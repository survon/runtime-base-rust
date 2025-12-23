use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{Color, Modifier, Style, Widget},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::modules::Module;
use crate::ui::components::UiComponent;
use super::{SideQuestCard, ViewData};

impl SideQuestCard {
    pub(super) fn render_quest_list(
        &self,
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        module: &mut Module,
    ) {
        let ViewData {
            border_color,
            selected_index,
            quests,
            quest_count,
            status_message,
            has_status,
            ..
        } = self.get_view_data(is_selected, area, buf, module);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if has_status {
                vec![
                    Constraint::Length(3),  // Title
                    Constraint::Min(1),     // Quest list
                    Constraint::Length(3),  // Status
                    Constraint::Length(3),  // Help
                ]
            } else {
                vec![
                    Constraint::Length(3),  // Title
                    Constraint::Min(1),     // Quest list
                    Constraint::Length(3),  // Help
                ]
            })
            .split(area);

        // Title
        let title = Paragraph::new(format!("🗺️  Side Quests ({})", quest_count))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(title, chunks[0], buf);

        // Quest list
        if quests.is_empty() {
            let empty_msg = Paragraph::new("No side quests yet!\n\nPress '[n]' to create your first quest.\n\nTrack activities you want to do someday -\nrock climbing, that new restaurant,\nlearning a skill, etc.")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
                )
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            Widget::render(empty_msg, chunks[1], buf);
        } else {
            let list_items: Vec<ListItem> = quests
                .iter()
                .enumerate()
                .map(|(i, quest)| {
                    let style = if i == selected_index {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Magenta)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let prefix = if i == selected_index { "▶ " } else { "  " };
                    ListItem::new(format!("{}{}", prefix, quest)).style(style)
                })
                .collect();

            let list = List::new(list_items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(border_color))
                        .title(" Your Quests ")
                );
            Widget::render(list, chunks[1], buf);
        }

        // Status message if present
        let help_index = if has_status {
            if let Some(status) = status_message {
                let status_component = UiComponent::status(status);
                Widget::render(status_component, chunks[2], buf);
            }
            3
        } else {
            2
        };

        // Help
        let help_text = "[n]: New | [c]: Complete | [d]: Delete | [Esc]: Back";
        let help_component = UiComponent::help(help_text);
        Widget::render(help_component, chunks[help_index], buf);
    }
}
