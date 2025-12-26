use crossterm::event::KeyCode;

use crate::{
    module::Module,
    util::io::event::AppEvent,
};
use crate::module::strategies::valve_control::handler::ValveControlHandler;

impl ValveControlHandler {
    pub(in crate::module) fn _handle_key(&mut self, key_code: KeyCode, _module: &mut Module) -> Option<AppEvent> {
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
