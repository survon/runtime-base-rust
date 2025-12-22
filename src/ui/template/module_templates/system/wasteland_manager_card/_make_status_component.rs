use ratatui::{
    layout::Alignment,
    prelude::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use super::WastelandManagerCard;

impl WastelandManagerCard {
    pub(super) fn _make_status_component<'a>(&self, status: &'a str) -> Paragraph<'a> {
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
