use super::FieldValue;

impl FieldValue {
    pub fn as_display_string(&self) -> String {
        match self {
            Self::Text(s) => s.clone(),
            Self::Number(n) => format!("{}", n),
            Self::Bool(b) => if *b { "true".to_string() } else { "false".to_string() },
            Self::Enum { options, selected } => {
                options.get(*selected).cloned().unwrap_or_default()
            }
        }
    }
}
