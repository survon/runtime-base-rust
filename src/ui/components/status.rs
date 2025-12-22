use ratatui::{
    layout::Alignment,
    prelude::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use super::UiComponent;

impl UiComponent {
    pub fn status(status: &str) -> Paragraph {
        Paragraph::new(status)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title(" Status ")
            )
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center)
    }
}
