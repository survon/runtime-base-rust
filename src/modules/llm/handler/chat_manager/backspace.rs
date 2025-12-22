use super::ChatManager;

impl ChatManager {
    pub fn backspace(&mut self) {
        self.chat_input.pop();
    }
}
