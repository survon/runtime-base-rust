use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{Color, Modifier, Style, Widget},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::module::Module;
use crate::ui::components::UiComponent;
use super::{ViewData, OverseerCard};

impl OverseerCard {
    pub(super) fn render_manage_modules(
        &self,
        area: Rect,
        buf: &mut Buffer,
        module: &mut Module,
    ) {
        let ViewData {
            border_color,
            selected_index,
            installed_modules,
            ..
        } = self.get_view_data(false, area, buf, module);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(1),     // Module list
                Constraint::Length(3),  // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new(format!("⚙️  Installed Modules ({})", installed_modules.len()))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(title, chunks[0], buf);

        // Module list
        let list_items: Vec<ListItem> = installed_modules
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == selected_index {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
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
                    .title(" Manage installed modules ")
            );
        Widget::render(list, chunks[1], buf);

        // Help
        let help_text = "↑/↓: Navigate • 'a': Archive Module • Esc: Back";
        let help_component = UiComponent::help(help_text);
        Widget::render(help_component, chunks[2], buf);
    }
}
