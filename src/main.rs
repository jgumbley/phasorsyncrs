use clap::Parser;
use phasorsyncrs::{
    cli::{validate_device, Args},
    create_shared_state, handle_device_list,
    midi::MidirEngine,
    transport::{run_midi_input, run_timing_simulation},
    ui::{create_progress_bar, run_loading_simulation, run_state_inspector},
};
use std::{thread, time::Duration};

fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Get list of available devices
    let devices = handle_device_list();

    // Handle device listing if requested
    if args.device_list {
        println!("Available MIDI devices:");
        for device in devices {
            println!("  - {}", device);
        }
        return;
    }

    // Validate device if specified
    if let Some(device_name) = &args.bind_to_device {
        if let Err(error_msg) = validate_device(device_name, &devices) {
            eprintln!("{}", error_msg);
            std::process::exit(1);
        }
    }

    // Initialize MIDI engine if device binding is requested
    if let Some(device_name) = &args.bind_to_device {
        match MidirEngine::new(Some(device_name.clone())) {
            Ok(engine) => {
                println!("Successfully connected to MIDI device: {}", device_name);
                // Create shared state
                let shared_state = create_shared_state();

                // Start the MIDI input thread
                let midi_state = shared_state.clone();
                run_midi_input(engine, midi_state);

                // Start the timing simulation thread
                let timing_state = shared_state.clone();
                run_timing_simulation(timing_state);

                // Start the inspector thread
                let inspector_state = shared_state.clone();
                run_state_inspector(inspector_state);
            }
            Err(e) => {
                eprintln!("Error connecting to MIDI device: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Show a fancy progress bar
        let pb = create_progress_bar();
        run_loading_simulation(&pb);

        // Create shared state
        let shared_state = create_shared_state();

        // Start the timing simulation thread
        let timing_state = shared_state.clone();
        run_timing_simulation(timing_state);

        // Start the inspector thread
        let inspector_state = shared_state.clone();
        run_state_inspector(inspector_state);
    }

    // Keep the main thread running
    println!("\nPress Ctrl+C to exit...");
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
