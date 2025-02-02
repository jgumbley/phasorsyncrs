use crate::SharedState;
use indicatif::{ProgressBar, ProgressStyle};
use std::{thread, time::Duration};

pub fn create_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );
    pb
}

pub fn run_loading_simulation(pb: &ProgressBar) {
    for i in 0..100 {
        pb.set_position(i + 1);
        thread::sleep(Duration::from_millis(20));
    }
    pb.finish_with_message("Loading complete!");
}

pub fn run_state_inspector(state: SharedState) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        {
            let transport = state.lock().unwrap();
            println!(
                "Transport State - BPM: {:.1}, Tick: {}, Beat: {}, Bar: {}, Playing: {}",
                transport.bpm,
                transport.tick_count,
                transport.beat,
                transport.bar,
                transport.is_playing
            );
        }
        thread::sleep(Duration::from_millis(500));
    })
}
