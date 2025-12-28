//! Basic example of using vruffr to render an SVG as a sketch
//!
//! Run with: cargo run -p vruffr-cli --example basic
// this_file: cli/examples/basic.rs

use anyhow::Result;
use std::path::Path;

fn main() -> Result<()> {
    // Find example SVG (try examples/ first, fall back to extra/test-data/)
    let svg_path = ["examples/tiger.svg", "examples/sag.svg"]
        .iter()
        .map(Path::new)
        .find(|p| p.exists());

    let svg_path = match svg_path {
        Some(p) => p,
        None => {
            eprintln!("No example SVG found. Run from project root.");
            return Ok(());
        }
    };

    println!("Using: {:?}", svg_path);
    let svg_data = std::fs::read_to_string(svg_path)?;

    // Create sketch options
    let options = vruffr::SketchOptions {
        roughness: 1.5,
        bowing: 1.0,
        seed: 42,
        fill_style: vruffr::SketchFillStyle::CrossHatch,
        adaptive_strength: 1.0,
        reference_size: 100.0,
        ..Default::default()
    };

    // Render to PNG
    let pixmap = vruffr::render_sketch(&svg_data, &options)?;
    let output = "/tmp/vruffr-basic.png";
    pixmap.save_png(output)?;
    println!("Saved: {}", output);

    // Render to SVG
    let (svg_output, warnings) = vruffr::render_to_svg(&svg_data, &options)?;
    let svg_out = "/tmp/vruffr-basic.svg";
    std::fs::write(svg_out, svg_output)?;
    println!("Saved: {}", svg_out);

    if warnings.has_text {
        println!("Note: SVG contains text (not rendered)");
    }

    Ok(())
}
