use crate::module::strategies::overseer::config_editor::ConfigEditor;
use super::OverseerHandler;

impl OverseerHandler {
    pub(in crate::module) fn get_config_editor(&self) -> Option<&ConfigEditor> {
        self.config_editor.as_ref()
    }
}
