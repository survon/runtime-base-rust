// src/modules/llm/handler.rs

use std::any::Any;
use color_eyre::Result;
use ratatui::crossterm::event::KeyCode;
use crate::util::{
    io::event::AppEvent,
    llm::LlmService,
    database
};
use crate::modules::{
    Module,
    module_handler::ModuleHandler,
    llm::database::ChatMessage
};

/// Manages chat UI state
#[derive(Debug)]
pub struct ChatManager {
    pub chat_input: String,
    pub chat_scroll_offset: usize,
    pub current_link_index: Option<usize>,
    pub available_links: Vec<String>,
}

impl ChatManager {
    pub fn new() -> Self {
        Self {
            chat_input: String::new(),
            chat_scroll_offset: 0,
            current_link_index: None,
            available_links: Vec::new(),
        }
    }

    pub fn handle_input(&mut self, ch: char) {
        self.chat_input.push(ch);
    }

    pub fn backspace(&mut self) {
        self.chat_input.pop();
    }

    pub fn clear_input(&mut self) {
        self.chat_input.clear();
    }

    pub fn get_input(&self) -> &str {
        &self.chat_input
    }

    pub fn scroll_up(&mut self) {
        if self.chat_scroll_offset > 0 {
            self.chat_scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.chat_scroll_offset += 1;
    }

    pub fn calculate_max_scroll(&self, messages: &[String], visible_height: usize) -> usize {
        let mut total_lines = 0;

        for msg in messages {
            let parts: Vec<&str> = msg.splitn(2, ':').collect();
            if parts.len() == 2 {
                let content = parts[1];
                // Count lines in content (each line + spacing)
                total_lines += content.lines().count() + 2; // +2 for role line and spacing
            }
        }

        // max_scroll is total_lines minus visible area
        total_lines.saturating_sub(visible_height)
    }

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

    pub fn get_current_link(&self) -> Option<&String> {
        self.current_link_index
            .and_then(|idx| self.available_links.get(idx))
    }

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

impl Default for ChatManager {
    fn default() -> Self {
        Self::new()
    }
}

/// LLM Handler that integrates with the LLM utility service
#[derive(Debug)]
pub struct LlmHandler {
    chat_manager: ChatManager,
    llm_service: Option<LlmService>,
    session_id: String,
}

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

    pub fn get_manager(&self) -> &ChatManager {
        &self.chat_manager
    }

    pub fn get_manager_mut(&mut self) -> &mut ChatManager {
        &mut self.chat_manager
    }

    pub fn get_service(&self) -> Option<&LlmService> {
        self.llm_service.as_ref()
    }

    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }

    /// Submit a chat message
    pub async fn submit_message(
        &mut self,
        module_name: String,
        knowledge_module_names: Vec<String>,
    ) -> Result<()> {
        let input = self.chat_manager.get_input().trim();
        if input.is_empty() {
            return Ok(());
        }

        let service = match &self.llm_service {
            Some(s) => s,
            None => {
                println!("No LLM service available");
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

        // Update available links from the response
        self.chat_manager.update_available_links(service, &self.session_id);

        Ok(())
    }

    /// Format chat history for display
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

impl ModuleHandler for LlmHandler {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
        match key_code {
            KeyCode::Esc => Some(AppEvent::Back),
            KeyCode::Tab => {
                self.chat_manager.cycle_links(1);
                None
            },
            KeyCode::BackTab => {
                self.chat_manager.cycle_links(-1);
                None
            },
            KeyCode::Enter => {
                if let Some(file_path) = self.chat_manager.get_current_link() {
                    Some(AppEvent::OpenDocument(file_path.clone()))
                } else {
                    Some(AppEvent::ChatSubmit)
                }
            },
            KeyCode::Backspace => {
                self.chat_manager.backspace();
                None
            },
            KeyCode::Char(ch) => {
                self.chat_manager.handle_input(ch);
                None
            },
            KeyCode::PageUp | KeyCode::Up => {
                self.chat_manager.scroll_up();
                None
            },
            KeyCode::PageDown | KeyCode::Down => {
                self.chat_manager.scroll_down();
                None
            },
            _ => None,
        }
    }

    fn handle_event(&mut self, _event: &AppEvent, _module: &mut Module) -> Result<bool> {
        Ok(false)
    }

    fn update_bindings(&mut self, module: &mut Module) {
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

    fn module_type(&self) -> &str {
        "llm"
    }
}
