use ratatui::{
    buffer::Buffer,
    layout::Rect,
};

use crate::modules::Module;
use crate::ui::template::UiTemplate;
use super::{ViewData, OverseerCard};

impl UiTemplate for OverseerCard {
    fn render_overview(&self, is_selected: bool, area: Rect, buf: &mut Buffer, module: &mut Module) {
        self.render_overview_cta(is_selected, area, buf, module)
    }

    fn render_detail(&self, area: Rect, buf: &mut Buffer, module: &mut Module) {
        let is_selected = false;
        let ViewData { current_view, .. } = self.get_view_data(false, area, buf, module);

        match current_view {
            "Main" => self.render_main_menu(is_selected, area, buf, module),
            "PendingTrust" => self.render_pending_trust(area, buf, module),
            "AllDevices" => self.render_all_devices(area, buf, module),
            "InstallRegistry" => self.render_install_registry(area, buf, module),
            "ManageModules" => self.render_manage_modules(area, buf, module),
            "ArchivedModules" => self.render_archived_modules(area, buf, module),
            "EditConfig" => self.render_config_editor(area, buf, module),
            _ => self.render_main_menu(is_selected, area, buf, module),
        }
    }

    fn required_bindings(&self) -> &'static [&'static str] {
        &["current_view", "selected_index"]
    }

    fn docs(&self) -> &'static str {
        "Overseer interface for managing modules, trusting peripheral IoT devices, and locating registries. \
         Integrates device trust management with pending and known devices."
    }
}
