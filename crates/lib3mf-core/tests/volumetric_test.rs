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

#[test]
fn test_volumetric_layer_ordering() -> anyhow::Result<()> {
    // Test that multiple layers with increasing z-heights are parsed in declaration order
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <volumetricstack id="20">
            <layer z="0.0" path="/3D/layer0.png" />
            <layer z="0.2" path="/3D/layer1.png" />
            <layer z="0.4" path="/3D/layer2.png" />
            <layer z="0.6" path="/3D/layer3.png" />
            <layer z="0.8" path="/3D/layer4.png" />
            <layer z="1.0" path="/3D/layer5.png" />
        </volumetricstack>
        <object id="1" type="model" volumetricstackid="20" />
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let stack = model
        .resources
        .get_volumetric_stack(ResourceId(20))
        .expect("Volumetric stack 20 missing");

    assert_eq!(
        stack.layers.len(),
        6,
        "Stack should contain exactly 6 layers"
    );

    // Verify z-height values in order
    let expected_z_heights = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
    for (i, (layer, expected_z)) in stack
        .layers
        .iter()
        .zip(expected_z_heights.iter())
        .enumerate()
    {
        assert_eq!(
            layer.z_height, *expected_z,
            "Layer {} z_height mismatch: expected {}, got {}",
            i, expected_z, layer.z_height
        );
        assert_eq!(
            layer.content_path,
            format!("/3D/layer{}.png", i),
            "Layer {} path mismatch",
            i
        );
    }

    Ok(())
}

#[test]
fn test_volumetric_multiple_refs() -> anyhow::Result<()> {
    // Test multiple external references in a single stack
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <volumetricstack id="30">
            <layer z="0.0" path="/3D/base.png" />
            <volumetricref volumetricstackid="31" path="/3D/stack1.model" />
            <volumetricref volumetricstackid="32" path="/3D/stack2.model" />
            <layer z="1.0" path="/3D/top.png" />
            <volumetricref volumetricstackid="33" path="/3D/stack3.model" />
        </volumetricstack>
        <object id="1" type="model" volumetricstackid="30" />
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let stack = model
        .resources
        .get_volumetric_stack(ResourceId(30))
        .expect("Volumetric stack 30 missing");

    // Verify mixed layers and refs
    assert_eq!(
        stack.layers.len(),
        2,
        "Stack should contain 2 layers (base and top)"
    );
    assert_eq!(
        stack.refs.len(),
        3,
        "Stack should contain 3 external references"
    );

    // Check refs
    let expected_refs = [
        (31, "/3D/stack1.model"),
        (32, "/3D/stack2.model"),
        (33, "/3D/stack3.model"),
    ];
    for (i, (ref_item, (expected_id, expected_path))) in
        stack.refs.iter().zip(expected_refs.iter()).enumerate()
    {
        assert_eq!(
            ref_item.stack_id,
            ResourceId(*expected_id),
            "Ref {} stack_id mismatch",
            i
        );
        assert_eq!(ref_item.path, *expected_path, "Ref {} path mismatch", i);
    }

    Ok(())
}

#[test]
fn test_volumetric_layer_paths() -> anyhow::Result<()> {
    // Test various content path formats (absolute, relative, deep paths)
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <volumetricstack id="40">
            <layer z="0.0" path="/3D/texture.png" />
            <layer z="0.5" path="relative_texture.png" />
            <layer z="1.0" path="/Textures/volumetric/layer001.png" />
            <layer z="1.5" path="sub/path/image.jpg" />
        </volumetricstack>
        <object id="1" type="model" volumetricstackid="40" />
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let stack = model
        .resources
        .get_volumetric_stack(ResourceId(40))
        .expect("Volumetric stack 40 missing");

    assert_eq!(stack.layers.len(), 4, "Stack should contain 4 layers");

    // Verify all paths are preserved exactly
    let expected_paths = [
        "/3D/texture.png",
        "relative_texture.png",
        "/Textures/volumetric/layer001.png",
        "sub/path/image.jpg",
    ];
    for (i, (layer, expected_path)) in stack.layers.iter().zip(expected_paths.iter()).enumerate() {
        assert_eq!(
            layer.content_path, *expected_path,
            "Layer {} path mismatch: expected '{}', got '{}'",
            i, expected_path, layer.content_path
        );
    }

    Ok(())
}

#[test]
fn test_volumetric_empty_stack() -> anyhow::Result<()> {
    // Test edge case: stack with no layers and no refs (valid empty stack)
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <volumetricstack id="50" />
        <object id="1" type="model" volumetricstackid="50" />
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let stack = model
        .resources
        .get_volumetric_stack(ResourceId(50))
        .expect("Empty volumetric stack 50 should exist");

    assert_eq!(
        stack.layers.len(),
        0,
        "Empty stack should contain no layers"
    );
    assert_eq!(stack.refs.len(), 0, "Empty stack should contain no refs");

    Ok(())
}

#[test]
fn test_volumetric_missing_layer_z() -> anyhow::Result<()> {
    // Test layer without z attribute - parser should default to 0.0
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <volumetricstack id="60">
            <layer path="/3D/no_z_layer.png" />
            <layer z="1.0" path="/3D/with_z_layer.png" />
        </volumetricstack>
        <object id="1" type="model" volumetricstackid="60" />
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let stack = model
        .resources
        .get_volumetric_stack(ResourceId(60))
        .expect("Volumetric stack 60 missing");

    assert_eq!(stack.layers.len(), 2, "Stack should contain 2 layers");

    // Verify first layer defaults to z=0.0 when z attribute is missing
    assert_eq!(
        stack.layers[0].z_height, 0.0,
        "Layer without z attribute should default to 0.0"
    );
    assert_eq!(
        stack.layers[0].content_path, "/3D/no_z_layer.png",
        "First layer path mismatch"
    );

    // Verify second layer has explicit z value
    assert_eq!(
        stack.layers[1].z_height, 1.0,
        "Second layer z_height mismatch"
    );
    assert_eq!(
        stack.layers[1].content_path, "/3D/with_z_layer.png",
        "Second layer path mismatch"
    );

    Ok(())
}

#[test]
fn test_volumetric_object_geometry_binding() -> anyhow::Result<()> {
    // Test that object correctly references volumetric stack through geometry binding
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <volumetricstack id="70">
            <layer z="0.0" path="/3D/layer0.png" />
            <layer z="0.5" path="/3D/layer1.png" />
            <layer z="1.0" path="/3D/layer2.png" />
        </volumetricstack>
        <volumetricstack id="71">
            <layer z="0.0" path="/3D/other0.png" />
            <layer z="1.0" path="/3D/other1.png" />
        </volumetricstack>
        <object id="1" type="model" volumetricstackid="70" />
        <object id="2" type="model" volumetricstackid="71" />
    </resources>
    <build>
        <item objectid="1" />
        <item objectid="2" />
    </build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;

    // Verify object 1 binding
    let obj1 = model
        .resources
        .get_object(ResourceId(1))
        .expect("Object 1 missing");
    if let Geometry::VolumetricStack(stack_id) = obj1.geometry {
        assert_eq!(
            stack_id,
            ResourceId(70),
            "Object 1 should reference stack 70"
        );
    } else {
        panic!(
            "Object 1 geometry mismatch: expected VolumetricStack, got {:?}",
            obj1.geometry
        );
    }

    // Verify object 2 binding
    let obj2 = model
        .resources
        .get_object(ResourceId(2))
        .expect("Object 2 missing");
    if let Geometry::VolumetricStack(stack_id) = obj2.geometry {
        assert_eq!(
            stack_id,
            ResourceId(71),
            "Object 2 should reference stack 71"
        );
    } else {
        panic!(
            "Object 2 geometry mismatch: expected VolumetricStack, got {:?}",
            obj2.geometry
        );
    }

    // Verify stacks exist and have correct content
    let stack70 = model
        .resources
        .get_volumetric_stack(ResourceId(70))
        .expect("Stack 70 missing");
    assert_eq!(stack70.layers.len(), 3, "Stack 70 should have 3 layers");

    let stack71 = model
        .resources
        .get_volumetric_stack(ResourceId(71))
        .expect("Stack 71 missing");
    assert_eq!(stack71.layers.len(), 2, "Stack 71 should have 2 layers");

    Ok(())
}

/// Test that volumetricstackid takes precedence when object also has mesh content.
/// The parser should still produce Geometry::VolumetricStack (mesh is discarded).
#[test]
fn test_volumetric_stack_with_unexpected_mesh_content() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"
       xmlns:v="http://schemas.microsoft.com/3dmanufacturing/volumetric/2018/11">
    <resources>
        <v:volumestack id="10">
        </v:volumestack>
        <object id="1" type="model" volumetricstackid="10">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0" />
                    <vertex x="1" y="0" z="0" />
                    <vertex x="0" y="1" z="0" />
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2" />
                </triangles>
            </mesh>
        </object>
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;
    let obj = model
        .resources
        .get_object(ResourceId(1))
        .expect("Object 1 missing");

    // VolumetricStack takes precedence - mesh content is discarded with warning
    match &obj.geometry {
        Geometry::VolumetricStack(vsid) => assert_eq!(*vsid, ResourceId(10)),
        other => panic!("Expected VolumetricStack, got {:?}", other),
    }

    Ok(())
}
