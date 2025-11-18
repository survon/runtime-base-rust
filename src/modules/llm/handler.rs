// src/modules/llm/handler.rs

use std::any::Any;
use color_eyre::Result;
use ratatui::crossterm::event::KeyCode;
use crate::event::AppEvent;
use crate::modules::{Module, module_handler::ModuleHandler};
use crate::bus::BusMessage;
use super::{LlmEngine, LlmModuleManager};

#[derive(Debug)]
pub struct LlmHandler {
    module_manager: LlmModuleManager,
    engine: Option<LlmEngine>,
}

impl LlmHandler {
    pub fn new(engine: Option<LlmEngine>) -> Self {
        Self {
            module_manager: LlmModuleManager::new(),
            engine,
        }
    }

    pub fn get_manager(&self) -> &LlmModuleManager {
        &self.module_manager
    }

    pub fn get_manager_mut(&mut self) -> &mut LlmModuleManager {
        &mut self.module_manager
    }

    pub fn get_engine(&self) -> Option<&LlmEngine> {
        self.engine.as_ref()
    }

    pub async fn submit_message(
        &mut self,
        module_name: String,
        knowledge_module_names: Vec<String>,
    ) -> Result<()> {
        if self.module_manager.chat_manager.get_input().trim().is_empty() {
            return Ok(());
        }

        let engine = match &self.engine {
            Some(e) => e,
            None => return Ok(()),
        };

        let recent_messages = Vec::new(); // Could get from database if needed

        // Build context from provided knowledge module names
        let mut context = crate::modules::llm::LlmContext::new();
        for name in knowledge_module_names {
            context.add_knowledge_module(name);
        }

        // Get the query
        let query = self.module_manager.chat_manager.get_input().to_string();

        // Clear input
        self.module_manager.chat_manager.clear_input();
        self.module_manager.chat_manager.clear_document_links();

        // Process query directly using the engine
        let _response = engine.process_user_query(
            query,
            module_name,
            recent_messages,
            context,
        ).await?;

        // Update available links from response
        self.module_manager.chat_manager.update_available_links(engine);

        Ok(())
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
                self.module_manager.cycle_links_forward();
                None
            },
            KeyCode::BackTab => {
                self.module_manager.cycle_links_backward();
                None
            },
            KeyCode::Enter => {
                if let Some(file_path) = self.module_manager.get_current_link() {
                    Some(AppEvent::OpenDocument(file_path.clone()))
                } else {
                    Some(AppEvent::ChatSubmit)
                }
            },
            KeyCode::Backspace => {
                self.module_manager.handle_backspace();
                None
            },
            KeyCode::Char(ch) => {
                self.module_manager.handle_input(ch);
                None
            },
            KeyCode::PageUp | KeyCode::Up => {
                self.module_manager.scroll_up();
                None
            },
            KeyCode::PageDown | KeyCode::Down => {
                if let Some(engine) = &self.engine {
                    self.module_manager.scroll_down(engine);
                }
                None
            },
            _ => None,
        }
    }

    fn handle_event(&mut self, _event: &AppEvent, _module: &mut Module) -> Result<bool> {
        // Events are already handled in handle_key
        Ok(false)
    }

    fn update_bindings(&mut self, module: &mut Module) {
        self.module_manager.update_module_bindings(module, self.engine.as_ref());
    }

    fn module_type(&self) -> &str {
        "llm"
    }
}
