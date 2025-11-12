// src/ui/splash.rs
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph, Widget},
    text::Line,
};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct SplashScreen {
    pub start_time: Instant,
    pub animation_frame: f64,
}

impl SplashScreen {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            animation_frame: 0.0,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.start_time.elapsed() >= Duration::from_millis(2000)
    }

    pub fn should_fade(&self) -> bool {
        self.start_time.elapsed() >= Duration::from_millis(1700)
    }

    pub fn get_alpha(&self) -> f64 {
        if !self.should_fade() {
            return 1.0;
        }

        let fade_start = 1700.0;
        let fade_end = 2000.0;
        let elapsed = self.start_time.elapsed().as_millis() as f64;

        if elapsed >= fade_end {
            0.0
        } else {
            1.0 - ((elapsed - fade_start) / (fade_end - fade_start))
        }
    }

    pub fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_millis() as f64;
        self.animation_frame = elapsed / 100.0; // Slower animation
    }

    fn get_rainbow_color(&self, offset: f64) -> Color {
        let alpha = self.get_alpha();
        if alpha <= 0.0 {
            return Color::Black;
        }

        // Create rainbow effect that cycles
        let hue = ((self.animation_frame + offset) % 360.0) / 360.0;

        // Convert HSV to RGB (simplified)
        let rainbow_colors = [
            Color::Red,
            Color::Rgb(255, 127, 0), // Orange
            Color::Yellow,
            Color::Green,
            Color::Cyan,
            Color::Blue,
            Color::Magenta,
        ];

        let index = (hue * rainbow_colors.len() as f64) as usize % rainbow_colors.len();
        rainbow_colors[index]
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        self.update();

        // ASCII art for SURVON
        let logo = vec![
            "███████╗██╗   ██╗██████╗ ██╗   ██╗ ██████╗ ███╗   ██╗",
            "██╔════╝██║   ██║██╔══██╗██║   ██║██╔═══██╗████╗  ██║",
            "███████╗██║   ██║██████╔╝██║   ██║██║   ██║██╔██╗ ██║",
            "╚════██║██║   ██║██╔══██╗╚██╗ ██╔╝██║   ██║██║╚██╗██║",
            "███████║╚██████╔╝██║  ██║ ╚████╔╝ ╚██████╔╝██║ ╚████║",
            "╚══════╝ ╚═════╝ ╚═╝  ╚═╝  ╚═══╝   ╚═════╝ ╚═╝  ╚═══╝",
        ];

        let tagline = "Smart Homestead Operating System";

        // Calculate vertical center
        let logo_height = logo.len() + 4; // +4 for tagline and spacing
        let start_y = (area.height.saturating_sub(logo_height as u16)) / 2;

        // Render each line with rainbow colors
        for (i, line) in logo.iter().enumerate() {
            let y = start_y + i as u16;
            if y >= area.height {
                break;
            }

            let color = self.get_rainbow_color(i as f64 * 60.0);
            let styled_line = Line::from(line.to_string()).style(
                Style::default()
                    .fg(color)
                    .add_modifier(Modifier::BOLD)
            );

            let paragraph = Paragraph::new(styled_line)
                .alignment(Alignment::Center);

            let line_area = Rect {
                x: area.x,
                y: area.y + y,
                width: area.width,
                height: 1,
            };

            paragraph.render(line_area, buf);
        }

        // Render tagline
        let tagline_y = start_y + logo.len() as u16 + 2;
        if tagline_y < area.height {
            let tagline_color = self.get_rainbow_color(360.0);
            let tagline_line = Line::from(tagline).style(
                Style::default()
                    .fg(tagline_color)
                    .add_modifier(Modifier::ITALIC)
            );

            let tagline_paragraph = Paragraph::new(tagline_line)
                .alignment(Alignment::Center);

            let tagline_area = Rect {
                x: area.x,
                y: area.y + tagline_y,
                width: area.width,
                height: 1,
            };

            tagline_paragraph.render(tagline_area, buf);
        }

        // Render loading animation
        let loading_y = tagline_y + 2;
        if loading_y < area.height {
            let dots = ".".repeat(((self.animation_frame / 5.0) as usize % 4) + 1);
            let loading_text = format!("Loading{}", dots);
            let loading_color = self.get_rainbow_color(180.0);

            let loading_line = Line::from(loading_text).style(
                Style::default()
                    .fg(loading_color)
            );

            let loading_paragraph = Paragraph::new(loading_line)
                .alignment(Alignment::Center);

            let loading_area = Rect {
                x: area.x,
                y: area.y + loading_y,
                width: area.width,
                height: 1,
            };

            loading_paragraph.render(loading_area, buf);
        }

        // Render fade overlay if fading
        if self.should_fade() {
            let alpha = self.get_alpha();
            if alpha < 0.5 {
                // When mostly faded, render a dimming overlay
                let overlay = Block::default()
                    .style(Style::default().bg(Color::Black));
                overlay.render(area, buf);
            }
        }
    }
}
