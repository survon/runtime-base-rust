// src/ui/module_templates/monitoring/gauge_card.rs
use crate::modules::Module;
use crate::ui::template::UiTemplate;
use ratatui::prelude::*;
use ratatui::buffer::Buffer;
use ratatui::widgets::{Block, Borders, Gauge, Widget};

#[derive(Debug)]
pub struct GaugeCard;

impl UiTemplate for GaugeCard {
    fn render(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        // Get the pressure value from module bindings
        let pressure = module
            .config
            .bindings
            .get("pressure_psi")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Calculate percentage (assuming max 100 PSI)
        let max_pressure = 100.0;
        let percentage = ((pressure / max_pressure) * 100.0).min(100.0) as u16;

        let healthy_color_fg = Color::Green;
        let healthy_color_bg = Color::LightGreen;
        let warn_color_fg = Color::Yellow;
        let warn_color_bg = Color::LightYellow;
        let danger_color_fg = Color::Red;
        let danger_color_bg = Color::LightRed;
        let danger_color_fg_blink = Color::Black;

        // TODO USE A USER-SET THRESHOLDS instead of 85.5/60 magic constants.. maybe from module.config?
        let in_danger_zone = pressure > 85.0;
        let in_warn_zone = pressure > 60.0;

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
            .label(format!("{:.1} PSI", pressure));

        Widget::render(gauge, area, buf);
    }

    fn required_bindings(&self) -> &'static [&'static str] {
        &["pressure_psi"]
    }

    fn docs(&self) -> &'static str {
        "Real-time pressure gauge (0-100 PSI). Shows green/yellow/red based on thresholds. Green: 0-60 PSI, Yellow: 60-85 PSI, Red: 85+ PSI. Blinks when above 85 PSI if is_blinkable is true."
    }
}

impl Default for GaugeCard {
    fn default() -> Self {
        Self
    }
}
