use lib3mf_core::model::{Geometry, ResourceId};
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

/// Test basic slice extension parsing with single polygon
#[test]
fn test_parse_slice_extension_basic() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
    <resources>
        <slicestack id="10" zbottom="1.0">
            <slice ztop="2.0">
                <vertices>
                   <vertex x="0" y="0" />
                   <vertex x="10" y="0" />
                   <vertex x="10" y="10" />
                   <vertex x="0" y="10" />
                </vertices>
                <polygon start="0">
                   <segment v2="1" />
                   <segment v2="2" />
                   <segment v2="3" />
                   <segment v2="0" />
                </polygon>
            </slice>
            <sliceref slicestackid="5" slicepath="/3D/other.model" />
        </slicestack>
        <object id="1" type="model" slicestackid="10" />
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;

    // Check SliceStack
    let stack = model
        .resources
        .get_slice_stack(ResourceId(10))
        .expect("Slice stack 10 missing");
    assert_eq!(stack.z_bottom, 1.0);
    assert_eq!(stack.slices.len(), 1);
    assert_eq!(stack.refs.len(), 1);

    // Check Slice content
    let slice = &stack.slices[0];
    assert_eq!(slice.z_top, 2.0);
    assert_eq!(slice.vertices.len(), 4);
    assert_eq!(slice.polygons.len(), 1);

    let poly = &slice.polygons[0];
    assert_eq!(poly.start_segment, 0);
    assert_eq!(poly.segments.len(), 4);
    assert_eq!(poly.segments[0].v2, 1);

    // Check Refs
    let r = &stack.refs[0];
    assert_eq!(r.slice_stack_id, ResourceId(5));
    assert_eq!(r.slice_path, "/3D/other.model");

    // Check Object
    let obj = model
        .resources
        .get_object(ResourceId(1))
        .expect("Object 1 missing");
    if let Geometry::SliceStack(ssid) = obj.geometry {
        assert_eq!(ssid, ResourceId(10));
    } else {
        panic!("Object geometry mismatch");
    }

    Ok(())
}

/// Test multiple polygons per slice
#[test]
fn test_slice_multiple_polygons() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
    <resources>
        <slicestack id="10" zbottom="0.0">
            <slice ztop="0.2">
                <vertices>
                   <vertex x="0" y="0" />
                   <vertex x="5" y="0" />
                   <vertex x="5" y="5" />
                   <vertex x="0" y="5" />
                   <vertex x="10" y="10" />
                   <vertex x="15" y="10" />
                   <vertex x="15" y="15" />
                   <vertex x="10" y="15" />
                </vertices>
                <polygon start="0">
                   <segment v2="1" />
                   <segment v2="2" />
                   <segment v2="3" />
                   <segment v2="0" />
                </polygon>
                <polygon start="4">
                   <segment v2="5" />
                   <segment v2="6" />
                   <segment v2="7" />
                   <segment v2="4" />
                </polygon>
            </slice>
        </slicestack>
        <object id="1" type="model" slicestackid="10" />
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let stack = model.resources.get_slice_stack(ResourceId(10)).unwrap();

    assert_eq!(stack.slices.len(), 1);
    let slice = &stack.slices[0];
    assert_eq!(slice.vertices.len(), 8);
    assert_eq!(slice.polygons.len(), 2);

    // First polygon
    assert_eq!(slice.polygons[0].start_segment, 0);
    assert_eq!(slice.polygons[0].segments.len(), 4);

    // Second polygon
    assert_eq!(slice.polygons[1].start_segment, 4);
    assert_eq!(slice.polygons[1].segments.len(), 4);

    Ok(())
}

/// Test segment property interpolation (pid, p1, p2)
#[test]
fn test_slice_segment_property_interpolation() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
    <resources>
        <slicestack id="10" zbottom="0.0">
            <slice ztop="0.1">
                <vertices>
                   <vertex x="0" y="0" />
                   <vertex x="10" y="0" />
                   <vertex x="5" y="10" />
                </vertices>
                <polygon start="0">
                   <segment v2="1" pid="5" />
                   <segment v2="2" p1="10" p2="15" />
                   <segment v2="0" />
                </polygon>
            </slice>
        </slicestack>
        <object id="1" type="model" slicestackid="10" />
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let stack = model.resources.get_slice_stack(ResourceId(10)).unwrap();
    let poly = &stack.slices[0].polygons[0];

    // Segment 0: has pid property resource reference
    assert_eq!(poly.segments[0].v2, 1);
    assert_eq!(poly.segments[0].pid, Some(ResourceId(5)));
    assert_eq!(poly.segments[0].p1, None);
    assert_eq!(poly.segments[0].p2, None);

    // Segment 1: has p1, p2 property indices
    assert_eq!(poly.segments[1].v2, 2);
    assert_eq!(poly.segments[1].pid, None);
    assert_eq!(poly.segments[1].p1, Some(10));
    assert_eq!(poly.segments[1].p2, Some(15));

    // Segment 2: no properties
    assert_eq!(poly.segments[2].v2, 0);
    assert_eq!(poly.segments[2].pid, None);
    assert_eq!(poly.segments[2].p1, None);
    assert_eq!(poly.segments[2].p2, None);

    Ok(())
}

/// Test multiple slices with increasing z values
#[test]
fn test_slice_multiple_slices_with_z_progression() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
    <resources>
        <slicestack id="10" zbottom="0.0">
            <slice ztop="0.2">
                <vertices>
                   <vertex x="0" y="0" />
                   <vertex x="10" y="0" />
                   <vertex x="10" y="10" />
                   <vertex x="0" y="10" />
                </vertices>
                <polygon start="0">
                   <segment v2="1" />
                   <segment v2="2" />
                   <segment v2="3" />
                   <segment v2="0" />
                </polygon>
            </slice>
            <slice ztop="0.4">
                <vertices>
                   <vertex x="1" y="1" />
                   <vertex x="9" y="1" />
                   <vertex x="9" y="9" />
                   <vertex x="1" y="9" />
                </vertices>
                <polygon start="0">
                   <segment v2="1" />
                   <segment v2="2" />
                   <segment v2="3" />
                   <segment v2="0" />
                </polygon>
            </slice>
            <slice ztop="0.6">
                <vertices>
                   <vertex x="2" y="2" />
                   <vertex x="8" y="2" />
                   <vertex x="8" y="8" />
                   <vertex x="2" y="8" />
                </vertices>
                <polygon start="0">
                   <segment v2="1" />
                   <segment v2="2" />
                   <segment v2="3" />
                   <segment v2="0" />
                </polygon>
            </slice>
        </slicestack>
        <object id="1" type="model" slicestackid="10" />
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let stack = model.resources.get_slice_stack(ResourceId(10)).unwrap();

    assert_eq!(stack.z_bottom, 0.0);
    assert_eq!(stack.slices.len(), 3);

    // Verify z-top progression
    assert_eq!(stack.slices[0].z_top, 0.2);
    assert_eq!(stack.slices[1].z_top, 0.4);
    assert_eq!(stack.slices[2].z_top, 0.6);

    // Verify each slice has vertices and polygon
    for slice in &stack.slices {
        assert_eq!(slice.vertices.len(), 4);
        assert_eq!(slice.polygons.len(), 1);
        assert_eq!(slice.polygons[0].segments.len(), 4);
    }

    Ok(())
}

/// Test external slice references (sliceref elements)
#[test]
fn test_slice_external_references() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
    <resources>
        <slicestack id="100" zbottom="0.0">
            <slice ztop="0.1">
                <vertices>
                   <vertex x="0" y="0" />
                   <vertex x="1" y="0" />
                </vertices>
                <polygon start="0">
                   <segment v2="1" />
                   <segment v2="0" />
                </polygon>
            </slice>
            <sliceref slicestackid="10" slicepath="/3D/layer1.model" />
            <sliceref slicestackid="20" slicepath="/OPC/Parts/layer2.model" />
            <sliceref slicestackid="30" slicepath="relative/layer3.model" />
        </slicestack>
        <object id="1" type="model" slicestackid="100" />
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let stack = model.resources.get_slice_stack(ResourceId(100)).unwrap();

    assert_eq!(stack.slices.len(), 1);
    assert_eq!(stack.refs.len(), 3);

    // Ref 0: absolute path
    assert_eq!(stack.refs[0].slice_stack_id, ResourceId(10));
    assert_eq!(stack.refs[0].slice_path, "/3D/layer1.model");

    // Ref 1: OPC path
    assert_eq!(stack.refs[1].slice_stack_id, ResourceId(20));
    assert_eq!(stack.refs[1].slice_path, "/OPC/Parts/layer2.model");

    // Ref 2: relative path
    assert_eq!(stack.refs[2].slice_stack_id, ResourceId(30));
    assert_eq!(stack.refs[2].slice_path, "relative/layer3.model");

    Ok(())
}

/// Test empty slice stack (valid edge case)
#[test]
fn test_slice_empty_stack() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
    <resources>
        <slicestack id="10" zbottom="0.5" />
        <object id="1" type="model" slicestackid="10" />
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let stack = model.resources.get_slice_stack(ResourceId(10)).unwrap();

    assert_eq!(stack.z_bottom, 0.5);
    assert_eq!(stack.slices.len(), 0);
    assert_eq!(stack.refs.len(), 0);

    Ok(())
}

/// Test mixed slices and refs in one stack
#[test]
fn test_slice_mixed_slices_and_refs() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
    <resources>
        <slicestack id="100" zbottom="0.0">
            <slice ztop="0.2">
                <vertices>
                   <vertex x="0" y="0" />
                   <vertex x="5" y="0" />
                </vertices>
                <polygon start="0">
                   <segment v2="1" />
                   <segment v2="0" />
                </polygon>
            </slice>
            <sliceref slicestackid="10" slicepath="/ext1.model" />
            <slice ztop="0.6">
                <vertices>
                   <vertex x="1" y="1" />
                   <vertex x="4" y="1" />
                </vertices>
                <polygon start="0">
                   <segment v2="1" />
                   <segment v2="0" />
                </polygon>
            </slice>
            <sliceref slicestackid="20" slicepath="/ext2.model" />
        </slicestack>
        <object id="1" type="model" slicestackid="100" />
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let stack = model.resources.get_slice_stack(ResourceId(100)).unwrap();

    // Verify mixed content
    assert_eq!(stack.slices.len(), 2);
    assert_eq!(stack.refs.len(), 2);

    // Verify slices
    assert_eq!(stack.slices[0].z_top, 0.2);
    assert_eq!(stack.slices[1].z_top, 0.6);

    // Verify refs
    assert_eq!(stack.refs[0].slice_stack_id, ResourceId(10));
    assert_eq!(stack.refs[1].slice_stack_id, ResourceId(20));

    Ok(())
}

/// Test polygon with different start indices
#[test]
fn test_slice_polygon_start_indices() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
    <resources>
        <slicestack id="10" zbottom="0.0">
            <slice ztop="0.1">
                <vertices>
                   <vertex x="0" y="0" />
                   <vertex x="5" y="0" />
                   <vertex x="5" y="5" />
                   <vertex x="0" y="5" />
                   <vertex x="10" y="0" />
                   <vertex x="15" y="0" />
                </vertices>
                <polygon start="0">
                   <segment v2="1" />
                   <segment v2="2" />
                   <segment v2="3" />
                   <segment v2="0" />
                </polygon>
                <polygon start="4">
                   <segment v2="5" />
                   <segment v2="4" />
                </polygon>
                <polygon start="2">
                   <segment v2="3" />
                   <segment v2="2" />
                </polygon>
            </slice>
        </slicestack>
        <object id="1" type="model" slicestackid="10" />
    </resources>
    <build><item objectid="1" /></build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let stack = model.resources.get_slice_stack(ResourceId(10)).unwrap();
    let slice = &stack.slices[0];

    assert_eq!(slice.polygons.len(), 3);

    // Polygon 0: starts at vertex 0
    assert_eq!(slice.polygons[0].start_segment, 0);
    assert_eq!(slice.polygons[0].segments.len(), 4);

    // Polygon 1: starts at vertex 4
    assert_eq!(slice.polygons[1].start_segment, 4);
    assert_eq!(slice.polygons[1].segments.len(), 2);

    // Polygon 2: starts at vertex 2
    assert_eq!(slice.polygons[2].start_segment, 2);
    assert_eq!(slice.polygons[2].segments.len(), 2);

    Ok(())
}

/// Test multiple objects referencing different slice stacks
#[test]
fn test_slice_multiple_objects() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
    <resources>
        <slicestack id="10" zbottom="0.0">
            <slice ztop="0.5">
                <vertices><vertex x="0" y="0" /><vertex x="1" y="0" /></vertices>
                <polygon start="0"><segment v2="1" /><segment v2="0" /></polygon>
            </slice>
        </slicestack>
        <slicestack id="20" zbottom="1.0">
            <slice ztop="1.5">
                <vertices><vertex x="2" y="2" /><vertex x="3" y="2" /></vertices>
                <polygon start="0"><segment v2="1" /><segment v2="0" /></polygon>
            </slice>
        </slicestack>
        <object id="1" type="model" slicestackid="10" />
        <object id="2" type="model" slicestackid="20" />
        <object id="3" type="model" slicestackid="10" />
    </resources>
    <build>
        <item objectid="1" />
        <item objectid="2" />
        <item objectid="3" />
    </build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;

    // Verify slice stacks exist
    let stack10 = model.resources.get_slice_stack(ResourceId(10)).unwrap();
    let stack20 = model.resources.get_slice_stack(ResourceId(20)).unwrap();
    assert_eq!(stack10.z_bottom, 0.0);
    assert_eq!(stack20.z_bottom, 1.0);

    // Verify object 1 references stack 10
    let obj1 = model.resources.get_object(ResourceId(1)).unwrap();
    if let Geometry::SliceStack(ssid) = obj1.geometry {
        assert_eq!(ssid, ResourceId(10));
    } else {
        panic!("Object 1 should reference slice stack");
    }

    // Verify object 2 references stack 20
    let obj2 = model.resources.get_object(ResourceId(2)).unwrap();
    if let Geometry::SliceStack(ssid) = obj2.geometry {
        assert_eq!(ssid, ResourceId(20));
    } else {
        panic!("Object 2 should reference slice stack");
    }

    // Verify object 3 also references stack 10 (shared stack)
    let obj3 = model.resources.get_object(ResourceId(3)).unwrap();
    if let Geometry::SliceStack(ssid) = obj3.geometry {
        assert_eq!(ssid, ResourceId(10));
    } else {
        panic!("Object 3 should reference slice stack");
    }

    Ok(())
}

// ============================================================================
// ERROR PATH TESTS
// ============================================================================

/// Test EOF during slicestack element parsing
#[test]
fn test_slice_truncated_eof_in_slicestack() {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
    <resources>
        <slicestack id="10" zbottom="0.0">
            <slice ztop="0.1">
                <vertices>"##; // Truncated here - EOF in slicestack

    let result = parse_model(Cursor::new(xml));
    assert!(
        result.is_err(),
        "Should fail on truncated XML in slicestack element"
    );
}

/// Test EOF during slice element parsing
#[test]
fn test_slice_truncated_eof_in_slice() {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
    <resources>
        <slicestack id="10" zbottom="0.0">
            <slice ztop="0.1">
                <vertices>
                   <vertex x="0" y="0" />
                   <vertex x="1" y="0" />
                </vertices>
                <polygon start="0">"##; // Truncated here - EOF in slice

    let result = parse_model(Cursor::new(xml));
    assert!(
        result.is_err(),
        "Should fail on truncated XML in slice element"
    );
}

/// Test EOF during vertices element parsing
#[test]
fn test_slice_truncated_eof_in_vertices() {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
    <resources>
        <slicestack id="10" zbottom="0.0">
            <slice ztop="0.1">
                <vertices>
                   <vertex x="0" y="0" />"##; // Truncated here - EOF in vertices

    let result = parse_model(Cursor::new(xml));
    assert!(
        result.is_err(),
        "Should fail on truncated XML in vertices element"
    );
}

/// Test EOF during polygon element parsing
#[test]
fn test_slice_truncated_eof_in_polygon() {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:s="http://schemas.microsoft.com/3dmanufacturing/slice/2015/07">
    <resources>
        <slicestack id="10" zbottom="0.0">
            <slice ztop="0.1">
                <vertices>
                   <vertex x="0" y="0" />
                   <vertex x="1" y="0" />
                </vertices>
                <polygon start="0">
                   <segment v2="1" />"##; // Truncated here - EOF in polygon

    let result = parse_model(Cursor::new(xml));
    assert!(
        result.is_err(),
        "Should fail on truncated XML in polygon element"
    );
}
