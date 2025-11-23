#[allow(unused_imports)] /// Needed for Stylize in this scope
use ratatui::style::{Style, Stylize};
use ratatui::style::Color;

pub fn dim_unless_focused(is_focused: bool, style: Style) -> Style {
    if is_focused { style.bold() } else { style.dim().italic() }
}
