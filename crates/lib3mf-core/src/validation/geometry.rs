use crate::model::{Geometry, Mesh, Model, ObjectType, ResourceId};
use crate::validation::{ValidationLevel, ValidationReport};
use std::collections::HashMap;

pub fn validate_geometry(model: &Model, level: ValidationLevel, report: &mut ValidationReport) {
    for object in model.resources.iter_objects() {
        // Per spec: "The object type is ignored on objects that contain components"
        // Component-containing objects skip type-specific mesh validation
        if let Geometry::Mesh(mesh) = &object.geometry {
            validate_mesh(
                mesh,
                object.id,
                object.object_type,
                level,
                report,
                model.unit,
            );
        }
    }
}

fn validate_mesh(
    mesh: &Mesh,
    oid: ResourceId,
    object_type: ObjectType,
    level: ValidationLevel,
    report: &mut ValidationReport,
    unit: crate::model::Unit,
) {
    // Basic checks for ALL object types (degenerate triangles)
    for (i, tri) in mesh.triangles.iter().enumerate() {
        if tri.v1 == tri.v2 || tri.v2 == tri.v3 || tri.v1 == tri.v3 {
            report.add_warning(
                4001,
                format!(
                    "Triangle {} in Object {} ({}) is degenerate (duplicate vertices)",
                    i, oid.0, object_type
                ),
            );
        }
    }

    // Type-specific validation at Paranoid level
    if level >= ValidationLevel::Paranoid {
        if object_type.requires_manifold() {
            // Strict checks for Model and SolidSupport
            check_manifoldness(mesh, oid, report);
            check_vertex_manifoldness(mesh, oid, report);
            check_islands(mesh, oid, report);
            check_self_intersections(mesh, oid, report);
            check_orientation(mesh, oid, report);
            check_degenerate_faces(mesh, oid, report, unit);
        } else {
            // Relaxed checks for Support/Surface/Other - only basic geometry warnings
            // These are informational, not errors
            let manifold_issues = count_non_manifold_edges(mesh);
            if manifold_issues > 0 {
                report.add_info(
                    4100,
                    format!(
                        "Object {} ({}) has {} non-manifold edges (allowed for this type)",
                        oid.0, object_type, manifold_issues
                    ),
                );
            }
        }
    }
}

fn check_self_intersections(mesh: &Mesh, oid: ResourceId, report: &mut ValidationReport) {
    if mesh.triangles.len() < 2 {
        return;
    }

    use crate::validation::bvh::{AABB, BvhNode};

    let tri_indices: Vec<usize> = (0..mesh.triangles.len()).collect();
    let bvh = BvhNode::build(mesh, tri_indices);

    let mut intersections = Vec::new();

    for i in 0..mesh.triangles.len() {
        let tri_aabb = AABB::from_triangle(mesh, &mesh.triangles[i]);
        let mut results = Vec::new();
        bvh.find_intersections(mesh, i, &tri_aabb, &mut results);
        for &j in &results {
            intersections.push((i, j));
        }
    }

    if !intersections.is_empty() {
        report.add_warning(
            4008,
            format!(
                "Object {} has {} self-intersecting triangle pairs",
                oid.0,
                intersections.len()
            ),
        );
    }
}

fn check_islands(mesh: &Mesh, oid: ResourceId, report: &mut ValidationReport) {
    if mesh.triangles.is_empty() {
        return;
    }

    // 1. Adjacency list: tri -> neighbors
    let mut edge_to_tris: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (i, tri) in mesh.triangles.iter().enumerate() {
        let edges = [
            sort_edge(tri.v1, tri.v2),
            sort_edge(tri.v2, tri.v3),
            sort_edge(tri.v3, tri.v1),
        ];
        for e in edges {
            edge_to_tris.entry(e).or_default().push(i);
        }
    }

    let mut visited = vec![false; mesh.triangles.len()];
    let mut component_count = 0;

    for start_idx in 0..mesh.triangles.len() {
        if visited[start_idx] {
            continue;
        }

        component_count += 1;
        let mut stack = vec![start_idx];
        visited[start_idx] = true;

        while let Some(curr_idx) = stack.pop() {
            let tri = &mesh.triangles[curr_idx];
            let edges = [
                sort_edge(tri.v1, tri.v2),
                sort_edge(tri.v2, tri.v3),
                sort_edge(tri.v3, tri.v1),
            ];

            for e in edges {
                if let Some(neighbors) = edge_to_tris.get(&e) {
                    for &neigh_idx in neighbors {
                        if !visited[neigh_idx] {
                            visited[neigh_idx] = true;
                            stack.push(neigh_idx);
                        }
                    }
                }
            }
        }
    }

    if component_count > 1 {
        report.add_warning(
            4007,
            format!(
                "Object {} contains {} disconnected components (islands)",
                oid.0, component_count
            ),
        );
    }
}

fn check_vertex_manifoldness(mesh: &Mesh, oid: ResourceId, report: &mut ValidationReport) {
    if mesh.vertices.is_empty() || mesh.triangles.is_empty() {
        return;
    }

    // 1. Group triangles by vertex
    let mut vertex_to_triangles = vec![Vec::new(); mesh.vertices.len()];
    for (i, tri) in mesh.triangles.iter().enumerate() {
        vertex_to_triangles[tri.v1 as usize].push(i);
        vertex_to_triangles[tri.v2 as usize].push(i);
        vertex_to_triangles[tri.v3 as usize].push(i);
    }

    // 2. For each vertex, check connectivity of its triangles
    for (v_idx, tri_indices) in vertex_to_triangles.iter().enumerate() {
        if tri_indices.len() <= 1 {
            continue;
        }

        // We want to see if all triangles sharing this vertex are reachable from each other
        // through edges that ALSO share this vertex.
        let mut visited = vec![false; tri_indices.len()];
        let mut components = 0;

        for start_idx in 0..tri_indices.len() {
            if visited[start_idx] {
                continue;
            }

            components += 1;
            let mut stack = vec![start_idx];
            visited[start_idx] = true;

            while let Some(current_idx) = stack.pop() {
                let current_tri_idx = tri_indices[current_idx];
                let current_tri = &mesh.triangles[current_tri_idx];

                // Find neighbor triangles in the local neighbor list that share an edge with current_tri
                // AND that edge must contain the vertex v_idx.
                for (other_idx, &other_tri_idx) in tri_indices.iter().enumerate() {
                    if visited[other_idx] {
                        continue;
                    }

                    let other_tri = &mesh.triangles[other_tri_idx];

                    // Do they share an edge containing v_idx?
                    // An edge is shared if they share 2 vertices.
                    // Since both share v_idx, they just need to share ONE MORE vertex.
                    let shared_verts = count_shared_vertices(current_tri, other_tri);
                    if shared_verts >= 2 {
                        visited[other_idx] = true;
                        stack.push(other_idx);
                    }
                }
            }
        }

        if components > 1 {
            report.add_warning(
                4006,
                format!(
                    "Object {} has non-manifold vertex {} (points to {} disjoint triangle groups)",
                    oid.0, v_idx, components
                ),
            );
        }
    }
}

fn count_shared_vertices(t1: &crate::model::Triangle, t2: &crate::model::Triangle) -> usize {
    let mut count = 0;
    let v1 = [t1.v1, t1.v2, t1.v3];
    let v2 = [t2.v1, t2.v2, t2.v3];
    for &va in &v1 {
        for &vb in &v2 {
            if va == vb {
                count += 1;
            }
        }
    }
    count
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

fn check_degenerate_faces(
    mesh: &Mesh,
    oid: ResourceId,
    report: &mut ValidationReport,
    unit: crate::model::Unit,
) {
    // Determine epsilon based on unit.
    // Base reference: 1e-6 mm^2 (which is 1e-12 m^2)
    // Formula: threshold = 1e-12 / scale_factor^2
    let scale = unit.scale_factor();
    let epsilon = 1e-12 / (scale * scale);

    for (i, tri) in mesh.triangles.iter().enumerate() {
        if mesh.compute_triangle_area(tri) < epsilon {
            report.add_warning(
                4005,
                format!(
                    "Triangle {} in Object {} has zero/near-zero area (unit scaled)",
                    i, oid.0
                ),
            );
        }
    }
}

fn sort_edge(v1: u32, v2: u32) -> (u32, u32) {
    if v1 < v2 { (v1, v2) } else { (v2, v1) }
}

fn count_non_manifold_edges(mesh: &Mesh) -> usize {
    let mut edge_counts: HashMap<(u32, u32), usize> = HashMap::new();

    for tri in &mesh.triangles {
        let edges = [
            sort_edge(tri.v1, tri.v2),
            sort_edge(tri.v2, tri.v3),
            sort_edge(tri.v3, tri.v1),
        ];
        for e in edges {
            *edge_counts.entry(e).or_insert(0) += 1;
        }
    }

    // Non-manifold edges have count != 2
    edge_counts.values().filter(|&&c| c != 2).count()
}
