//! vruffr - Convert SVG to hand-drawn sketch-style output
//!
//! Takes a standard SVG and renders it with wobbly lines and crosshatch fills.
//! Supports PNG, SVG, and SVG-plain output formats.
//!
//! # Example
//!
//! ```
//! use vruffr::{render_sketch, SketchOptions};
//!
//! let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
//!     <rect x="10" y="10" width="80" height="80" fill="blue"/>
//! </svg>"#;
//!
//! let options = SketchOptions::default();
//! let pixmap = render_sketch(svg, &options).expect("render failed");
//! assert_eq!(pixmap.width(), 100);
//! ```

use anyhow::{Context, Result};
use palette::Srgba;
use vruffr_skia::{SkiaGenerator, SkiaOpset};
use vruffr_core::core::{FillStyle, OpSetType, OptionsBuilder};
use vruffr_core::dedup::{deduplicate_paths, PathSignature, StyledPath};
use std::fmt::Write as FmtWrite;
use std::sync::Arc;
use tiny_skia::{Pixmap, PixmapMut};

/// Fill style for sketch rendering
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SketchFillStyle {
    /// Single direction hatch lines
    Hachure,
    /// Two perpendicular hatch directions (default)
    #[default]
    CrossHatch,
}

impl std::str::FromStr for SketchFillStyle {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hachure" => Ok(Self::Hachure),
            "crosshatch" | "cross-hatch" => Ok(Self::CrossHatch),
            _ => Err(format!(
                "Unknown fill style: {s}. Valid: hachure, crosshatch"
            )),
        }
    }
}

impl From<SketchFillStyle> for FillStyle {
    fn from(s: SketchFillStyle) -> Self {
        match s {
            SketchFillStyle::Hachure => FillStyle::Hachure,
            SketchFillStyle::CrossHatch => FillStyle::CrossHatch,
        }
    }
}

/// Output format for sketch rendering
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// PNG raster image
    Png,
    /// Fresh SVG with sketch paths only (default)
    #[default]
    SvgPlain,
    /// Embed sketch paths back into original SVG structure
    Svg,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "png" => Ok(Self::Png),
            "svgplain" | "svg-plain" => Ok(Self::SvgPlain),
            "svg" => Ok(Self::Svg),
            _ => Err(format!("Unknown format: {s}. Valid: png, svgplain, svg")),
        }
    }
}

/// Color mode for post-processing
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ColorMode {
    /// Full color (default)
    #[default]
    Color,
    /// Grayscale (luminance-based)
    Grayscale,
    /// Sepia tone
    Sepia,
    /// Invert colors (negative)
    Invert,
}

impl std::str::FromStr for ColorMode {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "color" | "full" | "none" => Ok(Self::Color),
            "grayscale" | "grey" | "gray" | "mono" | "monochrome" => Ok(Self::Grayscale),
            "sepia" | "vintage" => Ok(Self::Sepia),
            "invert" | "negative" | "inverted" => Ok(Self::Invert),
            _ => Err(format!(
                "Unknown color mode: {s}. Valid: color, grayscale, sepia, invert"
            )),
        }
    }
}

/// A single sketch path element with its styling
#[derive(Debug, Clone)]
pub struct SketchElement {
    /// SVG path data string
    pub path_data: String,
    /// Stroke color (None = no stroke)
    pub stroke: Option<[u8; 4]>,
    /// Fill color (None = no fill)
    pub fill: Option<[u8; 4]>,
    /// Stroke width
    pub stroke_width: f32,
    /// Whether this is a fill pattern (hachure lines)
    pub is_fill_sketch: bool,
}

/// Options for sketch rendering
///
/// All fields have sensible defaults via [`Default`].
#[derive(Debug, Clone)]
pub struct SketchOptions {
    /// Line perturbation amount (0.0-10.0, default: 1.0)
    pub roughness: f64,
    /// Line curvature/bowing amount (0.0-10.0, default: 1.0)
    pub bowing: f64,
    /// Random seed for reproducible output (default: 42)
    pub seed: u64,
    /// Output width in pixels (None = use SVG width)
    pub width: Option<u32>,
    /// Output height in pixels (None = use SVG height)
    pub height: Option<u32>,
    /// Fill pattern style (default: CrossHatch)
    pub fill_style: SketchFillStyle,
    /// Angle of hachure lines in degrees (default: -41)
    pub hachure_angle: f32,
    /// Gap between hachure lines in pixels (default: 4.0)
    pub hachure_gap: f32,
    /// Override stroke width (None = use SVG stroke width)
    pub stroke_width: Option<f32>,
    /// Background color as RGBA tuple (default: white, None = transparent)
    pub background: Option<[u8; 4]>,
    /// Skip fill rendering (strokes only)
    pub no_fill: bool,
    /// Skip stroke rendering (fills only)
    pub no_stroke: bool,
    /// Weight/thickness of hachure fill lines (default: 0.5)
    pub fill_weight: f32,
    /// Scale factor for output dimensions (default: 1.0)
    pub scale: f32,
    /// Font family for text rendering (None = use SVG font or system default)
    pub font: Option<String>,
    /// Font size in points for text rendering (None = use SVG font size)
    pub font_size: Option<f32>,
    /// Adaptive roughness strength (0.0 = disabled, 1.0 = normal, 2.0 = aggressive)
    pub adaptive_strength: f32,
    /// Reference element size in pixels for adaptive roughness scaling (default: 100)
    pub reference_size: f32,
    /// Remove duplicate stacked paths before roughening
    pub deduplicate: bool,
    /// Tolerance in pixels for path deduplication matching
    pub dedup_epsilon: f32,
    /// Color mode post-processing (color, grayscale, sepia)
    pub color_mode: ColorMode,
    /// Noise/grain intensity (0.0 = none, 1.0 = heavy)
    pub noise: f32,
}

impl Default for SketchOptions {
    fn default() -> Self {
        Self {
            roughness: 1.0,
            bowing: 1.0,
            seed: 42,
            width: None,
            height: None,
            fill_style: SketchFillStyle::default(),
            hachure_angle: -41.0,
            hachure_gap: 4.0,
            stroke_width: None,
            background: Some([255, 255, 255, 255]), // white
            no_fill: false,
            no_stroke: false,
            fill_weight: 0.5,
            scale: 1.0,
            font: None,
            font_size: None,
            adaptive_strength: 0.0,
            reference_size: 100.0,
            deduplicate: false,
            dedup_epsilon: 0.1,
            color_mode: ColorMode::default(),
            noise: 0.0,
        }
    }
}

/// Warnings about unsupported SVG elements
#[derive(Debug, Default)]
pub struct RenderWarnings {
    /// SVG contains text that couldn't be rendered (no fonts available)
    pub has_text: bool,
    /// SVG contains embedded images (not supported in sketch mode)
    pub has_images: bool,
}

/// Information about a validated SVG
#[derive(Debug)]
pub struct SvgInfo {
    /// Width of the SVG in pixels
    pub width: u32,
    /// Height of the SVG in pixels
    pub height: u32,
    /// Number of path elements in the SVG
    pub path_count: usize,
    /// Warnings about unsupported elements
    pub warnings: RenderWarnings,
}

/// Build usvg options with fontdb loaded
fn build_usvg_options(sketch_options: Option<&SketchOptions>) -> usvg::Options<'static> {
    let mut fontdb = fontdb::Database::new();
    fontdb.load_system_fonts();

    let mut opts = usvg::Options { fontdb: Arc::new(fontdb), ..Default::default() };

    if let Some(sketch) = sketch_options {
        if let Some(ref font) = sketch.font {
            opts.font_family = font.clone();
        }
        if let Some(font_size) = sketch.font_size {
            opts.font_size = font_size;
        }
    }

    opts
}

/// Validate an SVG string without rendering (dry-run)
pub fn validate_svg(svg_data: &str) -> Result<SvgInfo> {
    let usvg_options = build_usvg_options(None);
    let tree = usvg::Tree::from_str(svg_data, &usvg_options).context("Failed to parse SVG")?;

    let svg_size = tree.size();
    let width = svg_size.width() as u32;
    let height = svg_size.height() as u32;

    let mut warnings = RenderWarnings::default();
    let path_count = count_and_check(tree.root(), &mut warnings);

    Ok(SvgInfo { width, height, path_count, warnings })
}

fn count_and_check(group: &usvg::Group, warnings: &mut RenderWarnings) -> usize {
    let mut count = 0;
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => count += count_and_check(g, warnings),
            usvg::Node::Image(_) => warnings.has_images = true,
            usvg::Node::Text(text) => {
                // Only warn if text couldn't be flattened (no fonts available)
                if text.flattened().children().is_empty() {
                    warnings.has_text = true;
                } else {
                    count += count_and_check(text.flattened(), warnings);
                }
            }
            usvg::Node::Path(_) => count += 1,
        }
    }
    count
}

/// Apply color mode transformation to a pixmap
fn apply_color_mode(pixmap: &mut Pixmap, mode: ColorMode) {
    if mode == ColorMode::Color {
        return;
    }
    let data = pixmap.data_mut();
    for chunk in data.chunks_exact_mut(4) {
        let r = chunk[0] as f32;
        let g = chunk[1] as f32;
        let b = chunk[2] as f32;
        // Luminance calculation (ITU-R BT.601)
        let lum = 0.299 * r + 0.587 * g + 0.114 * b;
        match mode {
            ColorMode::Grayscale => {
                let v = lum as u8;
                chunk[0] = v;
                chunk[1] = v;
                chunk[2] = v;
            }
            ColorMode::Sepia => {
                // Sepia tone matrix
                let new_r = (0.393 * r + 0.769 * g + 0.189 * b).min(255.0) as u8;
                let new_g = (0.349 * r + 0.686 * g + 0.168 * b).min(255.0) as u8;
                let new_b = (0.272 * r + 0.534 * g + 0.131 * b).min(255.0) as u8;
                chunk[0] = new_r;
                chunk[1] = new_g;
                chunk[2] = new_b;
            }
            ColorMode::Invert => {
                chunk[0] = 255 - chunk[0];
                chunk[1] = 255 - chunk[1];
                chunk[2] = 255 - chunk[2];
            }
            ColorMode::Color => {}
        }
    }
}

/// Apply noise/grain effect to a pixmap
fn apply_noise(pixmap: &mut Pixmap, intensity: f32, seed: u64) {
    if intensity <= 0.0 {
        return;
    }
    let intensity = intensity.clamp(0.0, 1.0);
    let max_noise = (intensity * 50.0) as i16; // Max noise range
    let data = pixmap.data_mut();
    let mut rng_state = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    for chunk in data.chunks_exact_mut(4) {
        // Simple LCG for reproducible noise
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let noise = ((rng_state >> 33) as i16 % (max_noise * 2 + 1)) - max_noise;
        // Apply to RGB, preserve alpha
        chunk[0] = (chunk[0] as i16 + noise).clamp(0, 255) as u8;
        chunk[1] = (chunk[1] as i16 + noise).clamp(0, 255) as u8;
        chunk[2] = (chunk[2] as i16 + noise).clamp(0, 255) as u8;
    }
}

/// Render an SVG string to a sketch-style PNG pixmap
pub fn render_sketch(svg_data: &str, options: &SketchOptions) -> Result<Pixmap> {
    let (pixmap, _warnings) = render_sketch_with_warnings(svg_data, options)?;
    Ok(pixmap)
}

/// Render an SVG string and return any warnings about unsupported elements
pub fn render_sketch_with_warnings(
    svg_data: &str,
    options: &SketchOptions,
) -> Result<(Pixmap, RenderWarnings)> {
    let usvg_options = build_usvg_options(Some(options));
    let tree = usvg::Tree::from_str(svg_data, &usvg_options).context("Failed to parse SVG")?;

    let svg_size = tree.size();
    let base_width = options.width.unwrap_or(svg_size.width() as u32);
    let base_height = options.height.unwrap_or(svg_size.height() as u32);
    let width = (base_width as f32 * options.scale) as u32;
    let height = (base_height as f32 * options.scale) as u32;

    let mut pixmap = Pixmap::new(width, height).context("Failed to create pixmap")?;

    if let Some([r, g, b, a]) = options.background {
        pixmap.fill(tiny_skia::Color::from_rgba8(r, g, b, a));
    }

    let mut warnings = RenderWarnings::default();

    if options.deduplicate {
        // Collect raw paths, deduplicate, then render
        let raw_paths = collect_raw_paths(tree.root(), options, &mut warnings);
        let deduped = apply_dedup(raw_paths, options.dedup_epsilon);
        for info in &deduped {
            render_raw_path(info, options, &mut pixmap.as_mut());
        }
    } else {
        // Original path: render each path as encountered
        render_group(tree.root(), options, &mut pixmap.as_mut(), &mut warnings);
    }

    // Post-processing: color mode (grayscale/sepia)
    apply_color_mode(&mut pixmap, options.color_mode);

    // Post-processing: noise/grain
    apply_noise(&mut pixmap, options.noise, options.seed);

    Ok((pixmap, warnings))
}

fn render_group(
    group: &usvg::Group,
    options: &SketchOptions,
    pixmap: &mut PixmapMut,
    warnings: &mut RenderWarnings,
) {
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => {
                render_group(g, options, pixmap, warnings);
            }
            usvg::Node::Path(path) => {
                render_path(path, options, pixmap);
            }
            usvg::Node::Image(_) => {
                warnings.has_images = true;
            }
            usvg::Node::Text(text) => {
                // Process flattened text (text converted to paths)
                render_group(text.flattened(), options, pixmap, warnings);
            }
        }
    }
}

/// Compute effective roughness using adaptive scaling based on path bounding box
fn compute_effective_roughness(path: &usvg::Path, options: &SketchOptions) -> f32 {
    if options.adaptive_strength <= 0.0 {
        return options.roughness as f32;
    }
    let bbox = path.abs_bounding_box();
    let width = bbox.width();
    let height = bbox.height();
    let characteristic_size = (width * height).sqrt();
    if characteristic_size <= 0.0 {
        return options.roughness as f32;
    }
    let reference_size = options.reference_size.max(1.0);
    let size_ratio = characteristic_size / reference_size;
    let raw_scale = size_ratio.powf(options.adaptive_strength * 0.5);
    let scale = raw_scale.clamp(0.2, 2.0);
    (options.roughness as f32) * scale
}

fn render_path(path: &usvg::Path, options: &SketchOptions, pixmap: &mut PixmapMut) {
    let svg_path = path_to_svg_string(path);
    if svg_path.is_empty() {
        return;
    }

    let effective_roughness = compute_effective_roughness(path, options);

    // Handle fill
    if !options.no_fill {
        if let Some(fill) = path.fill() {
            let fill_color = extract_color(fill.paint());
            let fill_options = OptionsBuilder::default()
                .roughness(effective_roughness)
                .bowing(options.bowing as f32)
                .seed(options.seed)
                .fill(fill_color)
                .fill_style(options.fill_style.into())
                .stroke(fill_color)
                .stroke_width(options.fill_weight)
                .fill_weight(options.fill_weight)
                .hachure_angle(options.hachure_angle)
                .hachure_gap(options.hachure_gap)
                .build()
                .unwrap();

            let fill_gen = SkiaGenerator::new(fill_options);
            let drawable = fill_gen.path::<f64>(svg_path.clone());
            drawable.draw(pixmap);
        }
    }

    // Handle stroke
    if !options.no_stroke {
        if let Some(stroke) = path.stroke() {
            let stroke_color = extract_color(stroke.paint());
            let stroke_width = options.stroke_width.unwrap_or_else(|| stroke.width().get());

            let stroke_options = OptionsBuilder::default()
                .roughness(effective_roughness)
                .bowing(options.bowing as f32)
                .seed(options.seed)
                .stroke(stroke_color)
                .stroke_width(stroke_width)
                .build()
                .unwrap();

            let stroke_gen = SkiaGenerator::new(stroke_options);
            let drawable = stroke_gen.path::<f64>(svg_path);
            drawable.draw(pixmap);
        }
    }
}

/// Render a RawPathInfo directly to pixmap (for dedup path)
fn render_raw_path(info: &RawPathInfo, options: &SketchOptions, pixmap: &mut PixmapMut) {
    // Handle fill
    if let Some(fill_color) = info.fill_color {
        let fill_options = OptionsBuilder::default()
            .roughness(info.effective_roughness)
            .bowing(options.bowing as f32)
            .seed(options.seed)
            .fill(fill_color)
            .fill_style(options.fill_style.into())
            .stroke(fill_color)
            .stroke_width(options.fill_weight)
            .fill_weight(options.fill_weight)
            .hachure_angle(options.hachure_angle)
            .hachure_gap(options.hachure_gap)
            .build()
            .unwrap();

        let fill_gen = SkiaGenerator::new(fill_options);
        let drawable = fill_gen.path::<f64>(info.path_data.clone());
        drawable.draw(pixmap);
    }

    // Handle stroke
    if let Some(stroke_color) = info.stroke_color {
        let stroke_options = OptionsBuilder::default()
            .roughness(info.effective_roughness)
            .bowing(options.bowing as f32)
            .seed(options.seed)
            .stroke(stroke_color)
            .stroke_width(info.stroke_width)
            .build()
            .unwrap();

        let stroke_gen = SkiaGenerator::new(stroke_options);
        let drawable = stroke_gen.path::<f64>(info.path_data.clone());
        drawable.draw(pixmap);
    }
}

fn path_to_svg_string(path: &usvg::Path) -> String {
    use std::fmt::Write;
    let mut svg = String::new();
    let ts = path.abs_transform();

    // Helper to apply transform to a point
    let transform = |p: tiny_skia::Point| -> (f32, f32) {
        let x = ts.sx * p.x + ts.kx * p.y + ts.tx;
        let y = ts.ky * p.x + ts.sy * p.y + ts.ty;
        (x, y)
    };

    for seg in path.data().segments() {
        match seg {
            tiny_skia::PathSegment::MoveTo(p) => {
                let (x, y) = transform(p);
                let _ = write!(svg, "M {} {} ", x, y);
            }
            tiny_skia::PathSegment::LineTo(p) => {
                let (x, y) = transform(p);
                let _ = write!(svg, "L {} {} ", x, y);
            }
            tiny_skia::PathSegment::QuadTo(p1, p2) => {
                let (x1, y1) = transform(p1);
                let (x2, y2) = transform(p2);
                let _ = write!(svg, "Q {} {} {} {} ", x1, y1, x2, y2);
            }
            tiny_skia::PathSegment::CubicTo(p1, p2, p3) => {
                let (x1, y1) = transform(p1);
                let (x2, y2) = transform(p2);
                let (x3, y3) = transform(p3);
                let _ = write!(svg, "C {} {} {} {} {} {} ", x1, y1, x2, y2, x3, y3);
            }
            tiny_skia::PathSegment::Close => {
                svg.push_str("Z ");
            }
        }
    }

    svg.trim().to_string()
}

fn extract_color(paint: &usvg::Paint) -> Srgba {
    match paint {
        usvg::Paint::Color(c) => Srgba::new(
            c.red as f32 / 255.0,
            c.green as f32 / 255.0,
            c.blue as f32 / 255.0,
            1.0,
        ),
        usvg::Paint::LinearGradient(grad) => average_gradient_color(grad.stops()),
        usvg::Paint::RadialGradient(grad) => average_gradient_color(grad.stops()),
        usvg::Paint::Pattern(_) => Srgba::new(0.5, 0.5, 0.5, 1.0),
    }
}

/// Extract average color from gradient stops
fn average_gradient_color(stops: &[usvg::Stop]) -> Srgba {
    if stops.is_empty() {
        return Srgba::new(0.5, 0.5, 0.5, 1.0);
    }

    let mut r_sum = 0.0_f32;
    let mut g_sum = 0.0_f32;
    let mut b_sum = 0.0_f32;
    let mut a_sum = 0.0_f32;

    for stop in stops {
        r_sum += stop.color().red as f32 / 255.0;
        g_sum += stop.color().green as f32 / 255.0;
        b_sum += stop.color().blue as f32 / 255.0;
        a_sum += stop.opacity().get();
    }

    let n = stops.len() as f32;
    Srgba::new(r_sum / n, g_sum / n, b_sum / n, a_sum / n)
}

/// Convert Srgba to [u8; 4] RGBA
fn srgba_to_rgba(color: Srgba) -> [u8; 4] {
    let (r, g, b, a): (u8, u8, u8, u8) = color.into_format().into_components();
    [r, g, b, a]
}

/// Convert tiny_skia::Path to SVG path data string
fn skia_path_to_svg_string(path: &tiny_skia::Path) -> String {
    let mut svg = String::new();
    for seg in path.segments() {
        match seg {
            tiny_skia::PathSegment::MoveTo(p) => {
                let _ = write!(svg, "M{:.2} {:.2} ", p.x, p.y);
            }
            tiny_skia::PathSegment::LineTo(p) => {
                let _ = write!(svg, "L{:.2} {:.2} ", p.x, p.y);
            }
            tiny_skia::PathSegment::QuadTo(p1, p2) => {
                let _ = write!(svg, "Q{:.2} {:.2} {:.2} {:.2} ", p1.x, p1.y, p2.x, p2.y);
            }
            tiny_skia::PathSegment::CubicTo(p1, p2, p3) => {
                let _ = write!(
                    svg,
                    "C{:.2} {:.2} {:.2} {:.2} {:.2} {:.2} ",
                    p1.x, p1.y, p2.x, p2.y, p3.x, p3.y
                );
            }
            tiny_skia::PathSegment::Close => {
                svg.push_str("Z ");
            }
        }
    }
    svg.trim().to_string()
}

/// Convert SkiaOpset to SketchElements
fn opset_to_elements(set: &SkiaOpset<f64>, options: &vruffr_core::core::Options) -> Vec<SketchElement> {
    let path = match &set.ops {
        Some(p) => p,
        None => return vec![],
    };
    let path_data = skia_path_to_svg_string(path);
    if path_data.is_empty() {
        return vec![];
    }

    let stroke_color = options.stroke.map(srgba_to_rgba);
    let fill_color = options.fill.map(srgba_to_rgba);
    let stroke_width = options.stroke_width.unwrap_or(1.0);
    let fill_weight = options.fill_weight.unwrap_or(0.5);

    match set.op_set_type {
        OpSetType::Path => vec![SketchElement {
            path_data,
            stroke: stroke_color,
            fill: None,
            stroke_width,
            is_fill_sketch: false,
        }],
        OpSetType::FillPath => vec![SketchElement {
            path_data,
            stroke: None,
            fill: fill_color,
            stroke_width: 0.0,
            is_fill_sketch: false,
        }],
        OpSetType::FillSketch => vec![SketchElement {
            path_data,
            stroke: fill_color,
            fill: None,
            stroke_width: fill_weight,
            is_fill_sketch: true,
        }],
    }
}

/// Raw path info extracted from usvg for deduplication
struct RawPathInfo {
    path_data: String,
    fill_color: Option<Srgba>,
    stroke_color: Option<Srgba>,
    stroke_width: f32,
    effective_roughness: f32,
}

/// Collect raw path info from usvg group for deduplication
fn collect_raw_paths(
    group: &usvg::Group,
    options: &SketchOptions,
    warnings: &mut RenderWarnings,
) -> Vec<RawPathInfo> {
    let mut paths = Vec::new();
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => {
                paths.extend(collect_raw_paths(g, options, warnings));
            }
            usvg::Node::Path(path) => {
                let svg_path = path_to_svg_string(path);
                if svg_path.is_empty() {
                    continue;
                }
                let effective_roughness = compute_effective_roughness(path, options);
                let fill_color = if options.no_fill {
                    None
                } else {
                    path.fill().map(|f| extract_color(f.paint()))
                };
                let (stroke_color, stroke_width) = if options.no_stroke {
                    (None, 1.0)
                } else if let Some(stroke) = path.stroke() {
                    (
                        Some(extract_color(stroke.paint())),
                        options.stroke_width.unwrap_or_else(|| stroke.width().get()),
                    )
                } else {
                    (None, 1.0)
                };
                paths.push(RawPathInfo {
                    path_data: svg_path,
                    fill_color,
                    stroke_color,
                    stroke_width,
                    effective_roughness,
                });
            }
            usvg::Node::Image(_) => {
                warnings.has_images = true;
            }
            usvg::Node::Text(text) => {
                paths.extend(collect_raw_paths(text.flattened(), options, warnings));
            }
        }
    }
    paths
}

/// Apply deduplication to raw paths and return deduplicated list
fn apply_dedup(paths: Vec<RawPathInfo>, epsilon: f32) -> Vec<RawPathInfo> {
    // Convert to StyledPath for dedup module
    let styled: Vec<StyledPath> = paths
        .iter()
        .enumerate()
        .map(|(i, p)| StyledPath {
            path_data: p.path_data.clone(),
            signature: PathSignature::from_path_data(&p.path_data),
            stroke: p.stroke_color.map(srgba_to_rgba),
            stroke_width: Some(p.stroke_width),
            fill: p.fill_color.map(srgba_to_rgba),
            original_index: i,
        })
        .collect();

    let groups = deduplicate_paths(styled, epsilon);

    // For each duplicate group, use the canonical path but keep all unique styles
    // For now, we just keep the canonical (first) path from each group
    // A more sophisticated approach would render multiple styles per geometry
    groups
        .into_iter()
        .map(|g| {
            let idx = g.canonical.original_index;
            RawPathInfo {
                path_data: g.canonical.path_data,
                fill_color: paths[idx].fill_color,
                stroke_color: paths[idx].stroke_color,
                stroke_width: paths[idx].stroke_width,
                effective_roughness: paths[idx].effective_roughness,
            }
        })
        .collect()
}

/// Sketch a raw path info into elements
fn sketch_raw_path(info: &RawPathInfo, options: &SketchOptions) -> Vec<SketchElement> {
    let mut elements = Vec::new();

    // Handle fill
    if let Some(fill_color) = info.fill_color {
        let fill_options = OptionsBuilder::default()
            .roughness(info.effective_roughness)
            .bowing(options.bowing as f32)
            .seed(options.seed)
            .fill(fill_color)
            .fill_style(options.fill_style.into())
            .stroke(fill_color)
            .stroke_width(options.fill_weight)
            .fill_weight(options.fill_weight)
            .hachure_angle(options.hachure_angle)
            .hachure_gap(options.hachure_gap)
            .build()
            .unwrap();

        let fill_gen = SkiaGenerator::new(fill_options.clone());
        let result = fill_gen.path::<f64>(info.path_data.clone());
        for set in &result.sets {
            elements.extend(opset_to_elements(set, &fill_options));
        }
    }

    // Handle stroke
    if let Some(stroke_color) = info.stroke_color {
        let stroke_options = OptionsBuilder::default()
            .roughness(info.effective_roughness)
            .bowing(options.bowing as f32)
            .seed(options.seed)
            .stroke(stroke_color)
            .stroke_width(info.stroke_width)
            .build()
            .unwrap();

        let stroke_gen = SkiaGenerator::new(stroke_options.clone());
        let result = stroke_gen.path::<f64>(info.path_data.clone());
        for set in &result.sets {
            elements.extend(opset_to_elements(set, &stroke_options));
        }
    }

    elements
}

/// Collect sketch elements from a usvg path
fn collect_path_elements(path: &usvg::Path, options: &SketchOptions) -> Vec<SketchElement> {
    let svg_path = path_to_svg_string(path);
    if svg_path.is_empty() {
        return vec![];
    }

    let effective_roughness = compute_effective_roughness(path, options);
    let mut elements = Vec::new();

    // Handle fill
    if !options.no_fill {
        if let Some(fill) = path.fill() {
            let fill_color = extract_color(fill.paint());
            let fill_options = OptionsBuilder::default()
                .roughness(effective_roughness)
                .bowing(options.bowing as f32)
                .seed(options.seed)
                .fill(fill_color)
                .fill_style(options.fill_style.into())
                .stroke(fill_color)
                .stroke_width(options.fill_weight)
                .fill_weight(options.fill_weight)
                .hachure_angle(options.hachure_angle)
                .hachure_gap(options.hachure_gap)
                .build()
                .unwrap();

            let fill_gen = SkiaGenerator::new(fill_options.clone());
            let result = fill_gen.path::<f64>(svg_path.clone());
            for set in &result.sets {
                elements.extend(opset_to_elements(set, &fill_options));
            }
        }
    }

    // Handle stroke
    if !options.no_stroke {
        if let Some(stroke) = path.stroke() {
            let stroke_color = extract_color(stroke.paint());
            let stroke_width = options.stroke_width.unwrap_or_else(|| stroke.width().get());

            let stroke_options = OptionsBuilder::default()
                .roughness(effective_roughness)
                .bowing(options.bowing as f32)
                .seed(options.seed)
                .stroke(stroke_color)
                .stroke_width(stroke_width)
                .build()
                .unwrap();

            let stroke_gen = SkiaGenerator::new(stroke_options.clone());
            let result = stroke_gen.path::<f64>(svg_path);
            for set in &result.sets {
                elements.extend(opset_to_elements(set, &stroke_options));
            }
        }
    }

    elements
}

/// Collect all sketch elements from a usvg group
fn collect_group_elements(
    group: &usvg::Group,
    options: &SketchOptions,
    warnings: &mut RenderWarnings,
) -> Vec<SketchElement> {
    let mut elements = Vec::new();
    for node in group.children() {
        match node {
            usvg::Node::Group(g) => {
                elements.extend(collect_group_elements(g, options, warnings));
            }
            usvg::Node::Path(path) => {
                elements.extend(collect_path_elements(path, options));
            }
            usvg::Node::Image(_) => {
                warnings.has_images = true;
            }
            usvg::Node::Text(text) => {
                // Process flattened text (text converted to paths)
                elements.extend(collect_group_elements(text.flattened(), options, warnings));
            }
        }
    }
    elements
}

/// Render sketch elements to SVG string
pub fn elements_to_svg(
    elements: &[SketchElement],
    width: u32,
    height: u32,
    background: Option<[u8; 4]>,
) -> String {
    let mut svg = String::new();
    let _ = writeln!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">"#,
        width, height, width, height
    );

    // Background rect if specified
    if let Some([r, g, b, a]) = background {
        let _ = writeln!(
            svg,
            r#"  <rect width="100%" height="100%" fill="rgba({},{},{},{:.2})"/>"#,
            r,
            g,
            b,
            a as f32 / 255.0
        );
    }

    for elem in elements {
        let mut attrs = vec![format!(r#"d="{}""#, elem.path_data)];

        if let Some([r, g, b, a]) = elem.fill {
            attrs.push(format!(
                r#"fill="rgba({},{},{},{:.2})""#,
                r,
                g,
                b,
                a as f32 / 255.0
            ));
        } else {
            attrs.push(r#"fill="none""#.to_string());
        }

        if let Some([r, g, b, a]) = elem.stroke {
            attrs.push(format!(
                r#"stroke="rgba({},{},{},{:.2})""#,
                r,
                g,
                b,
                a as f32 / 255.0
            ));
            attrs.push(format!(r#"stroke-width="{:.2}""#, elem.stroke_width));
            attrs.push(r#"stroke-linecap="round""#.to_string());
            attrs.push(r#"stroke-linejoin="round""#.to_string());
        } else {
            attrs.push(r#"stroke="none""#.to_string());
        }

        let _ = writeln!(svg, "  <path {}/>", attrs.join(" "));
    }

    svg.push_str("</svg>\n");
    svg
}

/// Render SVG to sketch elements (intermediate representation)
pub fn render_to_elements(
    svg_data: &str,
    options: &SketchOptions,
) -> Result<(Vec<SketchElement>, u32, u32, RenderWarnings)> {
    let usvg_options = build_usvg_options(Some(options));
    let tree = usvg::Tree::from_str(svg_data, &usvg_options).context("Failed to parse SVG")?;

    let svg_size = tree.size();
    let base_width = options.width.unwrap_or(svg_size.width() as u32);
    let base_height = options.height.unwrap_or(svg_size.height() as u32);
    let width = (base_width as f32 * options.scale) as u32;
    let height = (base_height as f32 * options.scale) as u32;

    let mut warnings = RenderWarnings::default();

    let elements = if options.deduplicate {
        // Collect raw paths, deduplicate, then sketch
        let raw_paths = collect_raw_paths(tree.root(), options, &mut warnings);
        let deduped = apply_dedup(raw_paths, options.dedup_epsilon);
        deduped
            .iter()
            .flat_map(|p| sketch_raw_path(p, options))
            .collect()
    } else {
        // Original path: sketch each path as encountered
        collect_group_elements(tree.root(), options, &mut warnings)
    };

    Ok((elements, width, height, warnings))
}

/// Render SVG to sketch-style SVG string (plain format)
pub fn render_to_svg(svg_data: &str, options: &SketchOptions) -> Result<(String, RenderWarnings)> {
    let (elements, width, height, warnings) = render_to_elements(svg_data, options)?;
    let svg = elements_to_svg(&elements, width, height, options.background);
    Ok((svg, warnings))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rect() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="blue" stroke="black"/>
        </svg>"#;

        let options = SketchOptions::default();
        let result = render_sketch(svg, &options);
        assert!(result.is_ok());

        let pixmap = result.unwrap();
        assert_eq!(pixmap.width(), 100);
        assert_eq!(pixmap.height(), 100);
    }

    #[test]
    fn test_empty_svg() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="50" height="50"></svg>"#;
        let options = SketchOptions::default();
        let result = render_sketch(svg, &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_gradient_svg() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <defs>
                <linearGradient id="grad1" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" style="stop-color:rgb(255,0,0);stop-opacity:1" />
                    <stop offset="100%" style="stop-color:rgb(0,0,255);stop-opacity:1" />
                </linearGradient>
            </defs>
            <rect x="10" y="10" width="80" height="80" fill="url(#grad1)"/>
        </svg>"#;

        let options = SketchOptions::default();
        let result = render_sketch(svg, &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_average_gradient_color_empty_stops() {
        let color = average_gradient_color(&[]);
        assert!((color.red - 0.5).abs() < 0.01);
        assert!((color.green - 0.5).abs() < 0.01);
        assert!((color.blue - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_validate_svg_basic() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50"></svg>"#;
        let info = validate_svg(svg).expect("validate failed");
        assert_eq!(info.width, 100);
        assert_eq!(info.height, 50);
        assert!(!info.warnings.has_text);
        assert!(!info.warnings.has_images);
    }

    #[test]
    fn test_validate_svg_invalid() {
        let result = validate_svg("not valid svg");
        assert!(result.is_err());
    }

    #[test]
    fn test_sketch_fill_style_from_str() {
        assert_eq!(
            "hachure".parse::<SketchFillStyle>().unwrap(),
            SketchFillStyle::Hachure
        );
        assert_eq!(
            "crosshatch".parse::<SketchFillStyle>().unwrap(),
            SketchFillStyle::CrossHatch
        );
        assert_eq!(
            "cross-hatch".parse::<SketchFillStyle>().unwrap(),
            SketchFillStyle::CrossHatch
        );
        assert!("invalid".parse::<SketchFillStyle>().is_err());
    }

    #[test]
    fn test_sketch_fill_style_into_fill_style() {
        let hachure: FillStyle = SketchFillStyle::Hachure.into();
        assert!(matches!(hachure, FillStyle::Hachure));

        let crosshatch: FillStyle = SketchFillStyle::CrossHatch.into();
        assert!(matches!(crosshatch, FillStyle::CrossHatch));
    }

    #[test]
    fn test_render_warnings_default() {
        let warnings = RenderWarnings::default();
        assert!(!warnings.has_text);
        assert!(!warnings.has_images);
    }

    #[test]
    fn test_sketch_options_default_values() {
        let opts = SketchOptions::default();
        assert!((opts.roughness - 1.0).abs() < 0.001);
        assert!((opts.bowing - 1.0).abs() < 0.001);
        assert_eq!(opts.seed, 42);
        assert!(opts.width.is_none());
        assert!(opts.height.is_none());
        assert_eq!(opts.fill_style, SketchFillStyle::CrossHatch);
        assert!((opts.hachure_angle - (-41.0)).abs() < 0.1);
        assert!((opts.hachure_gap - 4.0).abs() < 0.1);
        assert!(opts.stroke_width.is_none());
        assert_eq!(opts.background, Some([255, 255, 255, 255]));
        assert!(!opts.no_fill);
        assert!(!opts.no_stroke);
        assert!((opts.fill_weight - 0.5).abs() < 0.1);
        assert!((opts.scale - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_svg_info_fields() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="300" height="200">
            <rect x="10" y="10" width="100" height="100" fill="blue"/>
        </svg>"#;
        let info = validate_svg(svg).expect("validate failed");

        assert_eq!(info.width, 300);
        assert_eq!(info.height, 200);
        assert_eq!(info.path_count, 1);
        assert!(!info.warnings.has_text);
        assert!(!info.warnings.has_images);
    }

    #[test]
    fn test_empty_svg_no_paths() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="50" height="50"></svg>"#;
        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render empty SVG");

        assert_eq!(pixmap.width(), 50);
        assert_eq!(pixmap.height(), 50);
    }

    #[test]
    fn test_sketch_fill_style_debug() {
        let hachure = SketchFillStyle::Hachure;
        let crosshatch = SketchFillStyle::CrossHatch;

        assert_eq!(format!("{:?}", hachure), "Hachure");
        assert_eq!(format!("{:?}", crosshatch), "CrossHatch");
    }

    #[test]
    fn test_sketch_options_clone() {
        let opts = SketchOptions { roughness: 2.5, seed: 999, ..Default::default() };
        let cloned = opts.clone();

        assert!((cloned.roughness - 2.5).abs() < 0.001);
        assert_eq!(cloned.seed, 999);
    }

    #[test]
    fn test_render_warnings_debug() {
        let warnings = RenderWarnings { has_text: true, has_images: false };
        let debug_str = format!("{:?}", warnings);

        assert!(debug_str.contains("has_text"));
        assert!(debug_str.contains("true"));
    }

    #[test]
    fn test_svg_info_debug() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"></svg>"#;
        let info = validate_svg(svg).expect("validate failed");
        let debug_str = format!("{:?}", info);

        assert!(debug_str.contains("width"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_sketch_fill_style_copy() {
        let style = SketchFillStyle::Hachure;
        let copied = style; // Copy trait allows this
        let also_copied = style; // Can use original again

        assert_eq!(copied, SketchFillStyle::Hachure);
        assert_eq!(also_copied, SketchFillStyle::Hachure);
    }

    #[test]
    fn test_sketch_fill_style_partial_eq() {
        assert_eq!(SketchFillStyle::Hachure, SketchFillStyle::Hachure);
        assert_eq!(SketchFillStyle::CrossHatch, SketchFillStyle::CrossHatch);
        assert_ne!(SketchFillStyle::Hachure, SketchFillStyle::CrossHatch);
    }

    #[test]
    fn test_sketch_options_debug() {
        let opts = SketchOptions::default();
        let debug_str = format!("{:?}", opts);

        assert!(debug_str.contains("roughness"));
        assert!(debug_str.contains("bowing"));
        assert!(debug_str.contains("seed"));
    }

    #[test]
    fn test_sketch_fill_style_eq() {
        // Eq trait requires reflexive equality
        let style = SketchFillStyle::CrossHatch;
        assert!(style == style);
    }

    #[test]
    fn test_sketch_fill_style_default() {
        let default_style = SketchFillStyle::default();
        assert_eq!(default_style, SketchFillStyle::CrossHatch);
    }

    #[test]
    fn test_gradient_svg_renders() {
        // Test that gradients are handled correctly (average color extracted)
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <defs>
                <linearGradient id="g1">
                    <stop offset="0%" stop-color="red"/>
                    <stop offset="100%" stop-color="blue"/>
                </linearGradient>
            </defs>
            <rect fill="url(#g1)" width="100" height="100"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render gradient");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_style_attribute_parsing() {
        // Test inline style attribute parsing
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" style="fill:green;stroke:black;stroke-width:2"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render style attribute");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_css_class_in_defs() {
        // Test CSS class defined in defs/style
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <defs>
                <style>.box { fill: purple; }</style>
            </defs>
            <rect class="box" x="10" y="10" width="80" height="80"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render CSS class");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_preserve_aspect_ratio() {
        // Test preserveAspectRatio attribute
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 200 100" width="100" height="100" preserveAspectRatio="xMidYMid meet">
            <rect x="50" y="25" width="100" height="50" fill="orange"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render preserveAspectRatio");
        assert_eq!(pixmap.width(), 100);
        assert_eq!(pixmap.height(), 100);
    }

    #[test]
    fn test_use_element() {
        // Test defs with use element reference
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="100" height="100">
            <defs>
                <rect id="myRect" width="20" height="20" fill="blue"/>
            </defs>
            <use xlink:href="#myRect" x="10" y="10"/>
            <use xlink:href="#myRect" x="50" y="50"/>
        </svg>"##;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render use element");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_stroke_linecap_linejoin() {
        // Test stroke-linecap and stroke-linejoin attributes
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="M 10,10 L 50,50 L 90,10" fill="none" stroke="black" stroke-width="5" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render stroke styles");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_smooth_curve_s_command() {
        // Test path with S (smooth cubic bezier) command
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="M 10,50 C 20,10 40,10 50,50 S 80,90 90,50" fill="none" stroke="green" stroke-width="2"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render smooth curve");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_arc_path_command() {
        // Test path with A (arc) command
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="M 10,50 A 40,40 0 1,1 90,50 A 40,40 0 1,1 10,50" fill="none" stroke="red" stroke-width="2"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render arc path");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_nested_group_transforms() {
        // Test nested groups with cumulative transforms
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <g transform="translate(10, 10)">
                <g transform="rotate(45)">
                    <rect width="20" height="20" fill="blue"/>
                </g>
            </g>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render nested transforms");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_zero_dimension_svg() {
        // Test SVG with zero width or height - should fail gracefully
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="0" height="100">
            <rect width="50" height="50" fill="red"/>
        </svg>"#;

        let options = SketchOptions::default();
        let result = render_sketch(svg, &options);
        // Zero-dimension SVG should fail (can't create valid pixmap)
        assert!(result.is_err());
    }

    #[test]
    fn test_quadratic_bezier_q_command() {
        // Test path with Q (quadratic bezier) command
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="M 10,50 Q 50,10 90,50" fill="none" stroke="purple" stroke-width="2"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render quadratic bezier");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_stroke_dasharray() {
        // Test stroke-dasharray attribute
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <line x1="10" y1="50" x2="90" y2="50" stroke="black" stroke-width="2" stroke-dasharray="5,3"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render dashed stroke");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_multiple_paths() {
        // Test multiple paths in single SVG
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="M 10,10 L 50,10" stroke="red" fill="none"/>
            <path d="M 10,30 L 50,30" stroke="green" fill="none"/>
            <path d="M 10,50 L 50,50" stroke="blue" fill="none"/>
            <rect x="60" y="10" width="30" height="30" fill="yellow"/>
            <circle cx="75" cy="70" r="15" fill="orange"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render multiple paths");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_smooth_quadratic_t_command() {
        // Test path with T (smooth quadratic bezier) command
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="M 10,50 Q 30,10 50,50 T 90,50" fill="none" stroke="teal" stroke-width="2"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render smooth quadratic");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_opacity_attribute() {
        // Test opacity attribute on shapes
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="40" height="40" fill="red" opacity="0.5"/>
            <circle cx="70" cy="70" r="20" fill="blue" fill-opacity="0.7"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render opacity");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_radial_gradient() {
        // Test radial gradient rendering (average color extraction)
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <defs>
                <radialGradient id="rg1" cx="50%" cy="50%" r="50%">
                    <stop offset="0%" stop-color="yellow"/>
                    <stop offset="100%" stop-color="red"/>
                </radialGradient>
            </defs>
            <circle cx="50" cy="50" r="40" fill="url(#rg1)"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render radial gradient");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_stroke_opacity() {
        // Test stroke-opacity attribute
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <line x1="10" y1="10" x2="90" y2="90" stroke="red" stroke-width="5" stroke-opacity="0.5"/>
            <rect x="20" y="20" width="60" height="60" fill="none" stroke="blue" stroke-width="3" stroke-opacity="0.7"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render stroke opacity");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_elliptical_arc() {
        // Test elliptical arc with different rx and ry
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="M 10,50 A 40,20 0 1,1 90,50" fill="none" stroke="purple" stroke-width="2"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render elliptical arc");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_path_h_v_commands() {
        // Test path with H (horizontal) and V (vertical) line commands
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="M 10,10 H 90 V 90 H 10 V 10 Z" fill="none" stroke="black" stroke-width="2"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render H/V commands");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_marker_elements() {
        // Test SVG with marker elements (arrow heads) - markers are flattened by usvg
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <defs>
                <marker id="arrow" viewBox="0 0 10 10" refX="5" refY="5" markerWidth="4" markerHeight="4" orient="auto">
                    <path d="M 0,0 L 10,5 L 0,10 Z" fill="black"/>
                </marker>
            </defs>
            <line x1="10" y1="50" x2="80" y2="50" stroke="black" stroke-width="2" marker-end="url(#arrow)"/>
        </svg>"##;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render marker");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_clip_path() {
        // Test SVG with clipPath element
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <defs>
                <clipPath id="clip1">
                    <circle cx="50" cy="50" r="30"/>
                </clipPath>
            </defs>
            <rect x="10" y="10" width="80" height="80" fill="blue" clip-path="url(#clip1)"/>
        </svg>"##;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render clipPath");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_text_element_rendering() {
        // Test SVG with text element - usvg may convert to paths or skip without fonts
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <text x="10" y="50" font-size="20">Hello</text>
            <rect x="10" y="60" width="80" height="30" fill="green"/>
        </svg>"#;

        let options = SketchOptions::default();
        let (pixmap, _warnings) =
            render_sketch_with_warnings(svg, &options).expect("Failed to render with text");
        assert_eq!(pixmap.width(), 100);
        // Text handling depends on font availability - just verify rendering succeeds
    }

    #[test]
    fn test_image_element_warning() {
        // Test SVG with embedded image element - should set has_images warning
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="100" height="100">
            <image x="10" y="10" width="80" height="80" xlink:href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="/>
            <rect x="5" y="5" width="90" height="90" fill="none" stroke="red"/>
        </svg>"#;

        let options = SketchOptions::default();
        let (pixmap, warnings) =
            render_sketch_with_warnings(svg, &options).expect("Failed to render with image");
        assert_eq!(pixmap.width(), 100);
        // Image elements should set has_images warning flag
        assert!(warnings.has_images);
    }

    #[test]
    fn test_mask_element() {
        // Test SVG with mask element
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <defs>
                <mask id="mask1">
                    <rect x="0" y="0" width="100" height="100" fill="white"/>
                    <circle cx="50" cy="50" r="25" fill="black"/>
                </mask>
            </defs>
            <rect x="10" y="10" width="80" height="80" fill="blue" mask="url(#mask1)"/>
        </svg>"##;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render mask");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_pattern_fill() {
        // Test SVG with pattern fill
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <defs>
                <pattern id="pat1" width="10" height="10" patternUnits="userSpaceOnUse">
                    <rect width="5" height="5" fill="red"/>
                    <rect x="5" y="5" width="5" height="5" fill="blue"/>
                </pattern>
            </defs>
            <rect x="10" y="10" width="80" height="80" fill="url(#pat1)"/>
        </svg>"##;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render pattern");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_filter_element() {
        // Test SVG with filter element (blur) - usvg handles filter application
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <defs>
                <filter id="blur1">
                    <feGaussianBlur in="SourceGraphic" stdDeviation="2"/>
                </filter>
            </defs>
            <rect x="20" y="20" width="60" height="60" fill="green" filter="url(#blur1)"/>
        </svg>"##;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render filter");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_symbol_with_use() {
        // Test SVG with symbol element referenced by use
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="100" height="100">
            <defs>
                <symbol id="sym1" viewBox="0 0 20 20">
                    <circle cx="10" cy="10" r="8" fill="purple"/>
                </symbol>
            </defs>
            <use xlink:href="#sym1" x="10" y="10" width="30" height="30"/>
            <use xlink:href="#sym1" x="60" y="60" width="30" height="30"/>
        </svg>"##;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render symbol");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_switch_element() {
        // Test SVG with switch element for conditional rendering
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <switch>
                <rect x="10" y="10" width="80" height="80" fill="orange"/>
            </switch>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render switch");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_foreign_object() {
        // Test SVG with foreignObject element - typically ignored by usvg
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <foreignObject x="10" y="10" width="80" height="80">
                <div xmlns="http://www.w3.org/1999/xhtml">HTML content</div>
            </foreignObject>
            <rect x="5" y="5" width="90" height="90" fill="none" stroke="black"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render with foreignObject");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_large_svg_dimensions() {
        // Test SVG with larger dimensions (stress test)
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="1000" height="1000">
            <rect x="100" y="100" width="800" height="800" fill="blue"/>
            <circle cx="500" cy="500" r="300" fill="red"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render large SVG");
        assert_eq!(pixmap.width(), 1000);
        assert_eq!(pixmap.height(), 1000);
    }

    #[test]
    fn test_negative_coordinates() {
        // Test SVG with negative coordinates in viewBox
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="-50 -50 200 200" width="100" height="100">
            <rect x="-40" y="-40" width="80" height="80" fill="green"/>
            <circle cx="50" cy="50" r="30" fill="yellow"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render negative coords");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_relative_path_commands() {
        // Test path with relative commands (lowercase m, l, c, z)
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="m 10,10 l 40,0 l 0,40 l -40,0 z" fill="cyan" stroke="black"/>
            <path d="m 60,60 c 10,-20 30,-20 40,0" fill="none" stroke="red" stroke-width="2"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render relative commands");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_stroke_miterlimit() {
        // Test stroke-miterlimit attribute for sharp corners
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="M 10,90 L 50,10 L 90,90" fill="none" stroke="black" stroke-width="10" stroke-linejoin="miter" stroke-miterlimit="10"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render miterlimit");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_fill_rule_evenodd() {
        // Test fill-rule evenodd vs default nonzero
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <path d="M 25,10 L 10,80 L 90,30 L 10,30 L 90,80 Z" fill="purple" fill-rule="evenodd"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render evenodd fill");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_skew_transform() {
        // Test skewX and skewY transforms
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="20" y="20" width="30" height="30" fill="red" transform="skewX(10)"/>
            <rect x="60" y="60" width="20" height="20" fill="blue" transform="skewY(15)"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render skew transform");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_matrix_transform() {
        // Test matrix transform (a, b, c, d, e, f)
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="0" y="0" width="20" height="20" fill="green" transform="matrix(1.5, 0.5, -0.5, 1.5, 30, 30)"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render matrix transform");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_combined_scale_rotate() {
        // Test combined scale and rotate transforms
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="30" y="30" width="20" height="20" fill="orange" transform="translate(50,50) rotate(45) scale(1.5)"/>
        </svg>"#;

        let options = SketchOptions::default();
        let pixmap = render_sketch(svg, &options).expect("Failed to render combined transform");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_dedup_with_duplicate_rects() {
        // Test deduplication with identical overlapping rects
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="blue"/>
            <rect x="10" y="10" width="80" height="80" fill="red"/>
            <rect x="10" y="10" width="80" height="80" fill="green"/>
        </svg>"#;

        let options = SketchOptions {
            deduplicate: true,
            dedup_epsilon: 0.1,
            ..Default::default()
        };
        let pixmap = render_sketch(svg, &options).expect("Failed to render with dedup");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_dedup_disabled_renders_all() {
        // Test that dedup=false (default) still works
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="blue"/>
            <rect x="10" y="10" width="80" height="80" fill="red"/>
        </svg>"#;

        let options = SketchOptions { deduplicate: false, ..Default::default() };
        let pixmap = render_sketch(svg, &options).expect("Failed to render without dedup");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_dedup_svg_output() {
        // Test deduplication with SVG output
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="blue"/>
            <rect x="10" y="10" width="80" height="80" fill="red"/>
        </svg>"#;

        let options = SketchOptions {
            deduplicate: true,
            dedup_epsilon: 0.1,
            ..Default::default()
        };
        let (svg_out, _warnings) = render_to_svg(svg, &options).expect("Failed SVG output");
        assert!(svg_out.contains("<svg"));
        assert!(svg_out.contains("</svg>"));
    }

    #[test]
    fn test_dedup_unique_paths_preserved() {
        // Test that non-duplicate paths are preserved
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="30" height="30" fill="blue"/>
            <rect x="60" y="60" width="30" height="30" fill="red"/>
        </svg>"#;

        let options = SketchOptions {
            deduplicate: true,
            dedup_epsilon: 0.1,
            ..Default::default()
        };
        let pixmap = render_sketch(svg, &options).expect("Failed to render unique paths");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_color_mode_grayscale() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="red"/>
        </svg>"#;

        let options = SketchOptions {
            color_mode: ColorMode::Grayscale,
            ..Default::default()
        };
        let pixmap = render_sketch(svg, &options).expect("Failed to render grayscale");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_color_mode_sepia() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="blue"/>
        </svg>"#;

        let options = SketchOptions { color_mode: ColorMode::Sepia, ..Default::default() };
        let pixmap = render_sketch(svg, &options).expect("Failed to render sepia");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_noise_effect() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="green"/>
        </svg>"#;

        let options = SketchOptions { noise: 0.5, ..Default::default() };
        let pixmap = render_sketch(svg, &options).expect("Failed to render with noise");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_color_mode_from_str() {
        assert_eq!("color".parse::<ColorMode>().unwrap(), ColorMode::Color);
        assert_eq!(
            "grayscale".parse::<ColorMode>().unwrap(),
            ColorMode::Grayscale
        );
        assert_eq!("gray".parse::<ColorMode>().unwrap(), ColorMode::Grayscale);
        assert_eq!("mono".parse::<ColorMode>().unwrap(), ColorMode::Grayscale);
        assert_eq!("sepia".parse::<ColorMode>().unwrap(), ColorMode::Sepia);
        assert_eq!("invert".parse::<ColorMode>().unwrap(), ColorMode::Invert);
        assert_eq!("negative".parse::<ColorMode>().unwrap(), ColorMode::Invert);
        assert!("invalid".parse::<ColorMode>().is_err());
    }

    #[test]
    fn test_color_mode_invert() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="white"/>
        </svg>"#;

        let options = SketchOptions {
            color_mode: ColorMode::Invert,
            ..Default::default()
        };
        let pixmap = render_sketch(svg, &options).expect("Failed to render inverted");
        assert_eq!(pixmap.width(), 100);
    }

    #[test]
    fn test_combined_post_processing() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
            <rect x="10" y="10" width="80" height="80" fill="purple"/>
        </svg>"#;

        let options = SketchOptions {
            color_mode: ColorMode::Sepia,
            noise: 0.3,
            ..Default::default()
        };
        let pixmap = render_sketch(svg, &options).expect("Failed to render combined effects");
        assert_eq!(pixmap.width(), 100);
    }
}
