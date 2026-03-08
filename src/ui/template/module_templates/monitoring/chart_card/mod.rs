mod get_history;
mod get_view_data;
mod render_bar_chart;
mod render_detail;
mod render_line_chart;
mod render_overview_cta;
mod render_sparkline;
mod trait_default;
mod trait_ui_template;

use ratatui::{prelude::*, widgets::Widget};

use crate::ui::template::UiTemplate;

#[derive(Debug)]
pub struct ChartCard;

struct ViewData<'a> {
    module_name: &'a str,
    chart_type: &'a str,
    history: Vec<(f64, f64, i64)>, // this is field unit telemetry data { a, b, c }
    a: f64,                        // telemetry data float
    b: f64,                        // telemetry data float
    c: i64,                        // telemetry data int
    is_connected: bool,
    connected_icon: &'a str,
    status_suffix: &'a str,
    unit: &'a str, //  [of measure]
    border_color: Color,
    chart_title: &'a str,
    min_value: f64,
    max_value: f64,
}
