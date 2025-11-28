// src/ui/module_templates/monitoring/chart_card.rs
use crate::modules::Module;
use crate::ui::template::UiTemplate;
use ratatui::prelude::*;
use ratatui::buffer::Buffer;
use ratatui::widgets::{
    Block, Borders, Widget, BarChart, Sparkline, Paragraph,
};
use ratatui::symbols;
use ratatui::layout::{Constraint, Direction, Layout, Alignment};
use crate::log_debug;

#[derive(Debug)]
pub struct ChartCard;

impl ChartCard {
    /// Get historical data from module bindings (populated by handler)
    fn get_history(module: &Module) -> Vec<(f64, f64, i64)> {
        // Read history that was set by the handler
        if let Some(history_json) = module.config.bindings.get("_chart_history") {
            if let Some(arr) = history_json.as_array() {
                return arr.iter()
                    .filter_map(|v| {
                        let obj = v.as_object()?;
                        let a = obj.get("a")?.as_f64()?;
                        let b = obj.get("b")?.as_f64()?;
                        let c = obj.get("c")?.as_i64()?;
                        Some((a, b, c))
                    })
                    .collect();
            }
        }
        Vec::new()
    }

    fn render_line_chart(&self, module: &Module, area: Rect, buf: &mut Buffer, is_selected: bool) {
        // Get history from handler (via bindings)
        let history = Self::get_history(module);

        // Get current values
        let a = module.config.bindings.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);

        log_debug!("Line chart - current a={}, history size={}", a, history.len());

        let is_connected = module.config.bindings.get("is_connected")
            .and_then(|v| v.as_bool()).unwrap_or(true);

        let status_suffix = module.config.bindings.get("status_suffix")
            .and_then(|v| v.as_str()).unwrap_or("");

        let unit = module.config.bindings.get("unit_of_measure_label")
            .and_then(|v| v.as_str()).unwrap_or("units");

        let border_color = if !is_connected {
            Color::Red
        } else if is_selected {
            Color::White
        } else {
            Color::Cyan
        };

        // Create chart data - use primary value (a)
        let data: Vec<(f64, f64)> = history.iter()
            .enumerate()
            .map(|(i, (val_a, _, _))| (i as f64, *val_a))
            .collect();

        // Calculate bounds
        let max_value = module.config.bindings.get("max_value")
            .and_then(|v| v.as_f64()).unwrap_or(100.0);
        let min_value = 0.0;

        // Split area for chart and status
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),      // Chart
                Constraint::Length(2),   // Current value display
            ])
            .split(area);

        // Render using ASCII art since ratatui's Chart needs datasets
        let chart_area = Block::default()
            .title(format!(" {}{} (Line) ", module.config.name, status_suffix))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .inner(chunks[0]);

        Block::default()
            .title(format!(" {}{} (Line) ", module.config.name, status_suffix))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .render(chunks[0], buf);

        // Draw simple ASCII line chart
        if !data.is_empty() && chart_area.height > 2 {
            let height = chart_area.height as f64;
            let width = chart_area.width as f64;
            let data_points = data.len() as f64;

            for (i, (_, val)) in data.iter().enumerate() {
                let x = chart_area.x + ((i as f64 / data_points) * width) as u16;
                let normalized = ((val - min_value) / (max_value - min_value)).clamp(0.0, 1.0);
                let y = chart_area.y + chart_area.height - 1 - ((normalized * (height - 1.0)) as u16);

                if x < chart_area.x + chart_area.width && y >= chart_area.y {
                    let style = if !is_connected {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    buf.get_mut(x, y).set_symbol("•").set_style(style);
                }
            }
        }

        // Current value display
        let value_text = format!("Current: {:.1} {} (Last {} points)", a, unit, history.len());
        let value_widget = Paragraph::new(value_text)
            .style(Style::default().fg(if is_connected { Color::White } else { Color::Red }))
            .alignment(Alignment::Center);
        value_widget.render(chunks[1], buf);
    }

    fn render_bar_chart(&self, module: &Module, area: Rect, buf: &mut Buffer, is_selected: bool) {
        // Get history from handler (via bindings)
        let history = Self::get_history(module);

        // Get current values for display
        let a = module.config.bindings.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let is_connected = module.config.bindings.get("is_connected")
            .and_then(|v| v.as_bool()).unwrap_or(true);

        let status_suffix = module.config.bindings.get("status_suffix")
            .and_then(|v| v.as_str()).unwrap_or("");

        let unit = module.config.bindings.get("unit_of_measure_label")
            .and_then(|v| v.as_str()).unwrap_or("units");

        let border_color = if !is_connected {
            Color::Red
        } else if is_selected {
            Color::White
        } else {
            Color::Green
        };

        // Take last 10 data points for bar chart
        let recent_data: Vec<(f64, f64, i64)> = history.iter()
            .rev()
            .take(10)
            .rev()
            .copied()
            .collect();

        // Create bar data as tuples (&str, u64) - required by ratatui 0.29+
        let bar_data: Vec<(&str, u64)> = recent_data.iter()
            .enumerate()
            .map(|(i, (val_a, _, _))| {
                // We need static strings, so use a fixed set
                let labels = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"];
                let label = if i < labels.len() { labels[i] } else { "?" };
                (label, (*val_a).max(0.0) as u64)
            })
            .collect();

        let max_value = module.config.bindings.get("max_value")
            .and_then(|v| v.as_f64()).unwrap_or(100.0);

        let bar_chart = BarChart::default()
            .block(
                Block::default()
                    .title(format!(" {}{} (Bar) ", module.config.name, status_suffix))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .data(&bar_data)
            .bar_width(3)
            .bar_gap(1)
            .max(max_value as u64)
            .bar_style(Style::default().fg(if is_connected { Color::Green } else { Color::DarkGray }))
            .value_style(Style::default().fg(Color::Black).bg(if is_connected { Color::Green } else { Color::DarkGray }));

        Widget::render(bar_chart, area, buf);
    }

    fn render_sparkline(&self, module: &Module, area: Rect, buf: &mut Buffer, is_selected: bool) {
        // Get history from handler (via bindings)
        let history = Self::get_history(module);

        // Get current values for display
        let a = module.config.bindings.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);

        let is_connected = module.config.bindings.get("is_connected")
            .and_then(|v| v.as_bool()).unwrap_or(true);

        let status_suffix = module.config.bindings.get("status_suffix")
            .and_then(|v| v.as_str()).unwrap_or("");

        let unit = module.config.bindings.get("unit_of_measure_label")
            .and_then(|v| v.as_str()).unwrap_or("units");

        let border_color = if !is_connected {
            Color::Red
        } else if is_selected {
            Color::White
        } else {
            Color::Yellow
        };

        // Convert to u64 for Sparkline
        let spark_data: Vec<u64> = history.iter()
            .map(|(val_a, _, _)| val_a.round() as u64)
            .collect();

        // Split area for sparkline and value
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),      // Sparkline
                Constraint::Length(2),   // Current value
            ])
            .split(area);

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .title(format!(" {}{} (Spark) ", module.config.name, status_suffix))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .data(&spark_data)
            .style(Style::default().fg(if is_connected { Color::Yellow } else { Color::DarkGray }))
            .max(module.config.bindings.get("max_value")
                .and_then(|v| v.as_f64())
                .unwrap_or(100.0) as u64);

        Widget::render(sparkline, chunks[0], buf);

        // Current value
        let value_text = format!("Current: {:.1} {} | Min: {:.1} | Max: {:.1}",
                                 a,
                                 unit,
                                 history.iter().map(|(val_a, _, _)| val_a).fold(f64::INFINITY, |a, &b| a.min(b)),
                                 history.iter().map(|(val_a, _, _)| val_a).fold(f64::NEG_INFINITY, |a, &b| a.max(b))
        );
        let value_widget = Paragraph::new(value_text)
            .style(Style::default().fg(if is_connected { Color::White } else { Color::Red }))
            .alignment(Alignment::Center);
        value_widget.render(chunks[1], buf);
    }
}

impl UiTemplate for ChartCard {
    fn render(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        // Get chart type from config
        let chart_type = module.config.bindings
            .get("chart_type")
            .and_then(|v| v.as_str())
            .unwrap_or("line");

        match chart_type {
            "bar" => self.render_bar_chart(module, area, buf, is_selected),
            "sparkline" | "spark" => self.render_sparkline(module, area, buf, is_selected),
            "line" | _ => self.render_line_chart(module, area, buf, is_selected),
        }
    }

    fn required_bindings(&self) -> &'static [&'static str] {
        &["a", "chart_type"]
    }

    fn docs(&self) -> &'static str {
        "Multi-type chart display using SSP format. Key 'a' = primary sensor value. \
         Set 'chart_type' to 'line', 'bar', or 'sparkline'. Maintains history of last 50 points. \
         Shows connection status and automatically updates with telemetry."
    }
}

impl Default for ChartCard {
    fn default() -> Self {
        Self
    }
}
