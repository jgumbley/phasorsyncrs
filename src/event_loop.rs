// event_loop.rs

use crate::midi_output::MidiOutputManager;
use crate::state;
use log::{debug, error, info, trace};
use std::collections::HashMap;
use std::collections::VecDeque;
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
    rx: Receiver<EngineMessage>,
    last_tick_time: Mutex<Option<Instant>>,
    tick_history: Mutex<VecDeque<Duration>>,
    midi_output: Option<MidiOutputManager>,
    note_off_schedule: HashMap<u64, Vec<crate::midi_output::MidiMessage>>,
}

impl EventLoop {
    pub fn new(
        shared_state: Arc<Mutex<state::SharedState>>,
        rx: Receiver<EngineMessage>,
        midi_output: Option<MidiOutputManager>,
    ) -> Self {
        EventLoop {
            shared_state,
            rx,
            last_tick_time: Mutex::new(None),
            tick_history: Mutex::new(VecDeque::with_capacity(TICK_HISTORY_SIZE)),
            midi_output,
            note_off_schedule: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        let start_time = Instant::now();
        loop {
            match self.rx.recv() {
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

        let middle_c_triggered = self.update_state(now);

        // Second section - handle MIDI output after releasing the lock
        self.handle_midi_output(middle_c_triggered);

        self.process_scheduled_note_offs();
    }

    fn update_state(&mut self, now: Instant) -> bool {
        let mut state = self.shared_state.lock().unwrap();
        let mut last_tick_time = self.last_tick_time.lock().unwrap();

        if let Some(last_time) = *last_tick_time {
            let duration = now.duration_since(last_time);
            update_tick_history(&self.tick_history, duration);

            let bpm = calculate_bpm(&self.tick_history.lock().unwrap());
            state.bpm = bpm;
            debug!("Calculated BPM: {}", bpm);
        } else {
            info!("First tick received, initializing last_tick_time");
        }

        *last_tick_time = Some(now);
        state.tick_update();

        // Get the Middle C trigger state, but don't do MIDI operations yet
        let triggered = crate::musical_graph::process_tick(&mut state);

        trace!(
            "Shared state updated: tick_count={}, current_beat={}, current_bar={}, bpm={}",
            state.get_tick_count(),
            state.get_current_beat(),
            state.get_current_bar(),
            state.get_bpm()
        );

        triggered
    }

    fn process_scheduled_note_offs(&mut self) {
        if let Some(note_offs) = {
            let state = self.shared_state.lock().unwrap();
            let current_tick = state.get_tick_count();
            self.note_off_schedule.remove(&current_tick)
        } {
            for note_off in note_offs {
                self.send_midi_note_off(note_off);
            }
        }
    }
    fn send_midi_note_off(&mut self, note_off: crate::midi_output::MidiMessage) {
        if let Some(midi_output) = &mut self.midi_output {
            if let Err(e) = midi_output.send(note_off) {
                error!("Failed to send MIDI note off: {}", e);
            }
        }
    }

    fn handle_midi_output(&mut self, middle_c_triggered: bool) {
        if middle_c_triggered {
            if let Some(midi_output) = &mut self.midi_output {
                info!("Sending MIDI note for triggered Middle C");
                // Send note on
                if let Err(e) = midi_output.send(crate::midi_output::MidiMessage::NoteOn {
                    channel: 1,
                    note: 60, // Middle C
                    velocity: 100,
                }) {
                    error!("Failed to send MIDI Note On: {}", e);
                }

                const MIDDLE_C_DURATION_TICKS: u64 = 48;
                let state = self.shared_state.lock().unwrap();
                let current_tick = state.get_tick_count();
                let target_tick = current_tick + MIDDLE_C_DURATION_TICKS;

                let note_off = crate::midi_output::MidiMessage::NoteOff {
                    channel: 1,
                    note: 60,
                };
                self.note_off_schedule
                    .entry(target_tick)
                    .or_default()
                    .push(note_off);
            }
        }
    }

    fn handle_transport_command(&mut self, action: TransportAction) {
        let mut state = self.shared_state.lock().unwrap();
        match action {
            TransportAction::Start => state.transport_state = state::TransportState::Playing,
            TransportAction::Stop => {
                state.transport_state = state::TransportState::Stopped;
                state.tick_count = 0;
                // Reset bar/beat, etc.
                state.current_beat = 0;
                state.current_bar = 0;

                // Reset the musical graph tick count
                crate::musical_graph::reset_musical_tick_count();
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
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None);

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
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None);

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
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None);

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
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None);

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
    fn test_update_state_first_tick() {
        // Create a shared state
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();

        // Create the event loop
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None);

        // Set the transport state to Playing
        shared_state.lock().unwrap().transport_state = state::TransportState::Playing;

        // Call update_state for the first time
        let now = Instant::now();
        let triggered = event_loop.update_state(now);

        // Verify that last_tick_time is updated
        let last_tick_time = event_loop.last_tick_time.lock().unwrap();
        assert!(last_tick_time.is_some());

        // Verify that the tick count is incremented
        assert_eq!(shared_state.lock().unwrap().get_tick_count(), 1);

        // Verify that triggered is false (since we're not at a trigger point)
        assert!(!triggered);
    }

    #[test]
    fn test_update_state_subsequent_tick() {
        // Create a shared state
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();

        // Create the event loop
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None);

        // Set the transport state to Playing
        shared_state.lock().unwrap().transport_state = state::TransportState::Playing;

        // Set last_tick_time to simulate a previous tick
        let first_time = Instant::now();
        *event_loop.last_tick_time.lock().unwrap() = Some(first_time);

        // Wait a bit to ensure a measurable duration
        std::thread::sleep(Duration::from_millis(10));

        // Call update_state
        let second_time = Instant::now();
        let _triggered = event_loop.update_state(second_time);

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
    fn test_process_scheduled_note_offs() {
        // Create a shared state
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();

        // Create the event loop
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None);

        // Set up a note off message in the schedule
        let note_off = crate::midi_output::MidiMessage::NoteOff {
            channel: 1,
            note: 60,
        };

        // Schedule the note off for the current tick
        let current_tick = shared_state.lock().unwrap().get_tick_count();
        event_loop
            .note_off_schedule
            .insert(current_tick, vec![note_off]);

        // Process scheduled note offs
        event_loop.process_scheduled_note_offs();

        // Verify that the note off was removed from the schedule
        assert!(!event_loop.note_off_schedule.contains_key(&current_tick));
    }

    #[test]
    fn test_handle_midi_output_when_triggered() {
        // Create a shared state
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();

        // Create a real MidiOutputManager
        let midi_output = Some(crate::midi_output::MidiOutputManager::new());

        // Create the event loop with the MIDI output
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, midi_output);

        // Set the transport state to Playing
        shared_state.lock().unwrap().transport_state = state::TransportState::Playing;

        // Call handle_midi_output with middle_c_triggered = true
        event_loop.handle_midi_output(true);

        // Verify that a note off was scheduled
        let state = shared_state.lock().unwrap();
        let current_tick = state.get_tick_count();
        let target_tick = current_tick + 48; // MIDDLE_C_DURATION_TICKS

        assert!(event_loop.note_off_schedule.contains_key(&target_tick));

        // Verify the scheduled note off is for Middle C
        let scheduled_note_offs = event_loop.note_off_schedule.get(&target_tick).unwrap();
        assert_eq!(scheduled_note_offs.len(), 1);

        match &scheduled_note_offs[0] {
            crate::midi_output::MidiMessage::NoteOff { channel, note } => {
                assert_eq!(*channel, 1);
                assert_eq!(*note, 60); // Middle C
            }
            _ => panic!("Expected NoteOff message"),
        }
    }

    #[test]
    fn test_handle_midi_output_when_not_triggered() {
        // Create a shared state
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();

        // Create the event loop
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None);

        // Set the transport state to Playing
        shared_state.lock().unwrap().transport_state = state::TransportState::Playing;

        // Call handle_midi_output with middle_c_triggered = false
        event_loop.handle_midi_output(false);

        // Verify that no note offs were scheduled
        assert!(event_loop.note_off_schedule.is_empty());
    }

    #[test]
    fn test_send_midi_note_off_without_midi_output() {
        // Create a shared state
        let shared_state = Arc::new(Mutex::new(state::SharedState::new(120)));
        let (_tx, rx) = mpsc::channel();

        // Create the event loop with no MIDI output
        let mut event_loop = EventLoop::new(shared_state.clone(), rx, None);

        // Call send_midi_note_off
        let note_off = crate::midi_output::MidiMessage::NoteOff {
            channel: 1,
            note: 60,
        };

        // This should not panic even though midi_output is None
        event_loop.send_midi_note_off(note_off);
    }
}
