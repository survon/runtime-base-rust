use ratatui::{
    prelude::*,
    buffer::Buffer,
    widgets::{Block, Borders, Widget, BarChart, Sparkline, Paragraph, Padding},
    layout::{Constraint, Direction, Layout, Alignment},
};

use crate::{
    modules::Module,
    ui::template::UiTemplate,
};

#[derive(Debug)]
pub struct ChartCard;

struct ViewData<'a> {
    module_name: &'a str,
    chart_type: &'a str,
    history: Vec<(f64, f64, i64)>, // this is field unit telemetry data { a, b, c }
    a: f64, // telemetry data float
    b: f64, // telemetry data float
    c: i64, // telemetry data int
    is_connected: bool,
    connected_icon: &'a str,
    status_suffix: &'a str,
    unit: &'a str, //  [of measure]
    border_color: Color,
    chart_title: &'a str,
    min_value: f64,
    max_value: f64,
}

/// I'm leaving this as a reminder that comments like this are markdown friendly by the way!
/// # ChartCard
/// Check out [`Module`] and [`UiTemplate`] - fuckin rad..
///
/// TODO have an LLM add these comment blocks to all my structs and shit.
///
impl ChartCard {
    fn get_view_data<'a>(
        &self,
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        module: &'a mut Module
    ) -> ViewData<'a> {
        let module_name = &module.config.name;

        // Get chart type from config
        let chart_type = module.config.bindings
            .get("chart_type")
            .and_then(|v| v.as_str())
            .unwrap_or("line");

        // Get history from handler (via bindings)
        let history = Self::get_history(module);

        // Get current values for display
        let a = module.config.bindings.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let b = module.config.bindings.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let c = module.config.bindings.get("c").and_then(|v| v.as_i64()).unwrap_or(0);

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

        let chart_title = &module.config.name;

        let max_value = module.config.bindings.get("max_value")
            .and_then(|v| v.as_f64()).unwrap_or(100.0);

        let min_value = 0.0;

        let connected_icon = if is_connected { "🔗" } else { "⛓️‍💥" };

        ViewData {
            module_name,
            chart_type,
            history,
            a,
            b,
            c,
            is_connected,
            connected_icon,
            status_suffix,
            unit,
            border_color,
            chart_title,
            max_value,
            min_value,
        }
    }

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

    fn render_line_chart(
        &self,
        module: &mut Module,
        area: Rect,
        buf: &mut Buffer,
        is_selected: bool,
        is_contained: bool,
    ) {
        let ViewData {
            history,
            a,
            is_connected,
            connected_icon,
            status_suffix,
            unit,
            border_color,
            chart_title,
            min_value,
            max_value,
            ..
        } = self.get_view_data(is_selected, area, buf, module);

        // Split area for chart and status
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),      // Chart
                Constraint::Length(2),   // Current value display
            ])
            .split(area);

        // Render block
        let container: Block = if is_contained {
            Block::default()
                .borders(Borders::NONE)
                .padding(Padding::symmetric(1, 1))
        } else {
            Block::default()
                .title(format!(" {}{}{} ", connected_icon, chart_title, status_suffix))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
        };

        let inner_area = container.inner(chunks[0]);

        container.render(chunks[0], buf);

        // Calculate how many points fit in the width
        let max_visible = inner_area.width.saturating_sub(2) as usize;

        // Get sliding window of most recent data that fits
        let visible_data: Vec<(f64, f64)> = history.iter()
            .rev()
            .take(max_visible.max(1))
            .rev()
            .enumerate()
            .map(|(i, (val_a, _, _))| (i as f64, *val_a))
            .collect();

        // Draw simple ASCII line chart
        if !visible_data.is_empty() && inner_area.height > 2 {
            let height = inner_area.height as f64;
            let width = inner_area.width as f64;
            let data_points = visible_data.len() as f64;

            for (i, (_, val)) in visible_data.iter().enumerate() {
                let x = inner_area.x + ((i as f64 / data_points.max(1.0)) * width) as u16;
                let normalized = ((val - min_value) / (max_value - min_value)).clamp(0.0, 1.0);
                let y = inner_area.y + inner_area.height - 1 - ((normalized * (height - 1.0)) as u16);

                if x < inner_area.x + inner_area.width && y >= inner_area.y {
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
        let value_text = format!("Cur: {:.1} {} (Last {} points)", a, unit, history.len());
        let value_widget = Paragraph::new(value_text)
            .style(Style::default().fg(if is_connected { Color::White } else { Color::Red }))
            .alignment(Alignment::Center);
        value_widget.render(chunks[1], buf);
    }

    fn render_bar_chart(
        &self,
        module: &mut Module,
        area: Rect,
        buf: &mut Buffer,
        is_selected: bool,
        is_contained: bool,
    ) {
        let ViewData {
            module_name,
            history,
            is_connected,
            status_suffix,
            border_color,
            max_value,
            ..
        } = self.get_view_data(is_selected, area, buf, module);

        // Calculate how many bars can fit (bar_width=3 + bar_gap=1 = 4 chars per bar)
        let inner_width = area.width.saturating_sub(2) as usize; // account for borders
        let max_bars = (inner_width / 4).max(1).min(history.len());

        // Take last N data points that fit
        let recent_data: Vec<(f64, f64, i64)> = history.iter()
            .rev()
            .take(max_bars)
            .rev()
            .copied()
            .collect();

        // Create bar data as tuples (&str, u64) - required by ratatui 0.29+
        let bar_data: Vec<(&str, u64)> = recent_data.iter()
            .enumerate()
            .map(|(i, (val_a, _, _))| {
                // We need static strings, so use a fixed set (expanded to 30)
                let labels = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10",
                    "11", "12", "13", "14", "15", "16", "17", "18", "19", "20",
                    "21", "22", "23", "24", "25", "26", "27", "28", "29", "30"];
                let label = if i < labels.len() { labels[i] } else { "•" };
                (label, (*val_a).max(0.0) as u64)
            })
            .collect();

        let connected_icon = if is_connected { "🔗" } else { "⛓️‍💥" };

        let container = if is_contained {
            Block::default()
                .borders(Borders::NONE)
                .padding(Padding::symmetric(1, 1))
        } else {
            Block::default()
                .title(format!(" {}{}{} (Showing {} of {}) ",
                               connected_icon, module_name, status_suffix,
                               recent_data.len(), history.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
        };

        let bar_chart = BarChart::default()
            .block(container)
            .data(&bar_data)
            .bar_width(3)
            .bar_gap(1)
            .max(max_value as u64)
            .bar_style(Style::default().fg(if is_connected { Color::Green } else { Color::DarkGray }))
            .value_style(Style::default().fg(Color::Black).bg(if is_connected { Color::Green } else { Color::DarkGray }));

        Widget::render(bar_chart, area, buf);
    }

    fn render_sparkline(
        &self,
        module: &mut Module,
        area: Rect,
        buf: &mut Buffer,
        is_selected: bool,
        is_contained: bool,
    ) {
        let ViewData {
            module_name,
            history,
            a,
            is_connected,
            connected_icon,
            status_suffix,
            unit,
            border_color,
            max_value,
            ..
        } = self.get_view_data(is_selected, area, buf, module);

        // Split area for sparkline and value
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),      // Sparkline
                Constraint::Length(2),   // Current value
            ])
            .split(area);

        // Calculate how many points fit in sparkline width
        let sparkline_area = Block::default()
            .borders(Borders::ALL)
            .inner(chunks[0]);
        let max_visible = sparkline_area.width.saturating_sub(1) as usize;

        // Get sliding window of most recent data
        let spark_data: Vec<u64> = history.iter()
            .rev()
            .take(max_visible.max(1))
            .rev()
            .map(|(val_a, _, _)| val_a.round() as u64)
            .collect();

        let container = if is_contained {
            Block::default()
                .borders(Borders::NONE)
                .padding(Padding::symmetric(1, 1))
        } else {
            Block::default()
                .title(format!(" {}{}{} ", connected_icon, module_name, status_suffix))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color))
        };

        let sparkline = Sparkline::default()
            .block(container)
            .data(&spark_data)
            .style(Style::default().fg(if is_connected { Color::Yellow } else { Color::DarkGray }))
            .max(max_value as u64);

        Widget::render(sparkline, chunks[0], buf);

        // Current value
        let min_val = history.iter().map(|(val_a, _, _)| val_a).fold(f64::INFINITY, |a, &b| a.min(b));
        let max_val = history.iter().map(|(val_a, _, _)| val_a).fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let value_text = if history.is_empty() {
            format!("Cur: {:.1} {}", a, unit)
        } else {
            format!("Cur: {:.1} {} | Min: {:.1} | Max: {:.1}", a, unit, min_val, max_val)
        };

        let value_widget = Paragraph::new(value_text)
            .style(Style::default().fg(if is_connected { Color::White } else { Color::Red }))
            .alignment(Alignment::Center);
        value_widget.render(chunks[1], buf);
    }
}

impl UiTemplate for ChartCard {
    fn render_overview_cta(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        let is_contained = false;

        let ViewData { chart_type, .. } = self.get_view_data(is_selected, area, buf, module);

        match chart_type {
            "bar" => self.render_bar_chart(module, area, buf, is_selected, is_contained),
            "sparkline" | "spark" => self.render_sparkline(module, area, buf, is_selected, is_contained),
            "line" | _ => self.render_line_chart(module, area, buf, is_selected, is_contained),
        }
    }

    fn render_detail(&self, area: Rect, buf: &mut Buffer, module: &mut Module) {
        let is_selected = false;
        let is_contained = true;

        let ViewData { chart_type, .. } = self.get_view_data(is_selected, area, buf, module);

        match chart_type {
            "bar" => self.render_bar_chart(module, area, buf, is_selected, is_contained),
            "sparkline" | "spark" => self.render_sparkline(module, area, buf, is_selected, is_contained),
            "line" | _ => self.render_line_chart(module, area, buf, is_selected, is_contained),
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
