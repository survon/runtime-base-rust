use super::{ConfigEditor, EditorField, FieldValue};

impl ConfigEditor {
    pub(super) fn add_llm_defaults(&mut self) {
        let model_options = vec!["search".to_string(), "summarizer".to_string(), "council".to_string()];
        self.fields.push((
            "Model".to_string(),
            EditorField::Model,
            FieldValue::Enum { options: model_options, selected: 0 },
        ));
    }
}
