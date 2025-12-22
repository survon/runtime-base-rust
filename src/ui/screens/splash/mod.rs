use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Paragraph, Widget},
    text::Line,
};
use std::time::{Duration, Instant};
use crate::log_error;
use crate::util::{
    audio::SurvonAudioPlayer,
    image::ImageRenderer,
};

#[derive(Debug)]
pub struct SplashScreen {
    pub start_time: Instant,
    pub animation_frame: f64,
    pub is_running: bool,
    pub user_dismissed: bool,
    pub player: SurvonAudioPlayer,
    pub background_image: Option<ImageRenderer>,
}

impl SplashScreen {
    pub fn new() -> Self {
        let mut player = SurvonAudioPlayer::new_with_audio_jack(
            "assets/audio/theme_compressed.wav",
            0.1
        );

        if let Err(e) = player.play_looped() {
            log_error!("Failed to play theme: {}", e);
        }

        // Load background image
        let background_image = ImageRenderer::from_path("assets/images/homestead-scene-3-wide.png")
            .ok();

        if background_image.is_none() {
            log_error!("Failed to load splash background image");
        }

        Self {
            start_time: Instant::now(),
            animation_frame: 0.0,
            is_running: true,
            user_dismissed: false,
            player,
            background_image,
        }
    }

    pub fn bypass_theme(&mut self) -> bool {
        if self.start_time.elapsed() >= Duration::from_millis(2000) {
            self.is_running = false;
            self.user_dismissed = true;
            self.player.stop().ok();
            true
        } else {
            false
        }
    }

    pub fn is_complete(&self) -> bool {
        self.user_dismissed
    }

    pub fn update(&mut self) {
        let elapsed = self.start_time.elapsed().as_millis() as f64;
        self.animation_frame = elapsed / 20.0;
    }

    fn get_rainbow_color(&self, offset: f64) -> Color {
        let hue = ((self.animation_frame + offset) % 360.0) / 360.0;

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

        // Render background image if available
        if let Some(ref mut bg_image) = self.background_image {
            bg_image.render_splash(area, buf)
        }

        // Create a semi-transparent overlay for better text visibility
        let overlay = Block::default()
            .style(Style::default().bg(Color::Rgb(0, 0, 0))); // We'll layer text on top

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
        let logo_height = logo.len() + 4;
        let start_y = (area.height.saturating_sub(logo_height as u16)) / 2;

        // Render each line with rainbow colors and shadow effect for visibility
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
                    .add_modifier(Modifier::ITALIC | Modifier::BOLD)
                    .bg(Color::Rgb(0, 0, 0)) // Dark background for readability
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

        // Render loading animation or "Press any key" message
        let loading_y = tagline_y + 2;
        if loading_y < area.height {
            let elapsed = self.start_time.elapsed();
            let message = if elapsed >= Duration::from_millis(2000) && !self.user_dismissed {
                "Press any key to continue".to_string()
            } else {
                let dots = ".".repeat(((self.animation_frame / 5.0) as usize % 4) + 1);
                format!("Loading{}", dots)
            };

            let loading_color = self.get_rainbow_color(180.0);

            let loading_line = Line::from(message).style(
                Style::default()
                    .fg(loading_color)
                    .bg(Color::Rgb(0, 0, 0)) // Dark background for readability
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
    }
}
