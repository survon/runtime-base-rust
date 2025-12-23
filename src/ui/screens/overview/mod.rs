// src/ui/screens/overview/mod.rs
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Stylize, Style},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
    text::Line,
};
use crate::app::{App, OverviewFocus};
use crate::modules::ModuleManagerView;

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

    let is_none_focused = matches!(app.overview_focus, OverviewFocus::None);
    let is_jukebox_focused = matches!(app.overview_focus, OverviewFocus::Jukebox);
        let is_wasteland_modules_list_focused = matches!(app.overview_focus, OverviewFocus::WastelandModules);
    let is_wasteland_modules_list_view = matches!(app.wasteland_module_manager.current_view, ModuleManagerView::ModuleListView);
    let is_core_modules_list_focused = matches!(app.overview_focus, OverviewFocus::CoreModules);
    let is_core_modules_list_view = matches!(app.core_module_manager.current_view, ModuleManagerView::ModuleListView);
    let is_messages_focused = matches!(app.overview_focus, OverviewFocus::Messages);

    // Render title
    let title = Paragraph::new("🏡 Survon - Smart Homestead OS")
        .block(
            Block::bordered()
                .title(" Survon ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
        )
        .style(Style::default().fg(Color::Green))
        .alignment(Alignment::Center);
    title.render(header_layout[0], buf);

    // Render jukebox
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

        if is_wasteland_modules_list_view {
            app.modules_list_widget.render(
                &mut app.wasteland_module_manager,
                content_layout[0],
                buf,
                is_focused,
                &mut needs_redraw
            );
        } else {
            let selected_module_index = app.wasteland_module_manager.selected_module;

            // Render chrome (header/footer)
            let container = app.module_detail_widget.render_chrome(
                &app.wasteland_module_manager,
                selected_module_index,
                is_focused,
                content_layout[0],
                buf
            );

            let inner_area = container.inner(content_layout[0]);

            // Update bindings for this module
            app.wasteland_module_manager.update_module_bindings(selected_module_index);

            // Render the module's template content
            if let Some(module) = app.wasteland_module_manager.get_modules_mut().get_mut(selected_module_index) {
                if let Err(e) = module.render_detail(inner_area, buf) {
                    render_template_error(inner_area, buf, e);
                }
            }
        }
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
        if let Some(messages) = &mut app.messages_widget {
            messages.render(content_layout[1], buf, is_focused);
        }
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

        if is_core_modules_list_view {
            app.modules_list_widget.render(
                &mut app.core_module_manager,
                content_layout[2],
                buf,
                is_focused,
                &mut needs_redraw
            );
        } else {
            let selected_module_index = app.core_module_manager.selected_module;

            let container = app.module_detail_widget.render_chrome(
                &app.core_module_manager,
                selected_module_index,
                is_focused,
                content_layout[2],
                buf
            );

            let inner_area = container.inner(content_layout[2]);

            // Update bindings for this module
            app.core_module_manager.update_module_bindings(selected_module_index);

            // Render the module's template content
            if let Some(module) = app.core_module_manager.get_modules_mut().get_mut(selected_module_index) {
                if let Err(e) = module.render_detail(inner_area, buf) {
                    render_template_error(inner_area, buf, e);
                }
            }
        }
    }

    let wasteland_help_text: &str = {
        if app.wasteland_module_manager.get_modules().is_empty() {
            "No wasteland modules found."
        } else if is_wasteland_modules_list_view {
            "[←]/[→] Navigate Wasteland Modules"
        } else {
            "[Esc] Back to List"
        }
    };

    let core_help_text: &str = {
        if is_core_modules_list_view {
            "[←]/[→] Navigate Core Modules"
        } else {
            "[Esc] Back to List"
        }
    };

    let focus_hint = match app.overview_focus {
        OverviewFocus::None => "[Tab] Focus Wasteland Modules".to_string(),
        OverviewFocus::WastelandModules => format!("{} [Tab] Focus Messages", wasteland_help_text),
        OverviewFocus::Messages => "[↑]/[↓] Scroll  [Tab] Focus Core Modules".to_string(),
        OverviewFocus::CoreModules => format!("{}  [Tab] Focus Jukebox", core_help_text),
        OverviewFocus::Jukebox => "[   ] ⏯  [←]/[→] ⏮/⏭  [+]/[-] 🔈  [m] Library  [Tab] Remove Overview Focus".to_string(),
    };

    let help_text = format!("{}  [Enter] Select  [r] Refresh  [q] Quit", focus_hint);

    let help = Paragraph::new(help_text)
        .block(
            Block::bordered()
                .title(" Controls ")
                .border_type(BorderType::Rounded)
        )
        .fg(Color::Yellow)
        .alignment(Alignment::Center);
    help.render(main_layout[2], buf);
}

/// Helper function to render template errors inline
fn render_template_error(area: Rect, buf: &mut Buffer, error: String) {
    let error_lines = vec![
        Line::from(""),
        Line::from("⚠️ Template Rendering Error").style(Style::default().fg(Color::Red)),
        Line::from(""),
        Line::from(error.clone()).style(Style::default().fg(Color::Yellow)),
        Line::from(""),
        Line::from("Check your module's config.yml:").style(Style::default().fg(Color::Gray)),
        Line::from("  - Is the 'template' field correct?").style(Style::default().fg(Color::Gray)),
        Line::from("  - Are all required bindings present?").style(Style::default().fg(Color::Gray)),
    ];

    let error_widget = Paragraph::new(error_lines)
        .block(
            Block::bordered()
                .title(" Error ")
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Red))
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    error_widget.render(area, buf);
}
