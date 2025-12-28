use std::fmt::Display;
use std::ops::MulAssign;

use euclid::default::Point2D;
use euclid::Trig;
use num_traits::{Float, FromPrimitive};
use palette::Srgba;
use roughr::core::{Drawable, OpSet, OpSetType, OpType, Options};
use roughr::generator::Generator;
use tiny_skia::{
    FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, PixmapMut, Stroke, StrokeDash, Transform,
};

#[derive(Default)]
pub struct SkiaGenerator {
    gen: Generator,
    options: Option<Options>,
}

#[derive(Clone)]
pub struct SkiaOpset<F: Float + Trig> {
    pub op_set_type: OpSetType,
    /// The tiny-skia path. May be None if the path was empty or degenerate.
    pub ops: Option<Path>,
    pub size: Option<Point2D<F>>,
    pub path: Option<String>,
}

pub trait ToSkiaOpset<F: Float + Trig> {
    fn to_skia_opset(self) -> SkiaOpset<F>;
}

impl<F: Float + Trig + FromPrimitive> ToSkiaOpset<F> for OpSet<F> {
    fn to_skia_opset(self) -> SkiaOpset<F> {
        SkiaOpset {
            op_set_type: self.op_set_type.clone(),
            size: self.size,
            path: self.path.clone(),
            ops: opset_to_shape(&self),
        }
    }
}

pub struct SkiaDrawable<F: Float + Trig> {
    pub shape: String,
    pub options: Options,
    pub sets: Vec<SkiaOpset<F>>,
}

pub trait ToSkiaDrawable<F: Float + Trig> {
    fn to_skia_drawable(self) -> SkiaDrawable<F>;
}

impl<F: Float + Trig + FromPrimitive> ToSkiaDrawable<F> for Drawable<F> {
    fn to_skia_drawable(self) -> SkiaDrawable<F> {
        SkiaDrawable {
            shape: self.shape,
            options: self.options,
            sets: self.sets.into_iter().map(|s| s.to_skia_opset()).collect(),
        }
    }
}

impl SkiaGenerator {
    pub fn new(options: Options) -> Self {
        SkiaGenerator { gen: Generator::default(), options: Some(options) }
    }
}

impl<F: Float + Trig> SkiaDrawable<F> {
    pub fn draw(&self, ctx: &mut PixmapMut) {
        for set in self.sets.iter() {
            // Skip empty or degenerate paths
            let path = match &set.ops {
                Some(p) => p,
                None => continue,
            };

            match set.op_set_type {
                OpSetType::Path => {
                    if self.options.stroke_line_dash.is_some() {
                        let mut stroke = Stroke {
                            width: self.options.stroke_width.unwrap_or(1.0),
                            line_cap: convert_line_cap_from_roughr_to_piet(self.options.line_cap),
                            line_join: convert_line_join_from_roughr_to_piet(
                                self.options.line_join,
                            ),
                            ..Stroke::default()
                        };
                        let stroke_line_dash = self
                            .options
                            .stroke_line_dash
                            .clone()
                            .unwrap_or(Vec::new())
                            .iter()
                            .map(|&a| a as f32)
                            .collect();

                        stroke.dash = StrokeDash::new(
                            stroke_line_dash,
                            self.options.stroke_line_dash_offset.unwrap_or(1.0f64) as f32,
                        );

                        let stroke_color = self
                            .options
                            .stroke
                            .unwrap_or_else(|| Srgba::from_components((1.0, 1.0, 1.0, 1.0)));
                        let stroke_color_components: (u8, u8, u8, u8) =
                            stroke_color.into_format().into_components();

                        let mut paint = Paint::default();
                        paint.set_color_rgba8(
                            stroke_color_components.0,
                            stroke_color_components.1,
                            stroke_color_components.2,
                            stroke_color_components.3,
                        );
                        paint.anti_alias = true;

                        ctx.stroke_path(path, &paint, &stroke, Transform::identity(), None);
                    } else {
                        let mut stroke = Stroke::default();
                        stroke.width = self.options.stroke_width.unwrap_or(1.0);

                        let stroke_color = self
                            .options
                            .stroke
                            .unwrap_or_else(|| Srgba::from_components((1.0, 1.0, 1.0, 1.0)));
                        let stroke_color_components: (u8, u8, u8, u8) =
                            stroke_color.into_format().into_components();

                        let mut paint = Paint::default();
                        paint.set_color_rgba8(
                            stroke_color_components.0,
                            stroke_color_components.1,
                            stroke_color_components.2,
                            stroke_color_components.3,
                        );
                        paint.anti_alias = true;

                        ctx.stroke_path(path, &paint, &stroke, Transform::identity(), None);
                    }
                }
                OpSetType::FillPath => {
                    let fill_color = self
                        .options
                        .fill
                        .unwrap_or(Srgba::from_components((1.0, 1.0, 1.0, 1.0)));
                    let fill_color_components: (u8, u8, u8, u8) =
                        fill_color.into_format().into_components();

                    let mut paint = Paint::default();
                    paint.set_color_rgba8(
                        fill_color_components.0,
                        fill_color_components.1,
                        fill_color_components.2,
                        fill_color_components.3,
                    );
                    paint.anti_alias = true;
                    match self.shape.as_str() {
                        "curve" | "polygon" | "path" => {
                            ctx.fill_path(
                                path,
                                &paint,
                                FillRule::EvenOdd,
                                Transform::identity(),
                                None,
                            );
                        }
                        _ => {
                            ctx.fill_path(
                                path,
                                &paint,
                                FillRule::Winding,
                                Transform::identity(),
                                None,
                            );
                        }
                    }
                }
                OpSetType::FillSketch => {
                    if self.options.fill_line_dash.is_some() {
                        let mut stroke = Stroke::default();
                        stroke.width = self.options.fill_weight.unwrap_or(1.0);
                        stroke.line_cap =
                            convert_line_cap_from_roughr_to_piet(self.options.line_cap);
                        stroke.line_join =
                            convert_line_join_from_roughr_to_piet(self.options.line_join);
                        let fill_line_dash = self
                            .options
                            .fill_line_dash
                            .clone()
                            .unwrap_or(Vec::new())
                            .iter()
                            .map(|&a| a as f32)
                            .collect();

                        stroke.dash = StrokeDash::new(
                            fill_line_dash,
                            self.options.fill_line_dash_offset.unwrap_or(1.0f64) as f32,
                        );

                        let fill_color = self
                            .options
                            .fill
                            .unwrap_or(Srgba::from_components((1.0, 1.0, 1.0, 1.0)));
                        let fill_color_components: (u8, u8, u8, u8) =
                            fill_color.into_format().into_components();

                        let mut paint = Paint::default();
                        paint.set_color_rgba8(
                            fill_color_components.0,
                            fill_color_components.1,
                            fill_color_components.2,
                            fill_color_components.3,
                        );
                        paint.anti_alias = true;
                        ctx.stroke_path(path, &paint, &stroke, Transform::identity(), None);
                    } else {
                        let mut stroke = Stroke::default();
                        stroke.width = self.options.fill_weight.unwrap_or(1.0);
                        stroke.line_cap =
                            convert_line_cap_from_roughr_to_piet(self.options.line_cap);
                        stroke.line_join =
                            convert_line_join_from_roughr_to_piet(self.options.line_join);

                        let fill_color = self
                            .options
                            .fill
                            .unwrap_or(Srgba::from_components((1.0, 1.0, 1.0, 1.0)));
                        let fill_color_components: (u8, u8, u8, u8) =
                            fill_color.into_format().into_components();

                        let mut paint = Paint::default();
                        paint.set_color_rgba8(
                            fill_color_components.0,
                            fill_color_components.1,
                            fill_color_components.2,
                            fill_color_components.3,
                        );
                        paint.anti_alias = true;
                        ctx.stroke_path(path, &paint, &stroke, Transform::identity(), None);
                    }
                }
            }
        }
    }
}

/// Convert an OpSet to a tiny-skia Path.
///
/// Returns `None` if the path is empty, degenerate, or contains only MoveTo operations.
fn opset_to_shape<F: Trig + Float + FromPrimitive>(op_set: &OpSet<F>) -> Option<Path> {
    if op_set.ops.is_empty() {
        return None;
    }

    let mut path: PathBuilder = PathBuilder::new();
    let mut has_drawing_op = false;

    for item in op_set.ops.iter() {
        match item.op {
            OpType::Move => {
                if let (Some(x), Some(y)) = (item.data[0].to_f32(), item.data[1].to_f32()) {
                    path.move_to(x, y);
                }
            }
            OpType::BCurveTo => {
                if let (Some(x1), Some(y1), Some(x2), Some(y2), Some(x3), Some(y3)) = (
                    item.data[0].to_f32(),
                    item.data[1].to_f32(),
                    item.data[2].to_f32(),
                    item.data[3].to_f32(),
                    item.data[4].to_f32(),
                    item.data[5].to_f32(),
                ) {
                    path.cubic_to(x1, y1, x2, y2, x3, y3);
                    has_drawing_op = true;
                }
            }
            OpType::LineTo => {
                if let (Some(x), Some(y)) = (item.data[0].to_f32(), item.data[1].to_f32()) {
                    path.line_to(x, y);
                    has_drawing_op = true;
                }
            }
        }
    }

    if has_drawing_op {
        path.finish()
    } else {
        None
    }
}

impl SkiaGenerator {
    pub fn line<F: Trig + Float + FromPrimitive>(
        &self,
        x1: F,
        y1: F,
        x2: F,
        y2: F,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.line(x1, y1, x2, y2, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn rectangle<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.rectangle(x, y, width, height, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn ellipse<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.ellipse(x, y, width, height, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn circle<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        diameter: F,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.circle(x, y, diameter, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn linear_path<F: Trig + Float + FromPrimitive>(
        &self,
        points: &[Point2D<F>],
        close: bool,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.linear_path(points, close, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn polygon<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        points: &[Point2D<F>],
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.polygon(points, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn arc<F: Trig + Float + FromPrimitive>(
        &self,
        x: F,
        y: F,
        width: F,
        height: F,
        start: F,
        stop: F,
        closed: bool,
    ) -> SkiaDrawable<F> {
        let drawable = self
            .gen
            .arc(x, y, width, height, start, stop, closed, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn bezier_quadratic<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        start: Point2D<F>,
        cp: Point2D<F>,
        end: Point2D<F>,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.bezier_quadratic(start, cp, end, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn bezier_cubic<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        start: Point2D<F>,
        cp1: Point2D<F>,
        cp2: Point2D<F>,
        end: Point2D<F>,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.bezier_cubic(start, cp1, cp2, end, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn curve<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        points: &[Point2D<F>],
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.curve(points, &self.options);
        drawable.to_skia_drawable()
    }

    pub fn path<F: Trig + Float + FromPrimitive + MulAssign + Display>(
        &self,
        svg_path: String,
    ) -> SkiaDrawable<F> {
        let drawable = self.gen.path(svg_path, &self.options);
        drawable.to_skia_drawable()
    }
}

fn convert_line_cap_from_roughr_to_piet(roughr_line_cap: Option<roughr::core::LineCap>) -> LineCap {
    match roughr_line_cap {
        Some(roughr::core::LineCap::Butt) => LineCap::Butt,
        Some(roughr::core::LineCap::Round) => LineCap::Round,
        Some(roughr::core::LineCap::Square) => LineCap::Square,
        None => LineCap::Round,
    }
}

fn convert_line_join_from_roughr_to_piet(
    roughr_line_join: Option<roughr::core::LineJoin>,
) -> LineJoin {
    match roughr_line_join {
        Some(roughr::core::LineJoin::Miter { limit: _ }) => LineJoin::Miter,
        Some(roughr::core::LineJoin::Round) => LineJoin::Round,
        Some(roughr::core::LineJoin::Bevel) => LineJoin::Bevel,
        None => LineJoin::Miter,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roughr::core::{Op, OpSet, OpSetType, OpType, OptionsBuilder};

    /// Test that empty OpSet returns None instead of panicking
    #[test]
    fn test_opset_to_shape_empty() {
        let empty_opset: OpSet<f64> = OpSet {
            op_set_type: OpSetType::Path,
            ops: vec![],
            size: None,
            path: None,
        };
        let result = opset_to_shape(&empty_opset);
        assert!(result.is_none());
    }

    /// Test that OpSet with only MoveTo returns None
    #[test]
    fn test_opset_to_shape_only_moveto() {
        let moveto_only: OpSet<f64> = OpSet {
            op_set_type: OpSetType::Path,
            ops: vec![Op { op: OpType::Move, data: vec![10.0, 20.0] }],
            size: None,
            path: None,
        };
        let result = opset_to_shape(&moveto_only);
        assert!(result.is_none());
    }

    /// Test that OpSet with actual drawing ops returns Some(Path)
    #[test]
    fn test_opset_to_shape_with_line() {
        let opset_with_line: OpSet<f64> = OpSet {
            op_set_type: OpSetType::Path,
            ops: vec![
                Op { op: OpType::Move, data: vec![0.0, 0.0] },
                Op { op: OpType::LineTo, data: vec![100.0, 100.0] },
            ],
            size: None,
            path: None,
        };
        let result = opset_to_shape(&opset_with_line);
        assert!(result.is_some());
    }

    /// Test SkiaGenerator can create a line without panicking
    #[test]
    fn test_skia_generator_line() {
        let options = OptionsBuilder::default().build().unwrap();
        let gen = SkiaGenerator::new(options);
        let drawable: SkiaDrawable<f64> = gen.line(0.0, 0.0, 100.0, 100.0);
        assert!(!drawable.sets.is_empty());
    }

    /// Test SkiaGenerator can create a rectangle without panicking
    #[test]
    fn test_skia_generator_rectangle() {
        let options = OptionsBuilder::default().build().unwrap();
        let gen = SkiaGenerator::new(options);
        let drawable: SkiaDrawable<f64> = gen.rectangle(10.0, 10.0, 80.0, 60.0);
        assert!(!drawable.sets.is_empty());
    }

    /// Test SkiaGenerator can create a circle without panicking
    #[test]
    fn test_skia_generator_circle() {
        let options = OptionsBuilder::default().build().unwrap();
        let gen = SkiaGenerator::new(options);
        let drawable: SkiaDrawable<f64> = gen.circle(50.0, 50.0, 40.0);
        assert!(!drawable.sets.is_empty());
    }

    /// Test draw() handles empty paths gracefully
    #[test]
    fn test_drawable_draw_with_empty_path() {
        let options = OptionsBuilder::default().build().unwrap();
        let drawable: SkiaDrawable<f64> = SkiaDrawable {
            shape: "test".to_string(),
            options,
            sets: vec![SkiaOpset {
                op_set_type: OpSetType::Path,
                ops: None, // Empty path
                size: None,
                path: None,
            }],
        };

        let mut pixmap = tiny_skia::Pixmap::new(100, 100).unwrap();
        // Should not panic
        drawable.draw(&mut pixmap.as_mut());
    }

    /// Test rendering a path string that might cause issues
    #[test]
    fn test_skia_generator_path() {
        let options = OptionsBuilder::default().build().unwrap();
        let gen = SkiaGenerator::new(options);
        let drawable: SkiaDrawable<f64> = gen.path("M 10 10 L 90 10 L 90 90 L 10 90 Z".to_string());
        assert!(!drawable.sets.is_empty());
    }

    /// Test that the generator works with hachure fills
    #[test]
    fn test_skia_generator_with_fill() {
        use palette::Srgba;
        use roughr::core::FillStyle;

        let options = OptionsBuilder::default()
            .fill(Srgba::new(0.0, 0.0, 1.0, 1.0))
            .fill_style(FillStyle::Hachure)
            .build()
            .unwrap();
        let gen = SkiaGenerator::new(options);
        let drawable: SkiaDrawable<f64> = gen.rectangle(10.0, 10.0, 80.0, 60.0);
        assert!(!drawable.sets.is_empty());
    }
}
