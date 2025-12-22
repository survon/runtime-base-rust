use super::ChatManager;

impl ChatManager {
    pub fn calculate_max_scroll(&self, messages: &[String], visible_height: usize) -> usize {
        let mut total_lines = 0;

        for msg in messages {
            let parts: Vec<&str> = msg.splitn(2, ':').collect();
            if parts.len() == 2 {
                let content = parts[1];
                total_lines += content.lines().count() + 2; // +2 for line and spacing
            }
        }

        // max_scroll is total_lines minus visible area
        total_lines.saturating_sub(visible_height)
    }
}
