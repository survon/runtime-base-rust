use super::ChatManager;

impl ChatManager {
    pub fn cycle_links(&mut self, direction: i32) {
        if self.available_links.is_empty() {
            return;
        }

        match self.current_link_index {
            None => self.current_link_index = Some(0),
            Some(index) => {
                let len = self.available_links.len() as i32;
                let new_index = if direction > 0 {
                    (index as i32 + 1) % len
                } else {
                    (index as i32 - 1 + len) % len
                };
                self.current_link_index = Some(new_index as usize);
            }
        }
    }
}
