use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};
use crate::module::Module;

pub fn render_knowledge_module(module: &Module, area: Rect, buf: &mut Buffer) {
    let mut content_lines = vec![
        Line::from(format!("Bus Topic: {}", module.config.bus_topic)),
        Line::from(""),
    ];

    if module.has_knowledge_dir() {
        content_lines.push(Line::from("📁 Knowledge directory found"));
        content_lines.push(Line::from("This module contains knowledge files that can"));
        content_lines.push(Line::from("be used to enhance the LLM capabilities."));
    } else {
        content_lines.push(Line::from("⚠️  No knowledge directory found"));
        content_lines.push(Line::from("Create a 'knowledge' folder in this module"));
        content_lines.push(Line::from("and add text files, PDFs, or other documents."));
    }

    let content = Text::from(content_lines);
    let widget = Paragraph::new(content)
        .block(
            Block::bordered()
                .title("Knowledge Module")
                .border_type(BorderType::Rounded)
        )
        .wrap(Wrap { trim: true });

    widget.render(area, buf);
}
