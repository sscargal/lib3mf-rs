//! Integration tests for Beam Lattice Extension writer (roundtrip fidelity)
//!
//! Covers requirements BLW-01 through BLW-08.

use lib3mf_core::model::*;
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

// ============================================================================
// Helper
// ============================================================================

/// Creates a base Model with one Object (id=1) containing a Mesh with 4 vertices,
/// 4 triangles (tetrahedron), and the provided BeamLattice.
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
// BLW-01: Namespace declaration present in output XML
// ============================================================================

/// Verify that the beam lattice namespace URI is declared on the model element
/// and that the beamlattice element appears in the output XML.
#[test]
fn test_beam_lattice_namespace_present() {
    let lattice = BeamLattice {
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
    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();
    let xml_str = String::from_utf8(buffer).unwrap();

    // BLW-01: namespace URI must be declared
    assert!(
        xml_str.contains(
            r#"xmlns:bl="http://schemas.microsoft.com/3dmanufacturing/beamlattice/2017/02""#
        ),
        "Expected beam lattice namespace declaration in output XML.\nGot XML:\n{}",
        &xml_str[..xml_str.len().min(500)]
    );

    // Verify beamlattice element is present inside mesh
    assert!(
        xml_str.contains("<beamlattice"),
        "Expected <beamlattice element in output XML"
    );
}

// ============================================================================
// BLW-02, BLW-03, BLW-07: Basic roundtrip (lattice attrs, beams, radii)
// ============================================================================

/// Full roundtrip: write model with beam lattice, re-parse, verify all fields preserved.
#[test]
fn test_beam_lattice_basic_roundtrip() {
    let lattice = BeamLattice {
        min_length: 0.1,
        precision: 0.01,
        clipping_mode: ClippingMode::Inside,
        beams: vec![
            // Tapered beam (r1 != r2)
            Beam {
                v1: 0,
                v2: 1,
                r1: 1.5,
                r2: 0.5,
                p1: None,
                p2: None,
                cap_mode: CapMode::Sphere,
            },
            // Beam with property indices
            Beam {
                v1: 1,
                v2: 2,
                r1: 1.0,
                r2: 1.0,
                p1: Some(5),
                p2: Some(10),
                cap_mode: CapMode::Hemisphere,
            },
            // Default beam
            Beam {
                v1: 0,
                v2: 3,
                r1: 0.75,
                r2: 0.75,
                p1: None,
                p2: None,
                cap_mode: CapMode::Sphere,
            },
        ],
        beam_sets: vec![],
    };

    let model = create_base_model_with_lattice(lattice);

    // Write
    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();

    // Parse back
    let parsed = parse_model(Cursor::new(&buffer)).unwrap();
    let obj = parsed.resources.get_object(ResourceId(1)).unwrap();

    if let Geometry::Mesh(mesh) = &obj.geometry {
        let lat = mesh.beam_lattice.as_ref().expect("BeamLattice should be present after roundtrip");

        // BLW-02: lattice-level attributes preserved
        assert!((lat.min_length - 0.1).abs() < 1e-6, "min_length mismatch");
        assert!((lat.precision - 0.01).abs() < 1e-6, "precision mismatch");
        assert_eq!(lat.clipping_mode, ClippingMode::Inside, "clipping_mode mismatch");

        // BLW-03: beams preserved
        assert_eq!(lat.beams.len(), 3, "beam count mismatch");

        // Beam 0: tapered
        let b0 = &lat.beams[0];
        assert_eq!(b0.v1, 0);
        assert_eq!(b0.v2, 1);
        assert!((b0.r1 - 1.5).abs() < 1e-6, "beam0 r1 mismatch");
        assert!((b0.r2 - 0.5).abs() < 1e-6, "beam0 r2 mismatch");
        assert_eq!(b0.p1, None);
        assert_eq!(b0.p2, None);
        assert_eq!(b0.cap_mode, CapMode::Sphere);

        // Beam 1: with property indices
        let b1 = &lat.beams[1];
        assert_eq!(b1.v1, 1);
        assert_eq!(b1.v2, 2);
        assert!((b1.r1 - 1.0).abs() < 1e-6, "beam1 r1 mismatch");
        assert!((b1.r2 - 1.0).abs() < 1e-6, "beam1 r2 mismatch");
        assert_eq!(b1.p1, Some(5));
        assert_eq!(b1.p2, Some(10));
        assert_eq!(b1.cap_mode, CapMode::Hemisphere);

        // Beam 2: default
        let b2 = &lat.beams[2];
        assert_eq!(b2.v1, 0);
        assert_eq!(b2.v2, 3);
        assert!((b2.r1 - 0.75).abs() < 1e-6, "beam2 r1 mismatch");
        assert!((b2.r2 - 0.75).abs() < 1e-6, "beam2 r2 mismatch");
        assert_eq!(b2.cap_mode, CapMode::Sphere);
    } else {
        panic!("Expected Geometry::Mesh after roundtrip");
    }
}

// ============================================================================
// BLW-05: All CapMode variants roundtrip
// ============================================================================

/// Verify all three cap modes serialize and deserialize correctly.
#[test]
fn test_beam_lattice_cap_modes_roundtrip() {
    let lattice = BeamLattice {
        min_length: 0.1,
        precision: 0.0,
        clipping_mode: ClippingMode::None,
        beams: vec![
            Beam {
                v1: 0,
                v2: 1,
                r1: 1.0,
                r2: 1.0,
                p1: None,
                p2: None,
                cap_mode: CapMode::Sphere, // default: cap attr omitted in XML
            },
            Beam {
                v1: 1,
                v2: 2,
                r1: 1.0,
                r2: 1.0,
                p1: None,
                p2: None,
                cap_mode: CapMode::Hemisphere,
            },
            Beam {
                v1: 0,
                v2: 3,
                r1: 1.0,
                r2: 1.0,
                p1: None,
                p2: None,
                cap_mode: CapMode::Butt,
            },
        ],
        beam_sets: vec![],
    };

    let model = create_base_model_with_lattice(lattice);
    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();
    let parsed = parse_model(Cursor::new(&buffer)).unwrap();

    let obj = parsed.resources.get_object(ResourceId(1)).unwrap();
    if let Geometry::Mesh(mesh) = &obj.geometry {
        let lat = mesh.beam_lattice.as_ref().unwrap();
        assert_eq!(lat.beams.len(), 3);
        assert_eq!(lat.beams[0].cap_mode, CapMode::Sphere, "Sphere cap mode not preserved");
        assert_eq!(lat.beams[1].cap_mode, CapMode::Hemisphere, "Hemisphere cap mode not preserved");
        assert_eq!(lat.beams[2].cap_mode, CapMode::Butt, "Butt cap mode not preserved");
    } else {
        panic!("Expected Geometry::Mesh");
    }
}

// ============================================================================
// BLW-06: All ClippingMode variants roundtrip
// ============================================================================

/// Verify all three clipping modes serialize and deserialize correctly.
#[test]
fn test_beam_lattice_clipping_modes_roundtrip() {
    for (mode, label) in [
        (ClippingMode::None, "None"),
        (ClippingMode::Inside, "Inside"),
        (ClippingMode::Outside, "Outside"),
    ] {
        let lattice = BeamLattice {
            min_length: 0.1,
            precision: 0.0,
            clipping_mode: mode,
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
        let mut buffer = Vec::new();
        model.write_xml(&mut buffer, None).unwrap();
        let parsed = parse_model(Cursor::new(&buffer)).unwrap();

        let obj = parsed.resources.get_object(ResourceId(1)).unwrap();
        if let Geometry::Mesh(mesh) = &obj.geometry {
            let lat = mesh.beam_lattice.as_ref().unwrap();
            assert_eq!(
                lat.clipping_mode, mode,
                "ClippingMode::{} not preserved after roundtrip",
                label
            );
        } else {
            panic!("Expected Geometry::Mesh for ClippingMode::{}", label);
        }
    }
}

// ============================================================================
// BLW-04: BeamSets roundtrip
// ============================================================================

/// Verify beam sets with name, identifier, refs (and combinations) roundtrip correctly.
#[test]
fn test_beam_lattice_beam_sets_roundtrip() {
    let lattice = BeamLattice {
        min_length: 0.1,
        precision: 0.0,
        clipping_mode: ClippingMode::None,
        beams: vec![
            Beam { v1: 0, v2: 1, r1: 1.0, r2: 1.0, p1: None, p2: None, cap_mode: CapMode::Sphere },
            Beam { v1: 1, v2: 2, r1: 1.0, r2: 1.0, p1: None, p2: None, cap_mode: CapMode::Sphere },
            Beam { v1: 0, v2: 3, r1: 1.0, r2: 1.0, p1: None, p2: None, cap_mode: CapMode::Sphere },
        ],
        beam_sets: vec![
            // Full: name + identifier + refs
            BeamSet {
                name: Some("Set1".to_string()),
                identifier: Some("BS1".to_string()),
                refs: vec![0, 1],
            },
            // Name only
            BeamSet {
                name: Some("NameOnly".to_string()),
                identifier: None,
                refs: vec![2],
            },
            // Empty refs (no refs)
            BeamSet {
                name: None,
                identifier: None,
                refs: vec![],
            },
        ],
    };

    let model = create_base_model_with_lattice(lattice);
    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();
    let parsed = parse_model(Cursor::new(&buffer)).unwrap();

    let obj = parsed.resources.get_object(ResourceId(1)).unwrap();
    if let Geometry::Mesh(mesh) = &obj.geometry {
        let lat = mesh.beam_lattice.as_ref().unwrap();
        assert_eq!(lat.beam_sets.len(), 3, "beam_sets count mismatch");

        // BeamSet 0: name + identifier + refs
        let bs0 = &lat.beam_sets[0];
        assert_eq!(bs0.name, Some("Set1".to_string()), "bs0 name mismatch");
        assert_eq!(bs0.identifier, Some("BS1".to_string()), "bs0 identifier mismatch");
        assert_eq!(bs0.refs, vec![0, 1], "bs0 refs mismatch");

        // BeamSet 1: name only
        let bs1 = &lat.beam_sets[1];
        assert_eq!(bs1.name, Some("NameOnly".to_string()), "bs1 name mismatch");
        assert_eq!(bs1.identifier, None, "bs1 identifier should be None");
        assert_eq!(bs1.refs, vec![2], "bs1 refs mismatch");

        // BeamSet 2: empty (no name, identifier, or refs)
        let bs2 = &lat.beam_sets[2];
        assert_eq!(bs2.name, None, "bs2 name should be None");
        assert_eq!(bs2.identifier, None, "bs2 identifier should be None");
        assert!(bs2.refs.is_empty(), "bs2 refs should be empty");
    } else {
        panic!("Expected Geometry::Mesh");
    }
}

// ============================================================================
// Edge case: no beam sets (beamsets element should not appear in XML)
// ============================================================================

/// Verify that when beam_sets is empty, the <beamsets> element is omitted in output XML
/// and the model roundtrips correctly.
#[test]
fn test_beam_lattice_no_beam_sets() {
    let lattice = BeamLattice {
        min_length: 0.5,
        precision: 0.001,
        clipping_mode: ClippingMode::None,
        beams: vec![
            Beam { v1: 0, v2: 1, r1: 2.0, r2: 2.0, p1: None, p2: None, cap_mode: CapMode::Sphere },
        ],
        beam_sets: vec![],
    };

    let model = create_base_model_with_lattice(lattice);
    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();
    let xml_str = String::from_utf8(buffer.clone()).unwrap();

    // Verify beamsets element is NOT present when beam_sets is empty
    assert!(
        !xml_str.contains("<beamsets"),
        "<beamsets> element should not appear when beam_sets is empty"
    );

    // Roundtrip still works
    let parsed = parse_model(Cursor::new(&buffer)).unwrap();
    let obj = parsed.resources.get_object(ResourceId(1)).unwrap();
    if let Geometry::Mesh(mesh) = &obj.geometry {
        let lat = mesh.beam_lattice.as_ref().unwrap();
        assert!(lat.beam_sets.is_empty(), "beam_sets should be empty after roundtrip");
        assert_eq!(lat.beams.len(), 1, "beam count mismatch");
        assert!((lat.min_length - 0.5).abs() < 1e-6, "min_length mismatch");
        assert!((lat.precision - 0.001).abs() < 1e-6, "precision mismatch");
    } else {
        panic!("Expected Geometry::Mesh");
    }
}
