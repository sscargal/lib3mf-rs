use crate::model::{Geometry, Mesh, Model, ResourceId};
use crate::validation::{ValidationLevel, ValidationReport};
use std::collections::HashMap;

pub fn validate_geometry(
    model: &Model,
    level: ValidationLevel,
    report: &mut ValidationReport,
) {
    for object in model.resources.iter_objects() {
        if let Geometry::Mesh(mesh) = &object.geometry {
            validate_mesh(mesh, object.id, level, report);
        }
    }
}

fn validate_mesh(
    mesh: &Mesh,
    oid: ResourceId,
    level: ValidationLevel,
    report: &mut ValidationReport,
) {
    // Basic checks
    for (i, tri) in mesh.triangles.iter().enumerate() {
        if tri.v1 == tri.v2 || tri.v2 == tri.v3 || tri.v1 == tri.v3 {
            report.add_warning(
                4001,
                format!(
                    "Triangle {} in Object {} is degenerate (duplicate vertices)",
                    i, oid.0
                ),
            );
        }
    }

    if level >= ValidationLevel::Paranoid {
        check_manifoldness(mesh, oid, report);
        check_orientation(mesh, oid, report);
        check_degenerate_faces(mesh, oid, report);
    }
}

fn check_manifoldness(mesh: &Mesh, oid: ResourceId, report: &mut ValidationReport) {
    let mut edge_counts = HashMap::new();

    for tri in &mesh.triangles {
        let edges = [
            sort_edge(tri.v1, tri.v2),
            sort_edge(tri.v2, tri.v3),
            sort_edge(tri.v3, tri.v1),
        ];

        for edge in edges {
            *edge_counts.entry(edge).or_insert(0) += 1;
        }
    }

    for (edge, count) in edge_counts {
        if count == 1 {
            report.add_warning(
                4002,
                format!(
                    "Object {} has boundary edge {:?} (not watertight)",
                    oid.0, edge
                ),
            );
        } else if count > 2 {
            report.add_warning(
                4003,
                format!(
                    "Object {} has non-manifold edge {:?} (shared by {} triangles)",
                    oid.0, edge, count
                ),
            );
        }
    }
}

fn check_orientation(mesh: &Mesh, oid: ResourceId, report: &mut ValidationReport) {
    // Count occurrences of directed edges.
    // If any directed edge count > 1, then two faces have edges in same direction -> Orientation Mismatch.
    
    let mut directed_edge_counts = HashMap::new();
    for tri in &mesh.triangles {
         let edges = [(tri.v1, tri.v2), (tri.v2, tri.v3), (tri.v3, tri.v1)];
         for edge in edges {
             *directed_edge_counts.entry(edge).or_insert(0) += 1;
         }
    }
    
    for (edge, count) in directed_edge_counts {
        if count > 1 {
             report.add_warning(
                4004,
                format!(
                    "Object {} has orientation mismatch or duplicate faces at edge {:?}",
                    oid.0, edge
                ),
            );
        }
    }
}

fn check_degenerate_faces(mesh: &Mesh, oid: ResourceId, report: &mut ValidationReport) {
    for (i, tri) in mesh.triangles.iter().enumerate() {
        if mesh.compute_triangle_area(tri) < 1e-6 {
             report.add_warning(
                4005,
                format!(
                    "Triangle {} in Object {} has zero/near-zero area",
                    i, oid.0
                ),
            );
        }
    }
}

fn sort_edge(v1: u32, v2: u32) -> (u32, u32) {
    if v1 < v2 {
        (v1, v2)
    } else {
        (v2, v1)
    }
}
