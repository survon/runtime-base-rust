// src/ui/modules/monitoring/status_badge.rs
use crate::module::Module;
use crate::ui::template::UiTemplate;
use ratatui::prelude::*;
use ratatui::buffer::Buffer;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, Widget};
use ratatui::layout::{Alignment, Layout, Constraint, Direction};
use ratatui::text::{Line, Span};

#[derive(Debug)]
pub struct StatusBadge;

impl UiTemplate for StatusBadge {
    fn render(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        // Get the status from module bindings
        let status = module
            .config
            .bindings
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Get optional message
        let message = module
            .config
            .bindings
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Get optional timestamp
        let timestamp = module
            .config
            .bindings
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Get optional count/value
        let count = module
            .config
            .bindings
            .get("count")
            .and_then(|v| v.as_i64())
            .map(|n| n.to_string())
            .or_else(|| {
                module.config.bindings
                    .get("count")
                    .and_then(|v| v.as_f64())
                    .map(|n| format!("{:.1}", n))
            });

        // Determine styling based on status
        let (icon, color, status_display) = match status.to_lowercase().as_str() {
            "online" | "active" | "success" | "ok" | "operational" => {
                ("✓", Color::Green, "OPERATIONAL")
            }
            "offline" | "inactive" | "down" | "error" | "failed" => {
                ("✗", Color::Red, "ERROR")
            }
            "warning" | "degraded" | "slow" => {
                ("⚠", Color::Yellow, "WARNING")
            }
            "pending" | "loading" | "starting" => {
                ("⟳", Color::Cyan, "PENDING")
            }
            "maintenance" | "updating" => {
                ("⚙", Color::Blue, "MAINTENANCE")
            }
            _ => {
                ("?", Color::Gray, "UNKNOWN")
            }
        };

        let border_color = if is_selected { Color::White } else { color };

        // Create main container
        let block = Block::default()
            .title(format!(" {} ", module.config.name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        Widget::render(block, area, buf);

        // Split inner area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Icon and status
                Constraint::Length(2),  // Count/value if present
                Constraint::Min(2),     // Message
                Constraint::Length(1),  // Timestamp
            ])
            .split(inner);

        // Render icon and status
        let status_line = Line::from(vec![
            Span::styled(
                format!("{} ", icon),
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            ),
            Span::styled(
                status_display,
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            ),
        ]);
        let status_widget = Paragraph::new(status_line)
            .alignment(Alignment::Center);
        Widget::render(status_widget, chunks[0], buf);

        // Render count/value if present
        if let Some(count_str) = count {
            let count_widget = Paragraph::new(count_str)
                .style(Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            Widget::render(count_widget, chunks[1], buf);
        }

        // Render message if provided
        if !message.is_empty() {
            let message_widget = Paragraph::new(message)
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });
            Widget::render(message_widget, chunks[2], buf);
        }

        // Render timestamp if provided
        if !timestamp.is_empty() {
            let timestamp_widget = Paragraph::new(timestamp)
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            Widget::render(timestamp_widget, chunks[3], buf);
        }
    }

    fn required_bindings(&self) -> &'static [&'static str] {
        &["status"]
    }

    fn docs(&self) -> &'static str {
        "Status badge showing system/service health. Required: 'status' (online/offline/warning/pending/maintenance/etc). Optional: 'message' (string), 'timestamp' (string), 'count' (number). Color-coded by status type."
    }
}

impl Default for StatusBadge {
    fn default() -> Self {
        Self
    }
}
