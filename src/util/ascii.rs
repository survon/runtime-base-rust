use ratatui::style::Color;
use ratatui::text::{Line, Span};
use crate::ui::{
    style::AdaptiveColors,
    generated::background::{BG_W, BG_H, BG_PIXELS, unpack_pixel}
};


/// Braille patterns ordered by visual density (lightest to darkest)
const BRAILLE_GRADIENT: &[char] = &[
    ' ', '⠁', '⠂', '⠄', '⠈', '⠐', '⠠',
    '⠃', '⠅', '⠆', '⠉', '⠊', '⠌', '⠑', '⠒', '⠔', '⠘', '⠡', '⠢', '⠤', '⠨',
    '⠇', '⠋', '⠍', '⠎', '⠓', '⠕', '⠖', '⠙', '⠚', '⠜', '⠣', '⠥', '⠦', '⠩', '⠪', '⠬',
    '⠏', '⠗', '⠛', '⠝', '⠞', '⠧', '⠫', '⠭', '⠮', '⠯',
    '⠟', '⠯', '⠷', '⠻', '⠽', '⠾', '⠿',
];

/// Calculate luminance from RGB values (perceived brightness)
fn luminance(r: u8, g: u8, b: u8) -> f32 {
    // Use standard luminance formula matching human perception
    0.299 * (r as f32 / 255.0) +
        0.587 * (g as f32 / 255.0) +
        0.114 * (b as f32 / 255.0)
}

/// Select Braille character based on luminance
fn char_for_luminance(lum: f32) -> char {
    let idx = (lum * (BRAILLE_GRADIENT.len() - 1) as f32) as usize;
    BRAILLE_GRADIENT[idx.min(BRAILLE_GRADIENT.len() - 1)]
}

/// Apply hue shift to RGB color
fn shift_hue(r: u8, g: u8, b: u8, shift: f32) -> (u8, u8, u8) {
    if shift == 0.0 {
        return (r, g, b);
    }

    // Convert RGB to HSV
    let r_f = r as f32 / 255.0;
    let g_f = g as f32 / 255.0;
    let b_f = b as f32 / 255.0;

    let max = r_f.max(g_f).max(b_f);
    let min = r_f.min(g_f).min(b_f);
    let delta = max - min;

    if delta < 0.00001 {
        return (r, g, b);
    }

    let mut h = if (max - r_f).abs() < 0.00001 {
        ((g_f - b_f) / delta) % 6.0
    } else if (max - g_f).abs() < 0.00001 {
        (b_f - r_f) / delta + 2.0
    } else {
        (r_f - g_f) / delta + 4.0
    };

    h = (h * 60.0 + shift * 360.0) % 360.0;
    if h < 0.0 {
        h += 360.0;
    }

    let s = delta / max;
    let v = max;

    // Convert HSV back to RGB
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r_p, g_p, b_p) = match h as i32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r_p + m) * 255.0) as u8,
        ((g_p + m) * 255.0) as u8,
        ((b_p + m) * 255.0) as u8,
    )
}

/// Apply brightness multiplier to RGB
fn adjust_brightness(r: u8, g: u8, b: u8, brightness: f32) -> (u8, u8, u8) {
    (
        (r as f32 * brightness).min(255.0) as u8,
        (g as f32 * brightness).min(255.0) as u8,
        (b as f32 * brightness).min(255.0) as u8,
    )
}

/// Render the cover image as ASCII art using Braille characters
/// Uses actual image colors for foreground with dark/black background
pub fn render_cover_ascii(
    term_width: u16,
    term_height: u16,
    palette: &AdaptiveColors,
    hue_shift: Option<f32>,
    brightness: f32,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let shift = hue_shift.unwrap_or(0.0);

    // Calculate scaling - sample directly from image
    let scale_x = BG_W as f32 / term_width as f32;
    let scale_y = BG_H as f32 / term_height as f32;

    for y in 0..term_height as usize {
        let mut spans = Vec::new();

        for x in 0..term_width as usize {
            // Sample from source image
            let src_x = (x as f32 * scale_x) as usize;
            let src_y = (y as f32 * scale_y) as usize;

            if src_x >= BG_W || src_y >= BG_H {
                spans.push(Span::raw(" "));
                continue;
            }

            let idx = src_y * BG_W + src_x;
            if idx >= BG_PIXELS.len() {
                spans.push(Span::raw(" "));
                continue;
            }

            let (r, g, b, a) = unpack_pixel(BG_PIXELS[idx]);

            // Skip transparent pixels
            if a < 128 {
                spans.push(Span::raw(" "));
                continue;
            }

            // Apply transformations to get final color
            let (r, g, b) = shift_hue(r, g, b, shift);
            let (r, g, b) = adjust_brightness(r, g, b, brightness);

            // Calculate luminance to select appropriate Braille character
            let lum = luminance(r, g, b);
            let ch = char_for_luminance(lum);

            // Use the actual pixel color for the character
            let fg_color = palette.map_rgb_to_term_color(r, g, b);

            // Use a very dark background (or black) to let the colors show through
            let bg_color = Color::Black;

            spans.push(Span::styled(
                ch.to_string(),
                ratatui::style::Style::default()
                    .fg(fg_color)
                    .bg(bg_color)
            ));
        }

        lines.push(Line::from(spans));
    }

    lines
}

/// Convert a grid of lines to a Paragraph
pub fn paragraph_from_grid(grid: &[Line<'static>]) -> ratatui::widgets::Paragraph<'static> {
    ratatui::widgets::Paragraph::new(grid.to_vec())
}
