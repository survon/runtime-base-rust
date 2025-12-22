use super::ChatManager;

impl ChatManager {
    pub fn get_input(&self) -> &str {
        &self.chat_input
    }
}
