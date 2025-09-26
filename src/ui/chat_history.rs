use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};
use crate::app::App;

pub fn render_chat_history(app: &App, area: Rect, buf: &mut Buffer) {
    let content = if let Some(llm_engine) = app.get_llm_engine() {
        match llm_engine.get_chat_history(50) {
            Ok(messages) => {
                if messages.is_empty() {
                    Text::from(vec![
                        Line::from("Welcome to your Survon LLM assistant!"),
                        Line::from(""),
                        Line::from("I can help you with:"),
                        Line::from("• Homestead management questions"),
                        Line::from("• IoT device control and monitoring"),
                        Line::from("• Information from your knowledge modules"),
                        Line::from(""),
                        Line::from("Try asking: 'What can you do?' or 'Help with my gate system'"),
                        Line::from("Or try: 'survival tips' or 'wilderness safety'"),
                    ])
                } else {
                    let mut lines = Vec::new();
                    let mut link_index = 0;

                    for msg in messages {
                        let prefix = if msg.role == "user" { "You: " } else { "Assistant: " };
                        let style = if msg.role == "user" {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default().fg(Color::Green)
                        };

                        let content_lines: Vec<String> = msg.content.lines().map(|s| s.to_string()).collect();
                        let first_line = content_lines.first().cloned().unwrap_or_default();

                        lines.push(Line::from(vec![
                            Span::styled(prefix, style.add_modifier(Modifier::BOLD)),
                            Span::styled(first_line, Style::default().fg(Color::White)),
                        ]));

                        for line in content_lines.into_iter().skip(1) {
                            if line.contains("(from ./") {
                                let parts: Vec<&str> = line.split("(from ").collect();
                                if parts.len() == 2 {
                                    let file_part = parts[1].trim_end_matches(')');
                                    let filename = std::path::Path::new(file_part)
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or(file_part);

                                    let is_selected = app.current_link_index == Some(link_index);
                                    let link_style = if is_selected {
                                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                                    } else {
                                        Style::default().fg(Color::Blue).add_modifier(Modifier::UNDERLINED)
                                    };

                                    lines.push(Line::from(vec![
                                        Span::styled("    ", Style::default()),
                                        Span::styled(parts[0].to_string(), Style::default().fg(Color::White)),
                                        Span::styled("(from ", Style::default().fg(Color::White)),
                                        Span::styled(
                                            format!("{} {}", filename, if is_selected { "[SELECTED - Press Enter]" } else { "[Tab to select]" }),
                                            link_style
                                        ),
                                        Span::styled(")", Style::default().fg(Color::White)),
                                    ]));

                                    link_index += 1;
                                }
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
            }
            Err(_) => Text::from("Error loading chat history..."),
        }
    } else {
        Text::from("LLM engine not available...")
    };

    let user_scroll_offset = app.get_chat_scroll_offset() as u16;

    let chat_widget = Paragraph::new(content)
        .block(
            Block::bordered()
                .title("Chat History (↑↓ to scroll, Enter to open files)")
                .border_type(BorderType::Rounded)
        )
        .wrap(Wrap { trim: true })
        .scroll((user_scroll_offset, 0));

    chat_widget.render(area, buf);
}
