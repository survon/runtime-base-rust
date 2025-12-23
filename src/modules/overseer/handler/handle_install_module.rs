use super::{
    installer::ModuleInstaller,
    HandlerMessage,
    InstallSource,
    OverseerHandler,
};

impl OverseerHandler {
    pub(super) fn handle_install_module(&mut self) {
        if self.selected_index < self.registry_manifests.len() {
            let module = &self.registry_manifests[self.selected_index];
            let module_id = module.id.clone();
            let module_name = module.name.clone();

            // Clone self fields needed for async closure
            let wasteland_path = self.wasteland_path.clone();
            let archive_path = self.archive_path.clone();
            let registry_url = self.registry_url.clone();
            let tx = self.message_tx.clone();

            self.status_message = Some(format!("⏳ Installing {}...", module_name));

            tokio::spawn(async move {
                // Create a temporary handler-like struct for the install operation
                let installer = ModuleInstaller {
                    wasteland_path,
                    archive_path,
                    registry_url,
                };

                match installer
                    .install_module(InstallSource::Registry(module_id.clone()), None)
                    .await
                {
                    Ok(name) => {
                        let _ = tx.send(HandlerMessage::ModuleInstalled(name));
                    }
                    Err(e) => {
                        let _ = tx.send(HandlerMessage::OperationInProgress(format!(
                            "❌ Failed to install: {}",
                            e
                        )));
                    }
                }
            });
        }
    }
}
