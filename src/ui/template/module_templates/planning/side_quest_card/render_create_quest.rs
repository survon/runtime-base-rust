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
    pub(super) fn render_create_quest(
        &self,
        area: Rect,
        buf: &mut Buffer,
        module: &mut Module,
    ) {
        let ViewData {
            border_color,
            selected_index,
            create_step,
            form_title,
            form_description,
            form_topic,
            form_urgency,
            available_topics,
            urgency_options,
            ..
        } = self.get_view_data(false, area, buf, module);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(1),     // Form content
                Constraint::Length(3),  // Help
            ])
            .split(area);

        // Title
        let title = Paragraph::new(format!("🆕 New Side Quest - {}", create_step))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
            )
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(title, chunks[0], buf);

        // Form content based on step
        match create_step {
            "Title" => {
                let content = Paragraph::new(format!("\nQuest Title:\n\n{}_", form_title))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(border_color))
                    )
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Left);
                Widget::render(content, chunks[1], buf);
            }
            "Description" => {
                let content = Paragraph::new(format!("\nDescription (optional):\n\n{}_", form_description))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(border_color))
                    )
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: true });
                Widget::render(content, chunks[1], buf);
            }
            "Topic" => {
                let list_items: Vec<ListItem> = available_topics
                    .iter()
                    .enumerate()
                    .map(|(i, topic)| {
                        let style = if i == selected_index {
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Cyan)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        };

                        let prefix = if i == selected_index { "▶ " } else { "  " };
                        ListItem::new(format!("{}{}", prefix, topic)).style(style)
                    })
                    .collect();

                let list = List::new(list_items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(border_color))
                            .title(" Select Topic ")
                    );
                Widget::render(list, chunks[1], buf);
            }
            "Urgency" => {
                let list_items: Vec<ListItem> = urgency_options
                    .iter()
                    .enumerate()
                    .map(|(i, urgency)| {
                        let icon = match urgency.as_str() {
                            "Chill" => "☁️",
                            "Casual" => "🌤️",
                            "Moderate" => "⚡",
                            "Pressing" => "🔥",
                            "Critical" => "🚨",
                            _ => "•",
                        };

                        let style = if i == selected_index {
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        };

                        let prefix = if i == selected_index { "▶ " } else { "  " };
                        ListItem::new(format!("{}{} {}", prefix, icon, urgency)).style(style)
                    })
                    .collect();

                let list = List::new(list_items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(border_color))
                            .title(" Select Urgency ")
                    );
                Widget::render(list, chunks[1], buf);
            }
            "TriggerDate" => {
                let content = Paragraph::new("\nOptional: Set a trigger date\n\n\
                    [1] One week from now\n\
                    [2] One month from now\n\
                    [3] Three months from now\n\n\
                    [Enter] Skip (no deadline)")
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(border_color))
                    )
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Left);
                Widget::render(content, chunks[1], buf);
            }
            "Confirm" => {
                let summary = format!(
                    "\nReview your quest:\n\n\
                    Title: {}\n\
                    Description: {}\n\
                    Topic: {}\n\
                    Urgency: {}\n\n\
                    Create this quest? ([y]/[n])",
                    form_title, form_description, form_topic, form_urgency
                );

                let content = Paragraph::new(summary)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Green))
                    )
                    .style(Style::default().fg(Color::White))
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: true });
                Widget::render(content, chunks[1], buf);
            }
            _ => {}
        }

        // Help
        let help_text = match create_step {
            "Title" | "Description" => "[Enter] Next  [Esc] Cancel",
            "Topic" | "Urgency" => "[Esc] Cancel",
            "TriggerDate" => "[Esc] Cancel",
            "Confirm" => "[Esc] Cancel",
            _ => "[Esc] Cancel",
        };
        let help_component = UiComponent::help(help_text);
        Widget::render(help_component, chunks[2], buf);
    }
}
