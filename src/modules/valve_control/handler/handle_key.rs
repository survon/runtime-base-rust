use crossterm::event::KeyCode;

use crate::{
    modules::{
        Module,
        valve_control::handler::ValveControlHandler,
    },
    util::io::event::AppEvent,
};

impl ValveControlHandler {
    pub(super) fn _handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
        match key_code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.toggle_valve();
                None
            }
            KeyCode::Char('o') => {
                if !self.current_state {
                    self.toggle_valve();
                }
                None
            }
            KeyCode::Char('c') => {
                if self.current_state {
                    self.toggle_valve();
                }
                None
            }
            _ => None,
        }
    }
}
