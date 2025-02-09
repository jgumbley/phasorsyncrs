use crate::transport::TICKS_PER_BEAT;
use indicatif::{ProgressBar, ProgressStyle};

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
