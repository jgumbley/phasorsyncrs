// ui.rs

use crate::config::BEATS_PER_BAR;
use crate::state;
use indicatif::ProgressDrawTarget;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn create_beat_progress(multi_progress: &MultiProgress) -> ProgressBar {
    let pb = multi_progress.add(ProgressBar::new(BEATS_PER_BAR));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold} [{bar:40.cyan}] {pos}/{len}")
            .unwrap()
            .progress_chars("⣀⣤⣦⣶⣷⣿ "),
    );
    pb.set_prefix("Beat");
    pb
}

fn create_bar_progress(multi_progress: &MultiProgress) -> ProgressBar {
    let pb = multi_progress.add(ProgressBar::new(BEATS_PER_BAR));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold} [{bar:20.white/black}] {pos}/{len}")
            .unwrap()
            .progress_chars("█▊ "),
    );
    pb.set_prefix("Bar");
    pb
}

fn create_transport_spinner(multi_progress: &MultiProgress) -> ProgressBar {
    let pb = multi_progress.add(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{prefix:.bold.dim} {spinner} {wide_msg}")
            .unwrap(),
    );
    pb.set_prefix("Transport");
    pb
}

pub struct UI {
    shared_state: Arc<Mutex<state::SharedState>>,

    #[allow(dead_code)]
    multi_progress: MultiProgress,
    beat_pb: ProgressBar,
    bar_pb: ProgressBar,
    transport_pb: ProgressBar,
}

impl UI {
    pub fn new(shared_state: Arc<Mutex<state::SharedState>>) -> Self {
        let multi_progress = MultiProgress::with_draw_target(ProgressDrawTarget::stderr());
        let beat_pb = create_beat_progress(&multi_progress);
        let bar_pb = create_bar_progress(&multi_progress);
        let transport_pb = create_transport_spinner(&multi_progress);

        UI {
            shared_state,
            multi_progress,
            beat_pb,
            bar_pb,
            transport_pb,
        }
    }

    pub fn run(&self) {
        loop {
            thread::sleep(Duration::from_millis(100));
            let state = self.shared_state.lock().unwrap();

            // Update the beat progress bar.
            self.beat_pb
                .set_position(u64::from(state.get_current_beat()));

            // Update the bar progress bar.
            self.bar_pb.set_position(u64::from(state.get_current_bar()));

            // Update the transport spinner message.
            self.transport_pb.set_message(format!(
                "BPM: {}, Tick Count: {}, Transport: {:?}",
                state.get_bpm(),
                state.get_tick_count(),
                state.transport_state
            ));

            // Tick the spinner to animate it.
            self.transport_pb.tick();
        }
    }
}
