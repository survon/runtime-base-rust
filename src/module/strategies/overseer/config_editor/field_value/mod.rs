mod as_display_string;
mod to_json;

#[derive(Debug, Clone)]
pub enum FieldValue {
    Text(String),
    Number(f64),
    Bool(bool),
    Enum { options: Vec<String>, selected: usize },
}
