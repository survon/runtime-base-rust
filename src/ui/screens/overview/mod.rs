use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Stylize}
    ,
    widgets::{Block, BorderType, Paragraph, Widget},
};
use crate::app::{App, OverviewFocus};

pub mod messages;
pub mod modules_list;

pub fn render_overview(app: &mut App, area: Rect, buf: &mut Buffer) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(1),     // Content
            Constraint::Length(3),  // Status/Help
        ])
        .split(area);

    // Title
    let title = Paragraph::new("🏠 Survon - Smart Homestead OS")
        .block(
            Block::bordered()
                .title("Survon")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Green)
        .alignment(Alignment::Center);
    title.render(main_layout[0], buf);

    let wasteland_modules_cell_constraints = Constraint::Percentage(40);
    let messages_cell_constraints = Constraint::Percentage(20);
    let core_modules_cell_constraints = Constraint::Percentage(40);

    // Content area split between modules and messages
    let content_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            wasteland_modules_cell_constraints,
            messages_cell_constraints,
            core_modules_cell_constraints,
        ])
        .split(main_layout[1]);

    let should_use_template = true;
    let is_none_focused = matches!(app.overview_focus, OverviewFocus::None);
    let is_wasteland_modules_list_focused = matches!(app.overview_focus, OverviewFocus::WastelandModules);
    let is_core_modules_list_focused = matches!(app.overview_focus, OverviewFocus::CoreModules);
    let is_messages_focused = matches!(app.overview_focus, OverviewFocus::Messages);

    // Render wasteland modules
    let mut needs_redraw = false;

    {
        let is_focused: Option<bool> = if is_wasteland_modules_list_focused {
            Some(true)
        } else if is_none_focused{
            None
        } else {
            Some(false)
        };
        modules_list::render_modules_list(
            &mut app.wasteland_module_manager,
            content_layout[0],
            buf,
            should_use_template,
            is_focused,
            &mut needs_redraw
        );
    }


    if needs_redraw {
        app.request_redraw();
    }

    // Render messages panel
    {
        let is_focused: Option<bool> = if is_messages_focused {
            Some(true)
        } else if is_none_focused{
            None
        } else {
            Some(false)
        };
        app.messages_panel.render(content_layout[1], buf, is_focused);
    }

    // Render core modules
    {
        let is_focused: Option<bool> = if is_core_modules_list_focused {
            Some(true)
        } else if is_none_focused{
            None
        } else {
            Some(false)
        };
        modules_list::render_modules_list(
            &mut app.core_module_manager,
            content_layout[2],
            buf,
            should_use_template,
            is_focused,
            &mut needs_redraw
        );
    }

    let wasteland_help_text: &str = {
        if app.wasteland_module_manager.get_modules().is_empty() {
            "No wasteland modules found."
        } else {
            "←/→: Navigate Wasteland Modules"
        }
    };

    let focus_hint = match app.overview_focus {
        OverviewFocus::None => "Tab: Focus Wasteland Modules".to_string(),
        OverviewFocus::WastelandModules => format!("{} • Tab: Focus Messages", wasteland_help_text),
        OverviewFocus::Messages => "↑/↓: Scroll Messages • Tab: Focus Core Modules".to_string(),
        OverviewFocus::CoreModules => "←/➔: Navigate Core Modules • Tab: Remove Overview Focus".to_string(),
    };

    let help_text = format!("{} • Enter: Select • 'r': Refresh • 'q': Quit", focus_hint);

    let help = Paragraph::new(help_text)
        .block(
            Block::bordered()
                .title("Controls")
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Yellow)
        .alignment(Alignment::Center);
    help.render(main_layout[2], buf);
}
