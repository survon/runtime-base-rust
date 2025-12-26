use super::ChatMessage;

impl ChatMessage {
    pub fn new_assistant(session_id: String, content: String, module_name: String) -> Self {
        Self {
            id: None,
            session_id,
            role: "assistant".to_string(),
            content,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            module_name,
        }
    }
}
