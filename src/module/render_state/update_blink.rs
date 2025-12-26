use std::time::{Duration, Instant};

use super::ModuleRenderState;

impl ModuleRenderState {
    pub fn update_blink(&mut self, interval: Duration) -> bool {
        if self.last_blink.elapsed() >= interval {
            self.blink_state = !self.blink_state;
            self.last_blink = Instant::now();
            true
        } else {
            false
        }
    }
}
