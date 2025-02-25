use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::{error::Error, io, time::Duration};

use crate::config::{BARS_PER_PHRASE, BEATS_PER_BAR};
use crate::event_loop::{EngineMessage, TransportAction};
use crate::state;

// Key mapping function moved from input.rs
fn map_key_event(key: KeyEvent) -> Option<EngineMessage> {
    match key.code {
        KeyCode::Char(' ') => None, // We'll handle space key specially
        _ => None,
    }
}

fn check_for_quit_key(key_event: &crossterm::event::KeyEvent) -> Result<(), Box<dyn Error>> {
    if let KeyCode::Char(c) = key_event.code {
        if c == 'q' || c == 'Q' {
            log::info!("Quit key pressed. Exiting event loop.");
            // Clean up the terminal before exiting
            disable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
            log::info!("Terminal cleaned up, exiting TUI event loop");
            std::process::exit(0); // Exit the program directly
        }
    }
    Ok(())
}
fn handle_key_event(
    key_event: crossterm::event::KeyEvent,
    message_tx: &Sender<EngineMessage>,
    shared_state: &Arc<Mutex<state::SharedState>>,
) -> Result<(), Box<dyn Error>> {
    log::info!("Key event received: {:?}", key_event);
    check_for_quit_key(&key_event)?;

    // Special handling for space key to toggle transport
    if let KeyCode::Char(' ') = key_event.code {
        // Get current transport state
        let current_state = {
            let state = shared_state.lock().unwrap();
            state.transport_state
        };

        // Toggle based on current state
        let command = match current_state {
            state::TransportState::Playing => TransportAction::Stop,
            state::TransportState::Stopped => TransportAction::Start,
        };

        log::info!("Space pressed - sending transport command: {:?}", command);
        message_tx
            .send(EngineMessage::TransportCommand(command))
            .unwrap();
        return Ok(());
    }

    // For all other keys, use the mapper
    if let Some(message) = map_key_event(key_event) {
        log::info!("Sending message to event loop: {:?}", message);
        message_tx.send(message).unwrap();
    }

    Ok(())
}

fn render_ui<B: ratatui::backend::Backend>(
    f: &mut ratatui::Frame<B>,
    shared_state: &Arc<Mutex<state::SharedState>>,
) {
    let size = f.size();

    // Create layout with multiple sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title area
                Constraint::Length(5), // Transport state area
                Constraint::Min(1),    // Instructions area
            ]
            .as_ref(),
        )
        .split(size);

    // Title block
    let title_block = Block::default()
        .title("PhasorSyncRS Terminal UI")
        .borders(Borders::ALL);
    f.render_widget(title_block, chunks[0]);

    // Transport state display
    let state_info = {
        let state = shared_state.lock().unwrap();
        format!(
            "BPM: {}\nTick Count: {}\nBeat: {}/{}\nBar: {}/{}\nTransport: {:?}",
            state.get_bpm(),
            state.get_tick_count(),
            state.get_current_beat(),
            BEATS_PER_BAR,
            state.get_current_bar(),
            BARS_PER_PHRASE,
            state.transport_state
        )
    };

    let transport_block = Block::default()
        .title("Transport State")
        .borders(Borders::ALL);
    let transport_text = Paragraph::new(state_info)
        .style(Style::default().fg(Color::Green))
        .block(transport_block);
    f.render_widget(transport_text, chunks[1]);

    // Instructions area
    let instructions = Paragraph::new(Span::styled(
        "Press Space: Toggle Play/Stop | Press Q: Quit",
        Style::default().fg(Color::Yellow),
    ));
    let instructions_block = Block::default().title("Controls").borders(Borders::ALL);
    f.render_widget(instructions.block(instructions_block), chunks[2]);
}

pub fn run_tui_event_loop(
    shared_state: Arc<Mutex<state::SharedState>>,
    message_tx: Sender<EngineMessage>,
) -> Result<(), Box<dyn Error>> {
    log::info!("Starting TUI event loop");
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Main event loop
    loop {
        // Repaint the UI on every iteration
        terminal.draw(|f| render_ui(f, &shared_state))?;
        log::info!("Screen repainted");

        // Poll for an event with a timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                handle_key_event(key_event, &message_tx, &shared_state)?;
            }
        }
    }
}

#[cfg(test)]
pub fn run_hello_world_tui(
    shared_state: Arc<Mutex<state::SharedState>>,
    _message_tx: Sender<EngineMessage>,
) -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Render a single frame using our UI renderer
    terminal.draw(|frame| render_ui(frame, &shared_state))?;

    // Cleanup and exit
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SharedState;

    #[test]
    fn test_hello_world_tui_compiles_and_runs() {
        // Create a test shared state
        let shared_state = Arc::new(Mutex::new(SharedState::new(120)));

        // Create a dummy channel
        let (tx, _rx) = std::sync::mpsc::channel();

        // Ensure that the TUI function executes without panicking.
        if let Err(e) = run_hello_world_tui(shared_state, tx) {
            panic!("TUI test failed: {}", e);
        }
    }

    #[test]
    fn test_space_returns_none() {
        let key_event = KeyEvent::from(KeyCode::Char(' '));
        assert!(map_key_event(key_event).is_none());
    }

    #[test]
    fn test_other_key_returns_none() {
        let key_event = KeyEvent::from(KeyCode::Char('x'));
        assert!(map_key_event(key_event).is_none());
    }
}
