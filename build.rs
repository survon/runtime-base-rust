use std::fs;
use std::path::Path;
use std::io::Write;

fn main() {
    // Path to your image
    let img_path = Path::new("assets/images/circuit-board-2.png");
    if !img_path.exists() {
        panic!("Image not found at {:?}", img_path);
    }

    // Load image - we want higher resolution for true-color ASCII art
    // Each terminal character represents 2 "pixels" (fg + bg), so we can use more detail
    let img = image::open(img_path).expect("Failed to open image");

    // Since we're using background colors only (no fancy characters),
    // we can go even higher resolution for better quality
    // 320x120 gives excellent detail (~400KB file size)
    let target_width = 320;
    let target_height = 120;

    let img = img.resize_exact(
        target_width,
        target_height,
        image::imageops::FilterType::Lanczos3
    );
    let img = img.to_rgba8();
    let (w, h) = img.dimensions();

    println!("cargo:warning=Processed image to {}x{} ({} pixels)", w, h, w * h);

    // Create the generated directory if it doesn't exist
    let generated_dir = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("ui")
        .join("generated");

    fs::create_dir_all(&generated_dir).expect("Failed to create generated directory");

    // Generate background.rs
    let mut out = String::new();
    out.push_str("// Auto-generated - do not edit\n");
    out.push_str("// High-quality downsampled image for true-color ASCII art rendering\n");
    out.push_str(&format!("pub const BG_W: usize = {};\n", w));
    out.push_str(&format!("pub const BG_H: usize = {};\n", h));
    out.push_str("\n// RGBA pixel data stored as u32 for better compression\n");
    out.push_str("pub const BG_PIXELS: &[u32] = &[\n");

    // Pack RGBA into u32 for better space efficiency
    for y in 0..h {
        out.push_str("    ");
        for x in 0..w {
            let p = img.get_pixel(x, y);
            // Pack as 0xRRGGBBAA
            let packed = ((p[0] as u32) << 24) |
                ((p[1] as u32) << 16) |
                ((p[2] as u32) << 8) |
                (p[3] as u32);
            out.push_str(&format!("0x{:08X},", packed));
        }
        out.push('\n');
    }

    out.push_str("];\n\n");

    // Helper function to unpack
    out.push_str("#[inline]\n");
    out.push_str("pub fn unpack_pixel(packed: u32) -> (u8, u8, u8, u8) {\n");
    out.push_str("    let r = ((packed >> 24) & 0xFF) as u8;\n");
    out.push_str("    let g = ((packed >> 16) & 0xFF) as u8;\n");
    out.push_str("    let b = ((packed >> 8) & 0xFF) as u8;\n");
    out.push_str("    let a = (packed & 0xFF) as u8;\n");
    out.push_str("    (r, g, b, a)\n");
    out.push_str("}\n");

    let dest = generated_dir.join("background.rs");
    let mut f = fs::File::create(&dest).expect("Failed to create background.rs");
    f.write_all(out.as_bytes()).expect("Failed to write background.rs");

    let file_size = out.len();
    println!("cargo:warning=Generated background.rs is {} KB", file_size / 1024);

    println!("cargo:rerun-if-changed={}", img_path.display());
}
