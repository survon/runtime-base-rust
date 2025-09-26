use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, Paragraph, Widget, Wrap},
};
use crate::app::App;
use crate::module::Module;

pub fn render_modules_list(app: &App, area: Rect, buf: &mut Buffer) {
    let modules = app.get_modules();

    if modules.is_empty() {
        let empty_msg = Paragraph::new("No modules found.\n\nPlace module directories in:\n./wasteland/modules/\n\nEach directory should contain a config.yml file.")
            .block(
                Block::bordered()
                    .title("Modules")
                    .border_type(BorderType::Rounded)
            )
            .fg(Color::Red)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        empty_msg.render(area, buf);
        return;
    }

    let items: Vec<ListItem> = modules
        .iter()
        .enumerate()
        .map(|(i, module)| {
            let style = if i == app.selected_module {
                Style::default().bg(Color::Blue).fg(Color::White)
            } else {
                Style::default()
            };

            let icon = match module.config.module_type.as_str() {
                "com" => "🔌",
                "entertainment" => "🎮",
                "knowledge" => "📚",
                "llm" => "🤖",
                "monitoring" => "📊",
                _ => "⚙️",
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default()),
                Span::styled(
                    format!("{} ", module.config.name),
                    style.add_modifier(Modifier::BOLD)
                ),
                Span::styled(
                    format!("({})", module.config.module_type),
                    Style::default().fg(Color::Gray)
                ),
            ]);

            ListItem::new(line).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(format!("Modules ({})", modules.len()))
                .border_type(BorderType::Rounded)
        )
        .highlight_style(Style::default().bg(Color::Blue));

    list.render(area, buf);
}
