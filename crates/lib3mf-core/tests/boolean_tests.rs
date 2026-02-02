//! Tests for Boolean Operations Extension parsing, writing, and validation.

use lib3mf_core::model::{
    BooleanOperationType, Geometry, ResourceId,
};
use lib3mf_core::parser::parse_model;
use lib3mf_core::validation::ValidationLevel;
use std::io::Cursor;

/// Test parsing a simple boolean union operation
#[test]
fn test_parse_boolean_union() -> anyhow::Result<()> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:b="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0"/>
                    <vertex x="10" y="0" z="0"/>
                    <vertex x="0" y="10" z="0"/>
                    <vertex x="0" y="0" z="10"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                    <triangle v1="0" v2="2" v3="3"/>
                    <triangle v1="0" v2="3" v3="1"/>
                    <triangle v1="1" v2="3" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="2" type="model">
            <mesh>
                <vertices>
                    <vertex x="5" y="0" z="0"/>
                    <vertex x="15" y="0" z="0"/>
                    <vertex x="5" y="10" z="0"/>
                    <vertex x="5" y="0" z="10"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                    <triangle v1="0" v2="2" v3="3"/>
                    <triangle v1="0" v2="3" v3="1"/>
                    <triangle v1="1" v2="3" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <b:booleanshape id="3" objectid="1">
            <b:boolean operation="union" objectid="2"/>
        </b:booleanshape>
    </resources>
    <build>
        <item objectid="3"/>
    </build>
</model>"#;

    let model = parse_model(Cursor::new(xml))?;

    let obj = model.resources.get_object(ResourceId(3))
        .expect("BooleanShape object missing");

    if let Geometry::BooleanShape(bs) = &obj.geometry {
        assert_eq!(bs.base_object_id, ResourceId(1));
        assert_eq!(bs.base_transform, glam::Mat4::IDENTITY);
        assert!(bs.base_path.is_none());
        assert_eq!(bs.operations.len(), 1);

        let op = &bs.operations[0];
        assert_eq!(op.operation_type, BooleanOperationType::Union);
        assert_eq!(op.object_id, ResourceId(2));
        assert_eq!(op.transform, glam::Mat4::IDENTITY);
    } else {
        panic!("Expected BooleanShape geometry");
    }

    Ok(())
}

/// Test parsing all three operation types
#[test]
fn test_parse_all_operation_types() -> anyhow::Result<()> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:b="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07">
    <resources>
        <object id="1" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/></vertices><triangles/></mesh></object>
        <object id="2" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/></vertices><triangles/></mesh></object>
        <object id="3" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/></vertices><triangles/></mesh></object>
        <object id="4" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/></vertices><triangles/></mesh></object>
        <b:booleanshape id="10" objectid="1">
            <b:boolean operation="union" objectid="2"/>
            <b:boolean operation="difference" objectid="3"/>
            <b:boolean operation="intersection" objectid="4"/>
        </b:booleanshape>
    </resources>
    <build/>
</model>"#;

    let model = parse_model(Cursor::new(xml))?;

    let obj = model.resources.get_object(ResourceId(10))
        .expect("BooleanShape object missing");

    if let Geometry::BooleanShape(bs) = &obj.geometry {
        assert_eq!(bs.operations.len(), 3);
        assert_eq!(bs.operations[0].operation_type, BooleanOperationType::Union);
        assert_eq!(bs.operations[1].operation_type, BooleanOperationType::Difference);
        assert_eq!(bs.operations[2].operation_type, BooleanOperationType::Intersection);
    } else {
        panic!("Expected BooleanShape geometry");
    }

    Ok(())
}

/// Test that missing operation attribute defaults to union
#[test]
fn test_default_operation_type() -> anyhow::Result<()> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:b="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07">
    <resources>
        <object id="1" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/></vertices><triangles/></mesh></object>
        <object id="2" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/></vertices><triangles/></mesh></object>
        <b:booleanshape id="10" objectid="1">
            <b:boolean objectid="2"/>
        </b:booleanshape>
    </resources>
    <build/>
</model>"#;

    let model = parse_model(Cursor::new(xml))?;

    if let Some(obj) = model.resources.get_object(ResourceId(10)) {
        if let Geometry::BooleanShape(bs) = &obj.geometry {
            assert_eq!(bs.operations[0].operation_type, BooleanOperationType::Union);
        } else {
            panic!("Expected BooleanShape");
        }
    }

    Ok(())
}

/// Test parsing with transformation matrix
#[test]
fn test_parse_with_transform() -> anyhow::Result<()> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:b="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07">
    <resources>
        <object id="1" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/></vertices><triangles/></mesh></object>
        <object id="2" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/></vertices><triangles/></mesh></object>
        <b:booleanshape id="10" objectid="1" transform="1 0 0 0 1 0 0 0 1 10 20 30">
            <b:boolean objectid="2" operation="difference" transform="2 0 0 0 2 0 0 0 2 5 5 5"/>
        </b:booleanshape>
    </resources>
    <build/>
</model>"#;

    let model = parse_model(Cursor::new(xml))?;

    if let Some(obj) = model.resources.get_object(ResourceId(10)) {
        if let Geometry::BooleanShape(bs) = &obj.geometry {
            // Base transform: identity rotation + translation (10, 20, 30)
            assert_eq!(bs.base_transform.w_axis.x, 10.0);
            assert_eq!(bs.base_transform.w_axis.y, 20.0);
            assert_eq!(bs.base_transform.w_axis.z, 30.0);

            // Operation transform: scale 2x + translation (5, 5, 5)
            let op = &bs.operations[0];
            assert_eq!(op.transform.x_axis.x, 2.0);
            assert_eq!(op.transform.w_axis.x, 5.0);
        }
    }

    Ok(())
}

/// Test validation catches missing base object
#[test]
fn test_validation_missing_base_object() -> anyhow::Result<()> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:b="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07">
    <resources>
        <object id="2" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/></vertices><triangles/></mesh></object>
        <b:booleanshape id="10" objectid="99">
            <b:boolean objectid="2"/>
        </b:booleanshape>
    </resources>
    <build/>
</model>"#;

    let model = parse_model(Cursor::new(xml))?;
    let report = model.validate(ValidationLevel::Standard);

    assert!(report.has_errors());
    assert!(report.items.iter().any(|e| e.code == 2102));

    Ok(())
}

/// Test validation catches missing operation object
#[test]
fn test_validation_missing_operation_object() -> anyhow::Result<()> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:b="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07">
    <resources>
        <object id="1" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/></vertices><triangles/></mesh></object>
        <b:booleanshape id="10" objectid="1">
            <b:boolean objectid="99"/>
        </b:booleanshape>
    </resources>
    <build/>
</model>"#;

    let model = parse_model(Cursor::new(xml))?;
    let report = model.validate(ValidationLevel::Standard);

    assert!(report.has_errors());
    assert!(report.items.iter().any(|e| e.code == 2104));

    Ok(())
}

/// Test validation catches cycles in boolean graph
#[test]
fn test_validation_cycle_detection() -> anyhow::Result<()> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:b="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07">
    <resources>
        <object id="1" type="model"><mesh><vertices><vertex x="0" y="0" z="0"/></vertices><triangles/></mesh></object>
        <b:booleanshape id="2" objectid="3">
            <b:boolean objectid="1"/>
        </b:booleanshape>
        <b:booleanshape id="3" objectid="2">
            <b:boolean objectid="1"/>
        </b:booleanshape>
    </resources>
    <build/>
</model>"#;

    let model = parse_model(Cursor::new(xml))?;
    let report = model.validate(ValidationLevel::Standard);

    assert!(report.has_errors());
    assert!(report.items.iter().any(|e| e.code == 2100));

    Ok(())
}

/// Test round-trip: parse -> write -> parse produces equivalent model
#[test]
fn test_round_trip() -> anyhow::Result<()> {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US"
       xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"
       xmlns:b="http://schemas.3mf.io/3dmanufacturing/booleanoperations/2023/07">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
                    <vertex x="0" y="0" z="0"/>
                    <vertex x="10" y="0" z="0"/>
                    <vertex x="0" y="10" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <object id="2" type="model">
            <mesh>
                <vertices>
                    <vertex x="5" y="0" z="0"/>
                    <vertex x="15" y="0" z="0"/>
                    <vertex x="5" y="10" z="0"/>
                </vertices>
                <triangles>
                    <triangle v1="0" v2="1" v3="2"/>
                </triangles>
            </mesh>
        </object>
        <b:booleanshape id="3" objectid="1">
            <b:boolean operation="difference" objectid="2"/>
        </b:booleanshape>
    </resources>
    <build>
        <item objectid="3"/>
    </build>
</model>"#;

    // Parse original
    let model1 = parse_model(Cursor::new(xml))?;

    // Write to buffer
    let mut buffer = Vec::new();
    model1.write_xml(&mut buffer, None)?;

    // Parse written output
    let model2 = parse_model(Cursor::new(&buffer))?;

    // Compare
    let bs1 = model1.resources.get_object(ResourceId(3)).unwrap();
    let bs2 = model2.resources.get_object(ResourceId(3)).unwrap();

    if let (Geometry::BooleanShape(s1), Geometry::BooleanShape(s2)) =
        (&bs1.geometry, &bs2.geometry)
    {
        assert_eq!(s1.base_object_id, s2.base_object_id);
        assert_eq!(s1.operations.len(), s2.operations.len());
        assert_eq!(
            s1.operations[0].operation_type,
            s2.operations[0].operation_type
        );
        assert_eq!(s1.operations[0].object_id, s2.operations[0].object_id);
    } else {
        panic!("Expected BooleanShape geometry in both models");
    }

    Ok(())
}
