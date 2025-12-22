// src/util/image.rs
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    Frame,
};
use ratatui_image::{
    picker::Picker,
    StatefulImage,
    protocol::StatefulProtocol,
    Resize,
};
use image::DynamicImage;
use std::path::Path;

pub struct ImageRenderer {
    protocol: StatefulProtocol,
    image_dimensions: Option<(u32, u32)>,
}

// Manual Debug implementation since StatefulProtocol doesn't implement Debug
impl std::fmt::Debug for ImageRenderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageRenderer")
            .field("protocol", &"<StatefulProtocol>")
            .field("image_dimensions", &self.image_dimensions)
            .finish()
    }
}

impl ImageRenderer {
    /// Create a new image renderer from a file path
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let img = image::open(path)?;
        Self::from_dynamic_image(img)
    }

    /// Create a new image renderer from a DynamicImage
    pub fn from_dynamic_image(img: DynamicImage) -> Result<Self, Box<dyn std::error::Error>> {
        let dimensions = (img.width(), img.height());
        let picker = Picker::from_query_stdio()?;
        let protocol = picker.new_resize_protocol(img);

        Ok(Self {
            protocol,
            image_dimensions: Some(dimensions),
        })
    }

    /// Render the image to fill the given area using Frame (CORRECT WAY)
    pub fn render_to_frame(&mut self, f: &mut Frame, area: Rect) {
        let image = StatefulImage::default().resize(Resize::Fit(None));
        f.render_stateful_widget(image, area, &mut self.protocol);

        if let Err(e) = self.protocol.last_encoding_result().unwrap_or(Ok(())) {
            eprintln!("Image encoding error: {}", e);
        }
    }

    /// DEFAULT METHOD - Render using buffer with Fit (letterbox)
    /// Use this for small contained areas like headers
    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::StatefulWidget;

        let image = StatefulImage::default().resize(Resize::Fit(None));
        image.render(area, buf, &mut self.protocol);

        if let Err(e) = self.protocol.last_encoding_result().unwrap_or(Ok(())) {
            eprintln!("Image encoding error: {}", e);
        }
    }

    /// SPLASH SCREEN METHOD - Manual scaling to fill area
    /// Creates oversized render area, lets terminal clip the overflow
    pub fn render_splash(&mut self, area: Rect, buf: &mut Buffer) {
        use ratatui::widgets::StatefulWidget;

        if let Some((img_width, img_height)) = self.image_dimensions {
            let img_aspect = img_width as f32 / img_height as f32;
            let area_aspect = area.width as f32 / area.height as f32;

            let (render_area, image_resize) = if img_aspect > area_aspect {
                // Image is wider - fill HEIGHT, let width overflow
                let scaled_height = area.height;
                let scaled_width = (scaled_height as f32 * img_aspect) as u16;

                // Center horizontally
                let offset_x = area.x.saturating_sub((scaled_width.saturating_sub(area.width)) / 2);

                (
                    Rect {
                        x: offset_x,
                        y: area.y,
                        width: scaled_width,
                        height: scaled_height,
                    },
                    Resize::Fit(None)
                )
            } else {
                // Image is taller - fill WIDTH, let height overflow
                let scaled_width = area.width;
                let scaled_height = (scaled_width as f32 / img_aspect) as u16;

                // Center vertically
                let offset_y = area.y.saturating_sub((scaled_height.saturating_sub(area.height)) / 2);

                (
                    Rect {
                        x: area.x,
                        y: offset_y,
                        width: scaled_width,
                        height: scaled_height,
                    },
                    Resize::Fit(None)
                )
            };

            let image = StatefulImage::default().resize(image_resize);
            image.render(render_area, buf, &mut self.protocol);
        } else {
            // Fallback
            let image = StatefulImage::default().resize(Resize::Fit(None));
            image.render(area, buf, &mut self.protocol);
        }

        if let Err(e) = self.protocol.last_encoding_result().unwrap_or(Ok(())) {
            eprintln!("Image encoding error: {}", e);
        }
    }

    /// Render with specific brightness/opacity
    pub fn render_dimmed(&mut self, area: Rect, buf: &mut Buffer, _brightness: f32) {
        self.render(area, buf);
    }
}

/// Cache for loaded images to avoid reloading
pub struct ImageCache {
    splash_bg: Option<ImageRenderer>,
    overview_header_bg: Option<ImageRenderer>,
}

// Manual Debug implementation since ImageRenderer doesn't derive Debug
impl std::fmt::Debug for ImageCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageCache")
            .field("splash_bg", &self.splash_bg.as_ref().map(|_| "<ImageRenderer>"))
            .field("overview_header_bg", &self.overview_header_bg.as_ref().map(|_| "<ImageRenderer>"))
            .finish()
    }
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            splash_bg: None,
            overview_header_bg: None,
        }
    }

    pub fn load_splash(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        self.splash_bg = Some(ImageRenderer::from_path(path)?);
        Ok(())
    }

    pub fn load_overview_header(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        self.overview_header_bg = Some(ImageRenderer::from_path(path)?);
        Ok(())
    }

    pub fn get_splash_mut(&mut self) -> Option<&mut ImageRenderer> {
        self.splash_bg.as_mut()
    }

    pub fn get_overview_header_mut(&mut self) -> Option<&mut ImageRenderer> {
        self.overview_header_bg.as_mut()
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new()
    }
}
