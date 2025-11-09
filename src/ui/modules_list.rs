use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect, Layout, Direction},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph, Widget, Wrap},
};
use crate::app::App;
use crate::module::Module;

const MODULES_PER_ROW: usize = 3;

pub fn render_modules_list(app: &App, area: Rect, buf: &mut Buffer) {
    let modules = app.module_manager.get_modules();

    if modules.is_empty() {
        let empty_msg = Paragraph::new("No modules found.\n\nPlace module directories in:\n./wasteland/modules/\n\nEach directory should contain a config.yml file.")
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
        .title(format!("Modules ({})", modules.len()))
        .border_type(BorderType::Rounded);
    let inner_area = container.inner(area);
    container.render(area, buf);

    // Calculate grid layout
    let num_rows = (modules.len() + MODULES_PER_ROW - 1) / MODULES_PER_ROW;

    // Create row constraints - each row gets equal space
    let row_constraints: Vec<Constraint> = (0..num_rows)
        .map(|_| Constraint::Length(8)) // Height of each module box
        .collect();

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(inner_area);

    // Render each row
    for (row_idx, row_area) in rows.iter().enumerate() {
        let start_idx = row_idx * MODULES_PER_ROW;
        let end_idx = (start_idx + MODULES_PER_ROW).min(modules.len());
        let modules_in_row = &modules[start_idx..end_idx];

        // Create column constraints
        let col_constraints: Vec<Constraint> = (0..modules_in_row.len())
            .map(|_| Constraint::Percentage(100 / MODULES_PER_ROW as u16))
            .collect();

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(*row_area);

        // Render each module box in this row
        for (col_idx, module) in modules_in_row.iter().enumerate() {
            let module_idx = start_idx + col_idx;
            render_module_box(module, module_idx, app.module_manager.selected_module, cols[col_idx], buf);
        }
    }
}

fn render_module_box(module: &Module, index: usize, selected_index: usize, area: Rect, buf: &mut Buffer) {
    let is_selected = index == selected_index;

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

    // Metadata section - placeholder for future enhancements
    // This area can be used to display charts, real-time data, etc.
    if sections[2].height > 0 {
        let metadata_text = "..."; // Placeholder for module-specific data
        let metadata_line = Line::from(Span::styled(
            metadata_text,
            Style::default().fg(Color::DarkGray)
        ));
        Paragraph::new(metadata_line)
            .alignment(Alignment::Center)
            .render(sections[2], buf);
    }
}
