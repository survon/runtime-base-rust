use crossterm::event::KeyCode;

use crate::{
    modules::Module,
    util::io::event::AppEvent
};

use super::LlmHandler;

impl LlmHandler {
    pub(super) fn _handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
        match key_code {
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
}
