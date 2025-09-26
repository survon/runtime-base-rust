use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Clear, Paragraph, Widget, Wrap},
};
use crate::app::{App, AppMode};
use crate::module::Module;
use std::path::Path;
use tui_scrollview::{ScrollView, ScrollViewState};
use ratatui::widgets::StatefulWidget;
use ratatui::prelude::Size;

// Re-export submodules
pub mod overview;
pub mod modules_list;
pub mod messages;
pub mod module_detail;
pub mod monitoring;
pub mod com;
pub mod entertainment;
pub mod knowledge;
pub mod llm;
pub mod llm_chat;
pub mod chat_history;
pub mod document_viewer;
pub mod document_popup_widget;
pub mod external_viewer;

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &self.mode {
            AppMode::Overview => overview::render_overview(self, area, buf),
            AppMode::ModuleDetail(module_idx) => {
                if let Some(module) = self.get_modules().get(*module_idx) {
                    module_detail::render_module_detail(self, module, area, buf);
                } else {
                    overview::render_overview(self, area, buf);
                }
            }
            AppMode::LlmChat(module_idx) => {
                if let Some(module) = self.get_modules().get(*module_idx) {
                    llm_chat::render_llm_chat(self, module, area, buf);
                } else {
                    overview::render_overview(self, area, buf);
                }
            }
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
