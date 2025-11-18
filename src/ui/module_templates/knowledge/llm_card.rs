// src/ui/module_templates/core/llm_card.rs
use crate::modules::Module;
use crate::ui::template::UiTemplate;
use ratatui::prelude::*;
use ratatui::buffer::Buffer;
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

#[derive(Debug)]
pub struct LlmCard;

impl UiTemplate for LlmCard {
    fn render(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        // Get LLM state from module bindings
        let model_info = module
            .config
            .bindings
            .get("model_info")
            .and_then(|v| v.as_str())
            .unwrap_or("No model loaded");

        let chat_history = module
            .config
            .bindings
            .get("chat_history")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let chat_input = module
            .config
            .bindings
            .get("chat_input")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let scroll_offset = module
            .config
            .bindings
            .get("scroll_offset")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u16;

        // Layout: title, chat history, input, help
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Min(1),     // Chat history
                Constraint::Length(3),  // Input box
                Constraint::Length(3),  // Help
            ])
            .split(area);

        // Title
        let title_color = if is_selected { Color::White } else { Color::Green };
        let title = Paragraph::new(format!("🤖 {} - Interactive Chat", module.config.name))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(title_color))
                    .title(format!(" {} ", model_info))
            )
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        Widget::render(title, chunks[0], buf);

        // Chat history
        self.render_chat_history(&chat_history, scroll_offset, chunks[1], buf);

        // Input box
        let input_color = if is_selected { Color::Yellow } else { Color::DarkGray };
        let input_text = format!("> {}", chat_input);
        let input_widget = Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(input_color))
                    .title(" Type your message ")
            )
            .style(Style::default().fg(Color::Yellow));
        Widget::render(input_widget, chunks[2], buf);

        // Help
        let help_color = if is_selected { Color::Cyan } else { Color::DarkGray };
        let help = Paragraph::new("Enter: send • ↑↓: scroll • Tab: cycle links • Esc: back")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(help_color))
                    .title(" Controls ")
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        Widget::render(help, chunks[3], buf);
    }

    fn required_bindings(&self) -> &'static [&'static str] {
        &["model_info", "chat_history", "chat_input", "scroll_offset"]
    }

    fn docs(&self) -> &'static str {
        "Interactive LLM chat interface. Displays chat history, input field, and controls. \
         Supports document links navigation with Tab key. Bindings: model_info (string), \
         chat_history (array of strings), chat_input (string), scroll_offset (number)."
    }
}

impl LlmCard {
    fn render_chat_history(&self, messages: &[String], scroll_offset: u16, area: Rect, buf: &mut Buffer) {
        let content = if messages.is_empty() {
            Text::from(vec![
                Line::from("Welcome to your Survon LLM assistant!"),
                Line::from(""),
                Line::from("I can help you with:"),
                Line::from("• Homestead management questions"),
                Line::from("• IoT device control and monitoring"),
                Line::from("• Information from your knowledge modules"),
                Line::from(""),
                Line::from("Try asking: 'What can you do?' or 'Help with my gate system'"),
            ])
        } else {
            self.format_messages(messages)
        };

        let chat_widget = Paragraph::new(content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Chat History (↑↓ to scroll) ")
            )
            .wrap(Wrap { trim: true })
            .scroll((scroll_offset, 0));

        Widget::render(chat_widget, area, buf);
    }

    fn format_messages(&self, messages: &[String]) -> Text<'static> {
        let mut lines = Vec::new();

        for msg in messages {
            // Parse message format: "role:content"
            let parts: Vec<&str> = msg.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }

            let (role, content) = (parts[0], parts[1]);
            let (prefix, style) = if role == "user" {
                ("You: ", Style::default().fg(Color::Cyan))
            } else {
                ("Assistant: ", Style::default().fg(Color::Green))
            };

            // Parse content for links
            let content_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
            let first_line = content_lines.first().cloned().unwrap_or_default();

            lines.push(Line::from(vec![
                Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                Span::styled(first_line, Style::default().fg(Color::White)),
            ]));

            for line in content_lines.into_iter().skip(1) {
                if line.contains("(from ./") {
                    self.format_link_line(&line, &mut lines);
                } else {
                    lines.push(Line::from(vec![
                        Span::styled("    ", Style::default()),
                        Span::styled(line, Style::default().fg(Color::White)),
                    ]));
                }
            }
            lines.push(Line::from(""));
        }

        Text::from(lines)
    }

    fn format_link_line(&self, line: &str, lines: &mut Vec<Line<'static>>) {
        let parts: Vec<&str> = line.split("(from ").collect();
        if parts.len() == 2 {
            let file_part = parts[1].trim_end_matches(')');
            let filename = std::path::Path::new(file_part)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file_part);

            let link_style = Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::UNDERLINED);

            lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(parts[0].to_string(), Style::default().fg(Color::White)),
                Span::styled("(from ", Style::default().fg(Color::White)),
                Span::styled(format!("{} [Tab to select]", filename), link_style),
                Span::styled(")", Style::default().fg(Color::White)),
            ]));
        }
    }
}

impl Default for LlmCard {
    fn default() -> Self {
        Self
    }
}
