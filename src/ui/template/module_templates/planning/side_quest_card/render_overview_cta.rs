use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{Style, Widget},
    widgets::{Block, BorderType},
};

use crate::modules::Module;
use crate::ui::components::UiComponent;
use crate::util::string::StringUtils;
use super::{SideQuestCard, ViewData};

impl SideQuestCard {
    pub(super) fn render_overview_cta(
        &self,
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        module: &mut Module,
    ) {
        let title = format!(" ⚙️ {} ", &module.config.name);

        let ViewData {
            border_color,
            status_message,
            quest_count,
            has_status,
            ..
        } = self.get_view_data(is_selected, area, buf, module);

        let border_style = Style::default().fg(border_color);

        // Create and render the container
        let container = Block::bordered()
            .title(title)
            .style(border_style)
            .border_type(BorderType::Rounded);

        let inner_area = container.inner(area);
        container.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if has_status {
                vec![
                    Constraint::Min(1),
                    Constraint::Length(3),
                ]
            } else {
                vec![
                    Constraint::Min(1),
                ]
            })
            .split(inner_area);

        let overview_text = format!(
            "Manage Schedule Events & Tasks\n---\n[{}]",
            StringUtils::maybe_pluralize_count(quest_count, ("Open Item", "Open Items"))
        );
        let overview_component = UiComponent::empty_message(overview_text.as_str(), None);
        Widget::render(overview_component, chunks[0], buf);

        if has_status {
            if let Some(status) = status_message {
                let status_component = UiComponent::status(status);
                Widget::render(status_component, chunks[1], buf);
            }
        }
    }
}
