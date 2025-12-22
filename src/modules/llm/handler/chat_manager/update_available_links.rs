use crate::util::llm::LlmService;

use super::ChatManager;

impl ChatManager {

    pub fn update_available_links(&mut self, llm_service: &LlmService, session_id: &str) {
        self.available_links.clear();

        if let Ok(messages) = llm_service.get_chat_history(session_id, 50) {
            for msg in messages {
                if msg.role == "assistant" {
                    for line in msg.content.lines() {
                        if line.contains("(from ./") || line.contains("(from ") {
                            if let Some(start) = line.find("(from ") {
                                if let Some(end) = line[start..].find(')') {
                                    let path = &line[start + 6..start + end];
                                    self.available_links.push(path.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        self.current_link_index = None;
    }
}
