// event_loop.rs

use crate::midi_output::{MidiMessage, MidiOutput, MidiOutputManager};
use crate::state;
use chrono::Local;
use log::{debug, error, info, trace, warn};
use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const TICK_HISTORY_SIZE: usize = 24 * 4; // Store last 4 beats (1 bar)

#[derive(Debug)]
pub enum EngineMessage {
    Tick,
    TransportCommand(TransportAction),
}

#[derive(Debug)]
pub enum TransportAction {
    Start,
    Stop,
}

pub struct EventLoop {
    shared_state: Arc<Mutex<state::SharedState>>,
    engine_rx: Receiver<EngineMessage>,
    last_tick_time: Mutex<Option<Instant>>,
    tick_history: Mutex<VecDeque<Duration>>,
    midi_output: Option<MidiOutputManager>,
    audio_recorder: AudioRecorder,
}

impl EventLoop {
    pub fn new(
        shared_state: Arc<Mutex<state::SharedState>>,
        engine_rx: Receiver<EngineMessage>,
        midi_output: Option<MidiOutputManager>,
        recording_reference: String,
    ) -> Self {
        EventLoop {
            shared_state,
            engine_rx,
            last_tick_time: Mutex::new(None),
            tick_history: Mutex::new(VecDeque::with_capacity(TICK_HISTORY_SIZE)),
            midi_output,
            audio_recorder: AudioRecorder::new(recording_reference),
        }
    }

    pub fn run(&mut self) {
        let start_time = Instant::now();
        loop {
            match self.engine_rx.recv() {
                Ok(EngineMessage::Tick) => self.handle_tick(start_time),
                Ok(EngineMessage::TransportCommand(action)) => {
                    self.handle_transport_command(action)
                }
                Err(e) => {
                    error!("Tick channel error: {}", e);
                    break;
                }
            }
        }
    }

    fn handle_tick(&mut self, start_time: Instant) {
        let now = Instant::now();
        let elapsed = now.duration_since(start_time).as_millis();
        trace!("EventLoop received tick at {} ms", elapsed);

        // Update tick history and BPM
        self.update_tick_history(now);

        // Update shared state
        {
            let mut state = self.shared_state.lock().unwrap();
            state.tick_update();
        }
        let current_tick = self.shared_state.lock().unwrap().get_tick_count();

        // Get new musical events from the musical graph
        let events = self.get_midi_events_from_musical_graph();

        // Delegate both sending and scheduling to the unified MIDI method
        if let Some(midi_output) = &mut self.midi_output {
            midi_output.process_tick_events(current_tick, events);
        }
    }

    fn get_midi_events_from_musical_graph(&self) -> Vec<MidiMessage> {
        let mut state = self.shared_state.lock().unwrap();
        let middle_c_triggered = crate::musical_graph::process_tick(&mut state);

        let mut events = Vec::new();
        if middle_c_triggered {
            info!("Sending MIDI note for triggered Middle C");
            events.push(MidiMessage::NoteOn {
                channel: 1,
                note: 60, // Middle C
                velocity: 100,
                duration_ticks: 48, // MIDDLE_C_DURATION_TICKS
            });
        }
        events
    }

    fn update_tick_history(&mut self, now: Instant) {
        let mut last_tick_time = self.last_tick_time.lock().unwrap();

        if let Some(last_time) = *last_tick_time {
            let duration = now.duration_since(last_time);
            update_tick_history(&self.tick_history, duration);

            let bpm = calculate_bpm(&self.tick_history.lock().unwrap());
            let mut state = self.shared_state.lock().unwrap();
            state.bpm = bpm;
            debug!("Calculated BPM: {}", bpm);
        } else {
            info!("First tick received, initializing last_tick_time");
        }

        *last_tick_time = Some(now);

        trace!(
            "Shared state updated: tick_count={}, current_beat={}, current_bar={}, bpm={}",
            self.shared_state.lock().unwrap().get_tick_count(),
            self.shared_state.lock().unwrap().get_current_beat(),
            self.shared_state.lock().unwrap().get_current_bar(),
            self.shared_state.lock().unwrap().get_bpm()
        );
    }

    fn handle_transport_command(&mut self, action: TransportAction) {
        let mut state = self.shared_state.lock().unwrap();
        match action {
            TransportAction::Start => {
                state.transport_state = state::TransportState::Playing;
                let bpm = state.get_bpm();
                self.audio_recorder.start(bpm);
            }
            TransportAction::Stop => {
                state.transport_state = state::TransportState::Stopped;
                state.tick_count = 0;
                // Reset bar/beat, etc.
                state.current_beat = 0;
                state.current_bar = 0;

                // Reset the musical graph tick count
                crate::musical_graph::reset_musical_tick_count();

                self.audio_recorder.stop();
            }
        }
    }
}

fn update_tick_history(tick_history: &Mutex<VecDeque<Duration>>, duration: Duration) {
    let mut tick_history_lock = tick_history.lock().unwrap();
    tick_history_lock.push_back(duration);
    if tick_history_lock.len() > TICK_HISTORY_SIZE {
        tick_history_lock.pop_front();
    }
}
fn calculate_bpm(tick_history: &VecDeque<Duration>) -> u32 {
    if tick_history.is_empty() {
        return 60;
    }

    let total_duration: Duration = tick_history.iter().sum();
    trace!("calculate_bpm: total_duration={:?}", total_duration);

    let average_duration = total_duration / tick_history.len() as u32;
    trace!("calculate_bpm: average_duration={:?}", average_duration);

    // 60 seconds / (duration in seconds * 24 ticks per beat)
    let seconds = average_duration.as_secs_f64();
    trace!("calculate_bpm: seconds={}", seconds);

    if seconds == 0.0 {
        // Avoid division by zero
        return 60;
    }
    let bpm = 60.0 / (seconds * 24.0);
    trace!("calculate_bpm: bpm={}", bpm);

    let rounded_bpm = bpm.round() as u32;
    trace!("calculate_bpm: rounded_bpm={}", rounded_bpm);
    rounded_bpm
}

struct AudioRecorder {
    current: Option<AudioRecordingSession>,
    reference: String,
    enabled: bool,
}

struct AudioRecordingSession {
    arecord: Child,
    aplay: Child,
    io_thread: Option<std::thread::JoinHandle<()>>,
    stderr_threads: Vec<std::thread::JoinHandle<()>>,
    stop_requested: Arc<AtomicBool>,
}

struct AudioSessionGuard {
    arecord: Option<Child>,
    aplay: Option<Child>,
}

impl AudioSessionGuard {
    fn spawn(capture_device: &str, playback_device: &str) -> std::io::Result<Self> {
        let mut arecord = spawn_arecord(capture_device)?;
        let mut aplay = match spawn_aplay(playback_device) {
            Ok(child) => child,
            Err(err) => {
                let _ = arecord.kill();
                let _ = arecord.wait();
                return Err(err);
            }
        };

        if let Some(status) = arecord.try_wait()? {
            let _ = aplay.kill();
            let _ = aplay.wait();
            return Err(std::io::Error::other(format!(
                "arecord exited immediately (device={capture_device}, status={status})"
            )));
        }

        if let Some(status) = aplay.try_wait()? {
            let _ = arecord.kill();
            let _ = arecord.wait();
            return Err(std::io::Error::other(format!(
                "aplay exited immediately (device={playback_device}, status={status})"
            )));
        }

        Ok(Self {
            arecord: Some(arecord),
            aplay: Some(aplay),
        })
    }

    fn arecord_mut(&mut self) -> &mut Child {
        self.arecord
            .as_mut()
            .expect("AudioSessionGuard arecord missing")
    }

    fn aplay_mut(&mut self) -> &mut Child {
        self.aplay
            .as_mut()
            .expect("AudioSessionGuard aplay missing")
    }

    fn into_children(mut self) -> (Child, Child) {
        (
            self.arecord
                .take()
                .expect("AudioSessionGuard arecord missing"),
            self.aplay.take().expect("AudioSessionGuard aplay missing"),
        )
    }
}

impl Drop for AudioSessionGuard {
    fn drop(&mut self) {
        if let Some(child) = self.aplay.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(child) = self.arecord.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl AudioRecorder {
    fn new(reference: String) -> Self {
        let sanitized_reference = reference
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();

        AudioRecorder {
            current: None,
            reference: sanitized_reference,
            enabled: !cfg!(test) && std::env::var("PHASOR_DISABLE_RECORDING").is_err(),
        }
    }

    fn start(&mut self, bpm: u32) {
        if !self.enabled {
            debug!("Audio recording disabled - skipping start");
            return;
        }

        if let Err(e) = fs::create_dir_all("wav_files") {
            error!("Failed to create wav_files directory: {}", e);
            return;
        }

        self.stop();

        let filename = self.build_target_path(bpm);
        info!("Starting audio capture and monitor to {}", filename);
        match spawn_audio_recording_session(&filename) {
            Ok(session) => self.current = Some(session),
            Err(err) => error!("Failed to start audio capture/monitor: {}", err),
        }
    }

    fn stop(&mut self) {
        let Some(session) = self.current.take() else {
            return;
        };

        Self::stop_recording_session(session);
    }

    fn build_target_path(&self, bpm: u32) -> String {
        let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
        let bpm_label = if bpm == 0 {
            "000".to_string()
        } else {
            format!("{:03}", bpm)
        };
        format!(
            "wav_files/{}{}-{}.wav",
            bpm_label, self.reference, timestamp
        )
    }

    fn stop_recording_session(session: AudioRecordingSession) {
        let AudioRecordingSession {
            mut arecord,
            mut aplay,
            io_thread,
            stderr_threads,
            stop_requested,
        } = session;

        stop_requested.store(true, Ordering::Relaxed);
        Self::stop_child_process(&mut aplay, "audio monitor");
        Self::stop_child_process(&mut arecord, "audio recorder");
        Self::join_optional_thread(io_thread, "Audio capture thread");
        Self::join_threads(stderr_threads, "Audio stderr thread");
    }

    fn stop_child_process(child: &mut Child, label: &str) {
        if let Err(err) = child.kill() {
            warn!("Failed to stop {} cleanly: {}", label, err);
        }
        let _ = child.wait();
    }

    fn join_optional_thread(handle: Option<std::thread::JoinHandle<()>>, label: &str) {
        if let Some(handle) = handle {
            Self::join_thread(handle, label);
        }
    }

    fn join_threads(handles: Vec<std::thread::JoinHandle<()>>, label: &str) {
        for handle in handles {
            Self::join_thread(handle, label);
        }
    }

    fn join_thread(handle: std::thread::JoinHandle<()>, label: &str) {
        if let Err(err) = handle.join() {
            warn!("{} panicked: {:?}", label, err);
        }
    }
}

fn spawn_audio_recording_session(target_path: &str) -> std::io::Result<AudioRecordingSession> {
    let capture_device = read_env_device_or_default("PHASOR_ALSA_CAPTURE_DEVICE", "default")?;
    let playback_device = read_env_device_or_default("PHASOR_ALSA_PLAYBACK_DEVICE", "default")?;
    info!("Audio capture device: {}", capture_device);
    info!("Audio playback device: {}", playback_device);

    let mut guard = AudioSessionGuard::spawn(&capture_device, &playback_device)?;

    let arecord_stdout = guard.arecord_mut().stdout.take().ok_or_else(|| {
        std::io::Error::other("arecord stdout unavailable (expected piped stdout)")
    })?;
    let aplay_stdin =
        guard.aplay_mut().stdin.take().ok_or_else(|| {
            std::io::Error::other("aplay stdin unavailable (expected piped stdin)")
        })?;

    let mut stderr_threads = Vec::new();
    if let Some(stderr) = guard.arecord_mut().stderr.take() {
        stderr_threads.push(spawn_child_stderr_logger("arecord", stderr));
    }
    if let Some(stderr) = guard.aplay_mut().stderr.take() {
        stderr_threads.push(spawn_child_stderr_logger("aplay", stderr));
    }
    let file = fs::File::create(target_path)?;

    let stop_requested = Arc::new(AtomicBool::new(false));
    let stop_requested_for_thread = Arc::clone(&stop_requested);
    let io_thread = Some(std::thread::spawn(move || {
        if let Err(err) =
            stream_and_record_cd_audio(file, arecord_stdout, aplay_stdin, stop_requested_for_thread)
        {
            error!("Audio capture/monitor pipeline failed: {}", err);
        }
    }));

    let (arecord, aplay) = guard.into_children();
    Ok(AudioRecordingSession {
        arecord,
        aplay,
        io_thread,
        stderr_threads,
        stop_requested,
    })
}

fn read_env_device_or_default(var: &str, default: &str) -> std::io::Result<String> {
    match std::env::var(var) {
        Ok(value) => {
            if value.trim().is_empty() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("{var} is set but empty"),
                ))
            } else {
                Ok(value)
            }
        }
        Err(std::env::VarError::NotPresent) => Ok(default.to_string()),
        Err(err) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid {var}: {err}"),
        )),
    }
}

fn spawn_child_stderr_logger(
    name: &'static str,
    stderr: std::process::ChildStderr,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) => error!("{name}: {line}"),
                Err(err) => {
                    error!("{name}: failed to read stderr: {err}");
                    break;
                }
            }
        }
    })
}

fn spawn_arecord(capture_device: &str) -> std::io::Result<Child> {
    Command::new("arecord")
        .args(["-q", "-f", "cd", "-t", "raw", "-D"])
        .arg(capture_device)
        .arg("-")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
}

fn spawn_aplay(playback_device: &str) -> std::io::Result<Child> {
    Command::new("aplay")
        .args(["-q", "-f", "cd", "-D"])
        .arg(playback_device)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
}

fn stream_and_record_cd_audio(
    mut output_file: fs::File,
    mut capture_stream: impl Read,
    mut playback_stream: impl Write,
    stop_requested: Arc<AtomicBool>,
) -> std::io::Result<()> {
    write_cd_wav_header(&mut output_file, 0)?;

    let mut buffer = [0u8; 16 * 1024];
    let mut bytes_written: u64 = 0;
    let mut last_header_update = Instant::now();

    loop {
        let bytes_read = capture_stream.read(&mut buffer)?;
        if bytes_read == 0 {
            if stop_requested.load(Ordering::Relaxed) {
                break;
            }
            return Err(std::io::Error::other(
                "audio capture ended unexpectedly (arecord returned EOF)",
            ));
        }

        output_file.write_all(&buffer[..bytes_read])?;
        bytes_written += bytes_read as u64;

        if let Err(err) = playback_stream.write_all(&buffer[..bytes_read]) {
            if stop_requested.load(Ordering::Relaxed)
                && err.kind() == std::io::ErrorKind::BrokenPipe
            {
                break;
            }
            return Err(err);
        }

        if last_header_update.elapsed() >= Duration::from_secs(1) {
            update_cd_wav_header(&mut output_file, bytes_written)?;
            last_header_update = Instant::now();
        }
    }

    drop(playback_stream);

    update_cd_wav_header(&mut output_file, bytes_written)?;

    Ok(())
}

fn write_cd_wav_header(output_file: &mut fs::File, data_len: u32) -> std::io::Result<()> {
    const SAMPLE_RATE: u32 = 44_100;
    const CHANNELS: u16 = 2;
    const BITS_PER_SAMPLE: u16 = 16;

    let block_align = CHANNELS
        .checked_mul(BITS_PER_SAMPLE / 8)
        .ok_or_else(|| std::io::Error::other("wav header overflow: block_align"))?;
    let byte_rate = SAMPLE_RATE
        .checked_mul(u32::from(block_align))
        .ok_or_else(|| std::io::Error::other("wav header overflow: byte_rate"))?;
    let riff_chunk_size = 36u32
        .checked_add(data_len)
        .ok_or_else(|| std::io::Error::other("wav header overflow: riff_chunk_size"))?;

    output_file.seek(SeekFrom::Start(0))?;
    output_file.write_all(b"RIFF")?;
    output_file.write_all(&riff_chunk_size.to_le_bytes())?;
    output_file.write_all(b"WAVE")?;

    output_file.write_all(b"fmt ")?;
    output_file.write_all(&16u32.to_le_bytes())?;
    output_file.write_all(&1u16.to_le_bytes())?;
    output_file.write_all(&CHANNELS.to_le_bytes())?;
    output_file.write_all(&SAMPLE_RATE.to_le_bytes())?;
    output_file.write_all(&byte_rate.to_le_bytes())?;
    output_file.write_all(&block_align.to_le_bytes())?;
    output_file.write_all(&BITS_PER_SAMPLE.to_le_bytes())?;

    output_file.write_all(b"data")?;
    output_file.write_all(&data_len.to_le_bytes())?;

    Ok(())
}

fn update_cd_wav_header(output_file: &mut fs::File, data_len: u64) -> std::io::Result<()> {
    let data_len =
        u32::try_from(data_len).map_err(|_| std::io::Error::other("wav data too large (>4GiB)"))?;
    let current_pos = output_file.stream_position()?;
    write_cd_wav_header(output_file, data_len)?;
    output_file.seek(SeekFrom::Start(current_pos))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    // No mock needed, we're using the real MidiOutputManager

    #[test]
    fn test_handle_transport_command_start() {
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None, "internal".to_string());

        // Initially, the transport state should be Stopped.
        assert_eq!(
            shared_state.lock().unwrap().transport_state,
            state::TransportState::Stopped
        );

        // Send a Start command.
        event_loop.handle_transport_command(TransportAction::Start);

        // The transport state should now be Playing.
        assert_eq!(
            shared_state.lock().unwrap().transport_state,
            state::TransportState::Playing
        );
    }

    #[test]
    fn test_handle_transport_command_stop() {
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None, "internal".to_string());

        // Initially, the transport state should be Stopped.
        assert_eq!(
            shared_state.lock().unwrap().transport_state,
            state::TransportState::Stopped
        );

        // Set the transport state to Playing.
        shared_state.lock().unwrap().transport_state = state::TransportState::Playing;

        // Send a Stop command.
        event_loop.handle_transport_command(TransportAction::Stop);

        // The transport state should now be Stopped.
        assert_eq!(
            shared_state.lock().unwrap().transport_state,
            state::TransportState::Stopped
        );

        // Tick count should be reset to 0.
        assert_eq!(shared_state.lock().unwrap().tick_count, 0);
    }

    #[test]
    fn test_handle_tick() {
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None, "internal".to_string());

        // Call handle_tick
        let start_time = Instant::now();
        event_loop.handle_tick(start_time);

        // Check if last_tick_time is updated
        let last_tick_time = event_loop.last_tick_time.lock().unwrap();
        assert!(last_tick_time.is_some());
    }

    #[test]
    fn test_handle_tick_updates_state() {
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None, "internal".to_string());

        // Set the transport state to Playing
        shared_state.lock().unwrap().transport_state = state::TransportState::Playing;

        // Get initial tick count
        let initial_tick_count = shared_state.lock().unwrap().tick_count;

        // Call handle_tick
        let start_time = Instant::now();
        event_loop.handle_tick(start_time);

        // Check if tick count is incremented
        let current_tick_count = shared_state.lock().unwrap().tick_count;
        assert_eq!(current_tick_count, initial_tick_count + 1);

        // Check if last_tick_time is updated
        let last_tick_time = event_loop.last_tick_time.lock().unwrap();
        assert!(last_tick_time.is_some());
    }

    #[test]
    fn test_update_tick_history() {
        let tick_history = Mutex::new(VecDeque::with_capacity(TICK_HISTORY_SIZE));
        let duration = Duration::from_millis(100);

        update_tick_history(&tick_history, duration);

        let tick_history_lock = tick_history.lock().unwrap();
        assert_eq!(tick_history_lock.len(), 1);
        assert_eq!(tick_history_lock.front(), Some(&duration));
    }

    #[test]
    fn test_update_tick_history_overflow() {
        let tick_history = Mutex::new(VecDeque::with_capacity(TICK_HISTORY_SIZE));
        for _ in 0..TICK_HISTORY_SIZE {
            let duration = Duration::from_millis(100);
            update_tick_history(&tick_history, duration);
        }

        let duration = Duration::from_millis(200);
        update_tick_history(&tick_history, duration);

        let tick_history_lock = tick_history.lock().unwrap();
        assert_eq!(tick_history_lock.len(), TICK_HISTORY_SIZE);
        assert_eq!(tick_history_lock.back(), Some(&duration));
    }

    #[test]
    fn test_calculate_bpm() {
        let tick_history: VecDeque<Duration> = vec![
            Duration::from_millis(500),
            Duration::from_millis(500),
            Duration::from_millis(500),
        ]
        .into();
        let bpm = calculate_bpm(&tick_history);
        assert_eq!(bpm, 5);
    }

    #[test]
    fn test_calculate_bpm_empty_history() {
        let tick_history: VecDeque<Duration> = VecDeque::new();
        let bpm = calculate_bpm(&tick_history);
        assert_eq!(bpm, 60);
    }

    #[test]
    fn test_calculate_bpm_with_various_durations() {
        // Test with 50ms between ticks (50 BPM)
        let mut tick_history: VecDeque<Duration> = VecDeque::new();
        for _ in 0..10 {
            tick_history.push_back(Duration::from_millis(50));
        }
        let bpm = calculate_bpm(&tick_history);
        assert_eq!(bpm, 50);

        // Test with 20ms between ticks (125 BPM)
        let mut tick_history: VecDeque<Duration> = VecDeque::new();
        for _ in 0..10 {
            tick_history.push_back(Duration::from_millis(20));
        }
        let bpm = calculate_bpm(&tick_history);
        assert_eq!(bpm, 125);
    }

    #[test]
    fn test_update_tick_history_method() {
        // Create a shared state
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();

        // Create the event loop
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None, "internal".to_string());

        // Set the transport state to Playing
        shared_state.lock().unwrap().transport_state = state::TransportState::Playing;

        // Call update_tick_history for the first time
        let now = Instant::now();
        event_loop.update_tick_history(now);

        // Verify that last_tick_time is updated
        let last_tick_time = event_loop.last_tick_time.lock().unwrap();
        assert!(last_tick_time.is_some());
    }

    #[test]
    fn test_update_tick_history_subsequent_tick() {
        // Create a shared state
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();

        // Create the event loop
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None, "internal".to_string());

        // Set the transport state to Playing
        shared_state.lock().unwrap().transport_state = state::TransportState::Playing;

        // Set last_tick_time to simulate a previous tick
        let first_time = Instant::now();
        *event_loop.last_tick_time.lock().unwrap() = Some(first_time);

        // Wait a bit to ensure a measurable duration
        std::thread::sleep(Duration::from_millis(10));

        // Call update_tick_history
        let second_time = Instant::now();
        event_loop.update_tick_history(second_time);

        // Verify that last_tick_time is updated
        let last_tick_time = event_loop.last_tick_time.lock().unwrap();
        assert_eq!(*last_tick_time, Some(second_time));

        // Verify that the tick history is updated
        let tick_history = event_loop.tick_history.lock().unwrap();
        assert_eq!(tick_history.len(), 1);

        // Verify that the BPM is calculated
        assert!(shared_state.lock().unwrap().get_bpm() > 0);
    }

    #[test]
    fn test_get_midi_events_from_musical_graph() {
        // Create a shared state
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();

        // Create the event loop
        let event_loop = EventLoop::new(shared_state.clone(), rx, None, "internal".to_string());

        // Set the transport state to Playing
        shared_state.lock().unwrap().transport_state = state::TransportState::Playing;

        // Get events when Middle C is not triggered
        let events = event_loop.get_midi_events_from_musical_graph();
        assert!(events.is_empty());
    }

    #[test]
    fn test_handle_tick_with_midi_output() {
        // Create a shared state
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();

        // Create a real MidiOutputManager
        let midi_output = Some(MidiOutputManager::new());

        // Create the event loop with the MIDI output
        let mut event_loop = EventLoop::new(
            shared_state.clone(),
            rx,
            midi_output,
            "internal".to_string(),
        );

        // Set the transport state to Playing
        shared_state.lock().unwrap().transport_state = state::TransportState::Playing;

        // Call handle_tick
        let start_time = Instant::now();
        event_loop.handle_tick(start_time);

        // Verify that tick count is incremented
        assert_eq!(shared_state.lock().unwrap().get_tick_count(), 1);
    }
}
