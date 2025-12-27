//! Basic example of using vruffr to render an SVG as a sketch
//!
//! Run with: cargo run -p vruffr-cli --example basic

use anyhow::Result;
use std::path::Path;

fn main() -> Result<()> {
    // Find a test SVG
    let svg_path = Path::new("extra/test-data/tigr.svg");
    if !svg_path.exists() {
        eprintln!("Test SVG not found at {:?}", svg_path);
        eprintln!("Run from the project root directory");
        return Ok(());
    }

    // Read SVG data
    let svg_data = std::fs::read_to_string(svg_path)?;

    // Create sketch options with adaptive roughness
    let options = vruffr::SketchOptions {
        roughness: 1.5,
        adaptive_strength: 1.0,
        reference_size: 100.0,
        ..Default::default()
    };

    // Render to pixmap
    let pixmap = vruffr::render_sketch(&svg_data, &options)?;

    // Save to PNG
    let output_path = "/tmp/vruffr-example.png";
    pixmap.save_png(output_path)?;
    println!("Saved sketch to {}", output_path);

    Ok(())
}
