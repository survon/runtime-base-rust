use std::time;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect, Layout, Direction},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};
use std::time::Duration;

use crate::app::App;
use crate::module::Module;

const MODULES_PER_ROW: usize = 3;

pub fn render_modules_list(app: &mut App, area: Rect, buf: &mut Buffer, use_template: bool) {
    let selected_idx = app.module_manager.selected_module;

    let modules_count: usize = app.module_manager.get_modules()
        .iter()
        .count();

    let displayable_count: usize = app.module_manager.get_modules()
        .iter()
        .filter(|m| !m.config.template.is_empty())
        .count();

    if displayable_count == 0 {
        let empty_msg = Paragraph::new("No displayable modules found.\n\nPlace module directories in:\n./wasteland/modules/\n\nEach directory should contain a config.yml file with:\n  - name\n  - module_type\n  - template\n  - bindings\n\nNote: Knowledge modules don't need templates.")
            .block(
                Block::bordered()
                    .title("Modules")
                    .border_type(BorderType::Rounded)
            )
            .fg(Color::Red)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        empty_msg.render(area, buf);
        return;
    }

    // Create main container
    let container = Block::bordered()
        .title(format!("Modules ({} displayable, {} total)", displayable_count, modules_count))
        .border_type(BorderType::Rounded);
    let inner_area = container.inner(area);
    container.render(area, buf);

    // Calculate grid layout
    let num_rows = (displayable_count + MODULES_PER_ROW - 1) / MODULES_PER_ROW;

    // Create row constraints - each row gets equal space
    let row_constraints: Vec<Constraint> = (0..num_rows)
        .map(|_| Constraint::Length(8)) // Height of each module box
        .collect();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(inner_area);

    // Render each row
    for (row_idx, row) in rows.iter().enumerate() {
        let start_idx = row_idx * MODULES_PER_ROW;
        let end_idx = (start_idx + MODULES_PER_ROW).min(displayable_count);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100 / 3); 3])
            .split(*row);

        for (col_idx, col_area) in cols.iter().enumerate().take(3) {
            let module_idx = start_idx + col_idx;
            if module_idx >= displayable_count { break; }

            let blink_interval = Duration::from_millis(500);

            // Find the actual module index first (immutable borrow)
            let mut displayable_idx = 0;
            let mut actual_module_idx = 0;

            for (i, module) in app.module_manager.get_modules().iter().enumerate() {
                if !module.config.template.is_empty() {
                    if displayable_idx == module_idx {
                        actual_module_idx = i;
                        break;
                    }
                    displayable_idx += 1;
                }
            }

            // Check if we need to update blink and request redraw
            let needs_redraw = {
                let modules = app.module_manager.get_modules_mut();
                if let Some(module) = modules.get_mut(actual_module_idx) {
                    module.config.is_blinkable() && module.render_state.update_blink(blink_interval)
                } else {
                    false
                }
            }; // Mutable borrow ends here

            if needs_redraw {
                app.request_redraw();
            }

            // Now get another mutable borrow for rendering
            let modules = app.module_manager.get_modules_mut();
            let is_selected = module_idx == selected_idx;
            if let Some(module) = modules.get_mut(actual_module_idx) {
                render_module_box(
                    module,
                    is_selected,
                    *col_area,
                    buf,
                    use_template,
                );
            }
        }
    }
}

fn render_module_box(module: &mut Module, is_selected: bool, area: Rect, buf: &mut Buffer, use_template: bool) {
    // If use_template is true and module has a template, render it directly
    if use_template && !module.config.template.is_empty() {

        if let Err(e) = module.render(is_selected, area, buf) {
            // If template fails, fall back to metadata view
            eprintln!("Template render failed: {}", e);
            render_metadata_card(module, is_selected, area, buf);
        }
        return;
    }

    // Otherwise render metadata card
    render_metadata_card(module, is_selected, area, buf);
}

fn render_metadata_card(module: &Module, is_selected: bool, area: Rect, buf: &mut Buffer) {
    let border_style = if is_selected {
        Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let icon = match module.config.module_type.as_str() {
        "com" => "📌",
        "entertainment" => "🎮",
        "knowledge" => "📚",
        "llm" => "🤖",
        "monitoring" => "📊",
        _ => "⚙️",
    };

    // Create the box
    let block = Block::bordered()
        .border_type(if is_selected { BorderType::Double } else { BorderType::Rounded })
        .style(border_style);

    let inner_area = block.inner(area);
    block.render(area, buf);

    if inner_area.height < 3 {
        return; // Not enough space to render content
    }

    // Split inner area into sections
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Icon and title
            Constraint::Length(1), // Module type
            Constraint::Length(1), // Template name
            Constraint::Min(1),     // Additional metadata space
        ])
        .split(inner_area);

    // Render icon and title
    let title_style = if is_selected {
        Style::default().add_modifier(Modifier::BOLD).fg(Color::White)
    } else {
        Style::default()
    };

    let title_line = Line::from(vec![
        Span::styled(format!("{} ", icon), Style::default()),
        Span::styled(&module.config.name, title_style),
    ]);
    Paragraph::new(title_line)
        .alignment(Alignment::Center)
        .render(sections[0], buf);

    // Render module type
    let type_line = Line::from(vec![
        Span::styled(
            format!("({})", module.config.module_type),
            Style::default().fg(Color::DarkGray)
        ),
    ]);
    Paragraph::new(type_line)
        .alignment(Alignment::Center)
        .render(sections[1], buf);

    // Render template name
    let template_line = Line::from(vec![
        Span::styled(
            format!("[{}]", module.config.template),
            Style::default().fg(Color::Cyan)
        ),
    ]);
    Paragraph::new(template_line)
        .alignment(Alignment::Center)
        .render(sections[2], buf);

    // Metadata section - could show binding count or status
    if sections.len() > 3 && sections[3].height > 0 {
        let binding_count = module.config.bindings.len();
        let metadata_text = format!("{} bindings", binding_count);
        let metadata_line = Line::from(Span::styled(
            metadata_text,
            Style::default().fg(Color::DarkGray)
        ));
        Paragraph::new(metadata_line)
            .alignment(Alignment::Center)
            .render(sections[3], buf);
    }
}
