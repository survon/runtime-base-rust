use crate::ui::document::content::DocumentContent;

use super::ExternalViewer;

impl ExternalViewer {
    pub(super) fn create_document_html(&self, content: &DocumentContent) -> color_eyre::Result<String> {
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
}
