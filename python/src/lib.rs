//! Python bindings for vruffr sketch renderer
// this_file: python/src/lib.rs

use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// Sketch options for rendering
#[pyclass]
#[derive(Clone)]
pub struct SketchOptions {
    #[pyo3(get, set)]
    pub roughness: f64,
    #[pyo3(get, set)]
    pub bowing: f64,
    #[pyo3(get, set)]
    pub seed: u64,
    #[pyo3(get, set)]
    pub fill_style: String,
    #[pyo3(get, set)]
    pub hachure_angle: f32,
    #[pyo3(get, set)]
    pub hachure_gap: f32,
    #[pyo3(get, set)]
    pub fill_weight: f32,
    #[pyo3(get, set)]
    pub scale: f32,
    #[pyo3(get, set)]
    pub adaptive_strength: f32,
    #[pyo3(get, set)]
    pub reference_size: f32,
    #[pyo3(get, set)]
    pub deduplicate: bool,
    #[pyo3(get, set)]
    pub no_fill: bool,
    #[pyo3(get, set)]
    pub no_stroke: bool,
    #[pyo3(get, set)]
    pub background: Option<String>,
    #[pyo3(get, set)]
    pub color_mode: String,
    #[pyo3(get, set)]
    pub noise: f32,
    #[pyo3(get, set)]
    pub edge_roughen: f32,
}

#[pymethods]
impl SketchOptions {
    #[new]
    #[pyo3(signature = (
        roughness=1.0,
        bowing=1.0,
        seed=42,
        fill_style="crosshatch".to_string(),
        hachure_angle=-41.0,
        hachure_gap=4.0,
        fill_weight=0.5,
        scale=1.0,
        adaptive_strength=0.0,
        reference_size=100.0,
        deduplicate=false,
        no_fill=false,
        no_stroke=false,
        background=None,
        color_mode="color".to_string(),
        noise=0.0,
        edge_roughen=0.0
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        roughness: f64,
        bowing: f64,
        seed: u64,
        fill_style: String,
        hachure_angle: f32,
        hachure_gap: f32,
        fill_weight: f32,
        scale: f32,
        adaptive_strength: f32,
        reference_size: f32,
        deduplicate: bool,
        no_fill: bool,
        no_stroke: bool,
        background: Option<String>,
        color_mode: String,
        noise: f32,
        edge_roughen: f32,
    ) -> Self {
        SketchOptions {
            roughness,
            bowing,
            seed,
            fill_style,
            hachure_angle,
            hachure_gap,
            fill_weight,
            scale,
            adaptive_strength,
            reference_size,
            deduplicate,
            no_fill,
            no_stroke,
            background,
            color_mode,
            noise,
            edge_roughen,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "SketchOptions(roughness={}, bowing={}, seed={}, fill_style='{}')",
            self.roughness, self.bowing, self.seed, self.fill_style
        )
    }
}

impl SketchOptions {
    fn to_vruffr_options(&self) -> vruffr_lib::SketchOptions {
        let fill_style = match self.fill_style.as_str() {
            "hachure" => vruffr_lib::SketchFillStyle::Hachure,
            _ => vruffr_lib::SketchFillStyle::CrossHatch,
        };

        let background = self.background.as_ref().and_then(|bg| parse_color(bg));

        let color_mode = match self.color_mode.as_str() {
            "grayscale" | "gray" | "mono" => vruffr_lib::ColorMode::Grayscale,
            "sepia" => vruffr_lib::ColorMode::Sepia,
            "invert" | "negative" => vruffr_lib::ColorMode::Invert,
            _ => vruffr_lib::ColorMode::Color,
        };

        vruffr_lib::SketchOptions {
            roughness: self.roughness,
            bowing: self.bowing,
            seed: self.seed,
            fill_style,
            hachure_angle: self.hachure_angle,
            hachure_gap: self.hachure_gap,
            fill_weight: self.fill_weight,
            scale: self.scale,
            adaptive_strength: self.adaptive_strength,
            reference_size: self.reference_size,
            deduplicate: self.deduplicate,
            no_fill: self.no_fill,
            no_stroke: self.no_stroke,
            background,
            color_mode,
            noise: self.noise,
            edge_roughen: self.edge_roughen,
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
    Some([255, 255, 255, 255])
}

/// Render SVG to sketch-style PNG bytes
#[pyfunction]
#[pyo3(signature = (svg_data, options=None))]
fn render_to_png<'py>(
    py: Python<'py>,
    svg_data: &str,
    options: Option<&SketchOptions>,
) -> PyResult<Bound<'py, PyBytes>> {
    let opts = options.map(|o| o.to_vruffr_options()).unwrap_or_default();

    let pixmap = vruffr_lib::render_sketch(svg_data, &opts).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Render error: {}", e))
    })?;

    let png_data = pixmap.encode_png().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("PNG encode error: {}", e))
    })?;

    Ok(PyBytes::new_bound(py, &png_data))
}

/// Save SVG to sketch-style PNG file
#[pyfunction]
#[pyo3(signature = (svg_data, output_path, options=None))]
fn render_to_file(
    svg_data: &str,
    output_path: &str,
    options: Option<&SketchOptions>,
) -> PyResult<()> {
    let opts = options.map(|o| o.to_vruffr_options()).unwrap_or_default();

    let pixmap = vruffr_lib::render_sketch(svg_data, &opts).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Render error: {}", e))
    })?;

    pixmap
        .save_png(output_path)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Save error: {}", e)))?;

    Ok(())
}

/// Render SVG to sketch-style SVG string
#[pyfunction]
#[pyo3(signature = (svg_data, options=None))]
fn render_to_svg(svg_data: &str, options: Option<&SketchOptions>) -> PyResult<String> {
    let opts = options.map(|o| o.to_vruffr_options()).unwrap_or_default();

    let (svg_out, _warnings) = vruffr_lib::render_to_svg(svg_data, &opts).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Render error: {}", e))
    })?;

    Ok(svg_out)
}

/// Validate SVG without rendering
#[pyfunction]
fn validate_svg(svg_data: &str) -> PyResult<(u32, u32, usize, bool, bool)> {
    let info = vruffr_lib::validate_svg(svg_data).map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("SVG parse error: {}", e))
    })?;

    Ok((
        info.width,
        info.height,
        info.path_count,
        info.warnings.has_text,
        info.warnings.has_images,
    ))
}

/// Get vruffr version
#[pyfunction]
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Python module definition
#[pymodule]
fn vruffr(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SketchOptions>()?;
    m.add_function(wrap_pyfunction!(render_to_png, m)?)?;
    m.add_function(wrap_pyfunction!(render_to_file, m)?)?;
    m.add_function(wrap_pyfunction!(render_to_svg, m)?)?;
    m.add_function(wrap_pyfunction!(validate_svg, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
