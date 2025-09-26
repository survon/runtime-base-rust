use std::path::Path;
use ratatui::{prelude::*, widgets::*};
use crate::ui::document_viewer::DocumentContent;
use ratatui_image::{picker::Picker, StatefulImage, protocol::StatefulProtocol};
use std::collections::HashMap;

pub struct DocumentPopupWidget<'a> {
    pub content: &'a DocumentContent,
    pub file_path: &'a str,
    picker: Picker,
    image_states: HashMap<String, Box<dyn StatefulProtocol>>,
}

impl<'a> DocumentPopupWidget<'a> {
    pub fn new(content: &'a DocumentContent, file_path: &'a str) -> Self {
        // Use from_termios (the available method in 1.0)
        let mut picker = Picker::from_termios().unwrap_or_else(|_| Picker::new((8, 12)));

        let mut image_states = HashMap::new();

        for (image_id, image_path) in &content.image_mappings {
            if let Ok(dyn_img) = image::open(image_path) {
                let protocol = picker.new_resize_protocol(dyn_img);
                image_states.insert(image_id.clone(), protocol);
            }
        }

        Self {
            content,
            file_path,
            picker,
            image_states,
        }
    }

    fn process_content_with_images(&self, content: &str) -> Vec<ContentLine> {
        let mut processed_lines = Vec::new();

        for line in content.lines() {
            if line.contains("{{IMAGE_") {
                if let Some(image_id) = self.extract_image_id(line) {
                    if self.image_states.contains_key(&image_id) {
                        processed_lines.push(ContentLine::Image(image_id));
                    } else {
                        processed_lines.push(ContentLine::Text(format!("[IMAGE: {} - failed to load]", image_id)));
                    }
                } else {
                    processed_lines.push(ContentLine::Text(line.to_string()));
                }
            } else {
                processed_lines.push(ContentLine::Text(line.to_string()));
            }
        }

        processed_lines
    }

    fn extract_image_id(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find("{{IMAGE_") {
            if let Some(end) = line[start..].find("}}") {
                let image_ref = &line[start + 8..start + end];
                return Some(image_ref.to_string());
            }
        }
        None
    }
}

#[derive(Debug)]
enum ContentLine {
    Text(String),
    Image(String),
}

impl StatefulWidget for DocumentPopupWidget<'_> {
    type State = usize;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        Clear.render(area, buf);
        let popup_area = centered_rect(90, 90, area);

        let filename = Path::new(self.file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        let content_lines = self.process_content_with_images(&self.content.text);
        let inner_area = popup_area.inner(Margin { horizontal: 1, vertical: 1 });
        let scroll_offset = *state;

        // Convert everything to a uniform line-based system
        let mut virtual_lines = Vec::new();
        for content_line in content_lines {
            match content_line {
                ContentLine::Text(text) => {
                    for line in text.lines() {
                        virtual_lines.push(VirtualLine::Text(line.to_string()));
                    }
                }
                ContentLine::Image(image_id) => {
                    // Each image takes exactly 10 virtual lines
                    for i in 0..10 {
                        if i == 0 {
                            virtual_lines.push(VirtualLine::ImageStart(image_id.clone()));
                        } else {
                            virtual_lines.push(VirtualLine::ImageContinue(image_id.clone()));
                        }
                    }
                }
            }
        }

        // Render the border
        let block = Block::bordered()
            .title(format!("Document: {} (↑↓ to scroll, Esc to close)", filename))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);
        block.render(popup_area, buf);

        // Simple line-by-line rendering
        let visible_height = inner_area.height as usize;
        let visible_lines = virtual_lines
            .iter()
            .skip(scroll_offset)
            .take(visible_height)
            .collect::<Vec<_>>();

        let mut y_pos = 0;
        let mut current_image: Option<String> = None;
        let mut image_start_y = 0;

        for virtual_line in visible_lines {
            if y_pos >= visible_height {
                break;
            }

            match virtual_line {
                VirtualLine::Text(text) => {
                    current_image = None;
                    let text_area = Rect {
                        x: inner_area.x,
                        y: inner_area.y + y_pos as u16,
                        width: inner_area.width,
                        height: 1,
                    };

                    let paragraph = Paragraph::new(text.clone())
                        .style(Style::default().fg(Color::White));
                    paragraph.render(text_area, buf);

                    y_pos += 1;
                }
                VirtualLine::ImageStart(image_id) => {
                    current_image = Some(image_id.clone());
                    image_start_y = y_pos;
                    y_pos += 1;
                }
                VirtualLine::ImageContinue(_) => {
                    y_pos += 1;

                    // If we've completed the image (10 lines), render it
                    if let Some(ref image_id) = current_image {
                        if y_pos - image_start_y >= 10 {
                            if let Some(protocol) = self.image_states.get_mut(image_id) {
                                let image_area = Rect {
                                    x: inner_area.x,
                                    y: inner_area.y + image_start_y as u16,
                                    width: inner_area.width,
                                    height: 10,
                                };

                                let image_widget = StatefulImage::new(None).resize(ratatui_image::Resize::Fit(None));

                                image_widget.render(image_area, buf, protocol);
                            }
                            current_image = None;
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
enum VirtualLine {
    Text(String),
    ImageStart(String),
    ImageContinue(String),
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
