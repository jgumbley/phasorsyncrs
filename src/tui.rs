use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::{error::Error, io, time::Duration};

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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    f.render_widget(title_block(), chunks[0]);
    f.render_widget(transport_paragraph(shared_state), chunks[1]);
    f.render_widget(controls_paragraph(), chunks[2]);
}

fn title_block() -> Block<'static> {
    Block::default()
        .title("PhasorSyncRS Terminal UI")
        .borders(Borders::ALL)
}

fn transport_paragraph(shared_state: &Arc<Mutex<state::SharedState>>) -> Paragraph<'static> {
    let lines = {
        let state = shared_state.lock().unwrap();
        build_transport_lines(&state)
    };

    Paragraph::new(lines)
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true })
        .block(Block::default().title("Transport").borders(Borders::ALL))
}

fn build_transport_lines(state: &state::SharedState) -> Vec<Spans<'static>> {
    let recording_indicator = if state.recording {
        Span::styled("● Recording", Style::default().fg(Color::Red))
    } else {
        Span::styled("○ Idle", Style::default().fg(Color::DarkGray))
    };
    let recording_target = state
        .recording_target
        .as_deref()
        .unwrap_or("wav_files/take_%Y%m%d_%H%M%S_pair1.wav");

    vec![
        Spans::from(vec![
            Span::raw("Transport: "),
            Span::styled(
                format!("{:?}", state.transport_state),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Spans::from(vec![
            Span::raw("BPM: "),
            Span::styled(
                state.get_bpm().to_string(),
                Style::default().fg(Color::Green),
            ),
            Span::raw("    Tick: "),
            Span::styled(
                state.get_tick_count().to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Spans::from(vec![
            Span::raw("Bar: "),
            Span::styled(
                state.get_current_bar().to_string(),
                Style::default().fg(Color::Yellow),
            ),
            Span::raw("    Beat: "),
            Span::styled(
                state.get_current_beat().to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Spans::from(vec![
            recording_indicator,
            Span::raw("  "),
            Span::styled(
                recording_target.to_string(),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ]
}

fn controls_paragraph() -> Paragraph<'static> {
    Paragraph::new(Spans::from(vec![
        Span::styled("SPACE", Style::default().fg(Color::Yellow)),
        Span::raw(": Start/Stop   "),
        Span::styled("Q", Style::default().fg(Color::Yellow)),
        Span::raw(": Quit"),
    ]))
    .style(Style::default().fg(Color::White))
    .block(Block::default().borders(Borders::ALL).title("Controls"))
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
        log::debug!("Screen repainted");

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
    use ratatui::backend::TestBackend;

    #[test]
    fn test_hello_world_tui_compiles_and_runs() {
        // Create a test shared state
        let shared_state = Arc::new(Mutex::new(SharedState::new(120)));
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");

        terminal
            .draw(|frame| render_ui(frame, &shared_state))
            .expect("render should succeed");
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
