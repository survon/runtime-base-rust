#[allow(unused_imports)] /// Needed for Stylize in this scope
use ratatui::style::{Style, Stylize};

pub fn dim_unless_focused(is_focused: bool, style: Style) -> Style {
    if is_focused { style } else { style.dim() }
}
