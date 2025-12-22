// src/widgets/module_detail/widget.rs
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget},
};
use ratatui::prelude::Style;
use crate::modules::ModuleManager;
use crate::ui::style::dim_unless_focused;

#[derive(Debug)]
pub struct ModuleDetailWidget {
    // No state machine needed - purely presentational
}

impl ModuleDetailWidget {
    pub fn new() -> Self {
        Self {}
    }

    /// Renders the title and help sections (the "chrome" around the template content)
    pub fn render_chrome(
        &self,
        module_manager: &ModuleManager,
        module_idx: usize,
        area: Rect,
        buf: &mut Buffer
    ) -> Block {
        let is_focused = Some(true); // TODO inject as param

        let border_style = dim_unless_focused(is_focused, Style::default().fg(Color::Yellow));

        let module = match module_manager.get_modules().get(module_idx) {
            Some(m) => m,
            None => return Default::default(),
        };

        // Title with module info
        // TODO infer this from an enum
        let icon = match module.config.module_type.as_str() {
            "com" => "🔌",
            "entertainment" => "🎮",
            "knowledge" => "📚",
            "control" => "🤖",
            "monitoring" => "📊",
            "system" => "⚙️",
            _ => "🤷🏻‍♂️️",
        };

        let title = format!("{} {} - Press [Esc] To Close Module Window", icon, module.config.name);

        let container = Block::bordered()
            .title(title)
            .style(border_style)
            .border_type(BorderType::Rounded);

        let response = container.clone();

        container.render(area, buf);

        response
    }

    /// Returns the content area rect for template rendering
    pub fn get_content_area(&self, area: Rect) -> Rect {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),     // Content
                Constraint::Length(3),  // Help
            ])
            .split(area);

        main_layout[0]
    }
}
