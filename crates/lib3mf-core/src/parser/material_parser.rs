use crate::error::{Lib3mfError, Result};
use crate::model::{BaseMaterial, BaseMaterialsGroup, Color, ColorGroup, ResourceId};
use crate::parser::xml_parser::{get_attribute, XmlParser};
use quick_xml::events::Event;
use std::io::BufRead;

pub fn parse_base_materials<R: BufRead>(
    parser: &mut XmlParser<R>,
    id: ResourceId,
) -> Result<BaseMaterialsGroup> {
    let mut materials = Vec::new();

    loop {
        match parser.next()? {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"base" => {
                let name = get_attribute(&e, b"name")
                    .ok_or_else(|| Lib3mfError::Validation("base element missing 'name' attribute".to_string()))?;
                let color_hex = get_attribute(&e, b"displaycolor")
                     .ok_or_else(|| Lib3mfError::Validation("base element missing 'displaycolor' attribute".to_string()))?;
                let display_color = Color::from_hex(&color_hex).ok_or_else(|| {
                    Lib3mfError::Validation(format!("Invalid color format: {}", color_hex))
                })?;
                
                materials.push(BaseMaterial {
                    name,
                    display_color,
                });
            }
            Event::End(e) if e.name().as_ref() == b"basematerials" => break,
            Event::Eof => return Err(Lib3mfError::Validation("Unexpected EOF in basematerials".to_string())),
            _ => {}
        }
    }

    Ok(BaseMaterialsGroup { id, materials })
}

pub fn parse_color_group<R: BufRead>(
    parser: &mut XmlParser<R>,
    id: ResourceId,
) -> Result<ColorGroup> {
    let mut colors = Vec::new();

    loop {
        match parser.next()? {
             Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"color" => {
                let color_hex = get_attribute(&e, b"color")
                     .ok_or_else(|| Lib3mfError::Validation("color element missing 'color' attribute".to_string()))?;
                let color = Color::from_hex(&color_hex).ok_or_else(|| {
                    Lib3mfError::Validation(format!("Invalid color format: {}", color_hex))
                })?;
                colors.push(color);
            }
            Event::End(e) if e.name().as_ref() == b"colorgroup" => break,
            Event::Eof => return Err(Lib3mfError::Validation("Unexpected EOF in colorgroup".to_string())),
            _ => {}
        }
    }

    Ok(ColorGroup { id, colors })
}
