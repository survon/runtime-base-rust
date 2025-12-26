use super::ValveControlHandler;

impl ValveControlHandler {
    pub(in crate::module) fn is_in_cmd_window(&self) -> bool {
        self.current_mode.as_deref() == Some("cmd")
    }
}
