use ratatui::{
    layout::Alignment,
    prelude::{Color, Stylize, Style},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

use super::WastelandManagerCard;

impl WastelandManagerCard {
    pub(super) fn _make_empty_message_component<'a>(&self, text: &'a str, border_color: Option<Color>) -> Paragraph<'a> {
        let mut block = Block::default()
            .padding(Padding::symmetric(1, 1));

        if let Some(border_color) = border_color {
            block = block
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color));
        }

        Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(Color::Gray).italic())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
    }
}
