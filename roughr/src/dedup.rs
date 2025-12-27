//! Duplicate path filtering for SVG preprocessing.
//!
//! This module provides functionality to detect and remove duplicate paths
//! that are stacked at the same position. When SVGs contain identical paths
//! overlapping each other, roughening each separately creates visual chaos.
//! By deduplicating before roughening, we ensure consistent sketch output.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Signature representing a path's geometric identity.
/// Two paths with the same signature are considered duplicates.
#[derive(Debug, Clone)]
pub struct PathSignature {
    /// Bounding box: (min_x, min_y, max_x, max_y)
    pub bbox: (f32, f32, f32, f32),
    /// Total path length (arc length)
    pub path_length: f32,
    /// Number of path commands
    pub vertex_count: usize,
    /// Hash of command types (M, L, C, Z sequence)
    pub command_hash: u64,
    /// Geometric center of the path
    pub centroid: (f32, f32),
}

impl PathSignature {
    /// Create a new path signature from geometric properties.
    pub fn new(
        bbox: (f32, f32, f32, f32),
        path_length: f32,
        vertex_count: usize,
        command_hash: u64,
        centroid: (f32, f32),
    ) -> Self {
        Self {
            bbox,
            path_length,
            vertex_count,
            command_hash,
            centroid,
        }
    }

    /// Create a signature from a path string.
    /// This is a simplified implementation that parses basic SVG path data.
    pub fn from_path_data(path_data: &str) -> Self {
        let (bbox, path_length, vertex_count, command_hash, centroid) =
            parse_path_geometry(path_data);
        Self::new(bbox, path_length, vertex_count, command_hash, centroid)
    }

    /// Check if this signature matches another within tolerance.
    pub fn matches(&self, other: &PathSignature, epsilon: f32) -> bool {
        // Check bounding box
        if (self.bbox.0 - other.bbox.0).abs() > epsilon
            || (self.bbox.1 - other.bbox.1).abs() > epsilon
            || (self.bbox.2 - other.bbox.2).abs() > epsilon
            || (self.bbox.3 - other.bbox.3).abs() > epsilon
        {
            return false;
        }

        // Check vertex count (must be exact)
        if self.vertex_count != other.vertex_count {
            return false;
        }

        // Check command hash (must be exact)
        if self.command_hash != other.command_hash {
            return false;
        }

        // Check path length (relative tolerance)
        let max_length = self.path_length.max(other.path_length);
        if max_length > 0.0 {
            let length_diff = (self.path_length - other.path_length).abs();
            if length_diff > epsilon * max_length * 0.01 {
                return false;
            }
        }

        // Check centroid
        if (self.centroid.0 - other.centroid.0).abs() > epsilon
            || (self.centroid.1 - other.centroid.1).abs() > epsilon
        {
            return false;
        }

        true
    }

    /// Generate a bucket key for hash-based grouping.
    /// Paths with the same bucket key are candidates for detailed comparison.
    pub fn bucket_key(&self, epsilon: f32) -> (i32, i32, i32, i32, usize, u64) {
        // Quantize bounding box to grid cells of size epsilon
        let grid = if epsilon > 0.0 { epsilon } else { 1.0 };
        (
            (self.bbox.0 / grid).floor() as i32,
            (self.bbox.1 / grid).floor() as i32,
            (self.bbox.2 / grid).floor() as i32,
            (self.bbox.3 / grid).floor() as i32,
            self.vertex_count,
            self.command_hash,
        )
    }
}

/// A path with its associated style information.
#[derive(Debug, Clone)]
pub struct StyledPath {
    /// The path data string (SVG d attribute)
    pub path_data: String,
    /// Computed signature for deduplication
    pub signature: PathSignature,
    /// Stroke color (RGBA)
    pub stroke: Option<[u8; 4]>,
    /// Stroke width
    pub stroke_width: Option<f32>,
    /// Fill color (RGBA)
    pub fill: Option<[u8; 4]>,
    /// Original index in the input list
    pub original_index: usize,
}

/// A group of duplicate paths that share the same geometry.
#[derive(Debug)]
pub struct DuplicateGroup {
    /// The canonical path (first occurrence)
    pub canonical: StyledPath,
    /// All unique stroke/fill combinations found in duplicates
    pub styles: Vec<(Option<[u8; 4]>, Option<f32>, Option<[u8; 4]>)>,
}

/// Deduplicate a list of styled paths.
///
/// Returns groups of paths where each group contains:
/// - A single canonical path (the geometry to roughen)
/// - All unique style combinations found among duplicates
///
/// # Arguments
/// * `paths` - List of styled paths to deduplicate
/// * `epsilon` - Tolerance for position matching (in pixels)
pub fn deduplicate_paths(paths: Vec<StyledPath>, epsilon: f32) -> Vec<DuplicateGroup> {
    if paths.is_empty() {
        return vec![];
    }

    // Group paths by bucket key for efficient comparison
    let mut buckets: HashMap<_, Vec<StyledPath>> = HashMap::new();
    for path in paths {
        let key = path.signature.bucket_key(epsilon);
        buckets.entry(key).or_default().push(path);
    }

    let mut result = Vec::new();

    for (_, bucket) in buckets {
        // Within each bucket, find exact duplicates
        let groups = find_duplicates_in_bucket(bucket, epsilon);
        result.extend(groups);
    }

    // Sort by original index to maintain order
    result.sort_by_key(|g| g.canonical.original_index);
    result
}

/// Find duplicates within a bucket of similar paths.
fn find_duplicates_in_bucket(mut paths: Vec<StyledPath>, epsilon: f32) -> Vec<DuplicateGroup> {
    let mut groups: Vec<DuplicateGroup> = Vec::new();

    while let Some(path) = paths.pop() {
        let mut duplicates = vec![path.clone()];

        // Find all paths that match this one
        let mut i = 0;
        while i < paths.len() {
            if path.signature.matches(&paths[i].signature, epsilon) {
                duplicates.push(paths.remove(i));
            } else {
                i += 1;
            }
        }

        // Collect unique styles
        let mut styles: Vec<(Option<[u8; 4]>, Option<f32>, Option<[u8; 4]>)> = Vec::new();
        for dup in &duplicates {
            let style = (dup.stroke, dup.stroke_width, dup.fill);
            if !styles.contains(&style) {
                styles.push(style);
            }
        }

        // Use the first (by original index) as canonical
        duplicates.sort_by_key(|p| p.original_index);
        let canonical = duplicates.remove(0);

        groups.push(DuplicateGroup { canonical, styles });
    }

    groups
}

/// Parse path geometry to extract signature components.
/// This is a simplified parser that handles common SVG path commands.
fn parse_path_geometry(path_data: &str) -> ((f32, f32, f32, f32), f32, usize, u64, (f32, f32)) {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    let mut path_length = 0.0f32;
    let mut vertex_count = 0usize;
    let mut command_sequence = String::new();
    let mut sum_x = 0.0f32;
    let mut sum_y = 0.0f32;
    let mut point_count = 0usize;

    let mut current_x = 0.0f32;
    let mut current_y = 0.0f32;
    let mut start_x = 0.0f32;
    let mut start_y = 0.0f32;

    // Simple tokenizer
    let mut chars = path_data.chars().peekable();
    let mut current_cmd = ' ';

    while chars.peek().is_some() {
        // Skip whitespace
        while chars.peek().map_or(false, |c| c.is_whitespace() || *c == ',') {
            chars.next();
        }

        if let Some(&c) = chars.peek() {
            if c.is_alphabetic() {
                current_cmd = chars.next().unwrap();
                command_sequence.push(current_cmd.to_ascii_uppercase());
                vertex_count += 1;
            }
        }

        // Parse numbers for the current command
        let nums = parse_numbers(&mut chars);

        match current_cmd {
            'M' => {
                if nums.len() >= 2 {
                    current_x = nums[0];
                    current_y = nums[1];
                    start_x = current_x;
                    start_y = current_y;
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, current_x, current_y);
                    sum_x += current_x;
                    sum_y += current_y;
                    point_count += 1;
                }
            }
            'm' => {
                if nums.len() >= 2 {
                    current_x += nums[0];
                    current_y += nums[1];
                    start_x = current_x;
                    start_y = current_y;
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, current_x, current_y);
                    sum_x += current_x;
                    sum_y += current_y;
                    point_count += 1;
                }
            }
            'L' => {
                if nums.len() >= 2 {
                    let dx = nums[0] - current_x;
                    let dy = nums[1] - current_y;
                    path_length += (dx * dx + dy * dy).sqrt();
                    current_x = nums[0];
                    current_y = nums[1];
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, current_x, current_y);
                    sum_x += current_x;
                    sum_y += current_y;
                    point_count += 1;
                }
            }
            'l' => {
                if nums.len() >= 2 {
                    path_length += (nums[0] * nums[0] + nums[1] * nums[1]).sqrt();
                    current_x += nums[0];
                    current_y += nums[1];
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, current_x, current_y);
                    sum_x += current_x;
                    sum_y += current_y;
                    point_count += 1;
                }
            }
            'H' => {
                if !nums.is_empty() {
                    path_length += (nums[0] - current_x).abs();
                    current_x = nums[0];
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, current_x, current_y);
                    sum_x += current_x;
                    sum_y += current_y;
                    point_count += 1;
                }
            }
            'h' => {
                if !nums.is_empty() {
                    path_length += nums[0].abs();
                    current_x += nums[0];
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, current_x, current_y);
                    sum_x += current_x;
                    sum_y += current_y;
                    point_count += 1;
                }
            }
            'V' => {
                if !nums.is_empty() {
                    path_length += (nums[0] - current_y).abs();
                    current_y = nums[0];
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, current_x, current_y);
                    sum_x += current_x;
                    sum_y += current_y;
                    point_count += 1;
                }
            }
            'v' => {
                if !nums.is_empty() {
                    path_length += nums[0].abs();
                    current_y += nums[0];
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, current_x, current_y);
                    sum_x += current_x;
                    sum_y += current_y;
                    point_count += 1;
                }
            }
            'C' => {
                if nums.len() >= 6 {
                    // Approximate cubic bezier length
                    let ctrl_length = distance(current_x, current_y, nums[0], nums[1])
                        + distance(nums[0], nums[1], nums[2], nums[3])
                        + distance(nums[2], nums[3], nums[4], nums[5]);
                    let chord = distance(current_x, current_y, nums[4], nums[5]);
                    path_length += (ctrl_length + chord) / 2.0;
                    current_x = nums[4];
                    current_y = nums[5];
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, current_x, current_y);
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, nums[0], nums[1]);
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, nums[2], nums[3]);
                    sum_x += current_x;
                    sum_y += current_y;
                    point_count += 1;
                }
            }
            'c' => {
                if nums.len() >= 6 {
                    let x1 = current_x + nums[0];
                    let y1 = current_y + nums[1];
                    let x2 = current_x + nums[2];
                    let y2 = current_y + nums[3];
                    let x = current_x + nums[4];
                    let y = current_y + nums[5];
                    let ctrl_length = distance(current_x, current_y, x1, y1)
                        + distance(x1, y1, x2, y2)
                        + distance(x2, y2, x, y);
                    let chord = distance(current_x, current_y, x, y);
                    path_length += (ctrl_length + chord) / 2.0;
                    current_x = x;
                    current_y = y;
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, current_x, current_y);
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, x1, y1);
                    update_bounds(&mut min_x, &mut min_y, &mut max_x, &mut max_y, x2, y2);
                    sum_x += current_x;
                    sum_y += current_y;
                    point_count += 1;
                }
            }
            'Z' | 'z' => {
                let dx = start_x - current_x;
                let dy = start_y - current_y;
                path_length += (dx * dx + dy * dy).sqrt();
                current_x = start_x;
                current_y = start_y;
            }
            _ => {}
        }
    }

    // Compute command hash
    let mut hasher = DefaultHasher::new();
    command_sequence.hash(&mut hasher);
    let command_hash = hasher.finish();

    // Compute centroid
    let centroid = if point_count > 0 {
        (sum_x / point_count as f32, sum_y / point_count as f32)
    } else {
        (0.0, 0.0)
    };

    // Handle empty paths
    if min_x == f32::MAX {
        min_x = 0.0;
        min_y = 0.0;
        max_x = 0.0;
        max_y = 0.0;
    }

    ((min_x, min_y, max_x, max_y), path_length, vertex_count, command_hash, centroid)
}

/// Parse a sequence of numbers from the character iterator.
fn parse_numbers(chars: &mut std::iter::Peekable<std::str::Chars>) -> Vec<f32> {
    let mut numbers = Vec::new();

    loop {
        // Skip whitespace and commas
        while chars.peek().map_or(false, |c| c.is_whitespace() || *c == ',') {
            chars.next();
        }

        // Check if next char is a command letter or end
        if chars.peek().map_or(true, |c| c.is_alphabetic()) {
            break;
        }

        // Parse number
        let mut num_str = String::new();

        // Handle sign
        if chars.peek().map_or(false, |c| *c == '-' || *c == '+') {
            num_str.push(chars.next().unwrap());
        }

        // Parse digits and decimal point
        while chars.peek().map_or(false, |c| c.is_ascii_digit() || *c == '.') {
            num_str.push(chars.next().unwrap());
        }

        // Handle exponent
        if chars.peek().map_or(false, |c| *c == 'e' || *c == 'E') {
            num_str.push(chars.next().unwrap());
            if chars.peek().map_or(false, |c| *c == '-' || *c == '+') {
                num_str.push(chars.next().unwrap());
            }
            while chars.peek().map_or(false, |c| c.is_ascii_digit()) {
                num_str.push(chars.next().unwrap());
            }
        }

        if let Ok(n) = num_str.parse::<f32>() {
            numbers.push(n);
        } else if !num_str.is_empty() {
            break;
        }
    }

    numbers
}

fn update_bounds(min_x: &mut f32, min_y: &mut f32, max_x: &mut f32, max_y: &mut f32, x: f32, y: f32) {
    *min_x = min_x.min(x);
    *min_y = min_y.min(y);
    *max_x = max_x.max(x);
    *max_y = max_y.max(y);
}

fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_signature_from_simple_path() {
        let sig = PathSignature::from_path_data("M 0 0 L 100 0 L 100 100 L 0 100 Z");
        assert!((sig.bbox.0 - 0.0).abs() < 0.001);
        assert!((sig.bbox.1 - 0.0).abs() < 0.001);
        assert!((sig.bbox.2 - 100.0).abs() < 0.001);
        assert!((sig.bbox.3 - 100.0).abs() < 0.001);
        assert!(sig.path_length > 390.0 && sig.path_length < 410.0); // ~400
        assert_eq!(sig.vertex_count, 5); // M, L, L, L, Z
    }

    #[test]
    fn test_path_signature_matches_identical() {
        let sig1 = PathSignature::from_path_data("M 10 10 L 50 50");
        let sig2 = PathSignature::from_path_data("M 10 10 L 50 50");
        assert!(sig1.matches(&sig2, 0.1));
    }

    #[test]
    fn test_path_signature_matches_within_tolerance() {
        let sig1 = PathSignature::from_path_data("M 10 10 L 50 50");
        let sig2 = PathSignature::from_path_data("M 10.05 10.05 L 50.05 50.05");
        // Use larger tolerance to account for centroid differences
        assert!(sig1.matches(&sig2, 0.1));
    }

    #[test]
    fn test_path_signature_matches_exact_duplicate() {
        // Test exact same path
        let sig1 = PathSignature::from_path_data("M 0 0 L 100 0 L 100 100 Z");
        let sig2 = PathSignature::from_path_data("M 0 0 L 100 0 L 100 100 Z");
        assert!(sig1.matches(&sig2, 0.001));
    }

    #[test]
    fn test_path_signature_no_match_different_position() {
        let sig1 = PathSignature::from_path_data("M 10 10 L 50 50");
        let sig2 = PathSignature::from_path_data("M 20 20 L 60 60");
        assert!(!sig1.matches(&sig2, 0.1));
    }

    #[test]
    fn test_path_signature_no_match_different_commands() {
        let sig1 = PathSignature::from_path_data("M 10 10 L 50 50");
        let sig2 = PathSignature::from_path_data("M 10 10 L 50 50 L 100 100");
        assert!(!sig1.matches(&sig2, 0.1));
    }

    #[test]
    fn test_deduplicate_empty() {
        let result = deduplicate_paths(vec![], 0.1);
        assert!(result.is_empty());
    }

    #[test]
    fn test_deduplicate_no_duplicates() {
        let paths = vec![
            StyledPath {
                path_data: "M 0 0 L 100 100".to_string(),
                signature: PathSignature::from_path_data("M 0 0 L 100 100"),
                stroke: Some([0, 0, 0, 255]),
                stroke_width: Some(1.0),
                fill: None,
                original_index: 0,
            },
            StyledPath {
                path_data: "M 200 200 L 300 300".to_string(),
                signature: PathSignature::from_path_data("M 200 200 L 300 300"),
                stroke: Some([255, 0, 0, 255]),
                stroke_width: Some(2.0),
                fill: None,
                original_index: 1,
            },
        ];
        let result = deduplicate_paths(paths, 0.1);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_deduplicate_with_duplicates() {
        let paths = vec![
            StyledPath {
                path_data: "M 0 0 L 100 100".to_string(),
                signature: PathSignature::from_path_data("M 0 0 L 100 100"),
                stroke: Some([0, 0, 0, 255]),
                stroke_width: Some(1.0),
                fill: None,
                original_index: 0,
            },
            StyledPath {
                path_data: "M 0 0 L 100 100".to_string(),
                signature: PathSignature::from_path_data("M 0 0 L 100 100"),
                stroke: Some([255, 0, 0, 255]),
                stroke_width: Some(2.0),
                fill: None,
                original_index: 1,
            },
            StyledPath {
                path_data: "M 0 0 L 100 100".to_string(),
                signature: PathSignature::from_path_data("M 0 0 L 100 100"),
                stroke: Some([0, 0, 0, 255]),
                stroke_width: Some(1.0),
                fill: Some([0, 255, 0, 255]),
                original_index: 2,
            },
        ];
        let result = deduplicate_paths(paths, 0.1);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].canonical.original_index, 0);
        assert_eq!(result[0].styles.len(), 3); // 3 unique style combinations
    }

    #[test]
    fn test_deduplicate_preserves_order() {
        let paths = vec![
            StyledPath {
                path_data: "M 100 100 L 200 200".to_string(),
                signature: PathSignature::from_path_data("M 100 100 L 200 200"),
                stroke: None,
                stroke_width: None,
                fill: None,
                original_index: 0,
            },
            StyledPath {
                path_data: "M 0 0 L 50 50".to_string(),
                signature: PathSignature::from_path_data("M 0 0 L 50 50"),
                stroke: None,
                stroke_width: None,
                fill: None,
                original_index: 1,
            },
        ];
        let result = deduplicate_paths(paths, 0.1);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].canonical.original_index, 0);
        assert_eq!(result[1].canonical.original_index, 1);
    }

    #[test]
    fn test_parse_relative_path() {
        let sig = PathSignature::from_path_data("m 10 10 l 40 40");
        assert!((sig.bbox.0 - 10.0).abs() < 0.001);
        assert!((sig.bbox.1 - 10.0).abs() < 0.001);
        assert!((sig.bbox.2 - 50.0).abs() < 0.001);
        assert!((sig.bbox.3 - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_cubic_bezier() {
        let sig = PathSignature::from_path_data("M 0 0 C 10 20 30 40 50 50");
        assert!(sig.vertex_count == 2); // M, C
        assert!(sig.path_length > 0.0);
    }
}
