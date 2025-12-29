use std::{
    path::Path,
    process::Stdio,
};
use tokio::process::Command as AsyncCommand;

use crate::log_debug;

use super::ExternalViewer;

impl ExternalViewer {
    pub(super) async fn launch_browser(&self, html_path: &Path) -> color_eyre::Result<()> {
        // Prioritize lightweight browsers suitable for Pi/embedded systems
        let browsers = [
            "netsurf-gtk",      // ~20MB RAM, super lightweight, perfect for Pi
            "surf",             // ~30MB RAM, minimalist webkit
            "midori",           // ~40MB RAM, light but feature-complete
            "chromium-browser", // Linux Chrome
            "google-chrome",    // macOS/Linux Chrome
            "firefox",          // Cross-platform
            "open",             // macOS default opener (uses Safari or default browser)
            "epiphany",         // GNOME Web, also lightweight
        ];

        for browser in &browsers {
            if self.command_exists(browser).await {
                let mut cmd = AsyncCommand::new(browser);

                // Configure browser-specific flags
                match *browser {
                    "open" => {
                        // macOS open command - just pass the file path, no file:// prefix
                        cmd.arg(html_path);
                    },
                    "netsurf-gtk" | "surf" => {
                        // Minimal browsers - no special flags
                        cmd.arg(format!("file://{}", html_path.display()));
                    },
                    "midori" => {
                        cmd.arg("--app");
                        cmd.arg(format!("file://{}", html_path.display()));
                    },
                    "chromium-browser" | "google-chrome" | "firefox" => {
                        cmd.arg("--app");
                        cmd.arg(format!("file://{}", html_path.display()));
                    },
                    _ => {
                        cmd.arg(format!("file://{}", html_path.display()));
                    },
                }

                cmd.stdout(Stdio::null());
                cmd.stderr(Stdio::null());

                if let Ok(_) = cmd.spawn() {
                    log_debug!("Launched document viewer with: {}", browser);
                    return Ok(());
                }
            }
        }

        Err(color_eyre::eyre::eyre!(
            "No suitable browser found. Install one of: netsurf-gtk, surf, midori, chromium-browser"
        ))
    }
}
