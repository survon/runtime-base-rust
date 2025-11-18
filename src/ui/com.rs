use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};
use crate::modules::Module;

pub fn render_com_module(module: &Module, area: Rect, buf: &mut Buffer) {
    let mut content_lines = vec![
        Line::from(format!("Bus Topic: {}", module.config.bus_topic)),
    ];

    if let Some(ports) = &module.config.ports {
        content_lines.push(Line::from("Ports:"));
        for port in ports {
            content_lines.push(Line::from(format!("  • {}", port)));
        }
    }

    if let Some(messages) = &module.config.messages {
        content_lines.push(Line::from("Available Commands:"));
        for message in messages {
            content_lines.push(Line::from(format!("  • {}", message)));
        }
    }

    content_lines.push(Line::from(""));
    content_lines.push(Line::from("Press '1' to send close_gate command"));

    let content = Text::from(content_lines);
    let widget = Paragraph::new(content)
        .block(
            Block::bordered()
                .title("Communication Module")
                .border_type(BorderType::Rounded)
        )
        .wrap(Wrap { trim: true });

    widget.render(area, buf);
}
