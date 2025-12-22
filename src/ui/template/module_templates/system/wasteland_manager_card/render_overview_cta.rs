use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{Style, Widget},
    widgets::{Block, BorderType},
};

use crate::modules::Module;
use crate::ui::template::module_templates::system::wasteland_manager_card::{ViewData, WastelandManagerCard};

impl WastelandManagerCard {
    pub(in crate::ui::template::module_templates::system::wasteland_manager_card) fn render_overview_cta(
        &self,
        is_selected: bool,
        area: Rect,
        buf: &mut Buffer,
        module: &mut Module,
    ) {
        let title = format!("⚙️ {}", &module.config.name);

        let ViewData {
            border_color,
            status_message,
            is_scanning,
            scan_countdown,
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

        let overview_text = "Manage Modules & Devices";
        let overview_component = self._make_empty_message_component(overview_text, None);
        Widget::render(overview_component, chunks[0], buf);

        if is_scanning {
            let is_scanning_component = self._make_is_scanning_component(&scan_countdown);
            Widget::render(is_scanning_component, chunks[1], buf);
        } else if has_status {
            if let Some(status) = status_message {
                let status_component = self._make_status_component(status);
                Widget::render(status_component, chunks[1], buf);
            }
        }
    }
}
