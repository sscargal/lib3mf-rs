use lib3mf_core::parser::model_parser::parse_model;
use std::io::Cursor;

#[test]
fn test_large_model_streaming() {
    // Generate a medium-sized mock XML to verify the parser handles it efficiently
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>"#);
    
    for i in 0..1000 {
        xml.push_str(&format!(r#"<vertex x="{}" y="{}" z="{}" />"#, i, i, i));
    }
    
    xml.push_str(r#"</vertices>
                <triangles>"#);
    
    for i in 0..998 {
        xml.push_str(&format!(r#"<triangle v1="{}" v2="{}" v3="{}" />"#, i, i+1, i+2));
    }
    
    xml.push_str(r#"</triangles>
            </mesh>
        </object>
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"#);

    let cursor = Cursor::new(xml);
    let model = parse_model(cursor).expect("Failed to parse large model");
    
    assert_eq!(model.resources.iter_objects().count(), 1);
}
