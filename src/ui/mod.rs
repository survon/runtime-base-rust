pub mod document;
pub mod template;
pub mod screens;
pub mod style;
pub mod widgets;

mod components;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
};
use crate::{
    app::{App, AppMode},
    ui::screens::overview::render_overview
};

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &self.mode {
            AppMode::Splash => {},
            AppMode::Overview => render_overview(self, area, buf),
            AppMode::ModuleDetail(_source, _module_idx) => {},
        }
    }
}
