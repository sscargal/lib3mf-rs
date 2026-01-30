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
