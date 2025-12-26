use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{Color, Modifier, Style, Widget},
    widgets::{Block, Borders, List, ListItem},
};

use crate::module::Module;
use crate::ui::components::UiComponent;
use super::{ViewData, OverseerCard};

impl OverseerCard {
    pub(super) fn render_main_menu(
        &self,
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        module: &mut Module,
    ) {
        let ViewData {
            border_color,
            selected_index,
            status_message,
            pending_count,
            known_count,
            registry_count,
            installed_count,
            archived_count,
            is_scanning,
            scan_countdown,
            has_status,
            ..
        } = self.get_view_data(is_selected, area, buf, module);

        let menu_items = vec![
            format!("⚠️  Trust Pending Devices ({})", pending_count),
            format!("📡 Manage All Devices ({})", known_count),
            format!("📦 Install from Registry ({})", registry_count),
            format!("⚙️  Manage Installed Modules ({})", installed_count),
            format!("📚 View Archived Modules ({})", archived_count),
            "← Back".to_string(),
        ];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if has_status {
                vec![
                    Constraint::Min(1),     // Menu
                    Constraint::Length(3),  // Status/Scan
                    Constraint::Length(3),  // Help
                ]
            } else {
                vec![
                    Constraint::Min(1),     // Menu
                    Constraint::Length(3),  // Help
                ]
            })
            .split(area);

        // Menu list
        let list_items: Vec<ListItem> = menu_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if i == selected_index { "▶ " } else { "  " };
                ListItem::new(format!("{}{}", prefix, item)).style(style)
            })
            .collect();

        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(" Main Menu ")
            );
        Widget::render(list, chunks[0], buf);

        // Status message if present
        let help_index = if has_status {
            if is_scanning {
                let is_scanning_component = UiComponent::is_scanning(&scan_countdown);
                Widget::render(is_scanning_component, chunks[2], buf);
            } else if let Some(status) = status_message {
                let status_component = UiComponent::status(status);
                Widget::render(status_component, chunks[1], buf);
            }
            2
        } else {
            1
        };

        // Help
        let help_text = "[s] Scan  [r] Refresh  [Esc] Back";
        let help_component = UiComponent::help(help_text);
        Widget::render(help_component, chunks[help_index], buf);
    }
}
