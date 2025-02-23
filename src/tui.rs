#[cfg(test)]
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[cfg(test)]
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
#[cfg(test)]
use std::{error::Error, io};

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
