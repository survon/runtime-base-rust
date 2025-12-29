use crate::ui::document::content::DocumentContent;

use super::ExternalViewer;

impl ExternalViewer {
    /// Launch external viewer for documents with images
    pub async fn show_document_external(&self, document_path: &str, content: &DocumentContent) -> color_eyre::Result<()> {
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
}
