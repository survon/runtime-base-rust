#[allow(unused_imports)] /// Needed for Stylize in this scope
use ratatui::style::{Color, Style, Stylize};
use std::env;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AdaptiveColors {
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub info: Color,
    pub background: Color,
    pub foreground: Color,
    pub chart_colors: Vec<Color>,
}

impl AdaptiveColors {
    pub fn detect() -> Self {
        let term = env::var("TERM").unwrap_or_default();
        let colorterm = env::var("COLORTERM").unwrap_or_default();

        if colorterm.contains("truecolor") || colorterm.contains("24bit") {
            Self::true_color()
        } else if term.contains("256color") {
            Self::color_256()
        } else {
            Self::basic_16()
        }
    }

    pub fn true_color() -> Self {
        Self {
            primary: Color::Rgb(100, 200, 255),
            secondary: Color::Rgb(150, 100, 255),
            success: Color::Rgb(100, 255, 150),
            warning: Color::Rgb(255, 200, 100),
            danger: Color::Rgb(255, 100, 100),
            info: Color::Rgb(150, 200, 255),
            background: Color::Rgb(20, 20, 30),
            foreground: Color::Rgb(220, 220, 230),
            chart_colors: vec![
                Color::Rgb(100, 200, 255),
                Color::Rgb(255, 150, 100),
                Color::Rgb(150, 255, 150),
                Color::Rgb(255, 100, 200),
                Color::Rgb(200, 200, 100),
                Color::Rgb(150, 150, 255),
            ],
        }
    }

    pub fn color_256() -> Self {
        Self {
            primary: Color::Indexed(39),
            secondary: Color::Indexed(99),
            success: Color::Indexed(48),
            warning: Color::Indexed(214),
            danger: Color::Indexed(196),
            info: Color::Indexed(117),
            background: Color::Indexed(234),
            foreground: Color::Indexed(252),
            chart_colors: vec![
                Color::Indexed(39),
                Color::Indexed(214),
                Color::Indexed(48),
                Color::Indexed(201),
                Color::Indexed(226),
                Color::Indexed(51),
            ],
        }
    }

    pub fn basic_16() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Magenta,
            success: Color::Green,
            warning: Color::Yellow,
            danger: Color::Red,
            info: Color::Cyan,
            background: Color::Black,
            foreground: Color::White,
            chart_colors: vec![
                Color::Blue,
                Color::Yellow,
                Color::Green,
                Color::Magenta,
                Color::Cyan,
                Color::Red,
            ],
        }
    }

    /// Convert an (r,g,b) into a ratatui::style::Color following terminal capability.
    /// If truecolor supported, produce Color::Rgb. Else map to 256 / 16 as fallback.
    pub fn map_rgb_to_term_color(&self, r: u8, g: u8, b: u8) -> Color {
        // If this AdaptiveColors is using truecolor palette, return Rgb directly.
        match (&self.background, &self.foreground) {
            (Color::Rgb(_, _, _), _) | (_, Color::Rgb(_, _, _)) => {
                Color::Rgb(r, g, b)
            }
            _ => {
                // Check if we are on 256 mode: many of the colors are indexed.
                // We'll do a simple nearest mapping to xterm-256 palette.
                // Use small helper:
                rgb_to_256(r, g, b).map_or(Color::Black, |idx| Color::Indexed(idx))
            }
        }
    }
}

/// Map rgb to xterm-256 index (basic approx). Returns Option<u8>.
/// Implementation uses a simple conversion to 6x6x6 cube + grayscale mapping.
fn rgb_to_256(r: u8, g: u8, b: u8) -> Option<u8> {
    // If nearly gray, map to gray palette 232..255
    let r_f = r as f32;
    let g_f = g as f32;
    let b_f = b as f32;
    let avg = (r_f + g_f + b_f) / 3.0;
    if (r_f - avg).abs() < 5.0 && (g_f - avg).abs() < 5.0 && (b_f - avg).abs() < 5.0 {
        // grayscale range 232..255 maps 24 levels
        let level = ((avg / 255.0) * 23.0).round() as i32;
        return Some((232 + level) as u8);
    }
    // Map to 6x6x6 cube
    let conv = |x: u8| -> i32 {
        let xf = x as f32 / 255.0;
        // levels are 0,95,135,175,215,255 (equivalent to rounding *5)
        let idx = (xf * 5.0).round() as i32;
        idx
    };
    let ri = conv(r);
    let gi = conv(g);
    let bi = conv(b);
    let idx = 16 + (36 * ri + 6 * gi + bi);
    if idx >= 16 && idx <= 231 {
        Some(idx as u8)
    } else {
        None
    }
}


// Quick test program to verify colors
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_detection() {
        let colors = AdaptiveColors::detect();

        println!("Color test:");
        println!("TERM: {}", std::env::var("TERM").unwrap_or_default());
        println!("COLORTERM: {}", std::env::var("COLORTERM").unwrap_or_default());
        println!("Detected mode: {:?}", colors.primary);
    }
}

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
