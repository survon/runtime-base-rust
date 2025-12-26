use ratatui::{
    buffer::Buffer,
    layout::Rect,
};

use crate::{
    module::Module,
    ui::template::UiTemplate,
};

use super::ChartCard;

impl UiTemplate for ChartCard {
    fn render_overview_cta(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        self._render_overview_cta(is_selected, area, buf, module)
    }

    fn render_detail(&self, area: Rect, buf: &mut Buffer, module: &mut Module) {
        self._render_detail(area, buf, module)
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
