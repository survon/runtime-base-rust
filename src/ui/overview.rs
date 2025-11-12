use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Paragraph, Widget},
};
use crate::app::App;
use crate::ui::{messages, modules_list};

pub fn render_overview(app: &mut App, area: Rect, buf: &mut Buffer) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(1),     // Content
            Constraint::Length(3),  // Status/Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("🏠 Survon - Smart Homestead OS")
        .block(
            Block::bordered()
                .title("Survon")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Green)
        .alignment(Alignment::Center);
    title.render(main_layout[0], buf);

    // Content area split between modules and messages
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),  // Modules
            Constraint::Percentage(40),  // Messages
        ])
        .split(main_layout[1]);

    // Render modules list with templates
    // Pass true to render actual templates, false for metadata cards
    modules_list::render_modules_list(app, content_layout[0], buf, true);

    // Render recent messages
    messages::render_recent_messages_panel(content_layout[1], buf);

    // Help text
    let help_text = if app.module_manager.get_modules().is_empty() {
        "No modules found. Press 'r' to refresh • 'o' for LLM setup • 'q' to quit"
    } else if app.get_llm_engine().is_some() {
        "←/→: Navigate • Enter: Select • 'c': Chat • 'o': LLM Setup • '1': Close Gate • 'r': Refresh • 'q': Quit"
    } else {
        "←/→: Navigate • Enter: Select • 'o': LLM Setup • '1': Close Gate • 'r': Refresh • 'q': Quit"
    };

    let help = Paragraph::new(help_text)
        .block(
            Block::bordered()
                .title("Controls")
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Yellow)
        .alignment(Alignment::Center);
    help.render(main_layout[2], buf);


}
