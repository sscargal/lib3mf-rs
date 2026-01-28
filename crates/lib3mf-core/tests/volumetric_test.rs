use lib3mf_core::model::{Geometry, ResourceId};
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

#[test]
fn test_parse_volumetric_extension() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <volumetricstack id="10">
            <layer z="1.0" path="/3D/texture1.png" />
            <layer z="2.0" path="/3D/texture2.png" />
            <volumetricref volumetricstackid="5" path="/3D/other.model" />
        </volumetricstack>
        <object id="1" type="model" volumetricstackid="10" />
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;

    // Check VolumetricStack
    let stack = model
        .resources
        .get_volumetric_stack(ResourceId(10))
        .expect("Volumetric stack 10 missing");
    assert_eq!(stack.layers.len(), 2);
    assert_eq!(stack.refs.len(), 1);

    // Check Layer content
    let l1 = &stack.layers[0];
    assert_eq!(l1.z_height, 1.0);
    assert_eq!(l1.content_path, "/3D/texture1.png");

    let l2 = &stack.layers[1];
    assert_eq!(l2.z_height, 2.0);
    assert_eq!(l2.content_path, "/3D/texture2.png");

    // Check Refs
    let r = &stack.refs[0];
    assert_eq!(r.stack_id, ResourceId(5));
    assert_eq!(r.path, "/3D/other.model");

    // Check Object
    let obj = model
        .resources
        .get_object(ResourceId(1))
        .expect("Object 1 missing");
    if let Geometry::VolumetricStack(ssid) = obj.geometry {
        assert_eq!(ssid, ResourceId(10));
    } else {
        panic!(
            "Object geometry mismatch: expected VolumetricStack, got {:?}",
            obj.geometry
        );
    }

    Ok(())
}
