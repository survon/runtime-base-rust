/// ./src/ui/llm.rs

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};
use crate::module::Module;

pub fn render_llm_module(module: &Module, area: Rect, buf: &mut Buffer) {
    let content_lines = vec![
        Line::from(format!("Bus Topic: {}", module.config.bus_topic)),
        Line::from(format!("Model: {}",
                           module.config.model.as_deref().unwrap_or("Unknown")
        )),
        Line::from(""),
        Line::from("🤖 LLM Module Status: Ready"),
        Line::from(""),
        Line::from("This module processes queries and provides"),
        Line::from("responses based on available knowledge modules."),
        Line::from(""),
        Line::from("Future: Query interface will be implemented here."),
    ];

    let content = Text::from(content_lines);
    let widget = Paragraph::new(content)
        .block(
            Block::bordered()
                .title("LLM Module")
                .border_type(BorderType::Rounded)
        )
        .wrap(Wrap { trim: true });

    widget.render(area, buf);
}
