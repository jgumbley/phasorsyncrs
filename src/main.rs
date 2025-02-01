use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use phasorsyncrs::{get_user_name, Args};
use std::{thread, time::Duration};

fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Get name either from args or through interactive prompt
    let name = get_user_name(args.name);

    // Show a fancy progress bar
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    // Simulate some work
    for i in 0..100 {
        pb.set_position(i + 1);
        thread::sleep(Duration::from_millis(20));
    }
    pb.finish_with_message("Loading complete!");

    // Display the greeting
    println!("\nHello, {}! Welcome to PhasorSyncRS!", name);
}
