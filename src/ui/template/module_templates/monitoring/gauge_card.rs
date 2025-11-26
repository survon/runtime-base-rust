// src/ui/module_templates/monitoring/gauge_card.rs
use crate::modules::Module;
use crate::ui::template::UiTemplate;
use ratatui::prelude::*;
use ratatui::buffer::Buffer;
use ratatui::widgets::{Block, Borders, Gauge, Widget};

#[derive(Debug)]
pub struct GaugeCard;

// Compact SSP uses single-letter keys
// "a" maps to primary sensor value (defined in config.yml)
const GAUGE_VALUE_KEY: &str = "a";

impl UiTemplate for GaugeCard {
    fn render(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        // Get the primary sensor value from module bindings
        let gauge_value = module
            .config
            .bindings
            .get(GAUGE_VALUE_KEY)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Get display label from config
        let unit_of_measure_label = module
            .config
            .bindings
            .get("unit_of_measure_label")
            .and_then(|v| v.as_str())
            .unwrap_or("units");

        // Calculate percentage (assuming max 100)
        let max_value = module
            .config
            .bindings
            .get("max_value")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0);

        let percentage = ((gauge_value / max_value) * 100.0).min(100.0) as u16;

        // Color thresholds (can be configured in config.yml)
        let warn_threshold = module
            .config
            .bindings
            .get("warn_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(60.0);

        let danger_threshold = module
            .config
            .bindings
            .get("danger_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(85.0);

        let healthy_color_fg = Color::Green;
        let healthy_color_bg = Color::LightGreen;
        let warn_color_fg = Color::Yellow;
        let warn_color_bg = Color::LightYellow;
        let danger_color_fg = Color::Red;
        let danger_color_bg = Color::LightRed;
        let danger_color_fg_blink = Color::Black;

        let in_danger_zone = gauge_value > danger_threshold;
        let in_warn_zone = gauge_value > warn_threshold;

        if in_danger_zone && module.config.is_blinkable() {
            module.render_state.start_blinking();
        } else {
            module.render_state.stop_blinking();
        }

        let gauge_color_fg = if in_danger_zone {
            if module.render_state.is_actively_blinking {
                if module.render_state.blink_state {
                    danger_color_fg_blink
                } else {
                    danger_color_fg
                }
            } else {
                danger_color_fg
            }
        } else if in_warn_zone {
            warn_color_fg
        } else {
            healthy_color_fg
        };

        let gauge_color_bg = if in_danger_zone {
            danger_color_bg
        } else if in_warn_zone {
            warn_color_bg
        } else {
            healthy_color_bg
        };

        let border_color = if is_selected { Color::White } else { gauge_color_bg };

        // Create gauge widget
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .title(format!(" {} ", module.config.name))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
            )
            .gauge_style(Style::default().fg(gauge_color_fg).bg(gauge_color_bg))
            .percent(percentage)
            .label(format!("{:.1} {}", gauge_value, unit_of_measure_label));

        Widget::render(gauge, area, buf);
    }

    fn required_bindings(&self) -> &'static [&'static str] {
        &["a"]  // Only "a" is required, others are optional
    }

    fn docs(&self) -> &'static str {
        "Real-time gauge using compact SSP format. Key 'a' = primary sensor value. \
         Displays green (0-60), yellow (60-85), red (85+). \
         Configure thresholds via warn_threshold/danger_threshold bindings."
    }
}

impl Default for GaugeCard {
    fn default() -> Self {
        Self
    }
}
