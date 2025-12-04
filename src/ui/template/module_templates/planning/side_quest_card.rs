// src/ui/module_templates/planning/side_quest_card.rs

use crate::modules::Module;
use crate::ui::template::UiTemplate;
use ratatui::prelude::*;
use ratatui::buffer::Buffer;
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};

#[derive(Debug)]
pub struct SideQuestCard;

impl UiTemplate for SideQuestCard {
    fn render(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        let current_view = module
            .config
            .bindings
            .get("current_view")
            .and_then(|v| v.as_str())
            .unwrap_or("QuestList");

        let border_color = if is_selected { Color::White } else { Color::Magenta };

        match current_view {
            "QuestList" => self.render_quest_list(area, buf, border_color, module),
            "CreateQuest" => self.render_create_quest(area, buf, border_color, module),
            "QuestDetail" => self.render_quest_detail(area, buf, border_color, module),
            _ => self.render_quest_list(area, buf, border_color, module),
        }
    }

    fn required_bindings(&self) -> &'static [&'static str] {
        &["current_view", "quests", "selected_index"]
    }

    fn docs(&self) -> &'static str {
        "Side Quest manager - Track activities and experiences you want to do 'someday'. \
         Create quests with urgency levels and optional deadlines."
    }
}

impl SideQuestCard {
    fn render_quest_list(
        &self,
        area: Rect,
        buf: &mut Buffer,
        border_color: Color,
        module: &Module,
    ) {
        let selected_index = module
            .config
            .bindings
            .get("selected_index")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let quests = module
            .config
            .bindings
            .get("quests")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let quest_count = quests.len();

        let status_message = module
            .config
            .bindings
            .get("status_message")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let has_status = status_message.is_some();
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
            let empty_msg = Paragraph::new("No side quests yet!\n\nPress 'n' to create your first quest.\n\nTrack activities you want to do someday -\nrock climbing, that new restaurant,\nlearning a skill, etc.")
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
                let status_widget = Paragraph::new(status)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Yellow))
                            .title(" Status ")
                    )
                    .style(Style::default().fg(Color::Yellow))
                    .alignment(Alignment::Center);
                Widget::render(status_widget, chunks[2], buf);
            }
            3
        } else {
            2
        };

        // Help
        let help = Paragraph::new("↑/↓: Navigate • Enter: View Details • 'n': New Quest • 'c': Complete • 'd': Delete • Esc: Back")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        Widget::render(help, chunks[help_index], buf);
    }

    fn render_create_quest(
        &self,
        area: Rect,
        buf: &mut Buffer,
        border_color: Color,
        module: &Module,
    ) {
        let create_step = module
            .config
            .bindings
            .get("create_step")
            .and_then(|v| v.as_str())
            .unwrap_or("Title");

        let selected_index = module
            .config
            .bindings
            .get("selected_index")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

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
                let form_title = module
                    .config
                    .bindings
                    .get("form_title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

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
                let form_description = module
                    .config
                    .bindings
                    .get("form_description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

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
                let available_topics = module
                    .config
                    .bindings
                    .get("available_topics")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

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
                let urgency_options = module
                    .config
                    .bindings
                    .get("urgency_options")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

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
                    1. One week from now\n\
                    2. One month from now\n\
                    3. Three months from now\n\n\
                    Enter: Skip (no deadline)")
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
                let title = module
                    .config
                    .bindings
                    .get("form_title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let description = module
                    .config
                    .bindings
                    .get("form_description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(none)");

                let topic = module
                    .config
                    .bindings
                    .get("form_topic")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let urgency = module
                    .config
                    .bindings
                    .get("form_urgency")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let summary = format!(
                    "\nReview your quest:\n\n\
                    Title: {}\n\
                    Description: {}\n\
                    Topic: {}\n\
                    Urgency: {}\n\n\
                    Create this quest? (y/n)",
                    title, description, topic, urgency
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
            "Title" | "Description" => "Type to enter text • Enter: Next • Esc: Cancel",
            "Topic" | "Urgency" => "↑/↓: Navigate • Enter: Select • Esc: Cancel",
            "TriggerDate" => "1/2/3: Select • Enter: Skip • Esc: Cancel",
            "Confirm" => "'y': Create Quest • 'n': Cancel • Esc: Cancel",
            _ => "Esc: Cancel",
        };

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        Widget::render(help, chunks[2], buf);
    }

    fn render_quest_detail(
        &self,
        area: Rect,
        buf: &mut Buffer,
        border_color: Color,
        module: &Module,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(1),     // Quest details
                Constraint::Length(3),  // Help
            ])
            .split(area);

        let title = module
            .config
            .bindings
            .get("selected_quest_title")
            .and_then(|v| v.as_str())
            .unwrap_or("Quest Details");

        let description = module
            .config
            .bindings
            .get("selected_quest_description")
            .and_then(|v| v.as_str())
            .unwrap_or("No description");

        let topic = module
            .config
            .bindings
            .get("selected_quest_topic")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let urgency = module
            .config
            .bindings
            .get("selected_quest_urgency")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let trigger = module
            .config
            .bindings
            .get("selected_quest_trigger")
            .and_then(|v| v.as_str())
            .unwrap_or("No deadline");

        // Title
        let title_widget = Paragraph::new(format!("📋 {}", title))
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
            topic, urgency, trigger, description
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
        let help = Paragraph::new("'c': Mark Complete • 'b'/Esc: Back to List")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        Widget::render(help, chunks[2], buf);
    }
}

impl Default for SideQuestCard {
    fn default() -> Self {
        Self
    }
}
