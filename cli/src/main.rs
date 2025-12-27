use anyhow::{Context, Result};
use clap::Parser;
use std::io::Read;
use std::path::PathBuf;
use vruffr::{OutputFormat, SketchFillStyle};

#[derive(Parser, Debug)]
#[command(name = "vruffr")]
#[command(about = "Convert SVG to hand-drawn sketch-style output (SVG or PNG)")]
#[command(version)]
struct Args {
    /// Input SVG file (use "-" for stdin)
    input: PathBuf,

    /// Output file (SVG or PNG depending on --format)
    #[arg(short, long)]
    output: PathBuf,

    /// Output format: png, svgplain, svg (default: inferred from output extension)
    #[arg(short = 'f', long)]
    format: Option<OutputFormat>,

    /// Roughness of lines (0.0 - 10.0)
    #[arg(short = 'r', long, default_value = "1.0")]
    roughness: f64,

    /// Bowing of lines (0.0 - 10.0)
    #[arg(short = 'b', long, default_value = "1.0")]
    bowing: f64,

    /// Fill style: hachure, crosshatch
    #[arg(long, default_value = "crosshatch")]
    fill_style: SketchFillStyle,

    /// Random seed for reproducibility
    #[arg(long)]
    seed: Option<u64>,

    /// Output width (default: from SVG)
    #[arg(long)]
    width: Option<u32>,

    /// Output height (default: from SVG)
    #[arg(long)]
    height: Option<u32>,

    /// Angle of hachure lines in degrees (default: -41)
    #[arg(long, default_value = "-41")]
    hachure_angle: f32,

    /// Gap between hachure lines in pixels (default: 4.0)
    #[arg(long, default_value = "4.0")]
    hachure_gap: f32,

    /// Override stroke width (default: from SVG)
    #[arg(long)]
    stroke_width: Option<f32>,

    /// Background color: "transparent", "white", "black", or hex "#RRGGBB"/"#RRGGBBAA"
    #[arg(long, default_value = "white")]
    background: String,

    /// Skip fill rendering (strokes only)
    #[arg(long)]
    no_fill: bool,

    /// Skip stroke rendering (fills only)
    #[arg(long)]
    no_stroke: bool,

    /// Weight/thickness of hachure fill lines (default: 0.5)
    #[arg(long, default_value = "0.5")]
    fill_weight: f32,

    /// Suppress output messages
    #[arg(short = 'q', long)]
    quiet: bool,

    /// Scale factor for output dimensions (e.g., 2.0 for 2x size)
    #[arg(short = 's', long, default_value = "1.0")]
    scale: f32,

    /// Validate SVG without rendering (checks if parseable)
    #[arg(long)]
    dry_run: bool,

    /// Font family name for text elements (e.g., "Arial", "Helvetica")
    #[arg(long)]
    font: Option<String>,

    /// Font size in points for text elements (default: from SVG)
    #[arg(long)]
    font_size: Option<f32>,

    /// Adaptive roughness strength (0.0 = disabled, 1.0 = normal, 2.0 = aggressive)
    /// Scales roughness based on element size - smaller elements get less roughness
    #[arg(long, default_value = "0.0")]
    adaptive_strength: f32,

    /// Reference element size in pixels for adaptive roughness scaling (default: 100)
    #[arg(long, default_value = "100.0")]
    reference_size: f32,

    /// Remove duplicate stacked paths before roughening (default: false)
    #[arg(long)]
    deduplicate: bool,

    /// Tolerance in pixels for path deduplication matching (default: 0.1)
    #[arg(long, default_value = "0.1")]
    dedup_epsilon: f32,
}

fn print_warnings(warnings: &vruffr::RenderWarnings) {
    if warnings.has_text {
        eprintln!("Warning: SVG contains text elements which are not rendered");
    }
    if warnings.has_images {
        eprintln!("Warning: SVG contains embedded images which are not rendered");
    }
}

fn infer_format(output: &std::path::Path, explicit: Option<OutputFormat>) -> OutputFormat {
    if let Some(fmt) = explicit {
        return fmt;
    }
    // Infer from extension
    match output.extension().and_then(|e| e.to_str()) {
        Some("png") => OutputFormat::Png,
        Some("svg") => OutputFormat::SvgPlain,
        _ => OutputFormat::SvgPlain, // default
    }
}

fn parse_background(s: &str) -> Result<Option<[u8; 4]>> {
    match s.to_lowercase().as_str() {
        "transparent" | "none" => Ok(None),
        "white" => Ok(Some([255, 255, 255, 255])),
        "black" => Ok(Some([0, 0, 0, 255])),
        hex if hex.starts_with('#') => {
            let hex = &hex[1..];
            match hex.len() {
                6 => {
                    let r = u8::from_str_radix(&hex[0..2], 16)?;
                    let g = u8::from_str_radix(&hex[2..4], 16)?;
                    let b = u8::from_str_radix(&hex[4..6], 16)?;
                    Ok(Some([r, g, b, 255]))
                }
                8 => {
                    let r = u8::from_str_radix(&hex[0..2], 16)?;
                    let g = u8::from_str_radix(&hex[2..4], 16)?;
                    let b = u8::from_str_radix(&hex[4..6], 16)?;
                    let a = u8::from_str_radix(&hex[6..8], 16)?;
                    Ok(Some([r, g, b, a]))
                }
                _ => anyhow::bail!("Invalid hex color: {s}"),
            }
        }
        _ => anyhow::bail!("Unknown color: {s}. Use transparent, white, black, or #RRGGBB"),
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let svg_data = if args.input.as_os_str() == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .context("Failed to read SVG from stdin")?;
        buf
    } else {
        std::fs::read_to_string(&args.input)
            .with_context(|| format!("Failed to read SVG: {}", args.input.display()))?
    };

    // Dry-run mode: validate only, no rendering
    if args.dry_run {
        let info = vruffr::validate_svg(&svg_data).context("Failed to validate SVG")?;
        if !args.quiet {
            if info.warnings.has_text {
                eprintln!("Warning: SVG contains text elements which are not rendered");
            }
            if info.warnings.has_images {
                eprintln!("Warning: SVG contains embedded images which are not rendered");
            }
            eprintln!(
                "Valid SVG: {} ({}x{})",
                args.input.display(),
                info.width,
                info.height
            );
        }
        return Ok(());
    }

    let options = vruffr::SketchOptions {
        roughness: args.roughness,
        bowing: args.bowing,
        seed: args.seed.unwrap_or(42),
        width: args.width,
        height: args.height,
        fill_style: args.fill_style,
        hachure_angle: args.hachure_angle,
        hachure_gap: args.hachure_gap,
        stroke_width: args.stroke_width,
        background: parse_background(&args.background)?,
        no_fill: args.no_fill,
        no_stroke: args.no_stroke,
        fill_weight: args.fill_weight,
        scale: args.scale,
        font: args.font,
        font_size: args.font_size,
        adaptive_strength: args.adaptive_strength,
        reference_size: args.reference_size,
        deduplicate: args.deduplicate,
        dedup_epsilon: args.dedup_epsilon,
    };

    let format = infer_format(&args.output, args.format);
    match format {
        OutputFormat::Png => {
            let (pixmap, warnings) = vruffr::render_sketch_with_warnings(&svg_data, &options)
                .context("Failed to render sketch")?;

            pixmap
                .save_png(&args.output)
                .with_context(|| format!("Failed to save PNG: {}", args.output.display()))?;

            if !args.quiet {
                print_warnings(&warnings);
                eprintln!(
                    "Rendered {} -> {} ({}x{}) [PNG]",
                    args.input.display(),
                    args.output.display(),
                    pixmap.width(),
                    pixmap.height()
                );
            }
        }
        OutputFormat::SvgPlain => {
            let (svg_output, warnings) =
                vruffr::render_to_svg(&svg_data, &options).context("Failed to render sketch")?;

            std::fs::write(&args.output, &svg_output)
                .with_context(|| format!("Failed to save SVG: {}", args.output.display()))?;

            if !args.quiet {
                print_warnings(&warnings);
                eprintln!(
                    "Rendered {} -> {} [SVG plain]",
                    args.input.display(),
                    args.output.display()
                );
            }
        }
        OutputFormat::Svg => {
            // For now, SVG format acts same as SvgPlain
            // TODO: Embed sketch paths back into original SVG structure
            let (svg_output, warnings) =
                vruffr::render_to_svg(&svg_data, &options).context("Failed to render sketch")?;

            std::fs::write(&args.output, &svg_output)
                .with_context(|| format!("Failed to save SVG: {}", args.output.display()))?;

            if !args.quiet {
                print_warnings(&warnings);
                eprintln!(
                    "Rendered {} -> {} [SVG]",
                    args.input.display(),
                    args.output.display()
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_background_named() {
        assert_eq!(
            parse_background("white").unwrap(),
            Some([255, 255, 255, 255])
        );
        assert_eq!(
            parse_background("WHITE").unwrap(),
            Some([255, 255, 255, 255])
        );
        assert_eq!(parse_background("black").unwrap(), Some([0, 0, 0, 255]));
        assert_eq!(parse_background("transparent").unwrap(), None);
        assert_eq!(parse_background("none").unwrap(), None);
    }

    #[test]
    fn test_parse_background_hex() {
        assert_eq!(parse_background("#ff0000").unwrap(), Some([255, 0, 0, 255]));
        assert_eq!(parse_background("#00ff00").unwrap(), Some([0, 255, 0, 255]));
        assert_eq!(parse_background("#0000ff").unwrap(), Some([0, 0, 255, 255]));
        assert_eq!(parse_background("#FF0000").unwrap(), Some([255, 0, 0, 255]));
    }

    #[test]
    fn test_parse_background_hex_with_alpha() {
        assert_eq!(
            parse_background("#ff000080").unwrap(),
            Some([255, 0, 0, 128])
        );
        assert_eq!(
            parse_background("#00ff00ff").unwrap(),
            Some([0, 255, 0, 255])
        );
    }

    #[test]
    fn test_parse_background_invalid() {
        assert!(parse_background("invalid").is_err());
        assert!(parse_background("#fff").is_err()); // 3-char hex not supported
        assert!(parse_background("#fffff").is_err()); // 5-char hex invalid
    }
}
