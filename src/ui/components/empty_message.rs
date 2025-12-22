use ratatui::{
    layout::Alignment,
    prelude::{Color, Stylize, Style},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

use super::UiComponent;

impl UiComponent {
    pub fn empty_message(text: &str, border_color: Option<Color>) -> Paragraph {
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
