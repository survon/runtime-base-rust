// src/ui/modules/monitoring/history_chart.rs
use crate::module::Module;
use crate::ui::template::UiTemplate;
use ratatui::prelude::*;
use ratatui::buffer::Buffer;
use ratatui::widgets::{Block, Borders, Axis, Chart, Dataset, GraphType, Widget};
use ratatui::symbols;

#[derive(Debug)]
pub struct HistoryChart;

impl UiTemplate for HistoryChart {
    fn render(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        // Get the history data from module bindings
        let history = module
            .config
            .bindings
            .get("history")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_f64())
                    .collect::<Vec<f64>>()
            })
            .unwrap_or_else(Vec::new);

        // Get optional configuration
        let chart_title = module
            .config
            .bindings
            .get("chart_title")
            .and_then(|v| v.as_str())
            .unwrap_or("History");

        let y_label = module
            .config
            .bindings
            .get("y_label")
            .and_then(|v| v.as_str())
            .unwrap_or("Value");

        let x_label = module
            .config
            .bindings
            .get("x_label")
            .and_then(|v| v.as_str())
            .unwrap_or("Time");

        let line_color = module
            .config
            .bindings
            .get("line_color")
            .and_then(|v| v.as_str())
            .and_then(|s| match s.to_lowercase().as_str() {
                "red" => Some(Color::Red),
                "green" => Some(Color::Green),
                "blue" => Some(Color::Blue),
                "yellow" => Some(Color::Yellow),
                "cyan" => Some(Color::Cyan),
                "magenta" => Some(Color::Magenta),
                "white" => Some(Color::White),
                _ => None,
            })
            .unwrap_or(Color::Cyan);

        // If no data, show empty chart with message
        if history.is_empty() {
            let empty_block = Block::default()
                .title(format!(" {} - No Data ", module.config.name))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray));
            Widget::render(empty_block, area, buf);
            return;
        }

        // Convert data to chart points (x, y)
        let data_points: Vec<(f64, f64)> = history
            .iter()
            .enumerate()
            .map(|(i, &value)| (i as f64, value))
            .collect();

        // Calculate bounds
        let max_x = (history.len() - 1).max(1) as f64;
        let min_y = history.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_y = history.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Add some padding to y-axis
        let y_range = (max_y - min_y).max(1.0);
        let y_padding = y_range * 0.1;
        let y_min = (min_y - y_padding).floor();
        let y_max = (max_y + y_padding).ceil();

        // Create dataset
        let datasets = vec![
            Dataset::default()
                .name(chart_title)
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(line_color))
                .data(&data_points)
        ];

        // Create x-axis with labels
        let x_labels = vec![
            format!("{:.0}", 0.0),
            format!("{:.0}", max_x / 2.0),
            format!("{:.0}", max_x),
        ];
        let x_label_refs: Vec<&str> = x_labels.iter().map(|s| s.as_str()).collect();

        let x_axis = Axis::default()
            .title(x_label)
            .style(Style::default().fg(Color::Gray))
            .labels(x_label_refs)
            .bounds([0.0, max_x]);

        // Create y-axis with labels
        let y_labels = vec![
            format!("{:.1}", y_min),
            format!("{:.1}", (y_min + y_max) / 2.0),
            format!("{:.1}", y_max),
        ];
        let y_label_refs: Vec<&str> = y_labels.iter().map(|s| s.as_str()).collect();

        let y_axis = Axis::default()
            .title(y_label)
            .style(Style::default().fg(Color::Gray))
            .labels(y_label_refs)
            .bounds([y_min, y_max]);

        let border_color = if is_selected { Color::White } else { Color::Green };

        // Create the chart
        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(format!(" {} ", module.config.name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .x_axis(x_axis)
            .y_axis(y_axis);

        Widget::render(chart, area, buf);
    }

    fn required_bindings(&self) -> &'static [&'static str] {
        &["history"]
    }

    fn docs(&self) -> &'static str {
        "Line chart showing historical data over time. Required: 'history' (array of numbers). Optional: 'chart_title' (string), 'y_label' (string), 'x_label' (string), 'line_color' (red/green/blue/yellow/cyan/magenta/white)."
    }
}

impl Default for HistoryChart {
    fn default() -> Self {
        Self
    }
}
