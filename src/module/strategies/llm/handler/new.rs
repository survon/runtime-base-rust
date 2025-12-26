use crate::module::strategies::llm::{
    handler::{
        LlmHandler,
        chat_manager::ChatManager
    },

};
use crate::util::llm::LlmService;

impl LlmHandler {
    pub fn new(llm_service: Option<LlmService>) -> Self {
        let session_id = format!(
            "session_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );
        Self {
            chat_manager: ChatManager::new(),
            llm_service,
            session_id,
        }
    }
}
