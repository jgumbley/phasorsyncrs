use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use phasorsyncrs::{create_shared_state, get_user_name, handle_device_list, Args};
use std::{thread, time::Duration};

fn run_timing_simulation(state: phasorsyncrs::SharedState) {
    let ticks_per_beat = 24; // MIDI standard PPQ

    thread::spawn(move || {
        loop {
            {
                let mut transport = state.lock().unwrap();
                if !transport.is_playing {
                    transport.is_playing = true;
                }

                transport.tick_count += 1;

                // Update beat and bar
                if transport.tick_count % ticks_per_beat == 0 {
                    transport.beat += 1;
                    if transport.beat > 4 {
                        // Assuming 4/4 time signature
                        transport.beat = 1;
                        transport.bar += 1;
                    }
                }
            }

            // Sleep for duration based on BPM
            // At 120 BPM, one beat is 500ms, so one tick is ~20.8ms
            thread::sleep(Duration::from_millis(21));
        }
    });
}

fn run_state_inspector(state: phasorsyncrs::SharedState) {
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
    });
}

fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Handle device listing if requested
    if args.device_list {
        let devices = handle_device_list();
        println!("Available MIDI devices:");
        for device in devices {
            println!("  - {}", device);
        }
        return;
    }

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

    // Create shared state
    let shared_state = create_shared_state();

    // Start the timing simulation thread
    let timing_state = shared_state.clone();
    run_timing_simulation(timing_state);

    // Start the inspector thread
    let inspector_state = shared_state.clone();
    run_state_inspector(inspector_state);

    // Keep the main thread running
    println!("\nPress Ctrl+C to exit...");
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
