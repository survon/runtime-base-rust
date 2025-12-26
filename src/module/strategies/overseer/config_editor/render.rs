use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::{Color, Modifier, Style, Widget},
    widgets::{Block, Borders},

};

use super::{ConfigEditor, FieldValue};

impl ConfigEditor {
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let title = if self.is_new_module {
            if self.fields.len() == 2 {
                " Create New Module - Select Type "
            } else {
                " Create New Module - Configure Fields "
            }
        } else {
            " Edit Module Configuration "
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(title);

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

        for (idx, (label, _field, value)) in self.fields.iter().enumerate() {
            if y >= max_y {
                break;
            }

            let is_selected = idx == self.selected_field;
            let is_editing_this = is_selected && self.is_editing;

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
            let value_width = left_area.width.saturating_sub(22);

            if is_editing_this {
                // Show edit buffer with cursor
                let display = if self.cursor_pos < self.edit_buffer.len() {
                    format!("{}│{}",
                            &self.edit_buffer[..self.cursor_pos],
                            &self.edit_buffer[self.cursor_pos..])
                } else {
                    format!("{}│", self.edit_buffer)
                };
                buf.set_string(value_x, y, &display, Style::default().fg(Color::Green));
            } else {
                let display = match value {
                    FieldValue::Bool(b) => {
                        if *b { "[X] true" } else { "[ ] false" }
                    }
                    FieldValue::Enum { options, selected } => {
                        &format!("< {} >", options.get(*selected).unwrap_or(&String::new()))
                    }
                    _ => &value.as_display_string(),
                };

                buf.set_string(value_x, y, display, style);
            }

            y += 1;
        }

        // Help text
        if right_area.width > 0 {
            let help_text = if self.is_new_module && self.fields.len() == 2 {
                vec![
                    "Create New Module:",
                    "",
                    "1. Enter module name",
                    "2. Select module type",
                    "3. Press Enter to continue",
                    "",
                    "↑/↓     - Navigate",
                    "←/→     - Change type",
                    "Enter   - Edit/Continue",
                    "Esc     - Cancel",
                ]
            } else if self.is_editing {
                vec![
                    "Editing mode:",
                    "",
                    "Enter - Save",
                    "Esc   - Cancel",
                    "←/→   - Move cursor",
                ]
            } else {
                vec![
                    "Navigation:",
                    "",
                    "↑/↓     - Select field",
                    "Enter   - Edit text",
                    "Space   - Edit text",
                    "←/→     - Toggle bool/enum",
                    "s       - Save config",
                    "Esc     - Close editor",
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
            let help = if self.is_new_module && self.fields.len() == 2 {
                "←/→: Change Type | Esc: Cancel"
            } else if self.is_editing {
                "Enter: Save | Esc: Cancel"
            } else {
                " ←/→: Toggle | s: Save | Esc: Close"
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
