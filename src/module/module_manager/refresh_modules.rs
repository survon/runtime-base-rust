use crate::module::ModuleManager;

impl ModuleManager {
    pub async fn refresh_modules(&mut self) {
        if let Err(e) = self.discover_modules() {
            panic!("Failed to refresh modules: {}", e);
        }

        let module_count = self.get_modules().len();
        if self.selected_module >= module_count && module_count > 0 {
            self.selected_module = 0;
        }
    }
}
