// event_loop.rs

use crate::midi_output::{MidiMessage, MidiOutput, MidiOutputManager};
use crate::state;
use log::{debug, error, info, trace, warn};
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::io;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
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
    recording_manager: ArecordManager,
}

impl EventLoop {
    pub fn new(
        shared_state: Arc<Mutex<state::SharedState>>,
        engine_rx: Receiver<EngineMessage>,
        midi_output: Option<MidiOutputManager>,
    ) -> Self {
        Self::with_recorder_spawner(
            shared_state,
            engine_rx,
            midi_output,
            Box::new(SystemRecordingSpawner),
        )
    }

    fn with_recorder_spawner(
        shared_state: Arc<Mutex<state::SharedState>>,
        engine_rx: Receiver<EngineMessage>,
        midi_output: Option<MidiOutputManager>,
        spawner: Box<dyn RecordingSpawner>,
    ) -> Self {
        EventLoop {
            shared_state,
            engine_rx,
            last_tick_time: Mutex::new(None),
            tick_history: Mutex::new(VecDeque::with_capacity(TICK_HISTORY_SIZE)),
            midi_output,
            recording_manager: ArecordManager::new(spawner),
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
        let current_state = self.shared_state.lock().unwrap().transport_state;

        match (current_state, action) {
            (state::TransportState::Stopped, TransportAction::Start) => {
                {
                    let mut state = self.shared_state.lock().unwrap();
                    state.transport_state = state::TransportState::Playing;
                }
                self.start_recording();
            }
            (state::TransportState::Playing, TransportAction::Stop) => {
                {
                    let mut state = self.shared_state.lock().unwrap();
                    state.transport_state = state::TransportState::Stopped;
                    state.tick_count = 0;
                    state.current_beat = 0;
                    state.current_bar = 0;
                }

                crate::musical_graph::reset_musical_tick_count();
                self.stop_recording();
            }
            (state::TransportState::Playing, TransportAction::Start) => {
                warn!("Start command received while already playing - ignoring");
            }
            (state::TransportState::Stopped, TransportAction::Stop) => {
                debug!("Stop command received while already stopped - ignoring");
            }
        }
    }

    fn start_recording(&mut self) {
        match self.recording_manager.start() {
            Ok(target) => {
                let mut state = self.shared_state.lock().unwrap();
                state.recording = true;
                state.recording_target = Some(target);
            }
            Err(err) => {
                error!("Failed to start arecord: {}", err);
                let mut state = self.shared_state.lock().unwrap();
                state.recording = false;
                state.recording_target = None;
            }
        }
    }

    fn stop_recording(&mut self) {
        if let Err(err) = self.recording_manager.stop() {
            warn!("Failed to stop arecord cleanly: {}", err);
        }

        let mut state = self.shared_state.lock().unwrap();
        state.recording = false;
        state.recording_target = None;
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

const ARECORD_FILENAME_TEMPLATE: &str = "wav_files/take_%Y%m%d_%H%M%S_pair1.wav";
const ARECORD_SAMPLE_RATE: &str = "48000";
const ARECORD_CHANNELS: &str = "2";
const ARECORD_FORMAT: &str = "S32_LE";
const ARECORD_WAIT_ATTEMPTS: u32 = 20;
const ARECORD_WAIT_STEP: Duration = Duration::from_millis(50);

struct ArecordManager {
    spawner: Box<dyn RecordingSpawner>,
    child: Option<Box<dyn ManagedChild>>,
}

impl ArecordManager {
    fn new(spawner: Box<dyn RecordingSpawner>) -> Self {
        Self {
            spawner,
            child: None,
        }
    }

    fn start(&mut self) -> io::Result<String> {
        if self.child.is_some() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "arecord already running",
            ));
        }

        ensure_pair_supported()?;

        fs::create_dir_all("wav_files")?;
        let device = read_capture_device()?;

        info!(
            "Starting arecord capture on device {} to template {}",
            device, ARECORD_FILENAME_TEMPLATE
        );

        let mut child = self.spawner.spawn(&device, ARECORD_FILENAME_TEMPLATE)?;

        if let Some(status) = child.try_wait()? {
            return Err(io::Error::other(format!(
                "arecord exited immediately with status {status}"
            )));
        }

        self.child = Some(child);
        Ok(ARECORD_FILENAME_TEMPLATE.to_string())
    }

    fn stop(&mut self) -> io::Result<()> {
        let Some(mut child) = self.child.take() else {
            debug!("Stop requested but arecord was not running");
            return Ok(());
        };

        let pid = child.id();
        if let Err(err) = self.spawner.send_signal(pid, Signal::Term) {
            warn!("Failed to send SIGTERM to arecord (pid {pid}): {err}");
        }

        let mut exited = false;
        for _ in 0..=ARECORD_WAIT_ATTEMPTS {
            match child.try_wait()? {
                Some(_status) => {
                    exited = true;
                    break;
                }
                None => thread::sleep(ARECORD_WAIT_STEP),
            }
        }

        if !exited {
            warn!(
                "arecord (pid {}) did not exit after SIGTERM - sending SIGKILL (WAV may be damaged)",
                pid
            );
            if let Err(err) = self.spawner.send_signal(pid, Signal::Kill) {
                warn!("Failed to SIGKILL arecord (pid {}): {}", pid, err);
            }
            let _ = child.wait();
        }

        Ok(())
    }
}

impl Drop for ArecordManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

trait ManagedChild: Send {
    fn id(&self) -> u32;
    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>>;
    fn wait(&mut self) -> io::Result<ExitStatus>;
}

struct RealChild {
    inner: Child,
}

impl ManagedChild for RealChild {
    fn id(&self) -> u32 {
        self.inner.id()
    }

    fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.inner.try_wait()
    }

    fn wait(&mut self) -> io::Result<ExitStatus> {
        self.inner.wait()
    }
}

#[derive(Clone, Copy, Debug)]
enum Signal {
    Term,
    Kill,
}

trait RecordingSpawner: Send {
    fn spawn(&self, device: &str, filename_template: &str) -> io::Result<Box<dyn ManagedChild>>;
    fn send_signal(&self, pid: u32, signal: Signal) -> io::Result<()>;
}

struct SystemRecordingSpawner;

impl RecordingSpawner for SystemRecordingSpawner {
    fn spawn(&self, device: &str, filename_template: &str) -> io::Result<Box<dyn ManagedChild>> {
        let stderr_target = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("wav_files/arecord.stderr.log")
            .map(Stdio::from)
            .unwrap_or_else(|err| {
                warn!("Failed to open arecord stderr log: {}", err);
                Stdio::null()
            });

        let child = Command::new("arecord")
            .arg("-D")
            .arg(device)
            .args([
                "-f",
                ARECORD_FORMAT,
                "-r",
                ARECORD_SAMPLE_RATE,
                "-c",
                ARECORD_CHANNELS,
                "-t",
                "wav",
                "-N",
                "--use-strftime",
            ])
            .arg(filename_template)
            .stdout(Stdio::null())
            .stderr(stderr_target)
            .spawn()?;

        Ok(Box::new(RealChild { inner: child }))
    }

    fn send_signal(&self, pid: u32, signal: Signal) -> io::Result<()> {
        let flag = match signal {
            Signal::Term => "-TERM",
            Signal::Kill => "-KILL",
        };

        let status = Command::new("kill")
            .args([flag, &pid.to_string()])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other(format!(
                "kill {} {} exited with status {}",
                flag, pid, status
            )))
        }
    }
}

fn read_capture_device() -> io::Result<String> {
    match env::var("PHASOR_ALSA_CAPTURE_DEVICE") {
        Ok(value) => {
            if value.trim().is_empty() {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "PHASOR_ALSA_CAPTURE_DEVICE is set but empty",
                ))
            } else {
                Ok(value)
            }
        }
        Err(env::VarError::NotPresent) => Ok("default".to_string()),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid PHASOR_ALSA_CAPTURE_DEVICE: {err}"),
        )),
    }
}

fn read_stereo_pair() -> io::Result<u32> {
    match env::var("PHASOR_ALSA_CAPTURE_STEREO_PAIR") {
        Ok(value) => {
            if value.trim().is_empty() {
                return Ok(1);
            }
            value.parse::<u32>().map_err(|err| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid PHASOR_ALSA_CAPTURE_STEREO_PAIR: {err}"),
                )
            })
        }
        Err(env::VarError::NotPresent) => Ok(1),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid PHASOR_ALSA_CAPTURE_STEREO_PAIR: {err}"),
        )),
    }
}

fn ensure_pair_supported() -> io::Result<()> {
    let pair = read_stereo_pair()?;
    if pair != 1 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "arecord backend currently supports only stereo pair 1/2; set AUDIO_CAPTURE_PAIR=1",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::os::unix::process::ExitStatusExt;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering as AtomicOrdering};
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    #[derive(Clone)]
    struct MockSpawner {
        starts: Arc<Mutex<Vec<(String, String)>>>,
        signals: Arc<Mutex<Vec<(u32, Signal)>>>,
        children: Arc<Mutex<HashMap<u32, Arc<AtomicBool>>>>,
        next_pid: Arc<AtomicU32>,
        exit_immediately: bool,
    }

    impl MockSpawner {
        fn new() -> Self {
            Self {
                starts: Arc::new(Mutex::new(Vec::new())),
                signals: Arc::new(Mutex::new(Vec::new())),
                children: Arc::new(Mutex::new(HashMap::new())),
                next_pid: Arc::new(AtomicU32::new(0)),
                exit_immediately: false,
            }
        }

        fn immediate_exit() -> Self {
            let mut spawner = Self::new();
            spawner.exit_immediately = true;
            spawner
        }
    }

    struct MockChild {
        pid: u32,
        should_exit: Arc<AtomicBool>,
    }

    impl ManagedChild for MockChild {
        fn id(&self) -> u32 {
            self.pid
        }

        fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
            if self.should_exit.swap(false, AtomicOrdering::SeqCst) {
                Ok(Some(ExitStatusExt::from_raw(0)))
            } else {
                Ok(None)
            }
        }

        fn wait(&mut self) -> io::Result<ExitStatus> {
            self.should_exit.store(false, AtomicOrdering::SeqCst);
            Ok(ExitStatusExt::from_raw(0))
        }
    }

    impl RecordingSpawner for MockSpawner {
        fn spawn(
            &self,
            device: &str,
            filename_template: &str,
        ) -> io::Result<Box<dyn ManagedChild>> {
            self.starts
                .lock()
                .unwrap()
                .push((device.to_string(), filename_template.to_string()));

            let pid = self.next_pid.fetch_add(1, AtomicOrdering::SeqCst) + 1;
            let should_exit = Arc::new(AtomicBool::new(self.exit_immediately));
            self.children
                .lock()
                .unwrap()
                .insert(pid, Arc::clone(&should_exit));

            Ok(Box::new(MockChild { pid, should_exit }))
        }

        fn send_signal(&self, pid: u32, signal: Signal) -> io::Result<()> {
            self.signals.lock().unwrap().push((pid, signal));
            if let Some(flag) = self.children.lock().unwrap().get(&pid) {
                flag.store(true, AtomicOrdering::SeqCst);
            }
            Ok(())
        }
    }

    fn build_event_loop(
        shared_state: Arc<Mutex<state::SharedState>>,
        spawner: MockSpawner,
    ) -> EventLoop {
        let (_tx, rx) = mpsc::channel();
        EventLoop::with_recorder_spawner(shared_state, rx, None, Box::new(spawner))
    }

    #[test]
    fn test_handle_transport_command_start() {
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let spawner = MockSpawner::new();
        let start_calls = spawner.starts.clone();
        let mut event_loop = build_event_loop(shared_state.clone(), spawner);

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
        assert!(shared_state.lock().unwrap().recording);
        assert_eq!(start_calls.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_handle_transport_command_stop() {
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let spawner = MockSpawner::new();
        let signal_calls = spawner.signals.clone();
        let mut event_loop = build_event_loop(shared_state.clone(), spawner);

        event_loop.handle_transport_command(TransportAction::Start);
        event_loop.handle_transport_command(TransportAction::Stop);

        assert_eq!(signal_calls.lock().unwrap().len(), 1);

        // The transport state should now be Stopped.
        assert_eq!(
            shared_state.lock().unwrap().transport_state,
            state::TransportState::Stopped
        );

        // Tick count should be reset to 0.
        assert_eq!(shared_state.lock().unwrap().tick_count, 0);
        assert!(!shared_state.lock().unwrap().recording);
    }

    #[test]
    fn test_start_command_only_runs_once() {
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let spawner = MockSpawner::new();
        let start_calls = spawner.starts.clone();
        let mut event_loop = build_event_loop(shared_state, spawner);

        event_loop.handle_transport_command(TransportAction::Start);
        event_loop.handle_transport_command(TransportAction::Start);

        assert_eq!(start_calls.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_stop_command_without_start_does_not_signal() {
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let spawner = MockSpawner::new();
        let signal_calls = spawner.signals.clone();
        let mut event_loop = build_event_loop(shared_state, spawner);

        event_loop.handle_transport_command(TransportAction::Stop);
        assert!(signal_calls.lock().unwrap().is_empty());
    }

    #[test]
    fn test_arecord_immediate_exit_surfaces_error() {
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let spawner = MockSpawner::immediate_exit();
        let mut event_loop = build_event_loop(shared_state.clone(), spawner);

        event_loop.handle_transport_command(TransportAction::Start);

        assert!(
            !shared_state.lock().unwrap().recording,
            "recording flag should be false when arecord exits immediately"
        );
    }

    #[test]
    fn test_handle_tick() {
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();
        let mut event_loop = EventLoop::with_recorder_spawner(
            shared_state.clone(),
            rx,
            None,
            Box::new(MockSpawner::new()),
        );

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
        let mut event_loop = EventLoop::with_recorder_spawner(
            shared_state.clone(),
            rx,
            None,
            Box::new(MockSpawner::new()),
        );

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
        let mut event_loop = EventLoop::with_recorder_spawner(
            shared_state.clone(),
            rx,
            None,
            Box::new(MockSpawner::new()),
        );

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
        let mut event_loop = EventLoop::with_recorder_spawner(
            shared_state.clone(),
            rx,
            None,
            Box::new(MockSpawner::new()),
        );

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
        let event_loop = EventLoop::with_recorder_spawner(
            shared_state.clone(),
            rx,
            None,
            Box::new(MockSpawner::new()),
        );

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
        let mut event_loop = EventLoop::with_recorder_spawner(
            shared_state.clone(),
            rx,
            midi_output,
            Box::new(MockSpawner::new()),
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
