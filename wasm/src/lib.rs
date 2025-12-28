//! WASM bindings for vruffr sketch renderer
// this_file: wasm/src/lib.rs

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Default)]
pub struct WasmOptions {
    pub roughness: Option<f64>,
    pub bowing: Option<f64>,
    pub seed: Option<u64>,
    pub fill_style: Option<String>,
    pub hachure_angle: Option<f32>,
    pub hachure_gap: Option<f32>,
}

/// Render SVG to sketch-style SVG output
#[wasm_bindgen]
pub fn render_to_svg(svg_data: &str, options: JsValue) -> Result<String, JsValue> {
    let opts: WasmOptions = serde_wasm_bindgen::from_value(options).unwrap_or_default();

    let fill_style = match opts.fill_style.as_deref() {
        Some("hachure") => roughr::core::FillStyle::Hachure,
        _ => roughr::core::FillStyle::CrossHatch,
    };

    // Parse SVG
    let tree = usvg::Tree::from_str(svg_data, &usvg::Options::default())
        .map_err(|e| JsValue::from_str(&format!("SVG parse error: {}", e)))?;

    let size = tree.size();
    let width = size.width() as u32;
    let height = size.height() as u32;

    // Create roughr options
    let rough_opts = roughr::core::Options {
        max_randomness_offset: opts.roughness.unwrap_or(1.0) * 2.0,
        roughness: opts.roughness.unwrap_or(1.0),
        bowing: opts.bowing.unwrap_or(1.0),
        seed: opts.seed.unwrap_or(42),
        fill_style,
        hachure_angle: opts.hachure_angle.unwrap_or(-41.0) as f64,
        hachure_gap: opts.hachure_gap.unwrap_or(4.0) as f64,
        ..Default::default()
    };

    // Build SVG output
    // TODO: Implement full rendering pipeline
    // For now, return a placeholder demonstrating the structure
    let svg_output = format!(
        r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">
  <!-- vruffr WASM sketch output -->
  <!-- roughness: {}, bowing: {}, seed: {} -->
  <text x="10" y="30" font-size="16" fill="#666">
    WASM rendering not yet implemented.
    Use CLI for full functionality.
  </text>
</svg>"#,
        width,
        height,
        opts.roughness.unwrap_or(1.0),
        opts.bowing.unwrap_or(1.0),
        opts.seed.unwrap_or(42)
    );

    Ok(svg_output)
}

/// Validate SVG without rendering
#[wasm_bindgen]
pub fn validate_svg(svg_data: &str) -> Result<JsValue, JsValue> {
    let tree = usvg::Tree::from_str(svg_data, &usvg::Options::default())
        .map_err(|e| JsValue::from_str(&format!("SVG parse error: {}", e)))?;

    let size = tree.size();

    #[derive(Serialize)]
    struct SvgInfo {
        width: u32,
        height: u32,
        valid: bool,
    }

    let info = SvgInfo {
        width: size.width() as u32,
        height: size.height() as u32,
        valid: true,
    };

    serde_wasm_bindgen::to_value(&info).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get version string
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
