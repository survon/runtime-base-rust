use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph, Widget},
};
use crate::app::{App, OverviewFocus};
use crate::util::ascii::{render_cover_ascii, paragraph_from_grid};

pub mod messages;
pub mod modules_list;

pub fn render_overview(app: &mut App, area: Rect, buf: &mut Buffer) {
    let header_constraints = Constraint::Length(10);
    let main_content_constraints = Constraint::Min(1);
    let footer_constraints = Constraint::Length(3);

    let title_constraints = Constraint::Percentage(50);
    let jukebox_constraints = Constraint::Percentage(50);

    let wasteland_modules_cell_constraints = Constraint::Percentage(40);
    let messages_cell_constraints = Constraint::Percentage(20);
    let core_modules_cell_constraints = Constraint::Percentage(40);

    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            header_constraints,
            main_content_constraints,
            footer_constraints,
        ])
        .split(area);

    let header_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            title_constraints,
            jukebox_constraints,
        ])
        .split(main_layout[0]);

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
    let is_jukebox_focused = matches!(app.overview_focus, OverviewFocus::Jukebox);
    let is_wasteland_modules_list_focused = matches!(app.overview_focus, OverviewFocus::WastelandModules);
    let is_core_modules_list_focused = matches!(app.overview_focus, OverviewFocus::CoreModules);
    let is_messages_focused = matches!(app.overview_focus, OverviewFocus::Messages);

    // Render true-color ASCII background in title area with gentle animation
    let elapsed = app.start_time.elapsed().as_secs_f32();
    let shift = elapsed * 0.03; // Very slow shift

    let title_bg_grid = render_cover_ascii(
        header_layout[0].width,
        header_layout[0].height,
        &app.palette,
        Some(shift),
        0.3, // Very dim so text is crystal clear
    );

    let title_bg_paragraph = paragraph_from_grid(&title_bg_grid);
    title_bg_paragraph.render(header_layout[0], buf);

    // Render title on top of background
    let title = Paragraph::new("🏡 Survon - Smart Homestead OS")
        .block(
            Block::bordered()
                .title("Survon")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Green)
        .alignment(Alignment::Center);
    title.render(header_layout[0], buf);

    if let Some(jukebox) = &mut app.jukebox_widget {
        let is_focused: Option<bool> = if is_jukebox_focused {
            Some(true)
        } else if is_none_focused {
            None
        } else {
            Some(false)
        };
        jukebox.render(header_layout[1], buf, is_focused);
    } else {
        let no_jukebox = Paragraph::new("Broken Jukebox")
            .fg(Color::Green)
            .alignment(Alignment::Center);
        no_jukebox.render(header_layout[1], buf);
    }

    // Render wasteland modules
    let mut needs_redraw = false;

    {
        let is_focused: Option<bool> = if is_wasteland_modules_list_focused {
            Some(true)
        } else if is_none_focused {
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
        } else if is_none_focused {
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
        } else if is_none_focused {
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
        OverviewFocus::CoreModules => "←/→: Navigate Core Modules • Tab: Remove Overview Focus".to_string(),
        _ => "".to_string(),
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
