use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect, Layout, Direction},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};
use std::time::Duration;

use crate::log_error;
use crate::modules::{Module, ModuleManager};
use crate::ui::style::{dim_unless_focused};

const MODULES_PER_ROW: usize = 3;

pub fn render_modules_list(
    module_manager: &mut ModuleManager,
    area: Rect,
    buf: &mut Buffer,
    use_template: bool,
    is_focused: bool,
    request_redraw: &mut bool
) {
    let selected_idx = module_manager.selected_module;
    let modules_count = module_manager.get_modules().len();
    let displayable_count = module_manager.get_displayable_modules().len();

    let text_style = dim_unless_focused(is_focused, Style::default().fg(Color::White));
    let border_style = dim_unless_focused(is_focused, Style::default().fg(Color::Yellow));

    let title_namespace_prefix = match module_manager.namespace.as_str() {
        "core" => "Core ".to_owned(),
        "wasteland" => "Wasteland ".to_owned(),
        _ => "".to_owned(),
    };

    // Handle empty case
    if displayable_count == 0 {
        let empty_msg = Paragraph::new("No displayable modules found.\n\nPlace module directories in the configured path.\n\nEach directory should contain a config.yml file with:\n  - name\n  - module_type\n  - template\n  - bindings\n\nNote: Knowledge modules don't need templates.")
            .block(
                Block::bordered()
                    .title(title_namespace_prefix + "Modules")
                    .border_type(BorderType::Rounded)
                    .style(border_style)
            )
            .fg(Color::Red)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        empty_msg.render(area, buf);
        return;
    }

    // Create main container
    let container = Block::bordered()
        .title(format!("{}Modules ({}/{} 👁️)", title_namespace_prefix, displayable_count, modules_count))
        .style(border_style)
        .border_type(BorderType::Rounded);
    let inner_area = container.inner(area);
    container.render(area, buf);

    // Calculate grid layout
    let num_rows = (displayable_count + MODULES_PER_ROW - 1) / MODULES_PER_ROW;
    let row_constraints: Vec<Constraint> = (0..num_rows)
        .map(|_| Constraint::Length(8))
        .collect();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(inner_area);

    let blink_interval = Duration::from_millis(500);

    // Build a mapping: displayable_idx -> actual_module_idx
    let mut displayable_to_actual: Vec<usize> = Vec::new();
    for (actual_idx, module) in module_manager.get_modules().iter().enumerate() {
        if !module.config.template.is_empty() {
            displayable_to_actual.push(actual_idx);
        }
    }

    // Render each row
    for (row_idx, row) in rows.iter().enumerate() {
        let start_idx = row_idx * MODULES_PER_ROW;
        let end_idx = (start_idx + MODULES_PER_ROW).min(displayable_count);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100 / 3); 3])
            .split(*row);

        for col_idx in 0..3 {
            let displayable_idx = start_idx + col_idx;
            if displayable_idx >= displayable_count { break; }

            // Get the actual module index from our mapping
            let actual_module_idx = displayable_to_actual[displayable_idx];
            let col_area = cols[col_idx];

            // Check if we need to update blink for this module
            let needs_redraw = {
                let modules = module_manager.get_modules_mut();
                if let Some(module) = modules.get_mut(actual_module_idx) {
                    module.config.is_blinkable() && module.render_state.update_blink(blink_interval)
                } else {
                    false
                }
            };

            if needs_redraw {
                *request_redraw = true;
            }

            // Check if this module is selected
            let is_selected = actual_module_idx == selected_idx;

            // Render the module
            let modules = module_manager.get_modules_mut();
            if let Some(module) = modules.get_mut(actual_module_idx) {
                render_module_box(
                    module,
                    is_selected,
                    col_area,
                    buf,
                    use_template,
                );
            }
        }
    }
}

fn render_module_box(module: &mut Module, is_selected: bool, area: Rect, buf: &mut Buffer, use_template: bool) {
    // If use_template is true and module has a template, render it directly
    if use_template && ModuleManager::is_displayable_module(module) {
        if let Err(e) = module.render(is_selected, area, buf) {
            // If template fails, fall back to metadata view
            log_error!("Template render failed: {}", e);
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
