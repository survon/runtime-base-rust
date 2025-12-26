use super::{ConfigEditor, FieldValue};

impl ConfigEditor {
    pub(in crate::module) fn start_editing(&mut self) {
        if let Some((_, _, value)) = self.fields.get(self.selected_field) {
            match value {
                FieldValue::Text(_) | FieldValue::Number(_) => {
                    self.edit_buffer = value.as_display_string();
                    self.cursor_pos = self.edit_buffer.len();
                    self.is_editing = true;
                }
                FieldValue::Bool(_) | FieldValue::Enum { .. } => {
                    // These are toggled with arrow keys, not edited
                }
            }
        }
    }
}
