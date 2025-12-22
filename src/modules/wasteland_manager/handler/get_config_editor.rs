use crate::modules::wasteland_manager::config_editor::ConfigEditor;
use super::WastelandManagerHandler;

impl WastelandManagerHandler {
    pub(super) fn get_config_editor(&self) -> Option<&ConfigEditor> {
        self.config_editor.as_ref()
    }
}
