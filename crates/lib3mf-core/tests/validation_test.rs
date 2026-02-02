use lib3mf_core::model::{Geometry, Mesh, Model, Object, ResourceId};
use lib3mf_core::validation::{
    ValidationLevel, ValidationReport, ValidationSeverity, validate_geometry,
};

fn create_cube() -> Mesh {
    let mut mesh = Mesh::new();
    // Vertices
    // Bottom
    mesh.add_vertex(0.0, 0.0, 0.0); // 0
    mesh.add_vertex(1.0, 0.0, 0.0); // 1
    mesh.add_vertex(1.0, 1.0, 0.0); // 2
    mesh.add_vertex(0.0, 1.0, 0.0); // 3
    // Top
    mesh.add_vertex(0.0, 0.0, 1.0); // 4
    mesh.add_vertex(1.0, 0.0, 1.0); // 5
    mesh.add_vertex(1.0, 1.0, 1.0); // 6
    mesh.add_vertex(0.0, 1.0, 1.0); // 7

    // Triangles (CCW outside)
    // Bottom
    mesh.add_triangle(0, 2, 1);
    mesh.add_triangle(0, 3, 2);
    // Top
    mesh.add_triangle(4, 5, 6);
    mesh.add_triangle(4, 6, 7);
    // Front
    mesh.add_triangle(0, 1, 5);
    mesh.add_triangle(0, 5, 4);
    // Right
    mesh.add_triangle(1, 2, 6);
    mesh.add_triangle(1, 6, 5);
    // Back
    mesh.add_triangle(2, 3, 7);
    mesh.add_triangle(2, 7, 6);
    // Left
    mesh.add_triangle(3, 0, 4);
    mesh.add_triangle(3, 4, 7);

    mesh
}

fn make_object(mesh: Mesh) -> Object {
    Object {
        id: ResourceId(1),
        geometry: Geometry::Mesh(mesh),
        name: None,
        part_number: None,
        uuid: None,
        pid: None,
        thumbnail: None,
        pindex: None,
    }
}

#[test]
fn test_perfect_cube() {
    let mut model = Model::default();
    let mesh = create_cube();
    let object = make_object(mesh);
    model.resources.add_object(object).unwrap();

    let mut report = ValidationReport::default();
    validate_geometry(&model, ValidationLevel::Paranoid, &mut report);

    assert!(!report.has_errors(), "Perfect cube should have no errors");
    assert!(
        !report
            .items
            .iter()
            .any(|i| i.severity == ValidationSeverity::Warning),
        "Perfect cube should have no warnings"
    );
}

#[test]
fn test_missing_face() {
    let mut model = Model::default();
    let mut mesh = create_cube();
    // Remove last two triangles (Left face)
    mesh.triangles.pop();
    mesh.triangles.pop();

    let object = make_object(mesh);
    model.resources.add_object(object).unwrap();

    let mut report = ValidationReport::default();
    validate_geometry(&model, ValidationLevel::Paranoid, &mut report);

    // Should detect boundary edges (code 4002)
    assert!(
        report.items.iter().any(|i| i.code == 4002),
        "Should detect boundary edges (4002). Got: {:?}",
        report.items
    );
}

#[test]
fn test_flipped_face() {
    let mut model = Model::default();
    let mut mesh = create_cube();
    // Flip first triangle
    let t0 = mesh.triangles[0];
    mesh.triangles[0] = lib3mf_core::model::Triangle {
        v1: t0.v1,
        v2: t0.v3,
        v3: t0.v2,
        ..Default::default()
    };

    let object = make_object(mesh);
    model.resources.add_object(object).unwrap();

    let mut report = ValidationReport::default();
    validate_geometry(&model, ValidationLevel::Paranoid, &mut report);

    // Should detect orientation mismatch (code 4004)
    assert!(
        report.items.iter().any(|i| i.code == 4004),
        "Should detect orientation mismatch (4004). Got: {:?}",
        report.items
    );
}

#[test]
fn test_degenerate_face() {
    let mut model = Model::default();
    let mut mesh = create_cube();

    // Add degenerate triangle (area 0, collinear)
    // 0=(0,0,0), 1=(1,0,0). Add 8=(2,0,0).
    mesh.add_vertex(2.0, 0.0, 0.0); // 8
    mesh.add_triangle(0, 1, 8);

    let object = make_object(mesh);
    model.resources.add_object(object).unwrap();

    let mut report = ValidationReport::default();
    validate_geometry(&model, ValidationLevel::Paranoid, &mut report);

    // Should detect zero area triangle (code 4005)
    assert!(
        report.items.iter().any(|i| i.code == 4005),
        "Should detect zero area triangle (4005). Got: {:?}",
        report.items
    );
}

#[test]
fn test_non_manifold_vertex() {
    let mut model = Model::default();
    let mut mesh = Mesh::new();

    // Two disjoint triangles sharing only vertex 0 (hourglass shape)
    mesh.add_vertex(0.0, 0.0, 0.0); // 0
    mesh.add_vertex(1.0, 0.0, 0.0); // 1
    mesh.add_vertex(0.0, 1.0, 0.0); // 2
    mesh.add_triangle(0, 1, 2);

    mesh.add_vertex(-1.0, 0.0, 0.0); // 3
    mesh.add_vertex(0.0, -1.0, 0.0); // 4
    mesh.add_triangle(0, 3, 4);

    let object = make_object(mesh);
    model.resources.add_object(object).unwrap();

    let mut report = ValidationReport::default();
    validate_geometry(&model, ValidationLevel::Paranoid, &mut report);

    // Should detect non-manifold vertex (code 4006)
    assert!(
        report.items.iter().any(|i| i.code == 4006),
        "Should detect non-manifold vertex (4006). Got: {:?}",
        report.items
    );
}

#[test]
fn test_islands() {
    let mut model = Model::default();
    let mut mesh = Mesh::new();

    // Island 1: Triangle (0, 1, 2)
    mesh.add_vertex(0.0, 0.0, 0.0);
    mesh.add_vertex(1.0, 0.0, 0.0);
    mesh.add_vertex(0.0, 1.0, 0.0);
    mesh.add_triangle(0, 1, 2);

    // Island 2: Triangle (3, 4, 5) - completely disconnected
    mesh.add_vertex(10.0, 0.0, 0.0);
    mesh.add_vertex(11.0, 0.0, 0.0);
    mesh.add_vertex(10.0, 1.0, 0.0);
    mesh.add_triangle(3, 4, 5);

    let object = make_object(mesh);
    model.resources.add_object(object).unwrap();

    let mut report = ValidationReport::default();
    validate_geometry(&model, ValidationLevel::Paranoid, &mut report);

    // Should detect islands (code 4007)
    assert!(
        report.items.iter().any(|i| i.code == 4007),
        "Should detect islands (4007). Got: {:?}",
        report.items
    );
}

#[test]
fn test_self_intersection() {
    let mut model = Model::default();
    let mut mesh = Mesh::new();

    // Two intersecting triangles (X-shape)
    // T1: (0,0,-1) - (0,0,1) - (1,0,0) -> XY plane roughly
    mesh.add_vertex(0.0, 1.0, 0.0); // 0
    mesh.add_vertex(0.0, -1.0, 0.0); // 1
    mesh.add_vertex(1.0, 0.0, 0.0); // 2
    mesh.add_triangle(0, 1, 2);

    // T2: (-1,0,0) - (1,0,0) - (0,1,0) -> wait, let's make them really intersect
    // T2: (0.5, 0.5, -1.0) - (0.5, 0.5, 1.0) - (-0.5, 0.5, 0.0)
    mesh.add_vertex(0.5, 0.0, -1.0); // 3
    mesh.add_vertex(0.5, 0.0, 1.0); // 4
    mesh.add_vertex(-0.5, 0.0, 0.0); // 5
    mesh.add_triangle(3, 4, 5);

    let object = make_object(mesh);
    model.resources.add_object(object).unwrap();

    let mut report = ValidationReport::default();
    validate_geometry(&model, ValidationLevel::Paranoid, &mut report);

    // Should detect self-intersection (code 4008)
    assert!(
        report.items.iter().any(|i| i.code == 4008),
        "Should detect self-intersection (4008). Got: {:?}",
        report.items
    );
}
