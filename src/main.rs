use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Input};
use indicatif::{ProgressBar, ProgressStyle};
use std::{thread, time::Duration};

/// Simple CLI demonstration
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional name to greet
    #[arg(short, long)]
    name: Option<String>,
}

fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Get name either from args or through interactive prompt
    let name = match args.name {
        Some(name) => name,
        None => {
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt("What's your name?")
                .interact_text()
                .unwrap()
        }
    };

    // Show a fancy progress bar
    let pb = ProgressBar::new(100);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    // Simulate some work
    for i in 0..100 {
        pb.set_position(i + 1);
        thread::sleep(Duration::from_millis(20));
    }
    pb.finish_with_message("Loading complete!");

    // Display the greeting
    println!("\nHello, {}! Welcome to PhasorSyncRS!", name);
}
