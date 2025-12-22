use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{Color, Modifier, Style, Widget},
    widgets::{Block, Borders, List, ListItem, Paragraph},

};

use crate::modules::Module;
use crate::ui::components::UiComponent;
use super::{ViewData, OverseerCard};

impl OverseerCard {
    pub(super) fn render_all_devices(
        &self,
        area: Rect,
        buf: &mut Buffer,
        module: &mut Module,
    ) {
        let ViewData {
            border_color,
            selected_index,
            status_message,
            has_status,
            is_scanning,
            scan_countdown,
            known_devices,
            ..
        } = self.get_view_data(false, area, buf, module);

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

        // Title
        let title = Paragraph::new(format!("📡 All Known Devices ({})", known_devices.len()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(title, chunks[0], buf);

        // Device list
        if known_devices.is_empty() {
            let empty_message = "No devices discovered yet.\n\nDevices will appear here when they're in range.\n\nPress 'r' to refresh scanning.";
            let empty_message_component = UiComponent::empty_message(empty_message, Some(border_color));
            Widget::render(empty_message_component, chunks[1], buf);
        } else {
            let list_items: Vec<ListItem> = known_devices
                .iter()
                .enumerate()
                .map(|(i, device)| {
                    // Parse device string format: "✓ Device Name (MAC) RSSI: -65 dBm"
                    let is_trusted = device.starts_with('✓');
                    let is_untrusted = device.starts_with('✗');

                    let style = if i == selected_index {
                        Style::default()
                            .fg(Color::Black)
                            .bg(if is_trusted { Color::Green } else { Color::Gray })
                            .add_modifier(Modifier::BOLD)
                    } else if is_trusted {
                        Style::default().fg(Color::Green)
                    } else if is_untrusted {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default().fg(Color::Gray)
                    };

                    let prefix = if i == selected_index { "▶ " } else { "  " };
                    ListItem::new(format!("{}{}", prefix, device)).style(style)
                })
                .collect();

            let list = List::new(list_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
                        .title(" All Known Devices ")
                );
            Widget::render(list, chunks[1], buf);
        }

        // Status message if present
        let help_index = if has_status {
            if is_scanning {
                let is_scanning_component = UiComponent::is_scanning(&scan_countdown);
                Widget::render(is_scanning_component, chunks[2], buf);
            } else if let Some(status) = status_message {
                let status_component = UiComponent::status(status);
                Widget::render(status_component, chunks[2], buf);
            }
            3
        } else {
            2
        };

        // Help
        let help_text = "↑/↓: Navigate • 't': Toggle Trust • 'd': Delete • 's': Scan Now • 'p': Pending • Esc: Back";
        let help_component = UiComponent::help(help_text);
        Widget::render(help_component, chunks[help_index], buf);
    }
}
