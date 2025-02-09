use clap::Parser;
use phasorsyncrs::{
    cli::{validate_device, Args},
    create_scheduler, create_shared_state, handle_device_list,
    midi::{run_external_clock, DefaultMidiEngine},
    transport::run_timing_simulation,
    ui::run_state_inspector,
    Scheduler, SharedState,
};
use std::{thread, time::Duration};
mod logging;

fn main() {
    initialize_logging();
    let args = parse_command_line_arguments();
    let devices = get_available_devices();

    if args.device_list {
        list_available_devices(&devices);
        return;
    }

    if let Some(device_name) = &args.bind_to_device {
        if let Err(error_msg) = validate_device(device_name, &devices) {
            log::error!("{}", error_msg);
            eprintln!("{}", error_msg);
            std::process::exit(1);
        }
    }

    let scheduler = create_scheduler();
    let shared_state = create_shared_state();

    if let Some(device_name) = &args.bind_to_device {
        initialize_midi_engine(device_name.clone(), &scheduler, &shared_state);
    } else {
        initialize_local_mode(&scheduler, &shared_state);
    }

    run_application_loop();
}

fn initialize_logging() {
    logging::init_logger().expect("Logger initialization failed");
    log::info!("Application starting");
}

fn parse_command_line_arguments() -> Args {
    Args::parse()
}

fn get_available_devices() -> Vec<String> {
    handle_device_list()
}

fn list_available_devices(devices: &[String]) {
    println!("Available MIDI devices:");
    for device in devices {
        println!("  - {}", device);
    }
}

fn initialize_midi_engine<T: Scheduler>(
    device_name: String,
    scheduler: &T,
    shared_state: &SharedState,
) {
    match DefaultMidiEngine::new(Some(device_name.clone())) {
        Ok(engine) => {
            log::info!("Successfully connected to MIDI device: {}", device_name);
            println!("Successfully connected to MIDI device: {}", device_name);

            let clock_state = shared_state.clone();
            let midi_engine = engine;
            scheduler.spawn(move || {
                run_external_clock(midi_engine, clock_state);
            });

            let inspector_state = shared_state.clone();
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
}

fn initialize_local_mode<T: Scheduler>(scheduler: &T, shared_state: &SharedState) {
    if let Ok(state) = shared_state.lock() {
        state.set_playing(true);
    }

    let timing_state = shared_state.clone();
    scheduler.spawn(move || {
        run_timing_simulation(timing_state);
    });

    let inspector_state = shared_state.clone();
    scheduler.spawn(move || {
        run_state_inspector(inspector_state);
    });
}

fn run_application_loop() {
    log::info!("Application running. Press Ctrl+C to exit...");
    println!("\nPress Ctrl+C to exit...");
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
