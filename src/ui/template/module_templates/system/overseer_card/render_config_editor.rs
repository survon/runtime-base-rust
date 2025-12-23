use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{Color, Modifier, Style, Widget},
    widgets::{Block, Borders},
};

use crate::modules::Module;
use super::OverseerCard;

impl OverseerCard {
    pub(super) fn render_config_editor(
        &self,
        area: Rect,
        buf: &mut Buffer,
        module: &mut Module,
    ) {
        // Extract editor state from bindings
        let module_name = module.config.bindings
            .get("editor_module_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        let selected_field = module.config.bindings
            .get("editor_selected_field")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let is_editing = module.config.bindings
            .get("editor_is_editing")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let fields = module.config.bindings
            .get("editor_fields")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let edit_buffer = if is_editing {
            module.config.bindings
                .get("editor_edit_buffer")
                .and_then(|v| v.as_str())
                .unwrap_or("")
        } else {
            ""
        };

        let cursor_pos = module.config.bindings
            .get("editor_cursor_pos")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        // Render the editor
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(format!(" Edit: {} ", module_name));

        let inner = block.inner(area);
        block.render(area, buf);

        // Split into two columns if space permits
        let (left_area, right_area) = if inner.width > 80 {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(inner);
            (chunks[0], chunks[1])
        } else {
            (inner, Rect::default())
        };

        // Render fields
        let mut y = left_area.y;
        let max_y = left_area.bottom();

        for (idx, field) in fields.iter().enumerate() {
            if y >= max_y {
                break;
            }

            let is_selected = idx == selected_field;
            let is_editing_this = is_selected && is_editing;

            let label = field.get("label").and_then(|v| v.as_str()).unwrap_or("");
            let display_value = field.get("display_value").and_then(|v| v.as_str()).unwrap_or("");
            let value_type = field.get("value_type").and_then(|v| v.as_str()).unwrap_or("text");

            let style = if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Render label
            let label_text = format!("{:20}", label);
            buf.set_string(left_area.x, y, &label_text, style);

            // Render value
            let value_x = left_area.x + 22;

            if is_editing_this {
                // Show edit buffer with cursor
                let display = if cursor_pos < edit_buffer.len() {
                    format!("{}│{}",
                            &edit_buffer[..cursor_pos],
                            &edit_buffer[cursor_pos..])
                } else {
                    format!("{}│", edit_buffer)
                };
                buf.set_string(value_x, y, &display, Style::default().fg(Color::Green));
            } else {
                let display = match value_type {
                    "bool" => {
                        let bool_val = field.get("bool_value").and_then(|v| v.as_bool()).unwrap_or(false);
                        if bool_val { "[X] true" } else { "[ ] false" }
                    }
                    "enum" => {
                        &format!("< {} >", display_value)
                    }
                    _ => display_value,
                };

                buf.set_string(value_x, y, display, style);
            }

            y += 1;
        }

        // Help text
        if right_area.width > 0 {
            let help_text = if is_editing {
                vec![
                    "Editing mode:",
                    "",
                    "[Ent]   - Save",
                    "[←]/[→] - Move cursor",
                    "[Esc]   - Cancel",
                ]
            } else {
                vec![
                    "Navigation:",
                    "",
                    "[↑]/[↓] - Select field",
                    "[Ent]   - Edit text",
                    "[Spc]   - Edit text",
                    "[←]/[→] - Toggle bool/enum",
                    "[s]     - Save config",
                    "[Esc]   - Close editor",
                ]
            };

            let mut help_y = right_area.y;
            for line in help_text {
                buf.set_string(
                    right_area.x + 2,
                    help_y,
                    line,
                    Style::default().fg(Color::DarkGray),
                );
                help_y += 1;
            }
        } else {
            // Show abbreviated help at bottom
            let help = if is_editing {
                "[Ent] Save  [Esc] Cancel"
            } else {
                "[↑]/[↓] Select  [Ent] Edit  [←]/[→] Toggle  [s] Save  [Esc] Close"
            };

            buf.set_string(
                area.x + 2,
                area.bottom().saturating_sub(1),
                help,
                Style::default().fg(Color::DarkGray),
            );
        }
    }
}
