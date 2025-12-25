use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{Color, Style, Widget},
    widgets::{Block, Borders, Padding, Paragraph, Sparkline},

};

use crate::modules::Module;

use super::{ChartCard, ViewData};

impl ChartCard {
    pub(super) fn render_sparkline(
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
