use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Paragraph, Widget},
};
use crate::app::App;
use crate::module::Module;
use crate::ui::chat_history;

pub fn render_llm_chat(app: &App, module: &Module, area: Rect, buf: &mut Buffer) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(1),     // Chat history
            Constraint::Length(3),  // Input box
            Constraint::Length(3),  // Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new(format!("🤖 {} - Interactive Chat", module.config.name))
        .block(
            Block::bordered()
                .title("LLM Chat")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Green)
        .alignment(Alignment::Center);
    title.render(main_layout[0], buf);

    // Chat history
    chat_history::render_chat_history(app, main_layout[1], buf);

    // Input box
    let input_text = format!("> {}", app.get_chat_input());
    let input_widget = Paragraph::new(input_text)
        .block(
            Block::bordered()
                .title("Type your message")
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Yellow);
    input_widget.render(main_layout[2], buf);

    // Help
    let help = Paragraph::new("Type your message and press Enter to send • Esc: Back to overview")
        .block(
            Block::bordered()
                .title("Controls")
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Yellow)
        .alignment(Alignment::Center);
    help.render(main_layout[3], buf);
}
