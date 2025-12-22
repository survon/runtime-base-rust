use super::{HandlerMessage, WastelandManagerHandler};

impl WastelandManagerHandler {
    pub(super) fn process_messages(&mut self) {
        // Non-blocking: process all queued messages
        while let Ok(msg) = self.message_rx.try_recv() {
            match msg {
                HandlerMessage::DevicesRefreshed(devices) => {
                    // These are discovered (not yet in DB) devices
                    self.pending_devices = devices;
                    self.status_message = None;
                }
                HandlerMessage::KnownDevicesRefreshed(devices) => {
                    self.known_devices = devices;
                    self.status_message = None;
                }
                HandlerMessage::RegistryRefreshed(modules) => {
                    self.registry_modules = modules;
                    self.status_message = None;
                }
                HandlerMessage::DeviceTrusted(mac) => {
                    self.pending_devices.retain(|(m, _, _)| m != &mac);
                    if self.selected_index > 0 && self.selected_index >= self.pending_devices.len()
                    {
                        self.selected_index -= 1;
                    }
                    self.status_message = Some(format!("✓ Device {} trusted!", mac));
                    self.refresh_known_devices();
                }
                HandlerMessage::DeviceDiscovered { mac, name, rssi } => {
                    // Add to pending list if not already there
                    if !self.pending_devices.iter().any(|(m, _, _)| m == &mac) {
                        self.pending_devices.push((mac, name, rssi));
                    }
                }
                HandlerMessage::ModuleInstalled(name) => {
                    self.refresh_installed_modules();
                    self.status_message = Some(format!("✓ Module {} installed!", name));
                }
                HandlerMessage::OperationInProgress(msg) => {
                    self.status_message = Some(msg);
                }
                HandlerMessage::ScanProgress(seconds) => {
                    self.scan_countdown = seconds;
                    self.status_message =
                        Some(format!("🔍 Scanning... {} seconds remaining", seconds));
                }
                HandlerMessage::ScanComplete(count) => {
                    self.is_scanning = false;
                    self.scan_countdown = 0;
                    self.status_message =
                        Some(format!("✅ Scan complete! {} new device(s) found", count));
                    self.refresh_data_async();
                }
                HandlerMessage::ScanFailed(err_msg) => {
                    self.is_scanning = false;
                    self.scan_countdown = 0;
                    self.status_message = Some(err_msg);
                }
                _ => {}
            }
        }
    }
}
