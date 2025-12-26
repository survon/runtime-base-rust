mod backspace;
mod trait_default;
mod handle_input;
mod new;
mod clear_input;
mod get_input;
mod scroll_up;
mod scroll_down;
mod calculate_max_scroll;
mod cycle_links;
mod get_current_link;
mod update_available_links;

/// Manages chat UI state
#[derive(Debug)]
pub struct ChatManager {
    pub chat_input: String,
    pub chat_scroll_offset: usize,
    pub current_link_index: Option<usize>,
    pub available_links: Vec<String>,
}
