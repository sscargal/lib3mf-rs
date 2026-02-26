//! Integration tests for BeamLattice `radius` attribute roundtrip fidelity.
//!
//! All 6 existing tests in `beamlattice_roundtrip.rs` use `radius: None`.
//! These tests cover the unchecked paths:
//!   - `radius: Some(f32)` — attribute must be emitted and survive parse
//!   - `radius: None`     — attribute must be omitted and stay None after parse

use lib3mf_core::model::*;
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

// ============================================================================
// Helper (mirrors beamlattice_roundtrip.rs)
// ============================================================================

/// Creates a base Model with one Object (id=1) containing a Mesh with 4 vertices,
/// 1 triangle (minimal valid mesh), and the provided BeamLattice.
fn create_base_model_with_lattice(lattice: BeamLattice) -> Model {
    let mut model = Model::default();

    let mut mesh = Mesh::new();
    // Tetrahedron corners
    mesh.add_vertex(0.0, 0.0, 0.0);
    mesh.add_vertex(10.0, 0.0, 0.0);
    mesh.add_vertex(0.0, 10.0, 0.0);
    mesh.add_vertex(0.0, 0.0, 10.0);
    // One triangle to make the mesh non-empty
    mesh.add_triangle(0, 1, 2);

    mesh.beam_lattice = Some(lattice);

    model
        .resources
        .add_object(Object {
            id: ResourceId(1),
            object_type: ObjectType::Model,
            name: None,
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::Mesh(mesh),
        })
        .unwrap();

    model
}

// ============================================================================
// Test: radius: Some(2.5) — attribute emitted and parsed correctly
// ============================================================================

/// Verify that `BeamLattice { radius: Some(2.5), .. }` survives a write-parse
/// roundtrip: the attribute appears in the serialized XML and is reconstructed
/// as `Some(r)` where `r ≈ 2.5`.
#[test]
fn test_beam_lattice_radius_some_roundtrip() {
    let lattice = BeamLattice {
        radius: Some(2.5),
        min_length: 0.1,
        precision: 0.01,
        clipping_mode: ClippingMode::None,
        beams: vec![Beam {
            v1: 0,
            v2: 1,
            r1: 1.0,
            r2: 1.0,
            p1: None,
            p2: None,
            cap_mode: CapMode::Sphere,
        }],
        beam_sets: vec![],
    };

    let model = create_base_model_with_lattice(lattice);

    // Serialize
    let mut buffer: Vec<u8> = Vec::new();
    model.write_xml(&mut buffer, None).expect("write_xml failed");
    let xml_str = String::from_utf8(buffer.clone()).expect("non-UTF-8 output");

    // The radius attribute must be present in the serialized XML
    assert!(
        xml_str.contains(r#"radius="2.5""#),
        "Expected radius=\"2.5\" in XML output when radius is Some(2.5).\nGot XML:\n{}",
        &xml_str[..xml_str.len().min(800)]
    );

    // Parse back
    let parsed = parse_model(Cursor::new(&buffer)).expect("parse_model failed");
    let obj = parsed
        .resources
        .get_object(ResourceId(1))
        .expect("Object 1 not found after roundtrip");

    if let Geometry::Mesh(mesh) = &obj.geometry {
        let lat = mesh
            .beam_lattice
            .as_ref()
            .expect("BeamLattice should be present after roundtrip");

        match lat.radius {
            Some(r) => assert!(
                (r - 2.5).abs() < 1e-6,
                "radius value mismatch: expected 2.5, got {}",
                r
            ),
            None => panic!("radius should be Some(2.5) after roundtrip, got None"),
        }
    } else {
        panic!("Expected Geometry::Mesh after roundtrip");
    }
}

// ============================================================================
// Test: radius: None — attribute omitted and stays None after parse
// ============================================================================

/// Verify that `BeamLattice { radius: None, .. }` correctly omits the `radius`
/// attribute from serialized XML and parses back as `None`.
#[test]
fn test_beam_lattice_radius_none_roundtrip() {
    let lattice = BeamLattice {
        radius: None,
        min_length: 0.1,
        precision: 0.01,
        clipping_mode: ClippingMode::None,
        beams: vec![Beam {
            v1: 0,
            v2: 1,
            r1: 1.0,
            r2: 1.0,
            p1: None,
            p2: None,
            cap_mode: CapMode::Sphere,
        }],
        beam_sets: vec![],
    };

    let model = create_base_model_with_lattice(lattice);

    // Serialize
    let mut buffer: Vec<u8> = Vec::new();
    model.write_xml(&mut buffer, None).expect("write_xml failed");
    let xml_str = String::from_utf8(buffer.clone()).expect("non-UTF-8 output");

    // The radius attribute must NOT appear in the XML when radius is None
    assert!(
        !xml_str.contains(r#"radius=""#),
        "radius attribute should be omitted when radius is None.\nGot XML:\n{}",
        &xml_str[..xml_str.len().min(800)]
    );

    // Parse back
    let parsed = parse_model(Cursor::new(&buffer)).expect("parse_model failed");
    let obj = parsed
        .resources
        .get_object(ResourceId(1))
        .expect("Object 1 not found after roundtrip");

    if let Geometry::Mesh(mesh) = &obj.geometry {
        let lat = mesh
            .beam_lattice
            .as_ref()
            .expect("BeamLattice should be present after roundtrip");

        assert_eq!(
            lat.radius, None,
            "radius should remain None after roundtrip when not set"
        );
    } else {
        panic!("Expected Geometry::Mesh after roundtrip");
    }
}
