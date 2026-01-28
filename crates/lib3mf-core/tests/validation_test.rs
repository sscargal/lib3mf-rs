use lib3mf_core::model::{Geometry, Mesh, Model, Object, ResourceCollection, ResourceId};
use lib3mf_core::validation::{ValidationLevel, ValidationSeverity};

#[test]
fn test_validation_clean_model() {
    let model = Model::default();
    let report = model.validate(ValidationLevel::Standard);
    assert!(!report.has_errors());
    assert!(report.items.is_empty());
}

#[test]
fn test_validation_invalid_pid() {
    let mut model = Model::default();
    let mesh = Mesh::new();
    let object = Object {
        id: ResourceId(1),
        name: None,
        part_number: None,
        pid: Some(ResourceId(999)), // Non-existent property group
        pindex: None,
        geometry: Geometry::Mesh(mesh),
    };
    model.resources.add_object(object).unwrap();

    // Standard level should catch this
    let report = model.validate(ValidationLevel::Standard);
    assert!(report.has_errors());
    let err = report.items.iter().find(|i| i.code == 2001).expect("Expected error 2001");
    assert_eq!(err.severity, ValidationSeverity::Error);

    // Minimal level should IGNORE this (structural only)
    let minimal_report = model.validate(ValidationLevel::Minimal);
    assert!(!minimal_report.has_errors());
}

#[test]
fn test_validation_invalid_triangle_indices() {
    let mut model = Model::default();
    let mut mesh = Mesh::new();
    mesh.add_vertex(0.0, 0.0, 0.0);
    // Triangle references index 1 and 2, but only 0 exists
    mesh.add_triangle(0, 1, 2);

    let object = Object {
        id: ResourceId(1),
        name: None,
        part_number: None,
        pid: None,
        pindex: None,
        geometry: Geometry::Mesh(mesh),
    };
    model.resources.add_object(object).unwrap();

    let report = model.validate(ValidationLevel::Standard);
    assert!(report.has_errors());
    let err = report.items.iter().find(|i| i.code == 3001).expect("Expected error 3001 (OOB index)");
    assert_eq!(err.severity, ValidationSeverity::Error);
}
