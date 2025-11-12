// src/ui/modules/com/activity_card.rs
use ratatui::prelude::*;
use ratatui::buffer::Buffer;
use ratatui::widgets::{Block, Borders, List, ListItem, Widget};

use crate::module::Module;
use crate::ui::template::UiTemplate;

#[derive(Debug)]
pub struct ActivityCard;

impl UiTemplate for ActivityCard {
    fn render(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        // Get the activity log from module bindings
        let activities = module
            .config
            .bindings
            .get("activity_log")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_else(Vec::new);

        // Get optional status from bindings for color coding
        let status = module
            .config
            .bindings
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("active");

        // Determine border color based on status
        let border_color = if is_selected { Color::White } else {
            match status {
                "error" => Color::Red,
                "warning" => Color::Yellow,
                "idle" => Color::Gray,
                _ => Color::Green, // "active" or default
            }
        };

        // Take last N items to fit the display (most recent at bottom)
        let display_count = (area.height.saturating_sub(2)) as usize; // Account for borders
        let recent_activities: Vec<String> = activities
            .iter()
            .rev()
            .take(display_count)
            .rev()
            .cloned()
            .collect();

        // Create list items with timestamps if available
        let items: Vec<ListItem> = recent_activities
            .iter()
            .map(|activity| {
                // Color-code different message types
                let style = if activity.contains("ERROR") || activity.contains("FAIL") {
                    Style::default().fg(Color::Red)
                } else if activity.contains("WARN") {
                    Style::default().fg(Color::Yellow)
                } else if activity.contains("INFO") {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(activity.as_str()).style(style)
            })
            .collect();

        // Create the list widget
        let list = List::new(items)
            .block(
                Block::default()
                    .title(format!(" {} [{}] ", module.config.name, status.to_uppercase()))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            );

        Widget::render(list, area, buf);
    }

    fn required_bindings(&self) -> &'static [&'static str] {
        &["activity_log"]
    }

    fn docs(&self) -> &'static str {
        "Scrollable activity feed showing recent messages. Supports 'activity_log' (array of strings) and optional 'status' (active/idle/warning/error). Messages containing ERROR/FAIL/WARN/INFO are color-coded."
    }
}

impl Default for ActivityCard {
    fn default() -> Self {
        Self
    }
}
