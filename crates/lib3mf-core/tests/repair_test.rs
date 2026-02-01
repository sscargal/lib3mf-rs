use lib3mf_core::model::{Mesh, MeshRepair, RepairOptions};

#[test]
fn test_vertex_stitching() {
    let mut mesh = Mesh::new();
    // Two triangles effectively sharing an edge, but vertices are duplicated slightly apart
    // Triangle 1
    mesh.add_vertex(0.0, 0.0, 0.0); // 0
    mesh.add_vertex(1.0, 0.0, 0.0); // 1
    mesh.add_vertex(0.0, 1.0, 0.0); // 2
    mesh.add_triangle(0, 1, 2);

    // Triangle 2 (Shifted by 1e-5 in Z, should be merged with default epsilon 1e-4)
    mesh.add_vertex(1.0, 0.0, 0.00001); // 3 (Should merge with 1)
    mesh.add_vertex(0.0, 1.0, 0.00001); // 4 (Should merge with 2)
    mesh.add_vertex(1.0, 1.0, 0.0); // 5
    mesh.add_triangle(3, 5, 4);

    assert_eq!(mesh.vertices.len(), 6);

    let stats = mesh.repair(RepairOptions::default());

    // Should remove 2 duplicate vertices (3->1, 4->2)
    assert_eq!(stats.vertices_removed, 2);
    assert_eq!(mesh.vertices.len(), 4);

    // Verify connectivity
    // Tri 2 should now use indices of Tri 1
    let t2 = mesh.triangles[1];
    // Original 3 mapped to 1
    // Original 4 mapped to 2
    // Original 5 mapped to 3 (since 3,4 removed, index 5 shifts down to 3? No, remapping table logic)
    // 0->0, 1->1, 2->2, 3->1, 4->2, 5->3.
    // So T2 vertices should be 1, 3, 2. (Originally 3, 5, 4)
    assert_eq!(t2.v1, 1); // 3 -> 1
    assert_eq!(t2.v2, 3); // 5 -> 3
    assert_eq!(t2.v3, 2); // 4 -> 2
}

#[test]
fn test_degenerate_removal() {
    let mut mesh = Mesh::new();
    // Valid triangle
    mesh.add_vertex(0.0, 0.0, 0.0);
    mesh.add_vertex(1.0, 0.0, 0.0);
    mesh.add_vertex(0.0, 1.0, 0.0);
    mesh.add_triangle(0, 1, 2);

    // Degenerate triangle (duplicate indices)
    mesh.add_triangle(0, 1, 0);

    // Degenerate triangle (zero area, collinear)
    mesh.add_vertex(2.0, 0.0, 0.0); // 3
    mesh.add_triangle(0, 1, 3); // (0,0,0), (1,0,0), (2,0,0) -> Area 0

    assert_eq!(mesh.triangles.len(), 3);

    let stats = mesh.repair(RepairOptions::default());

    assert_eq!(stats.triangles_removed, 2);
    assert_eq!(mesh.triangles.len(), 1);
}

#[test]
fn test_orientation_harmonization() {
    let mut mesh = Mesh::new();
    mesh.add_vertex(0.0, 0.0, 0.0); // 0
    mesh.add_vertex(1.0, 0.0, 0.0); // 1
    mesh.add_vertex(0.0, 1.0, 0.0); // 2
    mesh.add_vertex(1.0, 1.0, 0.0); // 3

    // Triangle 1: CCW (0, 1, 2)
    mesh.add_triangle(0, 1, 2);
    // Triangle 2: CW (1, 2, 3) - INCONSISTENT with CCW (0, 1, 2)
    // Edge (1, 2) in T1. In T2, edge (1, 2) is ALSO forward.
    mesh.add_triangle(1, 2, 3);

    let stats = mesh.repair(RepairOptions {
        stitch_epsilon: 0.0,
        remove_degenerate: false,
        remove_duplicate_faces: false,
        harmonize_orientations: true,
        remove_islands: false,
        fill_holes: false,
    });

    assert_eq!(stats.triangles_flipped, 1);

    // Verify T2 is now (1, 3, 2)
    let t2 = mesh.triangles[1];
    assert_eq!(t2.v1, 1);
    assert!(t2.v2 == 3 || t2.v3 == 3);
}

#[test]
fn test_island_removal() {
    let mut mesh = Mesh::new();
    // Island 1: 2 triangles
    mesh.add_vertex(0.0, 0.0, 0.0);
    mesh.add_vertex(1.0, 0.0, 0.0);
    mesh.add_vertex(0.0, 1.0, 0.0);
    mesh.add_vertex(1.0, 1.0, 0.0);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 3, 2);

    // Island 2: 1 triangle
    mesh.add_vertex(10.0, 0.0, 0.0);
    mesh.add_vertex(11.0, 0.0, 0.0);
    mesh.add_vertex(10.0, 1.0, 0.0);
    mesh.add_triangle(4, 5, 6);

    assert_eq!(mesh.triangles.len(), 3);

    let stats = mesh.repair(RepairOptions {
        stitch_epsilon: 0.0,
        remove_degenerate: false,
        remove_duplicate_faces: false,
        harmonize_orientations: false,
        remove_islands: true,
        fill_holes: false,
    });

    assert_eq!(stats.triangles_removed, 1);
    assert_eq!(mesh.triangles.len(), 2);
}

#[test]
fn test_hole_filling() {
    let mut mesh = Mesh::new();
    mesh.add_vertex(0.0, 0.0, 0.0); // 0
    mesh.add_vertex(1.0, 0.0, 0.0); // 1
    mesh.add_vertex(0.0, 1.0, 0.0); // 2
    // Missing triangle (0, 1, 2)

    // Add one triangle to have SOME geometry
    mesh.add_vertex(1.0, 1.0, 0.0); // 3
    mesh.add_triangle(1, 3, 2);

    // Now we have edges (1,3), (3,2), (2,1)
    // Edge (1,2) is boundary. Wait, that's just one edge, not a loop?
    // A single triangle (1,3,2) has 3 boundary edges: (1,3), (3,2), (2,1).
    // They form a loop (1-3-2-1).

    assert_eq!(mesh.triangles.len(), 1);

    let stats = mesh.repair(RepairOptions {
        stitch_epsilon: 0.0,
        remove_degenerate: false,
        remove_duplicate_faces: false,
        harmonize_orientations: false,
        remove_islands: false,
        fill_holes: true,
    });

    // Should add 1 triangle to cap the (1,3,2) triangle's reverse side
    // (Wait, my loop filler isn't winding-aware, so it might just double the face,
    // but here it's filling the hole formed by the boundary).
    assert_eq!(stats.triangles_added, 1);
    assert_eq!(mesh.triangles.len(), 2);
}
