use super::ChatManager;

impl ChatManager {
    pub fn scroll_down(&mut self) {
        self.chat_scroll_offset += 1;
    }
}
