use std::time::Instant;

use super::ModuleRenderState;

impl Default for ModuleRenderState {
    fn default() -> Self {
        Self {
            blink_state: false,
            last_blink: Instant::now(),
            animation_frame: 0,
            is_focused: false,
            is_actively_blinking: false,
        }
    }
}
