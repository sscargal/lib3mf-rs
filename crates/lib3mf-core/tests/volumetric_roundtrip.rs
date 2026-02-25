//! Integration tests for Volumetric Extension writer (roundtrip fidelity)
//!
//! Covers requirements VLW-01 through VLW-07 and edge cases.

use glam::Mat4;
use lib3mf_core::model::*;
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

// ============================================================================
// Helpers
// ============================================================================

/// Round-trip a model through write_xml -> parse_model.
fn roundtrip(model: &Model) -> Model {
    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();
    parse_model(Cursor::new(&buffer)).unwrap()
}

/// Helper to create a default build item for an object.
fn build_item(object_id: ResourceId) -> BuildItem {
    BuildItem {
        object_id,
        uuid: None,
        path: None,
        part_number: None,
        transform: Mat4::IDENTITY,
        printable: None,
    }
}

// ============================================================================
// VLW-01: Namespace declaration present in output XML
// ============================================================================

/// Verify that the volumetric namespace URI is declared on the model element.
#[test]
fn test_volumetric_namespace_present() {
    let mut model = Model::default();

    let stack = VolumetricStack {
        id: ResourceId(10),
        version: String::new(),
        layers: vec![VolumetricLayer {
            z_height: 0.0,
            content_path: "/3D/layer0.png".to_string(),
        }],
        refs: vec![],
    };
    model.resources.add_volumetric_stack(stack).unwrap();

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
            geometry: Geometry::VolumetricStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();
    let xml_str = String::from_utf8(buffer).unwrap();

    assert!(
        xml_str.contains(
            r#"xmlns:v="http://schemas.microsoft.com/3dmanufacturing/volumetric/2018/11""#
        ),
        "Expected volumetric namespace declaration in output XML.\nGot XML:\n{}",
        &xml_str[..xml_str.len().min(500)]
    );
}

// ============================================================================
// VLW-02 + VLW-03 + VLW-06 + VLW-07: Basic roundtrip
// ============================================================================

/// Full roundtrip: model with one VolumetricStack containing 2 layers and 1 ref.
/// Verifies stack id, layer count, z_height, content_path, ref count, ref fields,
/// and object geometry reference.
#[test]
fn test_volumetric_basic_roundtrip() {
    let mut model = Model::default();

    let stack = VolumetricStack {
        id: ResourceId(10),
        version: String::new(),
        layers: vec![
            VolumetricLayer {
                z_height: 0.0,
                content_path: "/3D/layer0.png".to_string(),
            },
            VolumetricLayer {
                z_height: 0.5,
                content_path: "/3D/layer1.png".to_string(),
            },
        ],
        refs: vec![VolumetricRef {
            stack_id: ResourceId(20),
            path: "/3D/external.model".to_string(),
        }],
    };
    model.resources.add_volumetric_stack(stack).unwrap();

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
            geometry: Geometry::VolumetricStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    let parsed = roundtrip(&model);

    // VolumetricStack preserved (VLW-02)
    let stack = parsed
        .resources
        .get_volumetric_stack(ResourceId(10))
        .expect("VolumetricStack 10 should exist after roundtrip");
    assert_eq!(stack.id, ResourceId(10));

    // Layers preserved (VLW-03)
    assert_eq!(stack.layers.len(), 2);
    assert_eq!(stack.layers[0].z_height, 0.0);
    assert_eq!(stack.layers[0].content_path, "/3D/layer0.png");
    assert_eq!(stack.layers[1].z_height, 0.5);
    assert_eq!(stack.layers[1].content_path, "/3D/layer1.png");

    // Refs preserved (VLW-04)
    assert_eq!(stack.refs.len(), 1);
    assert_eq!(stack.refs[0].stack_id, ResourceId(20));
    assert_eq!(stack.refs[0].path, "/3D/external.model");

    // Object geometry reference preserved (VLW-06/07)
    let obj = parsed
        .resources
        .get_object(ResourceId(1))
        .expect("Object 1 should exist");
    match &obj.geometry {
        Geometry::VolumetricStack(vsid) => assert_eq!(*vsid, ResourceId(10)),
        other => panic!("Expected Geometry::VolumetricStack, got {:?}", other),
    }
}

// ============================================================================
// VLW-03: Layers roundtrip with 3 layers
// ============================================================================

/// Verify 3 layers at z=0.0, 0.5, 1.0 with different content_path values all survive roundtrip.
#[test]
fn test_volumetric_layers_roundtrip() {
    let mut model = Model::default();

    let stack = VolumetricStack {
        id: ResourceId(10),
        version: String::new(),
        layers: vec![
            VolumetricLayer {
                z_height: 0.0,
                content_path: "/3D/layer_a.png".to_string(),
            },
            VolumetricLayer {
                z_height: 0.5,
                content_path: "/3D/layer_b.png".to_string(),
            },
            VolumetricLayer {
                z_height: 1.0,
                content_path: "/3D/layer_c.png".to_string(),
            },
        ],
        refs: vec![],
    };
    model.resources.add_volumetric_stack(stack).unwrap();

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
            geometry: Geometry::VolumetricStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    let parsed = roundtrip(&model);
    let stack = parsed
        .resources
        .get_volumetric_stack(ResourceId(10))
        .unwrap();

    assert_eq!(stack.layers.len(), 3);
    assert_eq!(stack.layers[0].z_height, 0.0);
    assert_eq!(stack.layers[0].content_path, "/3D/layer_a.png");
    assert_eq!(stack.layers[1].z_height, 0.5);
    assert_eq!(stack.layers[1].content_path, "/3D/layer_b.png");
    assert_eq!(stack.layers[2].z_height, 1.0);
    assert_eq!(stack.layers[2].content_path, "/3D/layer_c.png");
}

// ============================================================================
// VLW-04: Refs roundtrip
// ============================================================================

/// Verify 2 volumetricrefs (no layers) roundtrip with correct stack_id and path.
#[test]
fn test_volumetric_refs_roundtrip() {
    let mut model = Model::default();

    let stack = VolumetricStack {
        id: ResourceId(10),
        version: String::new(),
        layers: vec![],
        refs: vec![
            VolumetricRef {
                stack_id: ResourceId(10),
                path: "/3D/ref1.model".to_string(),
            },
            VolumetricRef {
                stack_id: ResourceId(20),
                path: "/3D/ref2.model".to_string(),
            },
        ],
    };
    model.resources.add_volumetric_stack(stack).unwrap();

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
            geometry: Geometry::VolumetricStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    let parsed = roundtrip(&model);
    let stack = parsed
        .resources
        .get_volumetric_stack(ResourceId(10))
        .unwrap();

    assert_eq!(stack.layers.len(), 0);
    assert_eq!(stack.refs.len(), 2);

    assert_eq!(stack.refs[0].stack_id, ResourceId(10));
    assert_eq!(stack.refs[0].path, "/3D/ref1.model");

    assert_eq!(stack.refs[1].stack_id, ResourceId(20));
    assert_eq!(stack.refs[1].path, "/3D/ref2.model");
}

// ============================================================================
// VLW-05: volumetricstackid on object element in XML
// ============================================================================

/// Verify the volumetricstackid attribute appears on object elements in written XML.
#[test]
fn test_volumetricstackid_in_xml() {
    let mut model = Model::default();

    let stack = VolumetricStack {
        id: ResourceId(10),
        version: String::new(),
        layers: vec![VolumetricLayer {
            z_height: 0.0,
            content_path: "/3D/layer0.png".to_string(),
        }],
        refs: vec![],
    };
    model.resources.add_volumetric_stack(stack).unwrap();

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
            geometry: Geometry::VolumetricStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();
    let xml_str = String::from_utf8(buffer).unwrap();

    assert!(
        xml_str.contains(r#"volumetricstackid="10""#),
        "Expected volumetricstackid=\"10\" in output XML.\nGot XML:\n{}",
        &xml_str[..xml_str.len().min(800)]
    );
}

// ============================================================================
// Edge case: Empty volumetric stack roundtrip
// ============================================================================

/// Verify a VolumetricStack with zero layers and zero refs roundtrips correctly.
#[test]
fn test_volumetric_empty_stack_roundtrip() {
    let mut model = Model::default();

    let stack = VolumetricStack {
        id: ResourceId(10),
        version: String::new(),
        layers: vec![],
        refs: vec![],
    };
    model.resources.add_volumetric_stack(stack).unwrap();

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
            geometry: Geometry::VolumetricStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    let parsed = roundtrip(&model);
    let stack = parsed
        .resources
        .get_volumetric_stack(ResourceId(10))
        .expect("Empty VolumetricStack should exist after roundtrip");

    assert_eq!(stack.id, ResourceId(10));
    assert_eq!(stack.layers.len(), 0);
    assert_eq!(stack.refs.len(), 0);
}

// ============================================================================
// Edge case: Multiple volumetric stacks roundtrip
// ============================================================================

/// Verify 2 volumetric stacks and 2 objects referencing them roundtrip correctly.
#[test]
fn test_volumetric_multiple_stacks_roundtrip() {
    let mut model = Model::default();

    // Stack 10
    let stack10 = VolumetricStack {
        id: ResourceId(10),
        version: String::new(),
        layers: vec![VolumetricLayer {
            z_height: 0.0,
            content_path: "/3D/stack10_layer0.png".to_string(),
        }],
        refs: vec![],
    };
    model.resources.add_volumetric_stack(stack10).unwrap();

    // Stack 20
    let stack20 = VolumetricStack {
        id: ResourceId(20),
        version: String::new(),
        layers: vec![VolumetricLayer {
            z_height: 1.0,
            content_path: "/3D/stack20_layer0.png".to_string(),
        }],
        refs: vec![],
    };
    model.resources.add_volumetric_stack(stack20).unwrap();

    // Object 1 -> stack 10
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
            geometry: Geometry::VolumetricStack(ResourceId(10)),
        })
        .unwrap();

    // Object 2 -> stack 20
    model
        .resources
        .add_object(Object {
            id: ResourceId(2),
            object_type: ObjectType::Model,
            name: None,
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::VolumetricStack(ResourceId(20)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));
    model.build.items.push(build_item(ResourceId(2)));

    let parsed = roundtrip(&model);

    // Both stacks exist
    let s10 = parsed
        .resources
        .get_volumetric_stack(ResourceId(10))
        .expect("Stack 10 should exist");
    let s20 = parsed
        .resources
        .get_volumetric_stack(ResourceId(20))
        .expect("Stack 20 should exist");

    assert_eq!(s10.layers.len(), 1);
    assert_eq!(s10.layers[0].z_height, 0.0);
    assert_eq!(s20.layers.len(), 1);
    assert_eq!(s20.layers[0].z_height, 1.0);

    // Object geometry references are correct
    let obj1 = parsed.resources.get_object(ResourceId(1)).unwrap();
    match &obj1.geometry {
        Geometry::VolumetricStack(vsid) => assert_eq!(*vsid, ResourceId(10)),
        other => panic!("Object 1: expected VolumetricStack, got {:?}", other),
    }

    let obj2 = parsed.resources.get_object(ResourceId(2)).unwrap();
    match &obj2.geometry {
        Geometry::VolumetricStack(vsid) => assert_eq!(*vsid, ResourceId(20)),
        other => panic!("Object 2: expected VolumetricStack, got {:?}", other),
    }
}

// ============================================================================
// Edge case: Content path passthrough (write-through without validation)
// ============================================================================

/// Verify paths with special characters or unusual formats are preserved exactly.
/// This validates the write-through behavior per locked decision.
#[test]
fn test_volumetric_path_passthrough() {
    let mut model = Model::default();

    let stack = VolumetricStack {
        id: ResourceId(10),
        version: String::new(),
        layers: vec![
            VolumetricLayer {
                z_height: 0.0,
                content_path: "/OPC/Deep/Nested/path.dat".to_string(),
            },
            VolumetricLayer {
                z_height: 0.1,
                content_path: "/3D/layer with spaces.png".to_string(),
            },
        ],
        refs: vec![],
    };
    model.resources.add_volumetric_stack(stack).unwrap();

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
            geometry: Geometry::VolumetricStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    let parsed = roundtrip(&model);
    let stack = parsed
        .resources
        .get_volumetric_stack(ResourceId(10))
        .unwrap();

    assert_eq!(stack.layers.len(), 2);
    assert_eq!(stack.layers[0].content_path, "/OPC/Deep/Nested/path.dat");
    assert_eq!(stack.layers[1].content_path, "/3D/layer with spaces.png");
}
