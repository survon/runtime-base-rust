#[allow(unused_imports)] /// Needed for Stylize in this scope
use ratatui::style::{Style, Stylize};
use ratatui::style::Color;

pub fn dim_unless_focused(is_focused: Option<bool>, style: Style) -> Style {
    match is_focused {
        Some(true) => {
            style.bold()
        },
        Some(false) => {
            style.dim().italic()
        }
        None => style
    }
}
