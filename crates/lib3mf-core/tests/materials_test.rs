use lib3mf_core::model::{Color, ResourceId};
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

#[test]
fn test_parse_materials() -> anyhow::Result<()> {
    let xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <basematerials id="1">
            <base name="Red" displaycolor="#FF0000" />
            <base name="Green" displaycolor="#00FF00" />
        </basematerials>
        <colorgroup id="2">
            <color color="#0000FFFF" />
            <color color="#FFFF00FF" />
        </colorgroup>
    </resources>
    <build />
</model>"##;

    let model = parse_model(Cursor::new(xml))?;

    // Check Base Materials
    let base_mats = model
        .resources
        .get_base_materials(ResourceId(1))
        .expect("Base materials not found");
    assert_eq!(base_mats.materials.len(), 2);
    assert_eq!(base_mats.materials[0].name, "Red");
    assert_eq!(
        base_mats.materials[0].display_color,
        Color::new(255, 0, 0, 255)
    );
    assert_eq!(base_mats.materials[1].name, "Green");
    assert_eq!(
        base_mats.materials[1].display_color,
        Color::new(0, 255, 0, 255)
    );

    // Check Color Group
    let color_group = model
        .resources
        .get_color_group(ResourceId(2))
        .expect("Color group not found");
    assert_eq!(color_group.colors.len(), 2);
    assert_eq!(color_group.colors[0], Color::new(0, 0, 255, 255));
    assert_eq!(color_group.colors[1], Color::new(255, 255, 0, 255));

    Ok(())
}
