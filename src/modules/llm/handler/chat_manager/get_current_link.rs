use super::ChatManager;

impl ChatManager {
    pub fn get_current_link(&self) -> Option<&String> {
        self.current_link_index
            .and_then(|idx| self.available_links.get(idx))
    }
}
