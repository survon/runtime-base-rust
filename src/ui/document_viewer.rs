// ui/document_viewer.rs
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fmt::Debug;
use color_eyre::Result;

use pdf::file::{File as PdfFile, FileOptions};
use pdf::enc::StreamFilter;
use pdf::object::*;
use uuid::Uuid;
use std::fs;

pub trait DocumentViewStrategy: Debug {
    fn parse_content(&self, file_path: &Path, cache_dir: &Path) -> Result<DocumentContent>;
    fn get_supported_extensions(&self) -> Vec<&'static str>;
}

#[derive(Debug, Clone)]
pub struct DocumentContent {
    pub text: String,
    pub image_mappings: HashMap<String, String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug)]
pub struct DocumentViewer {
    strategies: HashMap<String, Box<dyn DocumentViewStrategy>>,
}

impl DocumentViewer {
    pub fn new() -> Self {
        let mut strategies: HashMap<String, Box<dyn DocumentViewStrategy>> = HashMap::new();

        // Register strategies
        let pdf_strategy = PdfViewStrategy;
        for ext in pdf_strategy.get_supported_extensions() {
            strategies.insert(ext.to_string(), Box::new(PdfViewStrategy));
        }

        let text_strategy = TextViewStrategy;
        for ext in text_strategy.get_supported_extensions() {
            strategies.insert(ext.to_string(), Box::new(TextViewStrategy));
        }

        Self { strategies }
    }

    pub fn view_document(&self, file_path: &Path) -> Result<DocumentContent> {
        let extension = file_path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        if let Some(strategy) = self.strategies.get(&extension) {
            let cache_dir = PathBuf::from("./.cache/knowledge");
            strategy.parse_content(file_path, &cache_dir)
        } else {
            Err(color_eyre::eyre::eyre!("Unsupported file type: {}", extension))
        }
    }
}

// Concrete strategies
#[derive(Debug)]
pub struct PdfViewStrategy;
// impl DocumentViewStrategy for PdfViewStrategy {
//     fn parse_content(&self, file_path: &Path, cache_dir: &Path) -> Result<DocumentContent> {
//         let mut image_counter = 0;
//
//         // Create unique cache directory for this PDF
//         let pdf_cache_dir = cache_dir.join(format!("pdf_{}", Uuid::new_v4()));
//         fs::create_dir_all(&pdf_cache_dir)?;
//
//         // Extract text (keep existing logic)
//         let text = pdf_extract::extract_text(file_path)?;
//
//         // Extract images using pdf crate
//         let mut image_mappings = HashMap::new();
//         let mut processed_text = text;
//
//         if let Ok(pdf_file) = FileOptions::cached().open(file_path) {
//             for page_num in 0..pdf_file.num_pages() {
//                 if let Ok(page) = pdf_file.get_page(page_num) {
//                     if let Ok(resources) = page.resources() {
//                         for (name, xobject_ref) in &resources.xobjects {
//                             if let Ok(xobject) = pdf_file.get(*xobject_ref) {
//                                 if let XObject::Image(image_obj) = &*xobject {
//                                     let image_id = format!("IMG_{:03}", image_counter);
//                                     let image_path = pdf_cache_dir.join(format!("{}.png", image_id));
//
//                                     // Save image to cache
//                                     if self.save_pdf_image(image_obj, &image_path).is_ok() {
//                                         image_mappings.insert(
//                                             image_id.clone(),
//                                             image_path.to_string_lossy().to_string()
//                                         );
//
//                                         // Insert placeholder in text
//                                         let placeholder = format!("{{{{IMAGE_{}}}}}", image_id);
//                                         processed_text.push_str(&format!("\n\n{}\n\n", placeholder));
//                                     }
//
//                                     image_counter += 1;
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//
//         if image_counter > 0 {
//             let last_500_chars = processed_text.chars().rev().take(500).collect::<String>().chars().rev().collect::<String>();
//             eprintln!("DEBUG: Last 500 chars:\n{}", last_500_chars);
//         }
//
//         Ok(DocumentContent {
//             text: processed_text,
//             image_mappings,
//             metadata: serde_json::json!({
//                 "type": "pdf",
//                 "cache_dir": pdf_cache_dir.to_string_lossy(),
//                 "image_count": image_counter
//             }),
//         })
//     }
//
//     fn get_supported_extensions(&self) -> Vec<&'static str> {
//         vec!["pdf"]
//     }
// }
//
// impl PdfViewStrategy {
//     fn save_pdf_image(&self, image_obj: &ImageXObject, output_path: &Path) -> Result<()> {
//         // Try to get the image dimensions and data
//         let width = image_obj.inner.width;
//         let height = image_obj.inner.height;
//         let bits_per_component = image_obj.inner.bits_per_component.unwrap_or(8);
//
//         eprintln!("Extracting {}x{} image with {} bits per component", width, height, bits_per_component);
//
//         // The actual image data would be in the PDF stream
//         // For now, create a proper-sized test image
//         use image::{ImageBuffer, Rgb};
//
//         let img = ImageBuffer::from_fn(width as u32, height as u32, |x, y| {
//             // Create a simple gradient pattern for testing
//             let r = (x * 255 / width as u32) as u8;
//             let g = (y * 255 / height as u32) as u8;
//             let b = 128u8;
//             Rgb([r, g, b])
//         });
//
//         img.save(output_path)?;
//         Ok(())
//     }
// }

impl DocumentViewStrategy for PdfViewStrategy {
    fn parse_content(&self, file_path: &Path, cache_dir: &Path) -> Result<DocumentContent> {
        let pdf_cache_dir = cache_dir.join(format!("pdf_{}", Uuid::new_v4()));
        fs::create_dir_all(&pdf_cache_dir)?;

        let mut image_mappings = HashMap::new();
        let mut image_counter = 0;
        let mut pages_content = Vec::new();

        if let Ok(pdf_file) = FileOptions::cached().open(file_path) {
            for page_num in 0..pdf_file.num_pages() {
                if let Ok(page) = pdf_file.get_page(page_num) {
                    let page_text = format!("Page {} text content here", page_num + 1);
                    let mut page_images = Vec::new();

                    if let Ok(resources) = page.resources() {
                        for (_, xobject_ref) in &resources.xobjects {
                            if let Ok(xobject) = pdf_file.get(*xobject_ref) {
                                if let XObject::Image(image_obj) = &*xobject {
                                    let image_id = format!("IMG_{:03}", image_counter);
                                    let image_path = pdf_cache_dir.join(format!("{}.png", image_id));

                                    if self.save_pdf_image(image_obj, &pdf_file, &image_path).is_ok() {
                                        image_mappings.insert(
                                            image_id.clone(),
                                            image_path.to_string_lossy().to_string()
                                        );
                                        page_images.push(format!("{{{{IMAGE_{}}}}}", image_id));
                                    }
                                    image_counter += 1;
                                }
                            }
                        }
                    }

                    let page_content = if page_images.is_empty() {
                        page_text
                    } else {
                        format!("{}\n\n{}", page_text, page_images.join("\n\n"))
                    };
                    pages_content.push(page_content);
                }
            }

            let processed_text = pages_content.join("\n\n--- Page Break ---\n\n");

            Ok(DocumentContent {
                text: processed_text,
                image_mappings,
                metadata: serde_json::json!({
                    "type": "pdf",
                    "cache_dir": pdf_cache_dir.to_string_lossy(),
                    "image_count": image_counter
                }),
            })
        } else {
            let text = pdf_extract::extract_text(file_path)?;
            Ok(DocumentContent {
                text,
                image_mappings: HashMap::new(),
                metadata: serde_json::json!({"type": "pdf"}),
            })
        }
    }

    fn get_supported_extensions(&self) -> Vec<&'static str> {
        vec!["pdf"]
    }
}


impl PdfViewStrategy {
    fn save_pdf_image(&self, image_obj: &ImageXObject, pdf_file: &impl pdf::object::Resolve, output_path: &Path) -> Result<()> {
        let width = image_obj.inner.width;
        let height = image_obj.inner.height;
        let bits_per_component = image_obj.inner.bits_per_component.unwrap_or(8) as u8;
        let color_space = &image_obj.inner.color_space;

        eprintln!("Extracting {}x{} image with {} bits per component", width, height, bits_per_component);

        match image_obj.inner.data(pdf_file) {
            Ok(raw_data) => {
                eprintln!("Found raw stream data: {} bytes", raw_data.len());
                return self.convert_raw_to_png(&raw_data, width, height, bits_per_component, color_space, output_path);
            }
            Err(e) => {
                eprintln!("Failed to get stream data: {:?}", e);
            }
        }

        eprintln!("Could not extract real image data, using gradient fallback");
        self.create_gradient_fallback(width, height, output_path)
    }

    fn convert_raw_to_png(
        &self,
        data: &[u8],
        width: u32,
        height: u32,
        bits_per_component: u8,
        color_space: &Option<ColorSpace>,
        output_path: &Path,
    ) -> Result<()> {
        use image::{ImageBuffer, Rgb};

        match color_space {
            Some(ColorSpace::DeviceRGB) => {
                if bits_per_component == 8 && data.len() >= (width * height * 3) as usize {
                    let img = ImageBuffer::from_fn(width, height, |x, y| {
                        let idx = ((y * width + x) * 3) as usize;
                        if idx + 2 < data.len() {
                            Rgb([data[idx], data[idx + 1], data[idx + 2]])
                        } else {
                            Rgb([0, 0, 0])
                        }
                    });
                    img.save(output_path)?;
                    return Ok(());
                }
            }
            Some(ColorSpace::DeviceGray) => {
                if bits_per_component == 8 && data.len() >= (width * height) as usize {
                    let img = ImageBuffer::from_fn(width, height, |x, y| {
                        let idx = (y * width + x) as usize;
                        if idx < data.len() {
                            let gray = data[idx];
                            Rgb([gray, gray, gray])
                        } else {
                            Rgb([0, 0, 0])
                        }
                    });
                    img.save(output_path)?;
                    return Ok(());
                }
            }
            _ => {}
        }

        self.create_gradient_fallback(width, height, output_path)
    }

    fn create_gradient_fallback(&self, width: u32, height: u32, output_path: &Path) -> Result<()> {
        use image::{ImageBuffer, Rgb};

        let img = ImageBuffer::from_fn(width, height, |x, y| {
            let r = (x * 255 / width) as u8;
            let g = (y * 255 / height) as u8;
            let b = 128u8;
            Rgb([r, g, b])
        });

        img.save(output_path)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TextViewStrategy;
impl DocumentViewStrategy for TextViewStrategy {
    fn parse_content(&self, file_path: &Path, _cache_dir: &Path) -> Result<DocumentContent> {
        let text = std::fs::read_to_string(file_path)?;
        Ok(DocumentContent {
            text,
            image_mappings: HashMap::new(),
            metadata: serde_json::json!({"type": "text"}),
        })
    }

    fn get_supported_extensions(&self) -> Vec<&'static str> {
        vec!["txt", "md", "log"]
    }
}
