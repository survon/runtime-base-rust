use super::LlmHandler;

impl LlmHandler {
    pub fn format_chat_history(&self) -> Vec<String> {
        if let Some(service) = &self.llm_service {
            if let Ok(messages) = service.get_chat_history(&self.session_id, 50) {
                return messages
                    .iter()
                    .map(|msg| format!("{}:{}", msg.role, msg.content))
                    .collect();
            }
        }
        vec!["No chat history available".to_string()]
    }
}
