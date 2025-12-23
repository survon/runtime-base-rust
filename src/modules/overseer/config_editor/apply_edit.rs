use super::{ConfigEditor, FieldValue};

impl ConfigEditor {
    pub(super) fn apply_edit(&mut self) {
        if let Some((_, _, value)) = self.fields.get_mut(self.selected_field) {
            match value {
                FieldValue::Text(_) => {
                    *value = FieldValue::Text(self.edit_buffer.clone());
                }
                FieldValue::Number(_) => {
                    if let Ok(num) = self.edit_buffer.parse::<f64>() {
                        *value = FieldValue::Number(num);
                    }
                }
                _ => {}
            }
        }
        self.edit_buffer.clear();
    }
}
