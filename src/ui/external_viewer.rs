// ui/external_viewer.rs - Launch external viewers with egui (pure Rust)

use std::process::Stdio;
use std::path::Path;
use color_eyre::Result;
use tokio::process::Command as AsyncCommand;
use eframe::egui;
use std::sync::{Arc, Mutex};

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

    /// Check if egui viewer is available (always true since it's pure Rust)
    pub fn can_launch_wry(&self) -> bool {
        // Renamed for compatibility, but this now checks for egui availability
        // which is always true on systems with graphics
        let display = std::env::var("DISPLAY").is_ok();
        let wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        let framebuffer = Path::new("/dev/fb0").exists(); // For Pi without X

        eprintln!("DEBUG: DISPLAY={}, WAYLAND={}, FB={}", display, wayland, framebuffer);
        display || wayland || framebuffer
    }

    /// Launch egui-based document viewer
    pub fn show_document_wry(&self, document_path: &str, content: &crate::ui::document_viewer::DocumentContent) -> Result<()> {
        let content_clone = content.clone();
        let doc_path = document_path.to_string();

        // Run egui in a separate thread to avoid blocking
        std::thread::spawn(move || {
            if let Err(e) = Self::run_egui_viewer(doc_path, content_clone) {
                eprintln!("Egui viewer error: {}", e);
            }
        });

        Ok(())
    }

    fn run_egui_viewer(document_path: String, content: crate::ui::document_viewer::DocumentContent) -> Result<()> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([900.0, 700.0]),
            ..Default::default()
        };

        let app = DocumentViewerApp::new(document_path, content);

        eframe::run_native(
            "Survon Document Viewer",
            options,
            Box::new(|_cc| Box::new(app)),
        ).map_err(|e| color_eyre::eyre::eyre!("Failed to run egui app: {}", e))
    }

    /// Launch external viewer for documents with images
    pub async fn show_document_external(&self, document_path: &str, content: &crate::ui::document_viewer::DocumentContent) -> Result<()> {
        // First try to use egui viewer (pure Rust, works everywhere)
        if self.can_launch_wry() {
            return Ok(self.show_document_wry(document_path, content)?);
        }

        // Fallback: Create temporary HTML file for browser
        let html_path = self.temp_dir.join("document.html");
        let html_content = self.create_document_html(content)?;

        tokio::fs::write(&html_path, html_content).await?;
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

// Egui application for document viewing
struct DocumentViewerApp {
    document_path: String,
    content: crate::ui::document_viewer::DocumentContent,
    scroll_offset: f32,
    images: Arc<Mutex<Vec<Option<egui::TextureHandle>>>>,
}

impl DocumentViewerApp {
    fn new(document_path: String, content: crate::ui::document_viewer::DocumentContent) -> Self {
        Self {
            document_path,
            content,
            scroll_offset: 0.0,
            images: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl eframe::App for DocumentViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Title bar
            ui.horizontal(|ui| {
                ui.heading(&self.document_path);
                if ui.button("Close").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.separator();

            // Scrollable content area
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    // Display text content
                    let mut text = self.content.text.clone();

                    // Process image placeholders
                    for (image_id, image_path) in &self.content.image_mappings {
                        let placeholder = format!("{{{{IMAGE_{}}}}}", image_id);

                        // For egui, we'll show a button that can open the image
                        if text.contains(&placeholder) {
                            text = text.replace(&placeholder, &format!("[Image: {}]", image_id));
                        }
                    }

                    // Display the text with monospace font
                    ui.add(
                        egui::TextEdit::multiline(&mut text.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false)
                    );

                    // Add image viewer buttons
                    if !self.content.image_mappings.is_empty() {
                        ui.separator();
                        ui.heading("Images:");

                        for (image_id, image_path) in &self.content.image_mappings {
                            ui.horizontal(|ui| {
                                if ui.button(format!("View Image {}", image_id)).clicked() {
                                    // Try to open with system image viewer
                                    if let Err(e) = open_with_system_viewer(image_path) {
                                        eprintln!("Failed to open image: {}", e);
                                    }
                                }
                                ui.label(format!("({})", image_path));
                            });
                        }
                    }
                });
        });
    }
}

fn open_with_system_viewer(path: &str) -> Result<()> {
    // Try common image viewers
    for viewer in &["xdg-open", "feh", "eog", "gpicview"] {
        if let Ok(_) = std::process::Command::new(viewer)
            .arg(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            return Ok(());
        }
    }
    Err(color_eyre::eyre::eyre!("No image viewer found"))
}
