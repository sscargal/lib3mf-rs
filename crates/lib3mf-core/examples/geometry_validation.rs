use lib3mf_core::model::{Geometry, Mesh, Model, Object, ResourceId};
use lib3mf_core::validation::{ValidationLevel, ValidationReport, validate_geometry};

fn main() {
    println!("--- Geometry Validation Example ---");

    // 1. Create a simpler mesh that has issues
    let mut mesh = Mesh::new();

    // Triangle 1: (0,0,0) -> (1,0,0) -> (0,1,0) (Normal +Z)
    mesh.add_vertex(0.0, 0.0, 0.0);
    mesh.add_vertex(1.0, 0.0, 0.0);
    mesh.add_vertex(0.0, 1.0, 0.0);
    mesh.add_triangle(0, 1, 2);

    // Triangle 2: (0,0,0) -> (0,1,0) -> (1,0,0) (Normal -Z, Flipped relative to Tri 1)
    // Edge (0, 0, 0) -> (0, 1, 0) is shared.
    // In Tri 1: v0->v2 (0 -> 2).
    // In Tri 2: v0->v1 (0 -> 2).
    // Wait, indices: 0, 1, 2.
    // Tri 1: 0, 1, 2. Edges: 0->1, 1->2, 2->0.
    // Tri 2: 0, 2, 1. Edges: 0->2, 2->1, 1->0.
    // Shared edge is between 0 and 2.
    // Tri 1 has 2->0.
    // Tri 2 has 0->2.
    // This is CONSISTENT orientation (normals opposite, but winding consistent for a flat sheet).
    // Wait, separate issue.

    // Let's make a T-junction or non-manifold edge.
    // Add Triangle 3 attached to edge 0-1.
    // 0->1 is in Tri 1.
    // Add vertex (0.5, 0.5, 1.0).
    mesh.add_vertex(0.5, 0.5, 1.0); // 3
    mesh.add_triangle(0, 1, 3);

    // Add Triangle 4 ALSO attached to edge 0-1.
    mesh.add_vertex(0.5, -0.5, 1.0); // 4
    mesh.add_triangle(0, 1, 4);

    // Now edge 0-1 is shared by Tri 1, Tri 3, Tri 4. (Count = 3).
    // This is non-manifold (3 wings sharing an edge).

    let object = Object {
        id: ResourceId(1),
        geometry: Geometry::Mesh(mesh),
        name: Some("Problematic Mesh".to_string()),
        part_number: None,
        uuid: None,
        pid: None,
        pindex: None,
    };

    let mut model = Model::default();
    model.resources.add_object(object).unwrap();

    // 2. Standard Validation (Should pass geometry checks, as it mainly checks XML structure)
    let mut report = ValidationReport::default();
    validate_geometry(&model, ValidationLevel::Standard, &mut report);
    println!("Standard Validation Warning Count: {}", report.items.len());

    // 3. Paranoid Validation (Should detect non-manifold checks)
    let mut paranoid_report = ValidationReport::default();
    validate_geometry(&model, ValidationLevel::Paranoid, &mut paranoid_report);

    println!("Paranoid Validation Issues:");
    for item in paranoid_report.items {
        println!(
            " - [{:?}] Code {}: {}",
            item.severity, item.code, item.message
        );
    }
}
