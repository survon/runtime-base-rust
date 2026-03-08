pub mod document;
pub mod screens;
pub mod style;
pub mod template;
pub mod widgets;

mod components;

use crate::{
    app::{App, AppMode},
    ui::screens::council::CouncilScreen,
    ui::screens::overview::render_overview,
};
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &self.mode {
            AppMode::Splash => {}
            AppMode::Overview => render_overview(self, area, buf),
            AppMode::Council => {
                CouncilScreen::render(self, area, buf);
            }
            AppMode::ModuleDetail(_source, _module_idx) => {}
        }
    }
}
