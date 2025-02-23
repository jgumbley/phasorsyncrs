mod input;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
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
use std::{error::Error, io, time::Duration};

// Import the input mapping function
use crate::event_loop::{EngineMessage, TransportAction};
use crate::tui::input::map_key_event;

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

fn handle_key_event(key_event: crossterm::event::KeyEvent) -> Result<(), Box<dyn Error>> {
    log::info!("Key event received: {:?}", key_event);
    check_for_quit_key(&key_event)?;

    // Map the key event to a command and log it
    if let Some(message) = map_key_event(key_event) {
        match message {
            EngineMessage::TransportCommand(TransportAction::Start) => {
                log::info!("Would send Start command");
            }
            EngineMessage::TransportCommand(TransportAction::Stop) => {
                log::info!("Would send Stop command");
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn run_tui_event_loop() -> Result<(), Box<dyn Error>> {
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
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title("TUI - Key Mapping")
                .borders(Borders::ALL);
            let paragraph = Paragraph::new(Span::styled(
                "Press Space: Start | Press S: Stop | Press Q: Quit",
                Style::default().fg(Color::Yellow),
            ));
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
                .split(size);
            f.render_widget(block, chunks[0]);
            f.render_widget(paragraph, chunks[1]);
        })?;
        log::info!("Screen repainted");

        // Poll for an event with a timeout
        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(key_event) = event::read()? {
                handle_key_event(key_event)?;
            }
        }
    }
}

#[cfg(test)]
pub fn run_hello_world_tui() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Render a single frame
    terminal.draw(|frame| {
        let size = frame.size();
        let block = Block::default().title("Hello World").borders(Borders::ALL);
        let paragraph = Paragraph::new(Span::styled(
            "Hello from Ratatui + Crossterm!",
            Style::default().fg(Color::Yellow),
        ));
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
            .split(size);

        frame.render_widget(block, chunks[0]);
        frame.render_widget(paragraph, chunks[1]);
    })?;

    // Cleanup and exit
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world_tui_compiles_and_runs() {
        // Ensure that the TUI function executes without panicking.
        if let Err(e) = run_hello_world_tui() {
            panic!("TUI test failed: {}", e);
        }
    }
}
