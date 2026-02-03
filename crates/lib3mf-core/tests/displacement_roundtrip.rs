//! Integration tests for Displacement Extension

use lib3mf_core::model::*;
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

#[test]
fn test_displacement_mesh_roundtrip() {
    let model = create_test_model();

    // Write
    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();

    // Parse
    let parsed = parse_model(Cursor::new(&buffer)).unwrap();

    // Verify
    let obj = parsed.resources.get_object(ResourceId(1)).unwrap();
    if let Geometry::DisplacementMesh(dmesh) = &obj.geometry {
        assert_eq!(dmesh.vertices.len(), 3);
        assert_eq!(dmesh.triangles.len(), 1);
        assert_eq!(dmesh.normals.len(), 3);
    } else {
        panic!("Expected DisplacementMesh geometry");
    }
}

#[test]
fn test_displacement_2d_roundtrip() {
    let mut model = Model::default();
    let texture = Displacement2D {
        id: ResourceId(100),
        path: "/3D/Textures/test.png".to_string(),
        channel: Channel::R,
        tile_style: TileStyle::Mirror,
        filter: FilterMode::Nearest,
        height: -1.5,
        offset: 0.25,
    };
    model.resources.add_displacement_2d(texture).unwrap();

    // Write and parse
    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();
    let parsed = parse_model(Cursor::new(&buffer)).unwrap();

    // Verify all non-default values are preserved
    let tex = parsed
        .resources
        .get_displacement_2d(ResourceId(100))
        .unwrap();
    assert_eq!(tex.path, "/3D/Textures/test.png");
    assert_eq!(tex.channel, Channel::R); // Non-default, should be written
    assert_eq!(tex.tile_style, TileStyle::Mirror); // Non-default
    assert_eq!(tex.filter, FilterMode::Nearest); // Non-default
    assert!((tex.height - (-1.5)).abs() < 0.001);
    assert!((tex.offset - 0.25).abs() < 0.001);
}

#[test]
fn test_displacement_2d_defaults() {
    let mut model = Model::default();
    let texture = Displacement2D {
        id: ResourceId(101),
        path: "/3D/Textures/default.png".to_string(),
        channel: Channel::G,         // Default
        tile_style: TileStyle::Wrap, // Default
        filter: FilterMode::Linear,  // Default
        height: 1.0,
        offset: 0.0, // Default
    };
    model.resources.add_displacement_2d(texture).unwrap();

    // Write and parse
    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();
    let parsed = parse_model(Cursor::new(&buffer)).unwrap();

    // Verify defaults are preserved (even though not written explicitly)
    let tex = parsed
        .resources
        .get_displacement_2d(ResourceId(101))
        .unwrap();
    assert_eq!(tex.path, "/3D/Textures/default.png");
    assert_eq!(tex.channel, Channel::G); // Parser defaults to G
    assert_eq!(tex.tile_style, TileStyle::Wrap); // Parser defaults to Wrap
    assert_eq!(tex.filter, FilterMode::Linear); // Parser defaults to Linear
    assert!((tex.height - 1.0).abs() < 0.001);
    assert!((tex.offset - 0.0).abs() < 0.001);
}

#[test]
fn test_normal_validation() {
    use lib3mf_core::validation::ValidationLevel;

    let model = create_test_model();

    let report = model.validate(ValidationLevel::Paranoid);
    // Should pass with unit-length normals
    assert!(!report.has_errors());
}

fn create_test_model() -> Model {
    let mut model = Model::default();

    let dmesh = DisplacementMesh {
        vertices: vec![
            Vertex {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Vertex {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Vertex {
                x: 0.5,
                y: 1.0,
                z: 0.0,
            },
        ],
        triangles: vec![DisplacementTriangle {
            v1: 0,
            v2: 1,
            v3: 2,
            d1: None,
            d2: None,
            d3: None,
            p1: None,
            p2: None,
            p3: None,
            pid: None,
        }],
        normals: vec![
            NormalVector {
                nx: 0.0,
                ny: 0.0,
                nz: 1.0,
            },
            NormalVector {
                nx: 0.0,
                ny: 0.0,
                nz: 1.0,
            },
            NormalVector {
                nx: 0.0,
                ny: 0.0,
                nz: 1.0,
            },
        ],
        gradients: None,
    };

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
            geometry: Geometry::DisplacementMesh(dmesh),
        })
        .unwrap();

    model
}
