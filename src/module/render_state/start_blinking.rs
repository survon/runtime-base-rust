use std::time::Instant;

use super::ModuleRenderState;

impl ModuleRenderState {
    pub fn start_blinking(&mut self) {
        if !self.is_actively_blinking {
            self.is_actively_blinking = true;
            self.last_blink = Instant::now();
        }
    }
}
