use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::{Color, Style, Widget},
    widgets::{BarChart, Block, Borders, Padding},
};

use crate::module::Module;

use super::{ChartCard, ViewData};

impl ChartCard {
    pub(super) fn render_bar_chart(
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
}
