use lib3mf_core::model::Geometry;
use lib3mf_core::parser::parse_model;
use std::io::Cursor;
use uuid::Uuid;

#[test]
fn test_parse_production_extension() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06">
    <resources>
        <object id="1" uuid="6d1d4f20-8c2f-4a37-9d21-4f0e9b9d9d9d">
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
        <object id="2" p:uuid="f47ac10b-58cc-4372-a567-0e02b2c3d479">
             <components>
                <component objectid="1" uuid="a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11" />
            </components>
        </object>
    </resources>
    <build>
        <item objectid="1" uuid="123e4567-e89b-12d3-a456-426614174000" path="/3D/external.model" />
        <item objectid="2" p:uuid="98765432-1234-5678-90ab-cdef12345678" p:path="/Production/part.model" />
    </build>
</model>"##;

    let model = parse_model(Cursor::new(xml))?;

    // Check Object 1 UUID (no prefix)
    let obj1 = model
        .resources
        .get_object(lib3mf_core::model::ResourceId(1))
        .expect("Object 1 missing");
    assert_eq!(
        obj1.uuid,
        Some(Uuid::parse_str("6d1d4f20-8c2f-4a37-9d21-4f0e9b9d9d9d")?)
    );

    // Check Object 2 UUID (p: prefix)
    let obj2 = model
        .resources
        .get_object(lib3mf_core::model::ResourceId(2))
        .expect("Object 2 missing");
    assert_eq!(
        obj2.uuid,
        Some(Uuid::parse_str("f47ac10b-58cc-4372-a567-0e02b2c3d479")?)
    );

    // Check Component UUID
    if let Geometry::Components(comps) = &obj2.geometry {
        assert_eq!(comps.components.len(), 1);
        assert_eq!(
            comps.components[0].uuid,
            Some(Uuid::parse_str("a0eebc99-9c0b-4ef8-bb6d-6bb9bd380a11")?)
        );
    } else {
        panic!("Object 2 should be components");
    }

    // Check Build Items
    assert_eq!(model.build.items.len(), 2);

    // Item 1: "path"
    let item1 = &model.build.items[0];
    assert_eq!(
        item1.uuid,
        Some(Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000")?)
    );
    assert_eq!(item1.path, Some("/3D/external.model".to_string()));

    // Item 2: "p:path"
    let item2 = &model.build.items[1];
    assert_eq!(
        item2.uuid,
        Some(Uuid::parse_str("98765432-1234-5678-90ab-cdef12345678")?)
    );
    assert_eq!(item2.path, Some("/Production/part.model".to_string()));

    Ok(())
}
