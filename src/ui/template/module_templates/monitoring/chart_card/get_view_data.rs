use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Color,
};

use crate::module::Module;

use super::{ChartCard, ViewData};

impl ChartCard {
    pub(super) fn get_view_data<'a>(
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
}
