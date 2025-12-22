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
    pub(super) fn render_archived_modules(
        &self,
        area: Rect,
        buf: &mut Buffer,
        module: &mut Module,
    ) {
        let ViewData {
            border_color,
            selected_index,
            ..
        } = self.get_view_data(false, area, buf, module);

        let archived_modules = module
            .config
            .bindings
            .get("archived_modules")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(1),     // Module list
                Constraint::Length(3),  // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new(format!("📚 Archived Modules ({})", archived_modules.len()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(title, chunks[0], buf);

        // Module list
        if archived_modules.is_empty() {
            let empty_message = "No archived modules.";
            let empty_message_component = UiComponent::empty_message(empty_message, Some(border_color));
            Widget::render(empty_message_component, chunks[1], buf);
        } else {
            let list_items: Vec<ListItem> = archived_modules
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let style = if i == selected_index {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    };

                    let prefix = if i == selected_index { "↩ " } else { "  " };
                    ListItem::new(format!("{}{}", prefix, item)).style(style)
                })
                .collect();

            let list = List::new(list_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
                        .title(" Select module to restore ")
                );
            Widget::render(list, chunks[1], buf);
        }

        // Help
        let help_text = "↑/↓: Navigate • Enter: Restore Module • Esc: Back";
        let help_component = UiComponent::help(help_text);
        Widget::render(help_component, chunks[2], buf);
    }
}
