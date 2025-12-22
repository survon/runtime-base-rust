use crate::modules::overseer::config_editor::ConfigEditor;
use super::OverseerHandler;

impl OverseerHandler {
    pub(super) fn get_config_editor(&self) -> Option<&ConfigEditor> {
        self.config_editor.as_ref()
    }
}
