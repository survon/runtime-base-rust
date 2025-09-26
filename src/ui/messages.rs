use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color,Stylize},
    text::Text,
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};

pub fn render_recent_messages_panel(area: Rect, buf: &mut Buffer) {
    let content = Text::from("Message bus activity will appear here...\n\nRecent commands and sensor data\nwill be displayed in real-time.");

    let messages_widget = Paragraph::new(content)
        .block(
            Block::bordered()
                .title("Message Bus")
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Gray)
        .wrap(Wrap { trim: true });

    messages_widget.render(area, buf);
}
