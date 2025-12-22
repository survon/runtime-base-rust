use super::{HandlerMessage, WastelandManagerHandler};

impl WastelandManagerHandler {
    pub(super) fn handle_scan_devices(&mut self) {
        if self.is_scanning {
            self.status_message = Some("⚠️ Scan already in progress".to_string());
            return;
        }

        if let Some(discovery) = &self.discovery_manager {
            self.is_scanning = true;
            let scan_duration = 15;
            self.scan_countdown = scan_duration as u8;

            let discovery_clone = discovery.clone();
            let tx = self.message_tx.clone();

            tokio::spawn(async move {
                let countdown_task = async {
                    for i in (1..=scan_duration).rev() {
                        let _ = tx.send(HandlerMessage::ScanProgress(i as u8));
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                };

                let scan_task = async { discovery_clone.scan_once(scan_duration).await };

                let (_, scan_result) = tokio::join!(countdown_task, scan_task);

                match scan_result {
                    Ok(count) => {
                        let _ = tx.send(HandlerMessage::ScanComplete(count));
                    }
                    Err(e) => {
                        let _ =
                            tx.send(HandlerMessage::ScanFailed(format!("❌ Scan failed: {}", e)));
                    }
                }
            });

            self.status_message = Some("🔍 Starting BLE scan...".to_string());
        } else {
            self.status_message = Some("❌ Discovery manager not available".to_string());
        }
    }
}
