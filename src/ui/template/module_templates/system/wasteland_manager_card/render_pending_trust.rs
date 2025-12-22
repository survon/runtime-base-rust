use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{Color, Modifier, Style, Widget},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::modules::Module;
use super::{ViewData, WastelandManagerCard};

impl WastelandManagerCard {
    pub(super) fn render_pending_trust(
        &self,
        area: Rect,
        buf: &mut Buffer,
        module: &mut Module,
    ) {
        let ViewData {
            border_color,
            selected_index,
            status_message,
            is_scanning,
            pending_devices,
            scan_countdown,
            ..
        } = self.get_view_data(false, area, buf, module);

        let has_status = status_message.is_some() || is_scanning;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if has_status {
                vec![
                    Constraint::Length(3),  // Title
                    Constraint::Min(1),     // Device list
                    Constraint::Length(3),  // Status/Scan
                    Constraint::Length(3),  // Help
                ]
            } else {
                vec![
                    Constraint::Length(3),  // Title
                    Constraint::Min(1),     // Device list
                    Constraint::Length(3),  // Help
                ]
            })
            .split(area);

        // Title with alert styling
        let title = Paragraph::new(format!("⚠️  New Devices Discovered ({})", pending_devices.len()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(title, chunks[0], buf);

        // Device list
        if pending_devices.is_empty() {
            let empty_message = "No pending devices.\n\nNew devices will appear here when discovered.\n\nThey need to be trusted before registration.";
            let empty_message_component = self._make_empty_message_component(empty_message, Some(border_color.clone()));
            Widget::render(empty_message_component, chunks[1], buf);
        } else {
            let list_items: Vec<ListItem> = pending_devices
                .iter()
                .enumerate()
                .map(|(i, device)| {
                    let style = if i == selected_index {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let prefix = if i == selected_index { "▶ " } else { "  " };
                    ListItem::new(format!("{}{}", prefix, device)).style(style)
                })
                .collect();

            let list = List::new(list_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow))
                        .title(" Select device to trust ")
                );
            Widget::render(list, chunks[1], buf);
        }

        // Status message if present
        let help_index = if has_status {
            if is_scanning {
                let is_scanning_component = self._make_is_scanning_component(&scan_countdown);
                Widget::render(is_scanning_component, chunks[2], buf);
            } else if let Some(status) = status_message {
                let status_component = self._make_status_component(status);
                Widget::render(status_component, chunks[2], buf);
            }
            3
        } else {
            2
        };

        // Help
        let help_text = "↑/↓: Select • Enter: Trust & Register • 'i': Ignore • 's': Scan Now • 'v': View All • Esc: Back";
        let help_component = self._make_help_component(help_text);
        Widget::render(help_component, chunks[help_index], buf);
    }
}
