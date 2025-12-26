use crate::log_warn;

use super::LlmHandler;

impl LlmHandler {
    pub async fn submit_message(
        &mut self,
        module_name: String,
        knowledge_module_names: Vec<String>,
    ) -> color_eyre::Result<()> {
        let input = self.chat_manager.get_input().trim();
        if input.is_empty() {
            return Ok(());
        }

        let service = match &self.llm_service {
            Some(s) => s,
            None => {
                log_warn!("No LLM service available");
                return Ok(());
            }
        };

        let query = input.to_string();

        // Clear input immediately for better UX
        self.chat_manager.clear_input();
        self.chat_manager.available_links.clear();
        self.chat_manager.current_link_index = None;

        // Process the query
        let _response = service.process_query(
            &self.session_id,
            &module_name,
            &query,
            &knowledge_module_names,
        ).await?;

        self.chat_manager.update_available_links(service, &self.session_id);

        Ok(())
    }
}
