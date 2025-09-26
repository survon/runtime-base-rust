use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};
use crate::app::App;
use crate::module::Module;
use crate::ui::{com, entertainment, knowledge, llm, monitoring};

pub fn render_module_detail(app: &App, module: &Module, area: Rect, buf: &mut Buffer) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(1),     // Content
            Constraint::Length(3),  // Help
        ])
        .split(area);

    // Title with module info
    let icon = match module.config.module_type.as_str() {
        "com" => "🔌",
        "entertainment" => "🎮",
        "knowledge" => "📚",
        "llm" => "🤖",
        "monitoring" => "📊",
        _ => "⚙️",
    };

    let title = Paragraph::new(format!("{} {} - {}", icon, module.config.name, module.config.module_type))
        .block(
            Block::bordered()
                .title("Module Details")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Green)
        .alignment(Alignment::Center);
    title.render(main_layout[0], buf);

    // Module-specific content
    match module.config.module_type.as_str() {
        "monitoring" => monitoring::render_monitoring_module(module, main_layout[1], buf),
        "com" => com::render_com_module(module, main_layout[1], buf),
        "entertainment" => entertainment::render_entertainment_module(module, main_layout[1], buf),
        "knowledge" => knowledge::render_knowledge_module(module, main_layout[1], buf),
        "llm" => llm::render_llm_module(module, main_layout[1], buf),
        _ => render_default_module(module, main_layout[1], buf),
    }

    // Help
    let help = Paragraph::new("Backspace/h: Back to overview • '1': Close Gate • 'q': Quit")
        .block(
            Block::bordered()
                .title("Controls")
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Yellow)
        .alignment(Alignment::Center);
    help.render(main_layout[2], buf);
}

fn render_default_module(module: &Module, area: Rect, buf: &mut Buffer) {
    let content_lines = vec![
        Line::from(format!("Name: {}", module.config.name)),
        Line::from(format!("Type: {}", module.config.module_type)),
        Line::from(format!("Bus Topic: {}", module.config.bus_topic)),
        Line::from(format!("Path: {}", module.path.display())),
        Line::from(""),
        Line::from("This module type doesn't have a custom view yet."),
        Line::from("The module is loaded and available for messaging."),
    ];

    let content = Text::from(content_lines);
    let widget = Paragraph::new(content)
        .block(
            Block::bordered()
                .title("Module Information")
                .border_type(BorderType::Rounded)
        )
        .wrap(Wrap { trim: true });

    widget.render(area, buf);
}
