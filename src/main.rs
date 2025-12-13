use log::{debug, error, info};
use phasorsyncrs::{clock, config, event_loop, external_clock, logging, midi_output, state, tui};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::event_loop::EngineMessage;

fn initialize_clock(config: config::Config, engine_tx: Sender<EngineMessage>) {
    info!("Starting clock thread");

    // Create a new thread for the clock to run independently
    thread::spawn(move || {
        // Create the appropriate clock source based on configuration
        let clock_source: Box<dyn clock::ClockSource> = create_clock_source(&config, engine_tx);

        // Start the clock
        info!("Starting clock");
        clock_source.start();
    });
}

/// Creates the appropriate clock source based on configuration
fn create_clock_source(
    config: &config::Config,
    engine_tx: Sender<EngineMessage>,
) -> Box<dyn clock::ClockSource> {
    match config.clock_source {
        config::ClockSource::Internal => {
            info!("Initializing internal clock");
            Box::new(clock::InternalClock::new(engine_tx))
        }
        config::ClockSource::External => {
            info!("Initializing external clock");
            // Get the device name, panic with helpful message if not provided
            let device_name = config
                .bind_to_device
                .clone()
                .expect("Device binding required for external sync");

            Box::new(external_clock::ExternalClock::new(device_name, engine_tx))
        }
    }
}

fn start_ui(shared_state: Arc<Mutex<state::SharedState>>, engine_tx: Sender<EngineMessage>) {
    info!("Starting TUI");
    if let Err(e) = tui::run_tui_event_loop(shared_state, engine_tx) {
        eprintln!("TUI failed: {}", e);
        std::process::exit(1);
    }
}

fn initialize_logging() {
    // Initialize logging
    if let Err(e) = logging::init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
        std::process::exit(1);
    }
}

// Log configuration details
fn log_config_details(config: &config::Config) {
    debug!(
        "Clock source: {:?}",
        match config.clock_source {
            config::ClockSource::Internal => "Internal",
            config::ClockSource::External => "External",
        }
    );
    if let Some(device) = &config.bind_to_device {
        debug!("Bound to MIDI device: {}", device);
    }
}

fn recording_reference(config: &config::Config) -> String {
    if let Some(device) = &config.bind_to_device {
        return device.clone();
    }

    match config.clock_source {
        config::ClockSource::External => "external".to_string(),
        config::ClockSource::Internal => "internal".to_string(),
    }
}

fn send_http_response(stream: &mut TcpStream, status_line: &str, content_type: &str, body: &str) {
    let response = format!(
        "{status}\r\nContent-Length: {len}\r\nContent-Type: {ctype}\r\nConnection: close\r\n\r\n{body}",
        status = status_line,
        len = body.len(),
        ctype = content_type,
        body = body
    );
    if let Err(e) = stream.write_all(response.as_bytes()) {
        error!("Failed to send HTTP response: {}", e);
    }
}

fn handle_status_request(stream: &mut TcpStream, shared_state: &Arc<Mutex<state::SharedState>>) {
    let state = shared_state.lock().unwrap();
    let transport = match state.transport_state {
        state::TransportState::Playing => "Playing",
        state::TransportState::Stopped => "Stopped",
    };
    let body = format!(
        "{{\"transport\":\"{transport}\",\"bpm\":{},\"bar\":{},\"beat\":{}}}",
        state.get_bpm(),
        state.get_current_bar(),
        state.get_current_beat(),
    );
    send_http_response(
        stream,
        "HTTP/1.1 200 OK",
        "application/json; charset=utf-8",
        &body,
    );
}

fn handle_toggle_request(
    stream: &mut TcpStream,
    shared_state: &Arc<Mutex<state::SharedState>>,
    engine_tx: &Sender<EngineMessage>,
) {
    let current_state = {
        let state = shared_state.lock().unwrap();
        state.transport_state
    };

    let command = match current_state {
        state::TransportState::Playing => event_loop::TransportAction::Stop,
        state::TransportState::Stopped => event_loop::TransportAction::Start,
    };

    let target = match command {
        event_loop::TransportAction::Start => "Playing",
        event_loop::TransportAction::Stop => "Stopped",
    };

    if let Err(e) = engine_tx.send(EngineMessage::TransportCommand(command)) {
        error!("Failed to send transport toggle command: {}", e);
        send_http_response(
            stream,
            "HTTP/1.1 500 INTERNAL SERVER ERROR",
            "text/plain; charset=utf-8",
            "failed to toggle transport",
        );
        return;
    }

    let body = format!("{{\"requested\":\"{target}\"}}");
    send_http_response(
        stream,
        "HTTP/1.1 200 OK",
        "application/json; charset=utf-8",
        &body,
    );
}

fn handle_web_request(
    mut stream: TcpStream,
    shared_state: &Arc<Mutex<state::SharedState>>,
    engine_tx: &Sender<EngineMessage>,
) {
    let mut buffer = [0; 2048];
    let bytes_read = match stream.read(&mut buffer) {
        Ok(0) => return,
        Ok(n) => n,
        Err(e) => {
            error!("Failed to read from web client: {}", e);
            return;
        }
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut parts = request.lines().next().unwrap_or("").split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");

    match (method, path) {
        ("GET", "/") => {
            send_http_response(
                &mut stream,
                "HTTP/1.1 200 OK",
                "text/html; charset=utf-8",
                WEB_UI_HTML,
            );
        }
        ("GET", "/status") => handle_status_request(&mut stream, shared_state),
        ("POST", "/toggle") => handle_toggle_request(&mut stream, shared_state, engine_tx),
        _ => send_http_response(
            &mut stream,
            "HTTP/1.1 404 NOT FOUND",
            "text/plain; charset=utf-8",
            "not found",
        ),
    }
}

fn start_web_ui(shared_state: Arc<Mutex<state::SharedState>>, engine_tx: Sender<EngineMessage>) {
    thread::spawn(move || {
        let listener = match TcpListener::bind("0.0.0.0:8080") {
            Ok(listener) => listener,
            Err(e) => {
                error!("Failed to bind web UI port: {}", e);
                return;
            }
        };
        info!("Web UI available on all interfaces at http://0.0.0.0:8080");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => handle_web_request(stream, &shared_state, &engine_tx),
                Err(e) => error!("Web UI connection failed: {}", e),
            }
        }
    });
}

const WEB_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>PhasorSyncRS Transport</title>
  <style>
    body { font-family: Arial, sans-serif; margin: 1.5rem; background: #0b1021; color: #e6edf3; }
    .card { background: #131a33; padding: 1.25rem; border-radius: 10px; max-width: 420px; box-shadow: 0 10px 35px rgba(0,0,0,0.35); }
    h1 { margin-top: 0; letter-spacing: 0.5px; }
    button { margin-top: 1rem; padding: 0.75rem 1.25rem; border: none; border-radius: 8px; background: #41d1ff; color: #031225; font-size: 1rem; cursor: pointer; font-weight: bold; }
    button.playing { background: #ff784d; color: #1c0f08; }
    .status { font-size: 1.2rem; margin: 0.5rem 0; }
    .metrics { color: #9fb1d1; font-size: 0.95rem; }
  </style>
</head>
<body>
  <div class="card">
    <h1>Transport</h1>
    <div id="transport" class="status">Loading...</div>
    <div class="metrics">
      <div id="bpm">BPM: --</div>
      <div id="position">Bar: -- | Beat: --</div>
    </div>
    <button id="toggle">Toggle</button>
  </div>
  <script>
    const transportEl = document.getElementById('transport');
    const bpmEl = document.getElementById('bpm');
    const posEl = document.getElementById('position');
    const toggleBtn = document.getElementById('toggle');

    async function refreshStatus() {
      try {
        const res = await fetch('/status');
        if (!res.ok) return;
        const data = await res.json();
        transportEl.textContent = `Status: ${data.transport}`;
        bpmEl.textContent = `BPM: ${data.bpm}`;
        posEl.textContent = `Bar: ${data.bar} | Beat: ${data.beat}`;
        toggleBtn.textContent = data.transport === 'Playing' ? 'Pause' : 'Play';
        toggleBtn.className = data.transport === 'Playing' ? 'playing' : '';
      } catch (_) {
        transportEl.textContent = 'Status unavailable';
      }
    }

    async function toggleTransport() {
      try {
        await fetch('/toggle', { method: 'POST' });
        await refreshStatus();
      } catch (_) {
        transportEl.textContent = 'Toggle failed';
      }
    }

    toggleBtn.addEventListener('click', toggleTransport);
    refreshStatus();
    setInterval(refreshStatus, 500);
  </script>
</body>
</html>
"#;

// Initialize application components
fn initialize_components(
    config: config::Config,
) -> (Arc<Mutex<state::SharedState>>, Sender<EngineMessage>) {
    let recording_ref = recording_reference(&config);

    // Create shared state
    let shared_state = Arc::new(Mutex::new(state::SharedState::new(config.bpm)));
    info!("Shared state initialized with BPM: {}", config.bpm);

    // Create engine message channel
    let (engine_tx, engine_rx): (Sender<EngineMessage>, Receiver<EngineMessage>) = mpsc::channel();

    // Set up MIDI output - always initialize for musical graph
    info!("Setting up MIDI output for event loop");
    let mut output_manager = midi_output::MidiOutputManager::new();

    let result = if let Some(device) = &config.midi_output_device {
        output_manager.connect_to_device(device)
    } else {
        output_manager.connect_to_first_available()
    };

    let midi_output = if let Err(e) = result {
        error!("Failed to connect MIDI output: {}", e);
        None
    } else {
        info!("MIDI output connected successfully");
        Some(output_manager)
    };

    let midi_output = midi_output;

    // Start the clock thread
    initialize_clock(config, engine_tx.clone());

    // Start the event loop thread with MIDI output
    let event_loop_shared_state = Arc::clone(&shared_state);
    let recording_ref_for_loop = recording_ref.clone();
    info!("Starting event loop thread");
    thread::spawn(move || {
        let mut event_loop = event_loop::EventLoop::new(
            event_loop_shared_state,
            engine_rx,
            midi_output,
            recording_ref_for_loop,
        );
        event_loop.run();
    });

    (shared_state, engine_tx)
}

fn main() {
    initialize_logging();
    info!("Starting Phasorsyncrs");

    // Load configuration
    let config = config::Config::new();
    info!("Configuration loaded");

    // Log configuration details

    // Log configuration details
    log_config_details(&config);

    // Setup MIDI output
    info!("MIDI output setup complete");

    // Initialize components
    let (shared_state, engine_tx) = initialize_components(config);

    // Start the web UI thread
    start_web_ui(Arc::clone(&shared_state), engine_tx.clone());

    // Start the UI thread
    start_ui(Arc::clone(&shared_state), engine_tx.clone());

    info!("All threads started, entering main loop");
    // Keep the main thread alive to allow other threads to run
    loop {
        thread::park();
    }
}
