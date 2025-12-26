use super::{ConfigEditor, EditorField, FieldValue};

impl ConfigEditor {
    pub(in crate::module) fn add_com_defaults(&mut self) {
        self.fields.push((
            "Label".to_string(),
            EditorField::Label,
            FieldValue::Text("Device Control".to_string()),
        ));

        self.fields.push((
            "On Label".to_string(),
            EditorField::ToggleOnLabel,
            FieldValue::Text("On".to_string()),
        ));

        self.fields.push((
            "Off Label".to_string(),
            EditorField::ToggleOffLabel,
            FieldValue::Text("Off".to_string()),
        ));

        self.fields.push((
            "Description".to_string(),
            EditorField::Description,
            FieldValue::Text("Remote device control".to_string()),
        ));

        self.fields.push((
            "State".to_string(),
            EditorField::CustomBinding { key: "state".to_string() },
            FieldValue::Bool(false),
        ));
    }
}
