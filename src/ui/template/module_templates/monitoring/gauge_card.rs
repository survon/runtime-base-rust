// src/ui/module_templates/monitoring/gauge_card.rs - ENHANCED VERSION
// Add CMD window status indicator to existing gauge

use crate::module::Module;
use crate::ui::template::UiTemplate;
use ratatui::prelude::*;
use ratatui::buffer::Buffer;
use ratatui::widgets::{Block, Borders, Gauge, Paragraph, Widget};
use ratatui::layout::{Alignment, Constraint, Direction, Layout};

#[derive(Debug)]
pub struct GaugeCard;

struct ViewData<'a> {
    value: f64,
    max_value: f64,
    unit_label: &'a str,
    display_name: &'a str,
    is_connected: bool,
    cmd_status: &'a str,
    device_mode: &'a str,
    percentage: u16,
    warn_threshold: f64,
    danger_threshold: f64,
    gauge_color: Color,
    border_color: Color,
    connected_icon: &'a str,
}

impl GaugeCard {
    fn get_view_data<'a>(
        &self,
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        module: &'a mut Module
    ) -> ViewData<'a> {
        // Existing gauge value
        let value = module
            .config
            .bindings
            .get("a")  // Primary sensor value
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let max_value = module
            .config
            .bindings
            .get("max_value")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0);

        let unit_label = module
            .config
            .bindings
            .get("unit_of_measure_label")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let display_name = module
            .config
            .bindings
            .get("display_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&module.config.name);

        // Connection status
        let is_connected = module
            .config
            .bindings
            .get("is_connected")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // NEW: CMD window status
        let cmd_status = module
            .config
            .bindings
            .get("cmd_window_status")
            .and_then(|v| v.as_str())
            .unwrap_or("⚪ Unknown");

        let device_mode = module
            .config
            .bindings
            .get("device_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Calculate gauge percentage
        let percentage = ((value / max_value) * 100.0).clamp(0.0, 100.0) as u16;

        // Color based on thresholds
        let warn_threshold = module
            .config
            .bindings
            .get("warn_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(70.0);

        let danger_threshold = module
            .config
            .bindings
            .get("danger_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(85.0);

        let gauge_color = if !is_connected {
            Color::Gray
        } else if value >= danger_threshold {
            Color::Red
        } else if value >= warn_threshold {
            Color::Yellow
        } else {
            Color::Green
        };

        let border_color = if is_selected {
            Color::White
        } else {
            Color::Cyan
        };

        let connected_icon = if is_connected { "🔗" } else { "⛓️‍💥" };

        ViewData {
            value,
            max_value,
            unit_label,
            display_name,
            is_connected,
            cmd_status,
            device_mode,
            percentage,
            warn_threshold,
            danger_threshold,
            gauge_color,
            border_color,
            connected_icon,
        }
    }
}

impl UiTemplate for GaugeCard {
    fn render_overview_cta(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        let ViewData {
            value,
            unit_label,
            display_name,
            cmd_status,
            device_mode,
            percentage,
            gauge_color,
            border_color,
            connected_icon,
            ..
        } = self.get_view_data(is_selected, area, buf, module);

        let block = Block::default()
            .title(format!(" {}{} ", connected_icon, display_name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        Widget::render(block, area, buf);

        // Split inner area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Label
                Constraint::Length(1),  // Switch visual
                Constraint::Length(3),  // Status text
            ])
            .split(inner);

        // Gauge with value display
        let gauge_label = format!("{:.1} {}", value, unit_label);
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .border_style(Style::default().fg(border_color))
            )
            .gauge_style(Style::default().fg(gauge_color))
            .percent(percentage)
            .label(gauge_label);
        Widget::render(gauge, chunks[1], buf);

        // NEW: CMD Window Status Indicator
        let cmd_color = match device_mode {
            "cmd" => Color::Green,      // In CMD window
            "data" => Color::Yellow,    // In DATA mode
            _ => Color::Gray,           // Unknown
        };

        let cmd_widget = Paragraph::new(cmd_status)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(cmd_color))
                    .title(" CMD Window ")
            )
            .style(Style::default().fg(cmd_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(cmd_widget, chunks[2], buf);
    }

    fn render_detail(&self, area: Rect, buf: &mut Buffer, module: &mut Module) {
        let ViewData {
            value,
            unit_label,
            display_name,
            cmd_status,
            device_mode,
            percentage,
            gauge_color,
            border_color,
            connected_icon,
            ..
        } = self.get_view_data(false, area, buf, module);

        let block = Block::default()
            .title(format!(" {}{} ", connected_icon, display_name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        Widget::render(block, area, buf);

        // Split inner area into sections
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Label
                Constraint::Length(1),  // Switch visual
                Constraint::Length(3),  // Status text
            ])
            .split(inner);

        // Gauge with value display
        let gauge_label = format!("{:.1} {}", value, unit_label);
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .border_style(Style::default().fg(border_color))
            )
            .gauge_style(Style::default().fg(gauge_color))
            .percent(percentage)
            .label(gauge_label);
        Widget::render(gauge, chunks[1], buf);

        // NEW: CMD Window Status Indicator
        let cmd_color = match device_mode {
            "cmd" => Color::Green,      // In CMD window
            "data" => Color::Yellow,    // In DATA mode
            _ => Color::Gray,           // Unknown
        };

        let cmd_widget = Paragraph::new(cmd_status)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(cmd_color))
                    .title(" CMD Window ")
            )
            .style(Style::default().fg(cmd_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        Widget::render(cmd_widget, chunks[2], buf);
    }

    fn required_bindings(&self) -> &'static [&'static str] {
        &["a", "max_value"]  // Minimum required bindings
    }

    fn docs(&self) -> &'static str {
        "Displays a gauge with value, connection status, and CMD window schedule. \
         Shows when the device will accept commands based on its scheduled windows."
    }
}

impl Default for GaugeCard {
    fn default() -> Self {
        Self
    }
}
