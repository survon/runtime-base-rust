use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{Color, Style, Widget},
    widgets::{Block, Borders, Padding, Paragraph},
};

use crate::module::Module;

use super::{ChartCard, ViewData};

impl ChartCard {
    pub(super) fn render_line_chart(
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
}
