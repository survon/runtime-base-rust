use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::{Block, BorderType, Gauge, Paragraph, Widget, Wrap},
};
use crate::module::Module;

pub fn render_monitoring_module(module: &Module, area: Rect, buf: &mut Buffer) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Current value
            Constraint::Min(1),     // Config details
        ])
        .split(area);

    // Mock current value (in real implementation, this would come from sensors)
    let current_value = 45.0; // Mock pressure value
    let max_threshold = module.config.thresholds
        .as_ref()
        .and_then(|t| t.get("high"))
        .copied()
        .unwrap_or(100.0);

    let gauge = Gauge::default()
        .block(
            Block::bordered()
                .title("Current Reading")
                .border_type(BorderType::Rounded)
        )
        .gauge_style(if current_value > max_threshold {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        })
        .percent((current_value / max_threshold * 100.0) as u16)
        .label(format!("{:.1}", current_value));

    gauge.render(layout[0], buf);

    // Configuration details
    let mut config_lines = vec![
        Line::from(format!("Bus Topic: {}", module.config.bus_topic)),
        Line::from(format!("View Type: {}", module.get_view_type())),
    ];

    if let Some(thresholds) = &module.config.thresholds {
        config_lines.push(Line::from("Thresholds:"));
        for (key, value) in thresholds {
            config_lines.push(Line::from(format!("  {}: {}", key, value)));
        }
    }

    if let Some(rules) = &module.config.rules {
        config_lines.push(Line::from("Rules:"));
        for (key, value) in rules {
            config_lines.push(Line::from(format!("  {}: {}", key, value)));
        }
    }

    let config_text = Text::from(config_lines);
    let config_widget = Paragraph::new(config_text)
        .block(
            Block::bordered()
                .title("Configuration")
                .border_type(BorderType::Rounded)
        )
        .wrap(Wrap { trim: true });

    config_widget.render(layout[1], buf);
}
