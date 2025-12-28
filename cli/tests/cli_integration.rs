//! Integration tests for vruffr CLI binary

use std::process::Command;
use tempfile::TempDir;

fn vruffr_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vruffr"))
}

#[test]
fn test_cli_help() {
    let output = vruffr_bin()
        .arg("--help")
        .output()
        .expect("Failed to run vruffr");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vruffr"));
    assert!(stdout.contains("--roughness"));
    assert!(stdout.contains("--deduplicate"));
}

#[test]
fn test_cli_version() {
    let output = vruffr_bin()
        .arg("--version")
        .output()
        .expect("Failed to run vruffr");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("vruffr"));
}

#[test]
fn test_cli_render_png() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.png");

    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="50" height="50">
        <rect x="5" y="5" width="40" height="40" fill="blue"/>
    </svg>"#;

    let input_path = temp_dir.path().join("input.svg");
    std::fs::write(&input_path, svg).expect("Failed to write input SVG");

    let output = vruffr_bin()
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-q")
        .output()
        .expect("Failed to run vruffr");

    assert!(output.status.success(), "CLI failed: {:?}", output);
    assert!(output_path.exists(), "Output file not created");

    // Verify PNG header
    let png_data = std::fs::read(&output_path).expect("Failed to read output");
    assert!(png_data.len() > 8, "PNG too small");
    assert_eq!(&png_data[0..8], b"\x89PNG\r\n\x1a\n", "Invalid PNG header");
}

#[test]
fn test_cli_render_svg() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.svg");

    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="50" height="50">
        <circle cx="25" cy="25" r="20" fill="red"/>
    </svg>"#;

    let input_path = temp_dir.path().join("input.svg");
    std::fs::write(&input_path, svg).expect("Failed to write input SVG");

    let output = vruffr_bin()
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-q")
        .output()
        .expect("Failed to run vruffr");

    assert!(output.status.success(), "CLI failed: {:?}", output);
    assert!(output_path.exists(), "Output file not created");

    // Verify SVG content
    let svg_out = std::fs::read_to_string(&output_path).expect("Failed to read output");
    assert!(svg_out.contains("<svg"), "Missing SVG element");
    assert!(svg_out.contains("<path"), "Missing path elements");
    assert!(svg_out.contains("</svg>"), "Missing closing SVG tag");
}

#[test]
fn test_cli_dry_run() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.png");

    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100">
        <rect width="100" height="100" fill="green"/>
    </svg>"#;

    let input_path = temp_dir.path().join("input.svg");
    std::fs::write(&input_path, svg).expect("Failed to write input SVG");

    let output = vruffr_bin()
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--dry-run")
        .output()
        .expect("Failed to run vruffr");

    assert!(output.status.success(), "CLI failed: {:?}", output);
    assert!(
        !output_path.exists(),
        "Output should not be created in dry-run mode"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Valid SVG"), "Should report valid SVG");
}

#[test]
fn test_cli_with_options() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.png");

    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="50" height="50">
        <rect x="5" y="5" width="40" height="40" fill="blue" stroke="black"/>
    </svg>"#;

    let input_path = temp_dir.path().join("input.svg");
    std::fs::write(&input_path, svg).expect("Failed to write input SVG");

    let output = vruffr_bin()
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("-q")
        .arg("--roughness")
        .arg("2.0")
        .arg("--bowing")
        .arg("1.5")
        .arg("--seed")
        .arg("123")
        .arg("--deduplicate")
        .arg("--adaptive-strength")
        .arg("1.0")
        .output()
        .expect("Failed to run vruffr");

    assert!(output.status.success(), "CLI failed: {:?}", output);
    assert!(output_path.exists(), "Output file not created");
}

#[test]
fn test_cli_invalid_svg() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.png");

    let input_path = temp_dir.path().join("input.svg");
    std::fs::write(&input_path, "not valid svg").expect("Failed to write input");

    let output = vruffr_bin()
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("Failed to run vruffr");

    assert!(!output.status.success(), "CLI should fail on invalid SVG");
}

#[test]
fn test_cli_stdin() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp_dir.path().join("output.png");

    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" width="50" height="50">
        <rect width="50" height="50" fill="yellow"/>
    </svg>"#;

    let mut child = vruffr_bin()
        .arg("-")
        .arg("-o")
        .arg(&output_path)
        .arg("-q")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn vruffr");

    use std::io::Write;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(svg.as_bytes())
        .expect("Failed to write stdin");

    let output = child.wait_with_output().expect("Failed to wait");
    assert!(output.status.success(), "CLI failed: {:?}", output);
    assert!(output_path.exists(), "Output file not created");
}
