use lib3mf_core::model::mesh::Mesh;

#[test]
fn test_mesh_verification() {
    let mut mesh = Mesh::default();
    
    // Create a simple tetrahedron
    mesh.add_vertex(0.0, 0.0, 0.0);
    mesh.add_vertex(1.0, 0.0, 0.0);
    mesh.add_vertex(0.0, 1.0, 0.0);
    mesh.add_vertex(0.0, 0.0, 1.0);
    
    mesh.add_triangle(0, 2, 1);
    mesh.add_triangle(0, 1, 3);
    mesh.add_triangle(0, 3, 2);
    mesh.add_triangle(1, 2, 3);
    
    let aabb = mesh.compute_aabb().expect("AABB failed");
    assert_eq!(aabb.min, [0.0, 0.0, 0.0]);
    assert_eq!(aabb.max, [1.0, 1.0, 1.0]);
    
    let (area, volume) = mesh.compute_area_and_volume();
    assert!(area > 0.0);
    assert!(volume > 0.0);
}
