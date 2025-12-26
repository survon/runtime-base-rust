use super::ChatManager;

impl ChatManager {
    pub fn scroll_up(&mut self) {
        if self.chat_scroll_offset > 0 {
            self.chat_scroll_offset -= 1;
        }
    }
}
