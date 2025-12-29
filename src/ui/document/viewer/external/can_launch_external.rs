use std::process::Stdio;

use super::ExternalViewer;

impl ExternalViewer {
    /// Check if external viewing is possible (synchronous)
    pub fn can_launch_external(&self) -> bool {
        let browsers = ["netsurf-gtk", "chromium-browser", "firefox", "epiphany"];

        for browser in &browsers {
            if std::process::Command::new("which")
                .arg(browser)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
            {
                return true;
            }
        }
        false
    }
}
