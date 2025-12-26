use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{Style, Widget},
    widgets::{Block, BorderType},
};

use crate::module::Module;
use crate::ui::components::UiComponent;
use crate::util::string::StringUtils;

use super::{ViewData, OverseerCard};

impl OverseerCard {
    pub(super) fn render_overview_cta(
        &self,
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        module: &mut Module,
    ) {
        let title = format!(" 🗓️️ {} ", &module.config.name);

        let ViewData {
            border_color,
            status_message,
            is_scanning,
            scan_countdown,
            known_count,
            installed_count,
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
            .constraints(if has_status || is_scanning {
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
            "Configure Manifests & Peripherals\n---\n[{}]\n[{}]",
            StringUtils::maybe_pluralize_count(installed_count, ("Module", "Modules")),
            StringUtils::maybe_pluralize_count(known_count, ("Device", "Devices")),
        );
        let overview_component = UiComponent::empty_message(overview_text.as_str(), None);
        Widget::render(overview_component, chunks[0], buf);

        if is_scanning {
            let is_scanning_component = UiComponent::is_scanning(&scan_countdown);
            Widget::render(is_scanning_component, chunks[1], buf);
        } else if has_status {
            if let Some(status) = status_message {
                let status_component = UiComponent::status(status);
                Widget::render(status_component, chunks[1], buf);
            }
        }
    }
}
