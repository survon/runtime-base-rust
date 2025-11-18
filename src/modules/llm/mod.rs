// src/modules/llm/mod.rs
//! LLM Module - Complete implementation for LLM core modules

pub mod engine;
pub mod handler;

use color_eyre::Result;
use crate::database::{Database, ChatMessage};
use crate::bus::BusMessage;
use crate::modules::Module;
pub use engine::{LlmEngine, LlmContext, create_llm_engine_if_available, create_llm_strategy};

/// Manages chat UI state and interactions
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

    pub fn handle_chat_input(&mut self, ch: char) {
        self.chat_input.push(ch);
    }

    pub fn chat_backspace(&mut self) {
        self.chat_input.pop();
    }

    pub fn clear_input(&mut self) {
        self.chat_input.clear();
    }

    pub fn get_input(&self) -> &str {
        &self.chat_input
    }

    pub fn cycle_document_links(&mut self) {
        self.cycle_document_links_direction(1);
    }

    pub fn cycle_document_links_backward(&mut self) {
        self.cycle_document_links_direction(-1);
    }

    fn cycle_document_links_direction(&mut self, direction: i32) {
        if !self.available_links.is_empty() {
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

    pub fn clear_document_links(&mut self) {
        self.current_link_index = None;
        self.available_links.clear();
    }

    pub fn set_available_links(&mut self, links: Vec<String>) {
        self.available_links = links;
        self.current_link_index = None;
    }

    pub fn update_available_links(&mut self, llm_engine: &LlmEngine) {
        let mut links = Vec::new();

        if let Ok(messages) = llm_engine.get_chat_history(50) {
            for msg in messages {
                if msg.role == "assistant" {
                    for line in msg.content.lines() {
                        if line.contains("(from ./") {
                            if let Some(start) = line.find("(from ") {
                                if let Some(end) = line[start..].find(')') {
                                    let full_path = &line[start + 6..start + end];
                                    links.push(full_path.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        self.set_available_links(links);
    }

    pub fn scroll_chat_up(&mut self) {
        if self.chat_scroll_offset > 0 {
            self.chat_scroll_offset -= 1;
        }
    }

    pub fn scroll_chat_down(&mut self, llm_engine: &LlmEngine) {
        if let Ok(messages) = llm_engine.get_chat_history(50) {
            let total_lines = self.calculate_chat_content_lines(&messages);
            let visible_lines = self.get_chat_visible_lines();
            let max_scroll = total_lines.saturating_sub(visible_lines);

            if self.chat_scroll_offset < max_scroll {
                self.chat_scroll_offset += 1;
            }
        }
    }

    pub fn get_chat_scroll_offset(&self) -> usize {
        self.chat_scroll_offset
    }

    fn calculate_chat_content_lines(&self, messages: &[ChatMessage]) -> usize {
        let mut line_count = 0;
        for msg in messages {
            let content_lines: Vec<&str> = msg.content.lines().collect();
            line_count += content_lines.len() + 1; // +1 for spacing
        }
        line_count
    }

    fn get_chat_visible_lines(&self) -> usize {
        // This should match the chat history area height from your layout
        // You'll need to calculate this based on your UI layout
        20 // Placeholder - adjust based on actual chat area height
    }

    pub fn get_current_link(&self) -> Option<&String> {
        self.current_link_index
            .and_then(|idx| self.available_links.get(idx))
    }
}

impl Default for ChatManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages LLM module state and interactions
/// This separates LLM logic from App, making it modular
#[derive(Debug)]
pub struct LlmModuleManager {
    pub chat_manager: ChatManager,
    session_id: String,
}

impl LlmModuleManager {
    pub fn new() -> Self {
        let session_id = format!(
            "session_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        Self {
            chat_manager: ChatManager::new(),
            session_id,
        }
    }

    /// Update module bindings with current chat state
    pub fn update_module_bindings(&self, module: &mut Module, llm_engine: Option<&LlmEngine>) {
        // Update model info
        let model_info = llm_engine
            .map(|e| e.get_model_info().to_string())
            .unwrap_or_else(|| "No model loaded".to_string());

        module.config.bindings.insert(
            "model_info".to_string(),
            serde_json::Value::String(model_info),
        );

        // Update chat history
        let chat_history = if let Some(engine) = llm_engine {
            self.format_chat_history(engine)
        } else {
            Vec::new()
        };

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
            serde_json::Value::Number(self.chat_manager.get_chat_scroll_offset().into()),
        );
    }

    /// Format chat history for display
    fn format_chat_history(&self, llm_engine: &LlmEngine) -> Vec<String> {
        match llm_engine.get_chat_history(50) {
            Ok(messages) => {
                messages.iter()
                    .map(|msg| format!("{}:{}", msg.role, msg.content))
                    .collect()
            }
            Err(_) => vec!["Error loading chat history".to_string()],
        }
    }

    /// Handle chat input character
    pub fn handle_input(&mut self, ch: char) {
        self.chat_manager.handle_chat_input(ch);
    }

    /// Handle backspace
    pub fn handle_backspace(&mut self) {
        self.chat_manager.chat_backspace();
    }

    /// Scroll chat up
    pub fn scroll_up(&mut self) {
        self.chat_manager.scroll_chat_up();
    }

    /// Scroll chat down
    pub fn scroll_down(&mut self, llm_engine: &LlmEngine) {
        self.chat_manager.scroll_chat_down(llm_engine);
    }

    /// Cycle through document links
    pub fn cycle_links_forward(&mut self) {
        self.chat_manager.cycle_document_links();
    }

    /// Cycle through document links backward
    pub fn cycle_links_backward(&mut self) {
        self.chat_manager.cycle_document_links_backward();
    }

    /// Get current selected link
    pub fn get_current_link(&self) -> Option<&String> {
        self.chat_manager.get_current_link()
    }

    /// Clear document links
    pub fn clear_links(&mut self) {
        self.chat_manager.clear_document_links();
    }

    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }
}

impl Default for LlmModuleManager {
    fn default() -> Self {
        Self::new()
    }
}
