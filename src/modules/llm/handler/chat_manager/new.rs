use super::ChatManager;

impl ChatManager {
    pub fn new() -> Self {
        Self {
            chat_input: String::new(),
            chat_scroll_offset: 0,
            current_link_index: None,
            available_links: Vec::new(),
        }
    }
}
