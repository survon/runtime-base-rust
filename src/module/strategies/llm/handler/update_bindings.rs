use crate::module::Module;
use super::LlmHandler;

impl LlmHandler {
    pub(in crate::module) fn _update_bindings(&mut self, module: &mut Module) {
        // Update model info
        let model_info = self.llm_service
            .as_ref()
            .map(|s| s.get_model_info())
            .unwrap_or_else(|| "No model loaded".to_string());

        module.config.bindings.insert(
            "model_info".to_string(),
            serde_json::Value::String(model_info),
        );

        // Update chat history
        let chat_history = self.format_chat_history();
        module.config.bindings.insert(
            "chat_history".to_string(),
            serde_json::Value::Array(
                chat_history.iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect()
            ),
        );

        // Update input
        module.config.bindings.insert(
            "chat_input".to_string(),
            serde_json::Value::String(self.chat_manager.get_input().to_string()),
        );

        // Update scroll offset
        module.config.bindings.insert(
            "scroll_offset".to_string(),
            serde_json::Value::Number(self.chat_manager.chat_scroll_offset.into()),
        );

        // Update current link index for highlighting
        module.config.bindings.insert(
            "current_link_index".to_string(),
            match self.chat_manager.current_link_index {
                Some(idx) => serde_json::Value::Number(idx.into()),
                None => serde_json::Value::Null,
            },
        );
    }
}
