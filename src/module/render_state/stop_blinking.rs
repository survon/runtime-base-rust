use super::ModuleRenderState;

impl ModuleRenderState {
    pub fn stop_blinking(&mut self) {
        if self.is_actively_blinking {
            self.is_actively_blinking = false;
            self.blink_state = false; // Reset to normal state
        }
    }
}
