use serde_json::Value;

use super::FieldValue;

impl FieldValue {
    pub fn to_json(&self) -> Value {
        match self {
            Self::Text(s) => Value::String(s.clone()),
            Self::Number(n) => serde_json::json!(n),
            Self::Bool(b) => Value::Bool(*b),
            Self::Enum { options, selected } => {
                Value::String(options.get(*selected).cloned().unwrap_or_default())
            }
        }
    }
}
