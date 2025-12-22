use ratatui::{
    layout::Alignment,
    prelude::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

use super::UiComponent;

impl UiComponent {
    pub fn is_scanning(scan_countdown: &u8) -> Paragraph {
        let scan_msg = format!(
            "🔍 SCANNING FOR DEVICES... {} seconds remaining",
            scan_countdown
        );

        Paragraph::new(scan_msg)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .title(" Scan in Progress ")
            )
            .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
    }
}
