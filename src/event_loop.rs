// event_loop.rs

use crate::state;
use log::{error, info, trace};
use std::collections::VecDeque;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const TICK_HISTORY_SIZE: usize = 24 * 4; // Store last 4 beats (1 bar)

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
}

impl EventLoop {
    pub fn new(shared_state: Arc<Mutex<state::SharedState>>, rx: Receiver<EngineMessage>) -> Self {
        EventLoop {
            shared_state,
            rx,
            last_tick_time: Mutex::new(None),
            tick_history: Mutex::new(VecDeque::with_capacity(TICK_HISTORY_SIZE)),
        }
    }

    pub fn run(&self) {
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

    fn handle_tick(&self, start_time: Instant) {
        let now = Instant::now();
        let elapsed = now.duration_since(start_time).as_millis();
        trace!("EventLoop received tick at {} ms", elapsed);

        // A tick event has been received.
        let mut state = self.shared_state.lock().unwrap();
        let now = Instant::now();
        let mut last_tick_time = self.last_tick_time.lock().unwrap();

        if let Some(last_time) = *last_tick_time {
            let duration = now.duration_since(last_time);
            update_tick_history(&self.tick_history, duration);

            let bpm = calculate_bpm(&self.tick_history.lock().unwrap());
            state.bpm = bpm;
            info!("Calculated BPM: {}", bpm);
        } else {
            info!("First tick received, initializing last_tick_time");
        }

        *last_tick_time = Some(now);
        state.tick_update();
        trace!(
            "Shared state updated: tick_count={}, current_beat={}, current_bar={}, bpm={}",
            state.get_tick_count(),
            state.get_current_beat(),
            state.get_current_bar(),
            state.get_bpm()
        );
    }

    fn handle_transport_command(&self, action: TransportAction) {
        let mut state = self.shared_state.lock().unwrap();
        match action {
            TransportAction::Start => state.transport_state = state::TransportState::Playing,
            TransportAction::Stop => {
                state.transport_state = state::TransportState::Stopped;
                state.tick_count = 0;
                // Reset bar/beat, etc.
                state.current_beat = 0;
                state.current_bar = 0;
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
