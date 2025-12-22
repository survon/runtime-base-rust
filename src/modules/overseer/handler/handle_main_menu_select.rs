use crate::modules::overseer::{
    config_editor::ConfigEditor,
    handler::{OverseerHandler, WastelandView},
};

impl OverseerHandler {
    pub(super) fn handle_main_menu_select(&mut self) {
        match self.selected_index {
            0 => {
                self.current_view = WastelandView::PendingTrust;
                self.selected_index = 0;
            }
            1 => {
                self.current_view = WastelandView::AllDevices;
                self.selected_index = 0;
                self.refresh_known_devices();
            }
            2 => {
                self.current_view = WastelandView::InstallRegistry;
                self.selected_index = 0;
            }
            3 => {
                self.current_view = WastelandView::ManageModules;
                self.selected_index = 0;
                self.refresh_installed_modules();
            }
            4 => {
                self.current_view = WastelandView::ArchivedModules;
                self.selected_index = 0;
                self.refresh_data_async();
            }
            5 => {
                self.current_view = WastelandView::CreateNewModule;
                self.config_editor = Some(ConfigEditor::new_module());
                self.selected_index = 0;
            }
            6 => {
                // Back - will be handled by returning AppEvent::Back
            }
            _ => {}
        }
    }
}
