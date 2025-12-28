use euclid::default::Point2D;
use euclid::Trig;
use num_traits::{Float, FromPrimitive};
use palette::Srgba;
use rand::rngs::StdRng;
use rand::{random, Rng, SeedableRng};

pub struct Space;

#[allow(dead_code)]
pub struct Config {
    options: Option<Options>,
}

#[allow(dead_code)]
pub struct DrawingSurface {
    width: f32,
    height: f32,
}

#[derive(Clone, PartialEq, Debug, Copy, Eq)]
pub enum FillStyle {
    Solid,
    Hachure,
    ZigZag,
    CrossHatch,
    Dots,
    Dashed,
    ZigZagLine,
}

impl ToString for FillStyle {
    fn to_string(&self) -> String {
        match self {
            FillStyle::Solid => "Solid".into(),
            FillStyle::Hachure => "Hachure".into(),
            FillStyle::ZigZag => "ZigZag".into(),
            FillStyle::CrossHatch => "CrossHatch".into(),
            FillStyle::Dots => "Dots".into(),
            FillStyle::Dashed => "Dashed".into(),
            FillStyle::ZigZagLine => "ZigZagLine".into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LineCap {
    Butt,
    Round,
    Square,
}

impl Default for LineCap {
    fn default() -> Self {
        LineCap::Butt
    }
}

/// Options for angled joins in strokes.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LineJoin {
    Miter { limit: f64 },
    Round,
    Bevel,
}
impl LineJoin {
    pub const DEFAULT_MITER_LIMIT: f64 = 10.0;
}
impl Default for LineJoin {
    fn default() -> Self {
        LineJoin::Miter { limit: LineJoin::DEFAULT_MITER_LIMIT }
    }
}

#[derive(Clone, Builder)]
#[builder(setter(strip_option))]
pub struct Options {
    #[builder(default = "Some(2.0)")]
    pub max_randomness_offset: Option<f32>,
    #[builder(default = "Some(1.0)")]
    pub roughness: Option<f32>,
    #[builder(default = "Some(2.0)")]
    pub bowing: Option<f32>,
    #[builder(default = "Some(Srgba::new(0.0, 0.0, 0.0, 1.0))")]
    pub stroke: Option<Srgba>,
    #[builder(default = "Some(1.0)")]
    pub stroke_width: Option<f32>,
    #[builder(default = "Some(0.95)")]
    pub curve_fitting: Option<f32>,
    #[builder(default = "Some(0.0)")]
    pub curve_tightness: Option<f32>,
    #[builder(default = "Some(9.0)")]
    pub curve_step_count: Option<f32>,
    #[builder(default = "None")]
    pub fill: Option<Srgba>,
    #[builder(default = "None")]
    pub fill_style: Option<FillStyle>,
    #[builder(default = "Some(-1.0)")]
    pub fill_weight: Option<f32>,
    #[builder(default = "Some(-41.0)")]
    pub hachure_angle: Option<f32>,
    #[builder(default = "Some(-1.0)")]
    pub hachure_gap: Option<f32>,
    #[builder(default = "Some(1.0)")]
    pub simplification: Option<f32>,
    #[builder(default = "Some(-1.0)")]
    pub dash_offset: Option<f32>,
    #[builder(default = "Some(-1.0)")]
    pub dash_gap: Option<f32>,
    #[builder(default = "Some(-1.0)")]
    pub zigzag_offset: Option<f32>,
    #[builder(default = "Some(345_u64)")]
    pub seed: Option<u64>,
    #[builder(default = "None")]
    pub stroke_line_dash: Option<Vec<f64>>,
    #[builder(default = "None")]
    pub stroke_line_dash_offset: Option<f64>,
    #[builder(default = "None")]
    pub line_cap: Option<LineCap>,
    #[builder(default = "None")]
    pub line_join: Option<LineJoin>,
    #[builder(default = "None")]
    pub fill_line_dash: Option<Vec<f64>>,
    #[builder(default = "None")]
    pub fill_line_dash_offset: Option<f64>,
    #[builder(default = "Some(false)")]
    pub disable_multi_stroke: Option<bool>,
    #[builder(default = "Some(false)")]
    pub disable_multi_stroke_fill: Option<bool>,
    #[builder(default = "Some(false)")]
    pub preserve_vertices: Option<bool>,
    #[builder(default = "None")]
    pub fixed_decimal_place_digits: Option<f32>,
    #[builder(default = "None")]
    pub randomizer: Option<StdRng>,
    /// Adaptive roughness strength (0.0 = disabled, 1.0 = normal, 2.0 = aggressive).
    /// When enabled, roughness is scaled based on element size relative to reference_size.
    #[builder(default = "Some(0.0)")]
    pub adaptive_strength: Option<f32>,
    /// Reference element size in pixels for adaptive roughness scaling.
    /// Elements of this size use the base roughness value.
    #[builder(default = "Some(100.0)")]
    pub reference_size: Option<f32>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            max_randomness_offset: Some(2.0),
            roughness: Some(1.0),
            bowing: Some(2.0),
            stroke: Some(Srgba::new(0.0, 0.0, 0.0, 1.0)),
            stroke_width: Some(1.0),
            curve_tightness: Some(0.0),
            curve_fitting: Some(0.95),
            curve_step_count: Some(9.0),
            fill: None,
            fill_style: None,
            fill_weight: Some(-1.0),
            hachure_angle: Some(-41.0),
            hachure_gap: Some(-1.0),
            dash_offset: Some(-1.0),
            dash_gap: Some(-1.0),
            zigzag_offset: Some(-1.0),
            seed: Some(345_u64),
            disable_multi_stroke: Some(false),
            disable_multi_stroke_fill: Some(false),
            preserve_vertices: Some(false),
            simplification: Some(1.0),
            stroke_line_dash: None,
            stroke_line_dash_offset: None,
            line_cap: None,
            line_join: None,
            fill_line_dash: None,
            fill_line_dash_offset: None,
            fixed_decimal_place_digits: None,
            randomizer: None,
            adaptive_strength: Some(0.0),
            reference_size: Some(100.0),
        }
    }
}

impl Options {
    /// Calculate the effective roughness for an element of the given size.
    /// If adaptive_strength is 0 (disabled), returns the base roughness unchanged.
    /// Otherwise, scales roughness based on element size relative to reference_size.
    ///
    /// Formula: effective = base * (size / reference_size) ^ (strength * 0.5)
    /// - Small elements get reduced roughness (stay legible)
    /// - Large elements can get increased roughness
    /// - Scale is clamped to [0.2, 2.0] range
    pub fn effective_roughness(&self, element_size: f32) -> f32 {
        let base_roughness = self.roughness.unwrap_or(1.0);
        let adaptive_strength = self.adaptive_strength.unwrap_or(0.0);

        if adaptive_strength <= 0.0 || element_size <= 0.0 {
            return base_roughness;
        }

        let reference_size = self.reference_size.unwrap_or(100.0).max(1.0);
        let size_ratio = element_size / reference_size;
        let raw_scale = size_ratio.powf(adaptive_strength * 0.5);

        // Clamp scale to reasonable range
        let scale = raw_scale.clamp(0.2, 2.0);
        base_roughness * scale
    }

    /// Calculate characteristic size from bounding box dimensions.
    /// Uses geometric mean (sqrt of area) for balanced scaling.
    pub fn characteristic_size(width: f32, height: f32) -> f32 {
        (width * height).sqrt()
    }
}

impl Options {
    pub fn random(&mut self) -> f64 {
        match &mut self.randomizer {
            Some(r) => r.gen(),
            None => match self.seed {
                Some(s) => {
                    let rnd = self.randomizer.insert(StdRng::seed_from_u64(s));
                    rnd.gen()
                }
                None => {
                    let rnd = self.randomizer.insert(StdRng::seed_from_u64(random()));
                    rnd.gen()
                }
            },
        }
    }

    pub fn set_hachure_angle(&mut self, angle: Option<f32>) -> &mut Self {
        self.hachure_angle = angle;
        self
    }

    pub fn set_hachure_gap(&mut self, gap: Option<f32>) -> &mut Self {
        self.hachure_gap = gap;
        self
    }
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub enum OpType {
    Move,
    BCurveTo,
    LineTo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpSetType {
    Path,
    FillPath,
    FillSketch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Op<F: Float + Trig> {
    pub op: OpType,
    pub data: Vec<F>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpSet<F: Float + Trig> {
    pub op_set_type: OpSetType,
    pub ops: Vec<Op<F>>,
    pub size: Option<Point2D<F>>,
    pub path: Option<String>,
}

pub struct Drawable<F: Float + Trig> {
    pub shape: String,
    pub options: Options,
    pub sets: Vec<OpSet<F>>,
}

pub struct PathInfo {
    pub d: String,
    pub stroke: Option<Srgba>,
    pub stroke_width: Option<f32>,
    pub fill: Option<Srgba>,
}

pub fn _c<U: Float + FromPrimitive>(inp: f32) -> U {
    U::from(inp).expect("can not parse from f32")
}

pub fn _cc<U: Float + FromPrimitive>(inp: f64) -> U {
    U::from(inp).expect("can not parse from f64")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_roughness_disabled() {
        let opts = Options {
            roughness: Some(1.5),
            adaptive_strength: Some(0.0), // disabled
            ..Default::default()
        };
        // When disabled, any element size returns base roughness
        assert!((opts.effective_roughness(10.0) - 1.5).abs() < 0.001);
        assert!((opts.effective_roughness(100.0) - 1.5).abs() < 0.001);
        assert!((opts.effective_roughness(500.0) - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_effective_roughness_reference_size() {
        let opts = Options {
            roughness: Some(1.0),
            adaptive_strength: Some(1.0),
            reference_size: Some(100.0),
            ..Default::default()
        };
        // Element at reference size should have scale ~1.0
        let eff = opts.effective_roughness(100.0);
        assert!((eff - 1.0).abs() < 0.001, "Expected 1.0, got {}", eff);
    }

    #[test]
    fn test_effective_roughness_small_element() {
        let opts = Options {
            roughness: Some(1.0),
            adaptive_strength: Some(1.0),
            reference_size: Some(100.0),
            ..Default::default()
        };
        // 10px element: scale = (10/100)^0.5 = 0.316
        let eff = opts.effective_roughness(10.0);
        assert!(
            eff < 1.0,
            "Small element should have reduced roughness, got {}",
            eff
        );
        assert!(eff > 0.2, "Should not go below min scale, got {}", eff);
    }

    #[test]
    fn test_effective_roughness_large_element() {
        let opts = Options {
            roughness: Some(1.0),
            adaptive_strength: Some(1.0),
            reference_size: Some(100.0),
            ..Default::default()
        };
        // 400px element: scale = (400/100)^0.5 = 2.0
        let eff = opts.effective_roughness(400.0);
        assert!(
            eff > 1.0,
            "Large element should have increased roughness, got {}",
            eff
        );
        assert!(eff <= 2.0, "Should not exceed max scale, got {}", eff);
    }

    #[test]
    fn test_effective_roughness_clamping() {
        let opts = Options {
            roughness: Some(1.0),
            adaptive_strength: Some(2.0), // aggressive
            reference_size: Some(100.0),
            ..Default::default()
        };
        // Very small element should clamp to 0.2 scale
        let eff_small = opts.effective_roughness(1.0);
        assert!(
            (eff_small - 0.2).abs() < 0.001,
            "Should clamp to 0.2, got {}",
            eff_small
        );

        // Very large element should clamp to 2.0 scale
        let eff_large = opts.effective_roughness(10000.0);
        assert!(
            (eff_large - 2.0).abs() < 0.001,
            "Should clamp to 2.0, got {}",
            eff_large
        );
    }

    #[test]
    fn test_effective_roughness_zero_size() {
        let opts = Options {
            roughness: Some(1.5),
            adaptive_strength: Some(1.0),
            ..Default::default()
        };
        // Zero size should return base roughness
        assert!((opts.effective_roughness(0.0) - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_characteristic_size() {
        // Square: 100x100 = 100
        assert!((Options::characteristic_size(100.0, 100.0) - 100.0).abs() < 0.001);
        // Rectangle: 50x200 = sqrt(10000) = 100
        assert!((Options::characteristic_size(50.0, 200.0) - 100.0).abs() < 0.001);
        // Small: 10x10 = 10
        assert!((Options::characteristic_size(10.0, 10.0) - 10.0).abs() < 0.001);
    }
}
