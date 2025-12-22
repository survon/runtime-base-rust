use ratatui::{
    layout::Alignment,
    prelude::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use super::WastelandManagerCard;

impl WastelandManagerCard {
    pub(super) fn _make_help_component<'a>(&self, text: &'a str) -> Paragraph<'a> {
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
