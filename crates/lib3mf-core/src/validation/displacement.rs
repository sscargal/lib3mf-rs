use crate::model::{DisplacementMesh, Displacement2D, Geometry, Model, ResourceId};
use crate::validation::{ValidationLevel, ValidationReport};

/// Validate displacement-specific resources and geometry.
///
/// This function validates:
/// - Displacement2D texture resources (path, height, offset)
/// - DisplacementMesh geometry (normals, gradients, texture coordinates)
///
/// Validation is progressive based on level:
/// - Standard: Reference integrity, count matching
/// - Paranoid: Geometric correctness (unit normals, finite values)
pub fn validate_displacement(model: &Model, level: ValidationLevel, report: &mut ValidationReport) {
    // Validate Displacement2D resources
    validate_displacement_resources(model, level, report);

    // Validate DisplacementMesh geometry
    for object in model.resources.iter_objects() {
        if let Geometry::DisplacementMesh(dmesh) = &object.geometry {
            validate_displacement_mesh(
                dmesh,
                object.id,
                level,
                report,
                &model.resources,
            );
        }
    }
}

/// Validate Displacement2D texture resources.
fn validate_displacement_resources(model: &Model, level: ValidationLevel, report: &mut ValidationReport) {
    for res in model.resources.iter_displacement_2d() {
        // Standard level: Basic path validation
        if level >= ValidationLevel::Standard {
            if res.path.is_empty() {
                report.add_error(
                    5001,
                    format!("Displacement2D resource {} has empty path", res.id.0),
                );
            }

            // Check if path references existing attachment (warning, not error)
            if !res.path.is_empty() && !model.attachments.contains_key(&res.path) {
                report.add_warning(
                    5002,
                    format!(
                        "Displacement2D resource {} references non-existent attachment '{}'",
                        res.id.0, res.path
                    ),
                );
            }
        }

        // Paranoid level: Validate numeric parameters
        if level >= ValidationLevel::Paranoid {
            if !res.height.is_finite() {
                report.add_error(
                    5003,
                    format!(
                        "Displacement2D resource {} has non-finite height: {}",
                        res.id.0, res.height
                    ),
                );
            }

            if !res.offset.is_finite() {
                report.add_error(
                    5004,
                    format!(
                        "Displacement2D resource {} has non-finite offset: {}",
                        res.id.0, res.offset
                    ),
                );
            }

            // PNG validation (if feature enabled)
            #[cfg(feature = "png-validation")]
            {
                if !res.path.is_empty() {
                    if let Some(data) = model.attachments.get(&res.path) {
                        validate_png_texture(&res.path, res, data, report);
                    }
                }
            }
        }
    }
}

/// Validate PNG texture data (only with png-validation feature).
#[cfg(feature = "png-validation")]
fn validate_png_texture(
    path: &str,
    texture: &Displacement2D,
    data: &[u8],
    report: &mut ValidationReport,
) {
    use png::{ColorType, Decoder};
    use std::io::Cursor;

    let decoder = match Decoder::new(Cursor::new(data)) {
        Ok(d) => d,
        Err(e) => {
            report.add_error(
                5030,
                format!(
                    "Displacement texture {} PNG decode failed: {}",
                    texture.id.0, e
                ),
            );
            return;
        }
    };

    let reader = match decoder.read_info() {
        Ok(r) => r,
        Err(e) => {
            report.add_error(
                5030,
                format!(
                    "Displacement texture {} PNG info read failed: {}",
                    texture.id.0, e
                ),
            );
            return;
        }
    };

    let info = reader.info();
    let color_type = info.color_type;

    // Check if requested channel is available
    let channel_available = match texture.channel {
        crate::model::Channel::R => {
            matches!(color_type, ColorType::Rgb | ColorType::Rgba | ColorType::Grayscale | ColorType::GrayscaleAlpha)
        }
        crate::model::Channel::G => {
            matches!(color_type, ColorType::Rgb | ColorType::Rgba | ColorType::Grayscale | ColorType::GrayscaleAlpha)
        }
        crate::model::Channel::B => {
            matches!(color_type, ColorType::Rgb | ColorType::Rgba)
        }
        crate::model::Channel::A => {
            matches!(color_type, ColorType::Rgba | ColorType::GrayscaleAlpha)
        }
    };

    if !channel_available {
        report.add_warning(
            5031,
            format!(
                "Displacement texture {} specifies channel {:?} but PNG has color type {:?}",
                texture.id.0, texture.channel, color_type
            ),
        );
    }
}

/// Validate DisplacementMesh geometry.
fn validate_displacement_mesh(
    mesh: &DisplacementMesh,
    oid: ResourceId,
    level: ValidationLevel,
    report: &mut ValidationReport,
    resources: &crate::model::ResourceCollection,
) {
    // Minimal level: Basic structural validation (always run)
    if mesh.vertices.is_empty() {
        report.add_error(
            5010,
            format!("DisplacementMesh object {} has no vertices", oid.0),
        );
    }

    if mesh.triangles.is_empty() {
        report.add_error(
            5011,
            format!("DisplacementMesh object {} has no triangles", oid.0),
        );
    }

    // Critical: Normal count must match vertex count
    if mesh.normals.len() != mesh.vertices.len() {
        report.add_error(
            5012,
            format!(
                "Object {} has {} vertices but {} normals",
                oid.0,
                mesh.vertices.len(),
                mesh.normals.len()
            ),
        );
    }

    // Validate triangle vertex indices
    let vertex_count = mesh.vertices.len();
    for (i, tri) in mesh.triangles.iter().enumerate() {
        if tri.v1 as usize >= vertex_count
            || tri.v2 as usize >= vertex_count
            || tri.v3 as usize >= vertex_count
        {
            report.add_error(
                5013,
                format!(
                    "Triangle {} in object {} has out-of-bounds vertex index",
                    i, oid.0
                ),
            );
        }
    }

    // Standard level: Reference integrity
    if level >= ValidationLevel::Standard {
        // Validate displacement texture coordinate indices
        for (i, tri) in mesh.triangles.iter().enumerate() {
            if let Some(d1) = tri.d1 {
                validate_displacement_index(oid, i, d1, resources, report);
            }
            if let Some(d2) = tri.d2 {
                validate_displacement_index(oid, i, d2, resources, report);
            }
            if let Some(d3) = tri.d3 {
                validate_displacement_index(oid, i, d3, resources, report);
            }
        }

        // Validate gradient count if present
        if let Some(gradients) = &mesh.gradients {
            if gradients.len() != mesh.vertices.len() {
                report.add_error(
                    5015,
                    format!(
                        "Object {} has {} vertices but {} gradient vectors",
                        oid.0,
                        mesh.vertices.len(),
                        gradients.len()
                    ),
                );
            }
        }
    }

    // Paranoid level: Geometric correctness
    if level >= ValidationLevel::Paranoid {
        // Validate normal vectors
        for (i, normal) in mesh.normals.iter().enumerate() {
            // Check for finite values
            if !normal.nx.is_finite() || !normal.ny.is_finite() || !normal.nz.is_finite() {
                report.add_error(
                    5020,
                    format!(
                        "Normal {} in object {} contains non-finite values",
                        i, oid.0
                    ),
                );
                continue;
            }

            // Check unit length (should be ~1.0)
            let length_sq = normal.nx * normal.nx + normal.ny * normal.ny + normal.nz * normal.nz;
            let length = length_sq.sqrt();
            let tolerance = 1e-4;
            if (length - 1.0).abs() > tolerance {
                report.add_warning(
                    5021,
                    format!(
                        "Normal {} in object {} is not unit length (length: {:.6})",
                        i, oid.0, length
                    ),
                );
            }
        }

        // Validate gradient vectors if present
        if let Some(gradients) = &mesh.gradients {
            for (i, gradient) in gradients.iter().enumerate() {
                if !gradient.gu.is_finite() || !gradient.gv.is_finite() {
                    report.add_error(
                        5022,
                        format!(
                            "Gradient {} in object {} contains non-finite values",
                            i, oid.0
                        ),
                    );
                }
            }

            // Optionally check orthogonality (informational)
            // This is a quality check - gradients should be orthogonal to normals
            // for best displacement mapping results, but not strictly required
            for (i, (normal, gradient)) in mesh.normals.iter().zip(gradients.iter()).enumerate() {
                if normal.nx.is_finite() && normal.ny.is_finite() && normal.nz.is_finite()
                    && gradient.gu.is_finite() && gradient.gv.is_finite()
                {
                    // For a proper check we'd need to convert 2D gradient to 3D
                    // and check dot product with normal. This is complex and
                    // extension-specific, so we just provide an info message
                    // if gradients exist.
                    if i == 0 {
                        report.add_info(
                            5023,
                            format!(
                                "Object {} has gradient vectors (orthogonality not verified)",
                                oid.0
                            ),
                        );
                        break; // Only report once per object
                    }
                }
            }
        }
    }
}

/// Helper: Validate displacement texture coordinate index references.
fn validate_displacement_index(
    oid: ResourceId,
    tri_idx: usize,
    d_index: u32,
    resources: &crate::model::ResourceCollection,
    report: &mut ValidationReport,
) {
    // Displacement indices are stored as ResourceId in triangle
    // They should reference Displacement2D resources
    let rid = ResourceId(d_index);
    if resources.get_displacement_2d(rid).is_none() {
        report.add_error(
            5014,
            format!(
                "Triangle {} in object {} references non-existent displacement texture {}",
                tri_idx, oid.0, d_index
            ),
        );
    }
}
