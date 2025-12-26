use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{Color, Modifier, Style, Widget},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::module::Module;
use crate::ui::components::UiComponent;
use super::{SideQuestCard, ViewData};

impl SideQuestCard {
    pub(super) fn render_quest_detail(
        &self,
        area: Rect,
        buf: &mut Buffer,
        module: &mut Module,
    ) {
        let ViewData {
            selected_quest_title,
            selected_quest_description,
            selected_quest_topic,
            selected_quest_urgency,
            selected_quest_trigger,
            border_color,
            ..
        } = self.get_view_data(false, area, buf, module);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(1),     // Quest details
                Constraint::Length(3),  // Help
            ])
            .split(area);

        // Title
        let title_widget = Paragraph::new(format!("📋 {}", selected_quest_title))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(title_widget, chunks[0], buf);

        // Details
        let details = format!(
            "\nTopic: {}\nUrgency: {}\nDeadline: {}\n\nDescription:\n{}",
            selected_quest_topic,
            selected_quest_urgency,
            selected_quest_trigger,
            selected_quest_description
        );

        let details_widget = Paragraph::new(details)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        Widget::render(details_widget, chunks[1], buf);

        // Help
        let help_text = "[c] Complete  [Esc] Back";
        let help_component = UiComponent::help(help_text);
        Widget::render(help_component, chunks[2], buf);
    }
}
