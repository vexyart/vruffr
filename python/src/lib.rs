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
}

#[pymethods]
impl SketchOptions {
    #[new]
    #[pyo3(signature = (roughness=1.0, bowing=1.0, seed=42, fill_style="crosshatch".to_string(), hachure_angle=-41.0, hachure_gap=4.0))]
    fn new(
        roughness: f64,
        bowing: f64,
        seed: u64,
        fill_style: String,
        hachure_angle: f32,
        hachure_gap: f32,
    ) -> Self {
        SketchOptions {
            roughness,
            bowing,
            seed,
            fill_style,
            hachure_angle,
            hachure_gap,
        }
    }
}

/// Render SVG to sketch-style PNG bytes
#[pyfunction]
#[pyo3(signature = (svg_data, options=None))]
fn render_to_png<'py>(
    py: Python<'py>,
    svg_data: &str,
    options: Option<&SketchOptions>,
) -> PyResult<Bound<'py, PyBytes>> {
    let opts = options.cloned().unwrap_or_else(|| SketchOptions::new(
        1.0, 1.0, 42, "crosshatch".to_string(), -41.0, 4.0
    ));

    let fill_style = match opts.fill_style.as_str() {
        "hachure" => roughr::core::FillStyle::Hachure,
        _ => roughr::core::FillStyle::CrossHatch,
    };

    // Parse SVG
    let tree = usvg::Tree::from_str(svg_data, &usvg::Options::default())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("SVG parse error: {}", e)))?;

    let size = tree.size();
    let width = size.width() as u32;
    let height = size.height() as u32;

    // Create pixmap
    let mut pixmap = tiny_skia::Pixmap::new(width, height)
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("Failed to create pixmap"))?;

    // Fill with white background
    pixmap.fill(tiny_skia::Color::WHITE);

    // TODO: Implement full rendering pipeline
    // For now, return a placeholder PNG

    let png_data = pixmap.encode_png()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("PNG encode error: {}", e)))?;

    Ok(PyBytes::new(py, &png_data))
}

/// Render SVG to sketch-style SVG string
#[pyfunction]
#[pyo3(signature = (svg_data, options=None))]
fn render_to_svg(svg_data: &str, options: Option<&SketchOptions>) -> PyResult<String> {
    let opts = options.cloned().unwrap_or_else(|| SketchOptions::new(
        1.0, 1.0, 42, "crosshatch".to_string(), -41.0, 4.0
    ));

    // Parse SVG
    let tree = usvg::Tree::from_str(svg_data, &usvg::Options::default())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("SVG parse error: {}", e)))?;

    let size = tree.size();

    // TODO: Implement full rendering pipeline
    let svg_output = format!(
        r#"<svg viewBox="0 0 {} {}" xmlns="http://www.w3.org/2000/svg">
  <!-- vruffr Python sketch output -->
  <!-- roughness: {}, bowing: {}, seed: {} -->
  <text x="10" y="30" font-size="16" fill="#666">
    Python rendering not yet implemented.
    Use CLI for full functionality.
  </text>
</svg>"#,
        size.width() as u32,
        size.height() as u32,
        opts.roughness,
        opts.bowing,
        opts.seed
    );

    Ok(svg_output)
}

/// Validate SVG without rendering
#[pyfunction]
fn validate_svg(svg_data: &str) -> PyResult<(u32, u32, bool)> {
    let tree = usvg::Tree::from_str(svg_data, &usvg::Options::default())
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("SVG parse error: {}", e)))?;

    let size = tree.size();
    Ok((size.width() as u32, size.height() as u32, true))
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
    m.add_function(wrap_pyfunction!(render_to_svg, m)?)?;
    m.add_function(wrap_pyfunction!(validate_svg, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    Ok(())
}
