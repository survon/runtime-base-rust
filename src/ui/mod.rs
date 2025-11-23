pub mod document;
pub mod template;
pub mod screens;
pub mod style;


use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};
use crate::app::{App, AppMode};
use crate::ui::screens::overview::render_overview;

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &self.mode {
            AppMode::Splash => {},
            AppMode::Overview => render_overview(self, area, buf),
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
