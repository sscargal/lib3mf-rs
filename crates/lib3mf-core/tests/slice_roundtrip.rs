//! Integration tests for Slice Extension writer (roundtrip fidelity)
//!
//! Covers requirements SLW-01 through SLW-10.

use glam::Mat4;
use lib3mf_core::model::*;
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

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
// Helper
// ============================================================================

/// Round-trip a model through write_xml -> parse_model.
fn roundtrip(model: &Model) -> Model {
    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();
    parse_model(Cursor::new(&buffer)).unwrap()
}

/// Creates a minimal model with a single slice stack and one object referencing it.
/// The slice stack has the given id with one square slice at z_top.
fn create_model_with_square_slice(stack_id: u32, z_bottom: f32, z_top: f32) -> Model {
    let mut model = Model::default();

    let stack = SliceStack {
        id: ResourceId(stack_id),
        z_bottom,
        slices: vec![Slice {
            z_top,
            vertices: vec![
                Vertex2D { x: 0.0, y: 0.0 },
                Vertex2D { x: 10.0, y: 0.0 },
                Vertex2D { x: 10.0, y: 10.0 },
                Vertex2D { x: 0.0, y: 10.0 },
            ],
            polygons: vec![Polygon {
                start_segment: 0,
                segments: vec![
                    Segment { v2: 1, p1: None, p2: None, pid: None },
                    Segment { v2: 2, p1: None, p2: None, pid: None },
                    Segment { v2: 3, p1: None, p2: None, pid: None },
                    Segment { v2: 0, p1: None, p2: None, pid: None },
                ],
            }],
        }],
        refs: vec![],
    };
    model.resources.add_slice_stack(stack).unwrap();

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
            geometry: Geometry::SliceStack(ResourceId(stack_id)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    model
}

// ============================================================================
// SLW-01: Namespace declaration present in output XML
// ============================================================================

/// Verify that the slice namespace URI is declared on the model element.
#[test]
fn test_slice_namespace_present() {
    let model = create_model_with_square_slice(10, 0.0, 0.2);

    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();
    let xml_str = String::from_utf8(buffer).unwrap();

    assert!(
        xml_str.contains(
            r#"xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07""#
        ),
        "Expected slice namespace declaration in output XML.\nGot XML:\n{}",
        &xml_str[..xml_str.len().min(500)]
    );
}

// ============================================================================
// SLW-02 + SLW-03 + SLW-04: Basic slicestack/slice/vertices roundtrip
// ============================================================================

/// Full roundtrip: write model with slice stack, re-parse, verify all fields preserved.
#[test]
fn test_slice_basic_roundtrip() {
    let model = create_model_with_square_slice(10, 0.0, 0.2);
    let parsed = roundtrip(&model);

    // SliceStack preserved
    let stack = parsed
        .resources
        .get_slice_stack(ResourceId(10))
        .expect("Slice stack 10 should exist after roundtrip");
    assert_eq!(stack.id, ResourceId(10));
    assert_eq!(stack.z_bottom, 0.0);
    assert_eq!(stack.slices.len(), 1);
    assert_eq!(stack.refs.len(), 0);

    // Slice preserved
    let slice = &stack.slices[0];
    assert_eq!(slice.z_top, 0.2);
    assert_eq!(slice.vertices.len(), 4);

    // Vertex coordinates (SLW-04)
    assert_eq!(slice.vertices[0].x, 0.0);
    assert_eq!(slice.vertices[0].y, 0.0);
    assert_eq!(slice.vertices[1].x, 10.0);
    assert_eq!(slice.vertices[1].y, 0.0);
    assert_eq!(slice.vertices[2].x, 10.0);
    assert_eq!(slice.vertices[2].y, 10.0);
    assert_eq!(slice.vertices[3].x, 0.0);
    assert_eq!(slice.vertices[3].y, 10.0);

    // Polygon preserved
    assert_eq!(slice.polygons.len(), 1);
    let poly = &slice.polygons[0];
    assert_eq!(poly.start_segment, 0);
    assert_eq!(poly.segments.len(), 4);
    assert_eq!(poly.segments[0].v2, 1);
    assert_eq!(poly.segments[1].v2, 2);
    assert_eq!(poly.segments[2].v2, 3);
    assert_eq!(poly.segments[3].v2, 0);

    // Object geometry reference preserved
    let obj = parsed
        .resources
        .get_object(ResourceId(1))
        .expect("Object 1 should exist");
    match &obj.geometry {
        Geometry::SliceStack(ssid) => assert_eq!(*ssid, ResourceId(10)),
        other => panic!("Expected Geometry::SliceStack, got {:?}", other),
    }
}

// ============================================================================
// SLW-05 + SLW-06: Segment property attributes roundtrip
// ============================================================================

/// Verify that optional p1, p2, pid attributes on segments survive roundtrip.
#[test]
fn test_slice_segment_properties_roundtrip() {
    let mut model = Model::default();

    let stack = SliceStack {
        id: ResourceId(10),
        z_bottom: 0.0,
        slices: vec![Slice {
            z_top: 0.1,
            vertices: vec![
                Vertex2D { x: 0.0, y: 0.0 },
                Vertex2D { x: 10.0, y: 0.0 },
                Vertex2D { x: 5.0, y: 10.0 },
            ],
            polygons: vec![Polygon {
                start_segment: 0,
                segments: vec![
                    // Segment with pid only
                    Segment { v2: 1, pid: Some(ResourceId(5)), p1: None, p2: None },
                    // Segment with p1 and p2, no pid
                    Segment { v2: 2, pid: None, p1: Some(10), p2: Some(15) },
                    // Segment with no properties
                    Segment { v2: 0, pid: None, p1: None, p2: None },
                ],
            }],
        }],
        refs: vec![],
    };
    model.resources.add_slice_stack(stack).unwrap();

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
            geometry: Geometry::SliceStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    let parsed = roundtrip(&model);
    let stack = parsed.resources.get_slice_stack(ResourceId(10)).unwrap();
    let poly = &stack.slices[0].polygons[0];

    // Segment 0: has pid
    assert_eq!(poly.segments[0].v2, 1);
    assert_eq!(poly.segments[0].pid, Some(ResourceId(5)));
    assert_eq!(poly.segments[0].p1, None);
    assert_eq!(poly.segments[0].p2, None);

    // Segment 1: has p1, p2
    assert_eq!(poly.segments[1].v2, 2);
    assert_eq!(poly.segments[1].pid, None);
    assert_eq!(poly.segments[1].p1, Some(10));
    assert_eq!(poly.segments[1].p2, Some(15));

    // Segment 2: no properties
    assert_eq!(poly.segments[2].v2, 0);
    assert_eq!(poly.segments[2].pid, None);
    assert_eq!(poly.segments[2].p1, None);
    assert_eq!(poly.segments[2].p2, None);
}

// ============================================================================
// SLW-07: External slice references (sliceref) roundtrip
// ============================================================================

/// Verify sliceref elements roundtrip with correct paths and IDs.
#[test]
fn test_slice_refs_roundtrip() {
    let mut model = Model::default();

    let stack = SliceStack {
        id: ResourceId(10),
        z_bottom: 0.0,
        slices: vec![],
        refs: vec![
            SliceRef {
                slice_stack_id: ResourceId(10),
                slice_path: "/3D/layer1.model".to_string(),
            },
            SliceRef {
                slice_stack_id: ResourceId(20),
                slice_path: "/OPC/Parts/layer2.model".to_string(),
            },
        ],
    };
    model.resources.add_slice_stack(stack).unwrap();

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
            geometry: Geometry::SliceStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    let parsed = roundtrip(&model);
    let stack = parsed.resources.get_slice_stack(ResourceId(10)).unwrap();

    assert_eq!(stack.slices.len(), 0);
    assert_eq!(stack.refs.len(), 2);

    assert_eq!(stack.refs[0].slice_stack_id, ResourceId(10));
    assert_eq!(stack.refs[0].slice_path, "/3D/layer1.model");

    assert_eq!(stack.refs[1].slice_stack_id, ResourceId(20));
    assert_eq!(stack.refs[1].slice_path, "/OPC/Parts/layer2.model");
}

// ============================================================================
// SLW-08: slicestackid on object element in XML
// ============================================================================

/// Verify the slicestackid attribute appears on object elements in written XML.
#[test]
fn test_slicestackid_in_xml() {
    let model = create_model_with_square_slice(10, 0.0, 0.2);

    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None).unwrap();
    let xml_str = String::from_utf8(buffer).unwrap();

    assert!(
        xml_str.contains(r#"slicestackid="10""#),
        "Expected slicestackid=\"10\" in output XML.\nGot XML:\n{}",
        &xml_str[..xml_str.len().min(800)]
    );
}

// ============================================================================
// SLW-10: Multiple slices with z-progression roundtrip
// ============================================================================

/// Verify 3 slices at different z_top values roundtrip with correct data.
#[test]
fn test_slice_multiple_slices_roundtrip() {
    let mut model = Model::default();

    let stack = SliceStack {
        id: ResourceId(10),
        z_bottom: 0.0,
        slices: vec![
            Slice {
                z_top: 0.2,
                vertices: vec![
                    Vertex2D { x: 0.0, y: 0.0 },
                    Vertex2D { x: 10.0, y: 0.0 },
                    Vertex2D { x: 10.0, y: 10.0 },
                    Vertex2D { x: 0.0, y: 10.0 },
                ],
                polygons: vec![Polygon {
                    start_segment: 0,
                    segments: vec![
                        Segment { v2: 1, p1: None, p2: None, pid: None },
                        Segment { v2: 2, p1: None, p2: None, pid: None },
                        Segment { v2: 3, p1: None, p2: None, pid: None },
                        Segment { v2: 0, p1: None, p2: None, pid: None },
                    ],
                }],
            },
            Slice {
                z_top: 0.4,
                vertices: vec![
                    Vertex2D { x: 1.0, y: 1.0 },
                    Vertex2D { x: 9.0, y: 1.0 },
                    Vertex2D { x: 9.0, y: 9.0 },
                    Vertex2D { x: 1.0, y: 9.0 },
                ],
                polygons: vec![Polygon {
                    start_segment: 0,
                    segments: vec![
                        Segment { v2: 1, p1: None, p2: None, pid: None },
                        Segment { v2: 2, p1: None, p2: None, pid: None },
                        Segment { v2: 3, p1: None, p2: None, pid: None },
                        Segment { v2: 0, p1: None, p2: None, pid: None },
                    ],
                }],
            },
            Slice {
                z_top: 0.6,
                vertices: vec![
                    Vertex2D { x: 2.0, y: 2.0 },
                    Vertex2D { x: 8.0, y: 2.0 },
                    Vertex2D { x: 8.0, y: 8.0 },
                    Vertex2D { x: 2.0, y: 8.0 },
                ],
                polygons: vec![Polygon {
                    start_segment: 0,
                    segments: vec![
                        Segment { v2: 1, p1: None, p2: None, pid: None },
                        Segment { v2: 2, p1: None, p2: None, pid: None },
                        Segment { v2: 3, p1: None, p2: None, pid: None },
                        Segment { v2: 0, p1: None, p2: None, pid: None },
                    ],
                }],
            },
        ],
        refs: vec![],
    };
    model.resources.add_slice_stack(stack).unwrap();

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
            geometry: Geometry::SliceStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    let parsed = roundtrip(&model);
    let stack = parsed.resources.get_slice_stack(ResourceId(10)).unwrap();

    assert_eq!(stack.slices.len(), 3);
    assert_eq!(stack.slices[0].z_top, 0.2);
    assert_eq!(stack.slices[1].z_top, 0.4);
    assert_eq!(stack.slices[2].z_top, 0.6);

    // Verify vertex coordinates differ per slice
    assert_eq!(stack.slices[0].vertices[0].x, 0.0);
    assert_eq!(stack.slices[1].vertices[0].x, 1.0);
    assert_eq!(stack.slices[2].vertices[0].x, 2.0);

    // Verify each slice has 4 vertices and 1 polygon with 4 segments
    for slice in &stack.slices {
        assert_eq!(slice.vertices.len(), 4);
        assert_eq!(slice.polygons.len(), 1);
        assert_eq!(slice.polygons[0].segments.len(), 4);
    }
}

// ============================================================================
// SLW-10: Multiple objects referencing different slice stacks
// ============================================================================

/// Verify 2 slice stacks and 3 objects referencing them roundtrip correctly.
#[test]
fn test_slice_multiple_objects_roundtrip() {
    let mut model = Model::default();

    // Slice stack 10
    let stack10 = SliceStack {
        id: ResourceId(10),
        z_bottom: 0.0,
        slices: vec![Slice {
            z_top: 0.5,
            vertices: vec![
                Vertex2D { x: 0.0, y: 0.0 },
                Vertex2D { x: 1.0, y: 0.0 },
            ],
            polygons: vec![Polygon {
                start_segment: 0,
                segments: vec![
                    Segment { v2: 1, p1: None, p2: None, pid: None },
                    Segment { v2: 0, p1: None, p2: None, pid: None },
                ],
            }],
        }],
        refs: vec![],
    };
    model.resources.add_slice_stack(stack10).unwrap();

    // Slice stack 20
    let stack20 = SliceStack {
        id: ResourceId(20),
        z_bottom: 1.0,
        slices: vec![Slice {
            z_top: 1.5,
            vertices: vec![
                Vertex2D { x: 2.0, y: 2.0 },
                Vertex2D { x: 3.0, y: 2.0 },
            ],
            polygons: vec![Polygon {
                start_segment: 0,
                segments: vec![
                    Segment { v2: 1, p1: None, p2: None, pid: None },
                    Segment { v2: 0, p1: None, p2: None, pid: None },
                ],
            }],
        }],
        refs: vec![],
    };
    model.resources.add_slice_stack(stack20).unwrap();

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
            geometry: Geometry::SliceStack(ResourceId(10)),
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
            geometry: Geometry::SliceStack(ResourceId(20)),
        })
        .unwrap();

    // Object 3 -> stack 10 (shared)
    model
        .resources
        .add_object(Object {
            id: ResourceId(3),
            object_type: ObjectType::Model,
            name: None,
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::SliceStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));
    model.build.items.push(build_item(ResourceId(2)));
    model.build.items.push(build_item(ResourceId(3)));

    let parsed = roundtrip(&model);

    // Both slice stacks exist
    let s10 = parsed.resources.get_slice_stack(ResourceId(10)).unwrap();
    let s20 = parsed.resources.get_slice_stack(ResourceId(20)).unwrap();
    assert_eq!(s10.z_bottom, 0.0);
    assert_eq!(s20.z_bottom, 1.0);

    // Object references correct
    let obj1 = parsed.resources.get_object(ResourceId(1)).unwrap();
    match &obj1.geometry {
        Geometry::SliceStack(ssid) => assert_eq!(*ssid, ResourceId(10)),
        other => panic!("Object 1: expected SliceStack, got {:?}", other),
    }

    let obj2 = parsed.resources.get_object(ResourceId(2)).unwrap();
    match &obj2.geometry {
        Geometry::SliceStack(ssid) => assert_eq!(*ssid, ResourceId(20)),
        other => panic!("Object 2: expected SliceStack, got {:?}", other),
    }

    let obj3 = parsed.resources.get_object(ResourceId(3)).unwrap();
    match &obj3.geometry {
        Geometry::SliceStack(ssid) => assert_eq!(*ssid, ResourceId(10)),
        other => panic!("Object 3: expected SliceStack, got {:?}", other),
    }
}

// ============================================================================
// SLW-10: Mixed slices and refs in one stack
// ============================================================================

/// Verify a slice stack with both inline slices and sliceref entries roundtrips.
#[test]
fn test_slice_mixed_slices_and_refs_roundtrip() {
    let mut model = Model::default();

    let stack = SliceStack {
        id: ResourceId(10),
        z_bottom: 0.0,
        slices: vec![
            Slice {
                z_top: 0.2,
                vertices: vec![
                    Vertex2D { x: 0.0, y: 0.0 },
                    Vertex2D { x: 5.0, y: 0.0 },
                ],
                polygons: vec![Polygon {
                    start_segment: 0,
                    segments: vec![
                        Segment { v2: 1, p1: None, p2: None, pid: None },
                        Segment { v2: 0, p1: None, p2: None, pid: None },
                    ],
                }],
            },
            Slice {
                z_top: 0.6,
                vertices: vec![
                    Vertex2D { x: 1.0, y: 1.0 },
                    Vertex2D { x: 4.0, y: 1.0 },
                ],
                polygons: vec![Polygon {
                    start_segment: 0,
                    segments: vec![
                        Segment { v2: 1, p1: None, p2: None, pid: None },
                        Segment { v2: 0, p1: None, p2: None, pid: None },
                    ],
                }],
            },
        ],
        refs: vec![
            SliceRef {
                slice_stack_id: ResourceId(10),
                slice_path: "/ext1.model".to_string(),
            },
            SliceRef {
                slice_stack_id: ResourceId(20),
                slice_path: "/ext2.model".to_string(),
            },
        ],
    };
    model.resources.add_slice_stack(stack).unwrap();

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
            geometry: Geometry::SliceStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    let parsed = roundtrip(&model);
    let stack = parsed.resources.get_slice_stack(ResourceId(10)).unwrap();

    // Both slices and refs preserved
    assert_eq!(stack.slices.len(), 2);
    assert_eq!(stack.refs.len(), 2);

    assert_eq!(stack.slices[0].z_top, 0.2);
    assert_eq!(stack.slices[1].z_top, 0.6);

    assert_eq!(stack.refs[0].slice_stack_id, ResourceId(10));
    assert_eq!(stack.refs[0].slice_path, "/ext1.model");
    assert_eq!(stack.refs[1].slice_stack_id, ResourceId(20));
    assert_eq!(stack.refs[1].slice_path, "/ext2.model");
}

// ============================================================================
// Edge case: Empty slice stack roundtrip
// ============================================================================

/// Verify an empty slice stack (zero slices, zero refs) roundtrips with z_bottom preserved.
#[test]
fn test_slice_empty_stack_roundtrip() {
    let mut model = Model::default();

    let stack = SliceStack {
        id: ResourceId(10),
        z_bottom: 0.5,
        slices: vec![],
        refs: vec![],
    };
    model.resources.add_slice_stack(stack).unwrap();

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
            geometry: Geometry::SliceStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    let parsed = roundtrip(&model);
    let stack = parsed.resources.get_slice_stack(ResourceId(10)).unwrap();

    assert_eq!(stack.z_bottom, 0.5);
    assert_eq!(stack.slices.len(), 0);
    assert_eq!(stack.refs.len(), 0);
}

// ============================================================================
// Edge case: Multiple polygons in a single slice
// ============================================================================

/// Verify a single slice with 2 polygons (different start indices) roundtrips.
#[test]
fn test_slice_multiple_polygons_roundtrip() {
    let mut model = Model::default();

    let stack = SliceStack {
        id: ResourceId(10),
        z_bottom: 0.0,
        slices: vec![Slice {
            z_top: 0.2,
            vertices: vec![
                Vertex2D { x: 0.0, y: 0.0 },
                Vertex2D { x: 5.0, y: 0.0 },
                Vertex2D { x: 5.0, y: 5.0 },
                Vertex2D { x: 0.0, y: 5.0 },
                Vertex2D { x: 10.0, y: 10.0 },
                Vertex2D { x: 15.0, y: 10.0 },
                Vertex2D { x: 15.0, y: 15.0 },
                Vertex2D { x: 10.0, y: 15.0 },
            ],
            polygons: vec![
                Polygon {
                    start_segment: 0,
                    segments: vec![
                        Segment { v2: 1, p1: None, p2: None, pid: None },
                        Segment { v2: 2, p1: None, p2: None, pid: None },
                        Segment { v2: 3, p1: None, p2: None, pid: None },
                        Segment { v2: 0, p1: None, p2: None, pid: None },
                    ],
                },
                Polygon {
                    start_segment: 4,
                    segments: vec![
                        Segment { v2: 5, p1: None, p2: None, pid: None },
                        Segment { v2: 6, p1: None, p2: None, pid: None },
                        Segment { v2: 7, p1: None, p2: None, pid: None },
                        Segment { v2: 4, p1: None, p2: None, pid: None },
                    ],
                },
            ],
        }],
        refs: vec![],
    };
    model.resources.add_slice_stack(stack).unwrap();

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
            geometry: Geometry::SliceStack(ResourceId(10)),
        })
        .unwrap();

    model.build.items.push(build_item(ResourceId(1)));

    let parsed = roundtrip(&model);
    let stack = parsed.resources.get_slice_stack(ResourceId(10)).unwrap();
    let slice = &stack.slices[0];

    assert_eq!(slice.vertices.len(), 8);
    assert_eq!(slice.polygons.len(), 2);

    // First polygon
    assert_eq!(slice.polygons[0].start_segment, 0);
    assert_eq!(slice.polygons[0].segments.len(), 4);
    assert_eq!(slice.polygons[0].segments[0].v2, 1);

    // Second polygon
    assert_eq!(slice.polygons[1].start_segment, 4);
    assert_eq!(slice.polygons[1].segments.len(), 4);
    assert_eq!(slice.polygons[1].segments[0].v2, 5);
}

// ============================================================================
// Edge case: Non-zero zbottom
// ============================================================================

/// Verify a SliceStack with zbottom=1.5 (non-zero) roundtrips with exact value.
#[test]
fn test_slice_zbottom_nonzero_roundtrip() {
    let model = create_model_with_square_slice(10, 1.5, 2.0);
    let parsed = roundtrip(&model);

    let stack = parsed.resources.get_slice_stack(ResourceId(10)).unwrap();
    assert_eq!(stack.z_bottom, 1.5);
    assert_eq!(stack.slices[0].z_top, 2.0);
}
