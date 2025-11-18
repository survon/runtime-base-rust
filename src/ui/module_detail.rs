use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget},
};
use crate::modules::{ModuleManager};

/// Renders the title and help sections (the "chrome" around the template content)
/// The actual template content is rendered separately via Frame in the main loop
pub fn render_module_detail_chrome(module_manager: &ModuleManager, module_idx: usize, area: Rect, buf: &mut Buffer) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(1),     // Content (template renders here via Frame)
            Constraint::Length(3),  // Help
        ])
        .split(area);

    let module = match module_manager.get_modules().get(module_idx) {
        Some(m) => m,
        None => return,
    };

    // Title with module info
    let icon = match module.config.module_type.as_str() {
        "com" => "📌",
        "entertainment" => "🎮",
        "knowledge" => "📚",
        "llm" => "🤖",
        "monitoring" => "📊",
        _ => "⚙️",
    };

    let title = Paragraph::new(format!(
        "{} {} - {} [{}]",
        icon,
        module.config.name,
        module.config.module_type,
        module.config.template
    ))
        .block(
            Block::bordered()
                .title("Module Details")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Green)
        .alignment(Alignment::Center);
    title.render(main_layout[0], buf);

    // Help
    let help = Paragraph::new("SHIFT + Backspace: Back to overview  • SHIFT + Esc: Quit")
        .block(
            Block::bordered()
                .title("Controls")
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Yellow)
        .alignment(Alignment::Center);
    help.render(main_layout[2], buf);
}

/// Returns the content area rect for template rendering
pub fn get_content_area(area: Rect) -> Rect {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(1),     // Content
            Constraint::Length(3),  // Help
        ])
        .split(area);

    main_layout[1]
}
