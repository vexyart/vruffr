//! WASM bindings for vruffr sketch renderer
// this_file: wasm/src/lib.rs

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Options for sketch rendering (JS-compatible)
#[derive(Serialize, Deserialize, Default)]
pub struct WasmOptions {
    #[serde(default = "default_roughness")]
    pub roughness: f64,
    #[serde(default = "default_bowing")]
    pub bowing: f64,
    #[serde(default = "default_seed")]
    pub seed: u64,
    #[serde(default)]
    pub fill_style: Option<String>,
    #[serde(default)]
    pub hachure_angle: Option<f32>,
    #[serde(default)]
    pub hachure_gap: Option<f32>,
    #[serde(default)]
    pub fill_weight: Option<f32>,
    #[serde(default)]
    pub stroke_width: Option<f32>,
    #[serde(default)]
    pub scale: Option<f32>,
    #[serde(default)]
    pub adaptive_strength: Option<f32>,
    #[serde(default)]
    pub reference_size: Option<f32>,
    #[serde(default)]
    pub deduplicate: Option<bool>,
    #[serde(default)]
    pub no_fill: Option<bool>,
    #[serde(default)]
    pub no_stroke: Option<bool>,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub color_mode: Option<String>,
    #[serde(default)]
    pub noise: Option<f32>,
    #[serde(default)]
    pub edge_roughen: Option<f32>,
}

fn default_roughness() -> f64 {
    1.0
}
fn default_bowing() -> f64 {
    1.0
}
fn default_seed() -> u64 {
    42
}

impl WasmOptions {
    fn to_sketch_options(&self) -> vruffr::SketchOptions {
        let fill_style = match self.fill_style.as_deref() {
            Some("hachure") => vruffr::SketchFillStyle::Hachure,
            _ => vruffr::SketchFillStyle::CrossHatch,
        };

        let background = self
            .background
            .as_ref()
            .map(|bg| parse_color(bg).unwrap_or([255, 255, 255, 255]));

        let color_mode = match self.color_mode.as_deref() {
            Some("grayscale") | Some("gray") | Some("mono") => vruffr::ColorMode::Grayscale,
            Some("sepia") => vruffr::ColorMode::Sepia,
            Some("invert") | Some("negative") => vruffr::ColorMode::Invert,
            _ => vruffr::ColorMode::Color,
        };

        vruffr::SketchOptions {
            roughness: self.roughness,
            bowing: self.bowing,
            seed: self.seed,
            fill_style,
            hachure_angle: self.hachure_angle.unwrap_or(-41.0),
            hachure_gap: self.hachure_gap.unwrap_or(4.0),
            fill_weight: self.fill_weight.unwrap_or(0.5),
            stroke_width: self.stroke_width,
            scale: self.scale.unwrap_or(1.0),
            adaptive_strength: self.adaptive_strength.unwrap_or(0.0),
            reference_size: self.reference_size.unwrap_or(100.0),
            deduplicate: self.deduplicate.unwrap_or(false),
            no_fill: self.no_fill.unwrap_or(false),
            no_stroke: self.no_stroke.unwrap_or(false),
            background,
            color_mode,
            noise: self.noise.unwrap_or(0.0),
            edge_roughen: self.edge_roughen.unwrap_or(0.0),
            ..Default::default()
        }
    }
}

/// Parse a color string (#RRGGBB, #RGB, or named colors)
fn parse_color(s: &str) -> Option<[u8; 4]> {
    let s = s.trim();
    if s == "transparent" {
        return None;
    }
    if s == "white" {
        return Some([255, 255, 255, 255]);
    }
    if s == "black" {
        return Some([0, 0, 0, 255]);
    }
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some([r, g, b, 255]);
        } else if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            return Some([r, g, b, 255]);
        }
    }
    Some([255, 255, 255, 255]) // fallback to white
}

/// Render SVG to sketch-style PNG as base64
#[wasm_bindgen]
pub fn render_to_png_base64(svg_data: &str, options: JsValue) -> Result<String, JsValue> {
    let opts: WasmOptions = serde_wasm_bindgen::from_value(options).unwrap_or_default();
    let sketch_opts = opts.to_sketch_options();

    let pixmap = vruffr::render_sketch(svg_data, &sketch_opts)
        .map_err(|e| JsValue::from_str(&format!("Render error: {}", e)))?;

    let png_data = pixmap
        .encode_png()
        .map_err(|e| JsValue::from_str(&format!("PNG encode error: {}", e)))?;

    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);
    Ok(format!("data:image/png;base64,{}", b64))
}

/// Render SVG to sketch-style SVG output
#[wasm_bindgen]
pub fn render_to_svg(svg_data: &str, options: JsValue) -> Result<String, JsValue> {
    let opts: WasmOptions = serde_wasm_bindgen::from_value(options).unwrap_or_default();
    let sketch_opts = opts.to_sketch_options();

    let (svg_out, _warnings) = vruffr::render_to_svg(svg_data, &sketch_opts)
        .map_err(|e| JsValue::from_str(&format!("Render error: {}", e)))?;

    Ok(svg_out)
}

/// Validate SVG without rendering
#[wasm_bindgen]
pub fn validate_svg(svg_data: &str) -> Result<JsValue, JsValue> {
    let info = vruffr::validate_svg(svg_data)
        .map_err(|e| JsValue::from_str(&format!("SVG parse error: {}", e)))?;

    #[derive(Serialize)]
    struct SvgInfo {
        width: u32,
        height: u32,
        path_count: usize,
        valid: bool,
        has_text: bool,
        has_images: bool,
    }

    let result = SvgInfo {
        width: info.width,
        height: info.height,
        path_count: info.path_count,
        valid: true,
        has_text: info.warnings.has_text,
        has_images: info.warnings.has_images,
    };

    serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get version string
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
