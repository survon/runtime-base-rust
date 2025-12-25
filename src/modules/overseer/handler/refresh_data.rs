use crate::modules::overseer::database::OverseerDatabase;
use super::{HandlerMessage, OverseerHandler};

impl OverseerHandler {
    pub(super) fn refresh_async_data_only(&mut self) {
        let tx = self.message_tx.clone();

        // Refresh discovered devices (from discovery manager)
        if let Some(discovery) = &self.discovery_manager {
            let discovery_clone = discovery.clone();
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                let devices = discovery_clone.get_discovered_devices().await;
                let _ = tx_clone.send(HandlerMessage::DevicesRefreshed(devices));
            });
        }

        // Refresh registry (needs network fetch)
        let registry_url = self.registry_url.clone();
        tokio::spawn(async move {
            // Mock call - in real implementation would fetch from network
            let modules = Self::fetch_registry_manifests(&registry_url).await;
            if let Ok(modules) = modules {
                let _ = tx.send(HandlerMessage::RegistryRefreshed(modules));
            }
        });
    }

    pub(super) fn refresh_data_async(&mut self) {
        // Synchronous updates first (immediate)
        if let Ok(devices) = self.database.get_all_known_devices() {
            self.known_devices = devices;
        }

        self.refresh_installed_modules();

        if let Ok(archived) = self.list_archived_modules() {
            self.archived_modules = archived;
        }

        // Then trigger async updates
        self.refresh_async_data_only();
    }
}
