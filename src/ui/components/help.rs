use ratatui::{
    layout::Alignment,
    prelude::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use super::UiComponent;

impl UiComponent {
    pub fn help(text: &str) -> Paragraph {
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
    }
}
