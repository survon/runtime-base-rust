use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};
use crate::app::{App, AppMode};

// Re-export submodules
pub mod module_templates;

pub mod overview;
pub mod modules_list;
pub mod messages;
pub mod module_detail;
pub mod com;
pub mod entertainment;
pub mod knowledge;
pub mod chat_history;
pub mod document_viewer;
pub mod document_popup_widget;
pub mod external_viewer;
pub mod template;
pub mod splash;

pub mod style;

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &self.mode {
            AppMode::Splash => {},
            AppMode::Overview => overview::render_overview(self, area, buf),
            AppMode::ModuleDetail(_source, _module_idx) => {},
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
