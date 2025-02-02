use clap::Parser;
use phasorsyncrs::{
    cli::{validate_device, Args},
    create_scheduler, create_shared_state, handle_device_list,
    midi::{run_external_clock, DefaultMidiEngine},
    transport::run_timing_simulation,
    ui::run_state_inspector,
    Scheduler,
};
use std::{thread, time::Duration};
mod logging;

fn main() {
    // Initialize logging
    logging::init_logger().expect("Logger initialization failed");
    log::info!("Application starting");

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
            log::error!("{}", error_msg);
            eprintln!("{}", error_msg);
            std::process::exit(1);
        }
    }

    // Create scheduler for thread management
    let scheduler = create_scheduler();

    // Initialize MIDI engine if device binding is requested
    if let Some(device_name) = &args.bind_to_device {
        match DefaultMidiEngine::new(Some(device_name.clone())) {
            Ok(engine) => {
                log::info!("Successfully connected to MIDI device: {}", device_name);
                println!("Successfully connected to MIDI device: {}", device_name);
                // Create shared state
                let shared_state = create_shared_state();

                // Start the external clock thread
                let clock_state = shared_state.clone();
                let midi_engine = engine;
                scheduler.spawn(move || {
                    run_external_clock(midi_engine, clock_state);
                });

                // Start the inspector thread
                let inspector_state = shared_state;
                scheduler.spawn(move || {
                    run_state_inspector(inspector_state);
                });
            }
            Err(e) => {
                let error_msg = format!("Error connecting to MIDI device: {}", e);
                log::error!("{}", error_msg);
                eprintln!("{}", error_msg);
                std::process::exit(1);
            }
        }
    } else {
        // Create shared state
        let shared_state = create_shared_state();

        // Set initial playing state to true in local mode
        if let Ok(state) = shared_state.lock() {
            state.set_playing(true);
        }

        // Start the timing simulation thread
        let timing_state = shared_state.clone();
        scheduler.spawn(move || {
            run_timing_simulation(timing_state);
        });

        // Start the inspector thread
        let inspector_state = shared_state;
        scheduler.spawn(move || {
            run_state_inspector(inspector_state);
        });
    }

    // Keep the main thread running
    log::info!("Application running. Press Ctrl+C to exit...");
    println!("\nPress Ctrl+C to exit...");
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
