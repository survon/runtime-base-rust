// ui/external_viewer.rs - Launch external viewers on demand

use std::process::{Command, Stdio};
use std::path::Path;
use color_eyre::Result;
use tokio::process::Command as AsyncCommand;
use wry::{
    application::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    webview::WebViewBuilder,
};

#[derive(Debug)]
pub struct ExternalViewer {
    temp_dir: std::path::PathBuf,
}

impl ExternalViewer {
    pub fn new() -> Result<Self> {
        let temp_dir = std::path::PathBuf::from("/tmp/survon_viewer");
        std::fs::create_dir_all(&temp_dir)?;

        Ok(Self { temp_dir })
    }

    /// Check if external viewing is possible (synchronous)
    pub fn can_launch_external(&self) -> bool {
        let browsers = ["chromium-browser", "firefox", "epiphany"];

        for browser in &browsers {
            if Command::new("which")
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

    /// Check if wry webview is available (always true on desktop Linux)
    pub fn can_launch_wry(&self) -> bool {
        let display = std::env::var("DISPLAY").is_ok();
        let wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        eprintln!("DEBUG: DISPLAY={}, WAYLAND_DISPLAY={}", display, wayland);
        display || wayland
    }

    /// Launch wry-based document viewer
    pub fn show_document_wry(&self, document_path: &str, content: &crate::ui::document_viewer::DocumentContent) -> Result<()> {
        let html_content = self.create_document_html(content)?;

        // Run wry in a separate thread to avoid blocking
        std::thread::spawn(move || {
            if let Err(e) = Self::run_wry_viewer(html_content) {
                eprintln!("Wry viewer error: {}", e);
            }
        });

        Ok(())
    }

    fn run_wry_viewer(html_content: String) -> Result<()> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Survon Document Viewer")
            .with_inner_size(wry::application::dpi::LogicalSize::new(900, 700))
            .build(&event_loop)?;

        let _webview = WebViewBuilder::new(window)?
            .with_html(&html_content)?
            .build()?;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            }
        });
    }

    /// Launch external viewer for documents with images
    pub async fn show_document_external(&self, document_path: &str, content: &crate::ui::document_viewer::DocumentContent) -> Result<()> {
        // Create temporary HTML file with embedded images
        let html_path = self.temp_dir.join("document.html");
        let html_content = self.create_document_html(content)?;

        tokio::fs::write(&html_path, html_content).await?;

        // Launch with system default browser (lightweight on Pi)
        self.launch_browser(&html_path).await?;

        Ok(())
    }

    /// Launch external image viewer
    pub async fn show_images_external(&self, image_paths: &[String]) -> Result<()> {
        // Use feh, eog, or other lightweight viewers available on Pi
        let viewers = ["feh", "eog", "gpicview", "xviewer"];

        for viewer in &viewers {
            if self.command_exists(viewer).await {
                let mut cmd = AsyncCommand::new(viewer);
                cmd.args(image_paths);
                cmd.stdout(Stdio::null());
                cmd.stderr(Stdio::null());

                if let Ok(_) = cmd.spawn() {
                    return Ok(());
                }
            }
        }

        Err(color_eyre::eyre::eyre!("No suitable image viewer found"))
    }

    /// Launch external video player
    pub async fn show_video_external(&self, video_path: &str) -> Result<()> {
        let players = ["mpv", "vlc", "omxplayer"]; // omxplayer is Pi-optimized

        for player in &players {
            if self.command_exists(player).await {
                let mut cmd = AsyncCommand::new(player);
                cmd.arg(video_path);
                cmd.stdout(Stdio::null());
                cmd.stderr(Stdio::null());

                if let Ok(_) = cmd.spawn() {
                    return Ok(());
                }
            }
        }

        Err(color_eyre::eyre::eyre!("No suitable video player found"))
    }

    fn create_document_html(&self, content: &crate::ui::document_viewer::DocumentContent) -> Result<String> {
        let mut html = String::from(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Document Viewer</title>
    <style>
        body {
            font-family: monospace;
            margin: 20px;
            background: #1e1e1e;
            color: #ffffff;
            line-height: 1.6;
        }
        img {
            max-width: 100%;
            height: auto;
            border: 1px solid #444;
            margin: 10px 0;
        }
        .close-btn {
            position: fixed;
            top: 10px;
            right: 10px;
            background: #ff4444;
            color: white;
            border: none;
            padding: 10px;
            cursor: pointer;
        }
    </style>
</head>
<body>
    <button class="close-btn" onclick="window.close()">Close</button>
    <div id="content">
"#);

        // Process text content and replace image placeholders
        let mut processed_text = content.text.clone();

        for (image_id, image_path) in &content.image_mappings {
            let placeholder = format!("{{{{IMAGE_{}}}}}", image_id);
            let img_tag = format!(r#"<img src="file://{}" alt="{}" title="{}" />"#,
                                  image_path, image_id, image_id);
            processed_text = processed_text.replace(&placeholder, &img_tag);
        }

        // Convert line breaks to HTML
        processed_text = processed_text.replace('\n', "<br>");

        html.push_str(&processed_text);
        html.push_str("</div></body></html>");

        Ok(html)
    }

    async fn launch_browser(&self, html_path: &Path) -> Result<()> {
        let browsers = ["chromium-browser", "firefox", "epiphany"]; // Pi-friendly browsers

        for browser in &browsers {
            if self.command_exists(browser).await {
                let mut cmd = AsyncCommand::new(browser);
                cmd.arg("--app=true"); // App mode for cleaner UI
                cmd.arg(format!("file://{}", html_path.display()));
                cmd.stdout(Stdio::null());
                cmd.stderr(Stdio::null());

                if let Ok(_) = cmd.spawn() {
                    return Ok(());
                }
            }
        }

        Err(color_eyre::eyre::eyre!("No suitable browser found"))
    }

    async fn command_exists(&self, command: &str) -> bool {
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
