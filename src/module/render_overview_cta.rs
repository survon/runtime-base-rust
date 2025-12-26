use std::time::{Duration, Instant};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
};

use crate::module::Module;

impl Module {
    pub fn render_overview_cta(&mut self, is_selected: bool, area: Rect, buf: &mut Buffer) -> std::result::Result<(), String> {
        self.get_template()?;

        if self.config.is_blinkable() {
            // Get blink interval from bindings (default 500ms)
            let blink_interval_ms = self.config.bindings
                .get("blink_interval_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(500);

            let blink_interval = Duration::from_millis(blink_interval_ms);

            if self.render_state.last_blink.elapsed() >= blink_interval {
                self.render_state.blink_state = !self.render_state.blink_state;
                self.render_state.last_blink = Instant::now();
            }
        }

        let mut template = self.cached_template.take()
            .ok_or_else(|| "Template not loaded".to_string())?;

        template.render_overview_cta(is_selected, area, buf, self);

        self.cached_template = Some(template);

        Ok(())
    }
}
