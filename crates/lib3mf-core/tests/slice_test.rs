use lib3mf_core::model::{Geometry, ResourceId};
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

#[test]
fn test_parse_slice_extension() -> anyhow::Result<()> {
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
