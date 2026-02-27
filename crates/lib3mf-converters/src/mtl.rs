//! Wavefront MTL material library parser.
//!
//! Parses `.mtl` files referenced by OBJ `mtllib` directives, extracting diffuse color
//! (`Kd`) into [`MtlMaterial`] structs that can be mapped to 3MF [`BaseMaterial`] resources.
//!
//! ## Supported Directives
//!
//! - `newmtl <name>` - Define a new material
//! - `Kd <r> <g> <b>` - Diffuse color (floats 0.0-1.0, clamped)
//!
//! ## Ignored Directives
//!
//! - `Ka`, `Ks`, `Ns`, `Ke`, `illum`, `Ni`, `d`, `Tr` - Silently skipped
//! - `map_Kd` and other `map_*` - Warning printed to stderr, skipped
//!
//! ## Defaults
//!
//! Materials without a `Kd` line default to gray (#808080FF).
//!
//! [`BaseMaterial`]: lib3mf_core::model::BaseMaterial

use lib3mf_core::model::Color;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// Default gray color for materials without a Kd directive.
const DEFAULT_GRAY: Color = Color {
    r: 128,
    g: 128,
    b: 128,
    a: 255,
};

/// A parsed MTL material with a name and display color.
#[derive(Debug, Clone)]
pub struct MtlMaterial {
    /// Material name from the `newmtl` directive.
    pub name: String,
    /// Display color derived from the `Kd` directive, or gray (#808080FF) if absent.
    pub display_color: Color,
}

/// Parses an MTL material library from a reader.
///
/// Reads line-by-line, extracting `newmtl` and `Kd` directives. Materials without
/// a `Kd` line get a default gray color (#808080FF). Texture map references (`map_Kd`
/// and other `map_*` directives) emit a warning to stderr and are skipped.
///
/// Bad or unparseable lines are silently skipped without aborting.
///
/// # Returns
///
/// A `HashMap` mapping material name to [`MtlMaterial`].
pub fn parse_mtl<R: Read>(reader: R) -> HashMap<String, MtlMaterial> {
    let buf = BufReader::new(reader);
    let mut materials: HashMap<String, MtlMaterial> = HashMap::new();
    let mut current_name: Option<String> = None;
    let mut current_color: Option<Color> = None;

    for line_result in buf.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "newmtl" => {
                // Flush previous material
                if let Some(name) = current_name.take() {
                    let color = current_color.take().unwrap_or(DEFAULT_GRAY);
                    materials.insert(
                        name.clone(),
                        MtlMaterial {
                            name,
                            display_color: color,
                        },
                    );
                }
                // Start new material -- name is everything after "newmtl "
                if parts.len() >= 2 {
                    current_name = Some(parts[1..].join(" "));
                }
                current_color = None;
            }
            "Kd" => {
                if parts.len() >= 4
                    && let (Ok(r), Ok(g), Ok(b)) = (
                        parts[1].parse::<f32>(),
                        parts[2].parse::<f32>(),
                        parts[3].parse::<f32>(),
                    )
                {
                    current_color = Some(Color::new(
                        (r.clamp(0.0, 1.0) * 255.0).round() as u8,
                        (g.clamp(0.0, 1.0) * 255.0).round() as u8,
                        (b.clamp(0.0, 1.0) * 255.0).round() as u8,
                        255,
                    ));
                }
                // If parsing fails or not enough parts, skip the line (bad Kd)
            }
            directive if directive.starts_with("map_") => {
                // Warn about texture maps
                let texture_path = if parts.len() >= 2 {
                    parts[1..].join(" ")
                } else {
                    String::new()
                };
                let mat_name = current_name.as_deref().unwrap_or("unknown");
                eprintln!(
                    "Warning: texture map '{}' skipped for material '{}' (texture import not supported)",
                    texture_path, mat_name
                );
            }
            // Silently ignore: Ka, Ks, Ns, Ke, illum, Ni, d, Tr, and anything else
            _ => {}
        }
    }

    // Flush last material
    if let Some(name) = current_name.take() {
        let color = current_color.take().unwrap_or(DEFAULT_GRAY);
        materials.insert(
            name.clone(),
            MtlMaterial {
                name,
                display_color: color,
            },
        );
    }

    materials
}

/// Parses an MTL file from a filesystem path.
///
/// If the file does not exist or cannot be opened, a warning is printed to stderr
/// and an empty `HashMap` is returned. This allows OBJ import to proceed with
/// geometry only when the MTL file is missing.
pub fn parse_mtl_file(path: &Path) -> HashMap<String, MtlMaterial> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("Warning: MTL file not found: {}", path.display());
            } else {
                eprintln!(
                    "Warning: failed to open MTL file '{}': {}",
                    path.display(),
                    e
                );
            }
            return HashMap::new();
        }
    };
    parse_mtl(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_kd_parsing() {
        let mtl = b"newmtl Red\nKd 1.0 0.0 0.0\n";
        let materials = parse_mtl(&mtl[..]);
        assert_eq!(materials.len(), 1);
        let red = &materials["Red"];
        assert_eq!(red.name, "Red");
        assert_eq!(red.display_color, Color::new(255, 0, 0, 255));
    }

    #[test]
    fn test_missing_kd_defaults_to_gray() {
        let mtl = b"newmtl NoColor\n";
        let materials = parse_mtl(&mtl[..]);
        assert_eq!(materials.len(), 1);
        let mat = &materials["NoColor"];
        assert_eq!(mat.display_color, Color::new(128, 128, 128, 255));
    }

    #[test]
    fn test_kd_clamping() {
        let mtl = b"newmtl Bright\nKd 1.5 -0.5 0.5\n";
        let materials = parse_mtl(&mtl[..]);
        let mat = &materials["Bright"];
        assert_eq!(mat.display_color.r, 255); // 1.5 clamped to 1.0
        assert_eq!(mat.display_color.g, 0); // -0.5 clamped to 0.0
        assert_eq!(mat.display_color.b, 128); // 0.5 * 255 = 127.5 -> 128
    }

    #[test]
    fn test_map_kd_warning_falls_back() {
        // Material with map_Kd but also Kd should keep the Kd color
        let mtl = b"newmtl Textured\nKd 0.0 1.0 0.0\nmap_Kd texture.png\n";
        let materials = parse_mtl(&mtl[..]);
        let mat = &materials["Textured"];
        // Kd should be preserved (green)
        assert_eq!(mat.display_color, Color::new(0, 255, 0, 255));
    }

    #[test]
    fn test_map_kd_without_kd_gets_gray() {
        // Material with map_Kd but no Kd should default to gray
        let mtl = b"newmtl TexturedOnly\nmap_Kd texture.png\n";
        let materials = parse_mtl(&mtl[..]);
        let mat = &materials["TexturedOnly"];
        assert_eq!(mat.display_color, Color::new(128, 128, 128, 255));
    }

    #[test]
    fn test_multiple_materials() {
        let mtl = b"newmtl Red\nKd 1.0 0.0 0.0\nnewmtl Blue\nKd 0.0 0.0 1.0\nnewmtl Green\nKd 0.0 1.0 0.0\n";
        let materials = parse_mtl(&mtl[..]);
        assert_eq!(materials.len(), 3);
        assert_eq!(materials["Red"].display_color, Color::new(255, 0, 0, 255));
        assert_eq!(materials["Blue"].display_color, Color::new(0, 0, 255, 255));
        assert_eq!(materials["Green"].display_color, Color::new(0, 255, 0, 255));
    }

    #[test]
    fn test_bad_lines_are_skipped() {
        let mtl = b"newmtl Good\nKd 1.0 0.0 0.0\ngarbage line here\nKd bad bad bad\n";
        let materials = parse_mtl(&mtl[..]);
        assert_eq!(materials.len(), 1);
        // The first valid Kd should have been applied before the bad Kd
        // The bad Kd line is skipped (parse failure), so the first Kd sticks
        assert_eq!(materials["Good"].display_color, Color::new(255, 0, 0, 255));
    }

    #[test]
    fn test_empty_input() {
        let mtl = b"";
        let materials = parse_mtl(&mtl[..]);
        assert!(materials.is_empty());
    }

    #[test]
    fn test_comments_and_blank_lines() {
        let mtl = b"# comment\n\nnewmtl Mat\n# another comment\nKd 0.5 0.5 0.5\n\n";
        let materials = parse_mtl(&mtl[..]);
        assert_eq!(materials.len(), 1);
        assert_eq!(
            materials["Mat"].display_color,
            Color::new(128, 128, 128, 255)
        );
    }

    #[test]
    fn test_ignored_directives() {
        let mtl =
            b"newmtl Fancy\nKa 0.1 0.1 0.1\nKd 1.0 0.0 0.0\nKs 1.0 1.0 1.0\nNs 100.0\nNi 1.5\nillum 2\nd 0.5\nTr 0.5\nKe 0.0 0.0 0.0\n";
        let materials = parse_mtl(&mtl[..]);
        assert_eq!(materials.len(), 1);
        // Only Kd matters
        assert_eq!(materials["Fancy"].display_color, Color::new(255, 0, 0, 255));
    }

    #[test]
    fn test_material_name_with_spaces() {
        let mtl = b"newmtl My Material Name\nKd 0.0 0.0 1.0\n";
        let materials = parse_mtl(&mtl[..]);
        assert_eq!(materials.len(), 1);
        assert!(materials.contains_key("My Material Name"));
    }

    #[test]
    fn test_parse_mtl_file_nonexistent() {
        let materials = parse_mtl_file(Path::new("/tmp/nonexistent_test_mtl_file_12345.mtl"));
        assert!(materials.is_empty());
    }
}
