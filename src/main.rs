use log::{debug, error, info};
use phasorsyncrs::{clock, config, event_loop, external_clock, logging, midi_output, state, tui};
use std::cmp::Reverse;
use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::UNIX_EPOCH;

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
    thread::spawn(move || {
        info!("Starting TUI");
        if let Err(e) = tui::run_tui_event_loop(shared_state, engine_tx) {
            eprintln!("TUI failed: {} (continuing without TUI)", e);
            error!("TUI failed: {}", e);
        }
    });
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

fn send_binary_response(
    stream: &mut TcpStream,
    status_line: &str,
    content_type: &str,
    body: &[u8],
) {
    let header = format!(
        "{status}\r\nContent-Length: {len}\r\nContent-Type: {ctype}\r\nConnection: close\r\n\r\n",
        status = status_line,
        len = body.len(),
        ctype = content_type
    );

    if let Err(e) = stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(body))
    {
        error!("Failed to send binary HTTP response: {}", e);
    }
}

fn escape_json_string(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn wav_modified_secs(path: &Path) -> Option<u64> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    modified
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

fn list_recent_recordings(limit: usize) -> io::Result<Vec<(String, u64)>> {
    if limit == 0 || !Path::new("wav_files").exists() {
        return Ok(Vec::new());
    }

    let sample_name = "sample.wav";
    let sample_path = Path::new("wav_files").join(sample_name);
    let sample_entry = wav_modified_secs(&sample_path).map(|ts| (sample_name.to_string(), ts));

    let mut recordings: Vec<(String, u64)> = fs::read_dir("wav_files")?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            let is_wav = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("wav"))
                .unwrap_or(false);
            if !is_wav {
                return None;
            }
            let name = path.file_name()?.to_string_lossy().into_owned();
            wav_modified_secs(&path).map(|ts| (name, ts))
        })
        .filter(|(name, _)| name != sample_name)
        .collect();

    recordings.sort_by_key(|(_, ts)| Reverse(*ts));

    let max_non_sample = if sample_entry.is_some() {
        limit.saturating_sub(1)
    } else {
        limit
    };
    recordings.truncate(max_non_sample);

    if let Some(sample) = sample_entry {
        recordings.insert(0, sample);
    }

    Ok(recordings)
}

fn handle_status_request(stream: &mut TcpStream, shared_state: &Arc<Mutex<state::SharedState>>) {
    let state = shared_state.lock().unwrap();
    let transport = match state.transport_state {
        state::TransportState::Playing => "Playing",
        state::TransportState::Stopped => "Stopped",
    };
    let recording = if state.recording { "true" } else { "false" };
    let recording_target = state
        .recording_target
        .as_ref()
        .map(|s| format!("\"{}\"", s))
        .unwrap_or_else(|| "null".to_string());
    let body = format!(
        "{{\"transport\":\"{transport}\",\"bpm\":{},\"bar\":{},\"beat\":{},\"recording\":{recording},\"recording_target\":{recording_target}}}",
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

fn handle_recordings_request(stream: &mut TcpStream) {
    match list_recent_recordings(6) {
        Ok(recordings) => {
            let entries: Vec<String> = recordings
                .iter()
                .map(|(name, modified)| {
                    format!(
                        "{{\"name\":\"{}\",\"modified\":{}}}",
                        escape_json_string(name),
                        modified
                    )
                })
                .collect();
            let body = format!("[{}]", entries.join(","));
            send_http_response(
                stream,
                "HTTP/1.1 200 OK",
                "application/json; charset=utf-8",
                &body,
            );
        }
        Err(e) => {
            error!("Failed to list recordings: {}", e);
            send_http_response(
                stream,
                "HTTP/1.1 500 INTERNAL SERVER ERROR",
                "text/plain; charset=utf-8",
                "failed to list recordings",
            );
        }
    }
}

fn handle_wav_request(stream: &mut TcpStream, filename: &str) {
    if filename.is_empty()
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains("..")
    {
        send_http_response(
            stream,
            "HTTP/1.1 400 BAD REQUEST",
            "text/plain; charset=utf-8",
            "invalid file name",
        );
        return;
    }

    let path = Path::new("wav_files").join(filename);
    match fs::read(&path) {
        Ok(bytes) => send_binary_response(stream, "HTTP/1.1 200 OK", "audio/wav", &bytes),
        Err(e) if e.kind() == io::ErrorKind::NotFound => send_http_response(
            stream,
            "HTTP/1.1 404 NOT FOUND",
            "text/plain; charset=utf-8",
            "file not found",
        ),
        Err(e) => {
            error!("Failed to read wav file {}: {}", filename, e);
            send_http_response(
                stream,
                "HTTP/1.1 500 INTERNAL SERVER ERROR",
                "text/plain; charset=utf-8",
                "failed to read file",
            );
        }
    }
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

    if method == "GET" && path.starts_with("/wav/") {
        let filename = path.trim_start_matches("/wav/");
        handle_wav_request(&mut stream, filename);
        return;
    }

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
        ("GET", "/recordings") => handle_recordings_request(&mut stream),
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
    .stack { display: flex; flex-direction: column; gap: 1rem; max-width: 520px; }
    .card { background: #131a33; padding: 1.25rem; border-radius: 10px; box-shadow: 0 10px 35px rgba(0,0,0,0.35); }
    h1 { margin-top: 0; letter-spacing: 0.5px; }
    h2 { margin: 0 0 0.5rem 0; font-size: 1.1rem; color: #c8d6ff; }
    button { margin-top: 1rem; padding: 0.75rem 1.25rem; border: none; border-radius: 8px; background: #41d1ff; color: #031225; font-size: 1rem; cursor: pointer; font-weight: bold; }
    button.playing { background: #ff784d; color: #1c0f08; }
    .recording-buttons button { display: block; width: 100%; margin-top: 0.5rem; text-align: left; }
    .status { font-size: 1.2rem; margin: 0.5rem 0; }
    .metrics { color: #9fb1d1; font-size: 0.95rem; }
    audio { width: 100%; margin-top: 0.75rem; }
  </style>
</head>
<body>
  <div class="stack">
    <div class="card">
      <h1>Transport</h1>
      <div id="transport" class="status">Loading...</div>
      <div class="metrics">
        <div id="bpm">BPM: --</div>
        <div id="position">Bar: -- | Beat: --</div>
      </div>
      <button id="toggle">Toggle</button>
    </div>
    <div class="card">
      <h2>Recent Recordings</h2>
      <div id="recordings" class="recording-buttons">Loading...</div>
      <audio id="player" controls preload="none"></audio>
    </div>
  </div>
  <script>
    const transportEl = document.getElementById('transport');
    const bpmEl = document.getElementById('bpm');
    const posEl = document.getElementById('position');
    const toggleBtn = document.getElementById('toggle');
    const recordingsEl = document.getElementById('recordings');
    const playerEl = document.getElementById('player');

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

    function formatTimestamp(epochSeconds) {
      const date = new Date(epochSeconds * 1000);
      return date.toLocaleString();
    }

    async function refreshRecordings() {
      try {
        const res = await fetch('/recordings');
        if (!res.ok) {
          recordingsEl.textContent = 'Unable to load recordings';
          return;
        }
        const data = await res.json();
        recordingsEl.innerHTML = '';
        if (!Array.isArray(data) || data.length === 0) {
          recordingsEl.textContent = 'No recordings yet.';
          return;
        }
        data.forEach(item => {
          const btn = document.createElement('button');
          const when = typeof item.modified === 'number' ? formatTimestamp(item.modified) : '';
          btn.textContent = when ? `${item.name} — ${when}` : item.name;
          btn.addEventListener('click', () => {
            playerEl.src = `/wav/${item.name}`;
            playerEl.play().catch(() => {});
          });
          recordingsEl.appendChild(btn);
        });
      } catch (_) {
        recordingsEl.textContent = 'Unable to load recordings';
      }
    }

    toggleBtn.addEventListener('click', toggleTransport);
    refreshStatus();
    refreshRecordings();
    setInterval(refreshStatus, 500);
    setInterval(refreshRecordings, 4000);
  </script>
</body>
</html>
"#;

// Initialize application components
fn initialize_components(
    config: config::Config,
) -> (Arc<Mutex<state::SharedState>>, Sender<EngineMessage>) {
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
    info!("Starting event loop thread");
    thread::spawn(move || {
        let mut event_loop =
            event_loop::EventLoop::new(event_loop_shared_state, engine_rx, midi_output);
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
