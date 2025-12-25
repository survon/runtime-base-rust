use ratatui::{
    buffer::Buffer,
    layout::Rect,
};

use crate::modules::Module;

use super::{ChartCard, ViewData};

impl ChartCard {
    pub(super) fn _render_overview_cta(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        let is_contained = false;

        let ViewData { chart_type, .. } = self.get_view_data(is_selected, area, buf, module);

        match chart_type {
            "bar" => self.render_bar_chart(module, area, buf, is_selected, is_contained),
            "sparkline" | "spark" => self.render_sparkline(module, area, buf, is_selected, is_contained),
            "line" | _ => self.render_line_chart(module, area, buf, is_selected, is_contained),
        }
    }
}
