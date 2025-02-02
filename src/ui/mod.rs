use crate::{transport::TICKS_PER_BEAT, SharedState};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{thread, time::Duration};

const BEATS_PER_BAR: u32 = 4; // Matching transport's 4/4 time signature assumption

pub fn create_beat_progress() -> ProgressBar {
    let pb = ProgressBar::new(u64::from(TICKS_PER_BEAT));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold} [{bar:40.cyan}] {pos}/{len}")
            .unwrap()
            .progress_chars("⣀⣤⣦⣶⣷⣿ "),
    );
    pb.set_prefix("Beat");
    pb
}

pub fn create_bar_progress() -> ProgressBar {
    let pb = ProgressBar::new(u64::from(BEATS_PER_BAR));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold} [{bar:20.white/black}] {pos}/{len}")
            .unwrap()
            .progress_chars("█▊ "),
    );
    pb.set_prefix("Bar");
    pb
}

pub fn create_transport_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{prefix:.bold.dim} {spinner} {wide_msg}")
            .unwrap(),
    );
    pb.set_prefix("Transport");
    pb
}

pub fn run_state_inspector(state: SharedState) -> thread::JoinHandle<()> {
    let multi = MultiProgress::new();

    // Create our progress indicators
    let beat_progress = multi.add(create_beat_progress());
    let bar_progress = multi.add(create_bar_progress());
    let transport_spinner = multi.add(create_transport_spinner());

    thread::spawn(move || loop {
        {
            let transport = state.lock().unwrap();

            // Update beat progress (based on ticks)
            let tick_in_beat = transport.tick_count % TICKS_PER_BEAT;
            beat_progress.set_position(u64::from(tick_in_beat));

            // Update bar progress (based on beats)
            let beat_in_bar = ((transport.beat - 1) % BEATS_PER_BAR) + 1;
            bar_progress.set_position(u64::from(beat_in_bar));

            // Update transport spinner with details
            transport_spinner.set_message(format!(
                "BPM: {:.1} | Tick: {} | Beat: {} | Bar: {} | {}",
                transport.bpm,
                transport.tick_count,
                transport.beat,
                transport.bar,
                if transport.is_playing {
                    "▶ Playing"
                } else {
                    "⏸ Stopped"
                }
            ));
        }
        thread::sleep(Duration::from_millis(16)); // ~60fps update rate
    })
}
