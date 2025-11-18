use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};
use crate::modules::Module;

pub fn render_entertainment_module(module: &Module, area: Rect, buf: &mut Buffer) {
    let content_lines = vec![
        Line::from(format!("Bus Topic: {}", module.config.bus_topic)),
        Line::from(format!("Game Type: {}",
                           module.config.game_type.as_deref().unwrap_or("Unknown")
        )),
        Line::from(""),
        Line::from("Entertainment module ready."),
        Line::from("Game functionality would be implemented here."),
    ];

    let content = Text::from(content_lines);
    let widget = Paragraph::new(content)
        .block(
            Block::bordered()
                .title("Entertainment Module")
                .border_type(BorderType::Rounded)
        )
        .wrap(Wrap { trim: true });

    widget.render(area, buf);
}
