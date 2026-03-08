mod start_blinking;
mod stop_blinking;
mod trait_default;
mod update_blink;

use std::time::Instant;

/// Runtime rendering state for modules (not serialized)
#[derive(Debug, Clone)]
pub struct ModuleRenderState {
    pub blink_state: bool,
    pub last_blink: Instant,
    pub animation_frame: usize,
    pub is_focused: bool,
    pub is_actively_blinking: bool,
}
