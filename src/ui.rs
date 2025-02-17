use std::error::Error;
use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    layout::{Layout, Constraint, Direction},
    widgets::{Block, Borders, Paragraph},
};
use crate::module_manager::LoadedModule;

pub fn run_app(loaded_modules: Vec<LoadedModule>) -> Result<(), Box<dyn Error>> {
    // Setup terminal in raw mode.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            // Divide the screen vertically into 4 equal rows.
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ])
                .split(size);

            // Create an iterator over rows.
            for row in rows.iter() {
                // Each row is divided into 4 equal columns.
                let cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                        Constraint::Percentage(25),
                    ])
                    .split(*row);
                for area in cols.iter() {
                    let content = if let Some(loaded_module) = loaded_modules.iter().next() {
                        loaded_module.module.render()
                    } else {
                        String::from("")
                    };
                    let paragraph = Paragraph::new(content)
                        .block(Block::default().borders(Borders::ALL));
                    f.render_widget(paragraph, *area);
                }
            }
        })?;

        // Exit on 'q' key press.
        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // Restore terminal settings.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
