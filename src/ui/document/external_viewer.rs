// ui/external_viewer.rs - Launch external browser for viewing all media types

use std::process::Stdio;
use std::path::Path;
use color_eyre::Result;
use tokio::process::Command as AsyncCommand;

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

    /// Launch external viewer for documents with images
    pub async fn show_document_external(&self, document_path: &str, content: &crate::ui::document::DocumentContent) -> Result<()> {
        let path = std::path::Path::new(document_path);
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            // Documents - browsers can render these natively
            "pdf" => {
                self.launch_browser_with_file(document_path).await?;
            }

            // Video files - create simple HTML5 video player
            "mp4" | "webm" | "ogg" | "ogv" | "avi" | "mov" | "mkv" => {
                let html_path = self.temp_dir.join("video.html");
                let html_content = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Video Player</title>
    <style>
        body {{ margin: 0; background: #000; display: flex; justify-content: center; align-items: center; height: 100vh; }}
        video {{ max-width: 100%; max-height: 100vh; }}
    </style>
</head>
<body>
    <video controls autoplay>
        <source src="file://{}" type="video/{}">
        Your browser doesn't support this video format.
    </video>
</body>
</html>
"#, document_path, extension);
                tokio::fs::write(&html_path, html_content).await?;
                self.launch_browser(&html_path).await?;
            }

            // Audio files - create simple HTML5 audio player
            "mp3" | "wav" | "ogg" | "oga" | "flac" | "m4a" | "aac" => {
                let html_path = self.temp_dir.join("audio.html");
                let html_content = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Audio Player</title>
    <style>
        body {{
            margin: 0;
            background: #1e1e1e;
            color: #fff;
            display: flex;
            flex-direction: column;
            justify-content: center;
            align-items: center;
            height: 100vh;
            font-family: monospace;
        }}
        audio {{ width: 80%; max-width: 600px; margin: 20px; }}
        .info {{ text-align: center; }}
    </style>
</head>
<body>
    <div class="info">
        <h2>Now Playing</h2>
        <p>{}</p>
    </div>
    <audio controls autoplay>
        <source src="file://{}" type="audio/{}">
        Your browser doesn't support this audio format.
    </audio>
</body>
</html>
"#, path.file_name().unwrap_or_default().to_string_lossy(), document_path, extension);
                tokio::fs::write(&html_path, html_content).await?;
                self.launch_browser(&html_path).await?;
            }

            // Text/Markdown - convert to HTML
            "txt" | "md" | "log" | "rtf" => {
                let html_path = self.temp_dir.join("document.html");
                let html_content = self.create_document_html(content)?;
                tokio::fs::write(&html_path, html_content).await?;
                self.launch_browser(&html_path).await?;
            }

            // Images - create simple HTML image viewer
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" => {
                let html_path = self.temp_dir.join("image.html");
                let html_content = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Image Viewer</title>
    <style>
        body {{
            margin: 0;
            background: #000;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
        }}
        img {{ max-width: 100%; max-height: 100vh; object-fit: contain; }}
    </style>
</head>
<body>
    <img src="file://{}" alt="Image">
</body>
</html>
"#, document_path);
                tokio::fs::write(&html_path, html_content).await?;
                self.launch_browser(&html_path).await?;
            }

            // Unknown - try opening directly and let browser figure it out
            _ => {
                self.launch_browser_with_file(document_path).await?;
            }
        }

        Ok(())
    }

    async fn launch_browser_with_file(&self, file_path: &str) -> Result<()> {
        let path = std::path::Path::new(file_path);
        self.launch_browser(path).await
    }

    fn create_document_html(&self, content: &crate::ui::document::DocumentContent) -> Result<String> {
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
                    eprintln!("Launched document viewer with: {}", browser);
                    return Ok(());
                }
            }
        }

        Err(color_eyre::eyre::eyre!(
            "No suitable browser found. Install one of: netsurf-gtk, surf, midori, chromium-browser"
        ))
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
