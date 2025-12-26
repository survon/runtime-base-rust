use super::ChatManager;

impl ChatManager {
    pub fn handle_input(&mut self, ch: char) {
        self.chat_input.push(ch);
    }
}
