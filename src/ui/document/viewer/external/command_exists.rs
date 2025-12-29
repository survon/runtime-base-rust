use std::{
    process::Stdio,
};
use tokio::process::Command as AsyncCommand;

use super::ExternalViewer;

impl ExternalViewer {
    pub(super) async fn command_exists(&self, command: &str) -> bool {
        AsyncCommand::new("which")
            .arg(command)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|status| status.success())
            .unwrap_or(false)
    }
}
