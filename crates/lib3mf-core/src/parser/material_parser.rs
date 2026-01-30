use crate::error::{Lib3mfError, Result};
use crate::model::{
    BaseMaterial, BaseMaterialsGroup, BlendMethod, Color, ColorGroup, Composite,
    CompositeMaterials, Multi, MultiProperties, ResourceId, Texture2DCoord, Texture2DGroup,
};
use crate::parser::xml_parser::{XmlParser, get_attribute};
use quick_xml::events::Event;
use std::io::BufRead;

// ... existing code ...

pub fn parse_texture_2d_group<R: BufRead>(
    parser: &mut XmlParser<R>,
    id: ResourceId,
    texture_id: ResourceId,
) -> Result<Texture2DGroup> {
    let mut coords = Vec::new();

    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"tex2coord" => {
                let u = get_attribute(&e, b"u")
                    .ok_or_else(|| Lib3mfError::Validation("tex2coord missing u".to_string()))?
                    .parse::<f32>()
                    .map_err(|_| Lib3mfError::Validation("Invalid u value".to_string()))?;
                let v = get_attribute(&e, b"v")
                    .ok_or_else(|| Lib3mfError::Validation("tex2coord missing v".to_string()))?
                    .parse::<f32>()
                    .map_err(|_| Lib3mfError::Validation("Invalid v value".to_string()))?;
                coords.push(Texture2DCoord { u, v });
            }
            Event::End(e) if e.name().as_ref() == b"texture2dgroup" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in texture2dgroup".to_string(),
                ));
            }
            _ => {}
        }
    }

    Ok(Texture2DGroup {
        id,
        texture_id,
        coords,
    })
}

pub fn parse_composite_materials<R: BufRead>(
    parser: &mut XmlParser<R>,
    id: ResourceId,
    base_material_id: ResourceId,
    indices: Vec<u32>,
) -> Result<CompositeMaterials> {
    let mut composites = Vec::new();

    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"composite" => {
                let values_str = get_attribute(&e, b"values").ok_or_else(|| {
                    Lib3mfError::Validation("composite missing values".to_string())
                })?;
                let values = values_str
                    .split_whitespace()
                    .map(|s| {
                        s.parse::<f32>()
                            .map_err(|_| Lib3mfError::Validation("Invalid composite value".to_string()))
                    })
                    .collect::<Result<Vec<f32>>>()?;
                composites.push(Composite { values });
            }
            Event::End(e) if e.name().as_ref() == b"compositematerials" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in compositematerials".to_string(),
                ));
            }
            _ => {}
        }
    }

    Ok(CompositeMaterials {
        id,
        base_material_id,
        indices,
        composites,
    })
}

pub fn parse_multi_properties<R: BufRead>(
    parser: &mut XmlParser<R>,
    id: ResourceId,
    pids: Vec<ResourceId>,
    blend_methods: Vec<BlendMethod>,
) -> Result<MultiProperties> {
    let mut multis = Vec::new();

    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"multi" => {
                let pindices_str = get_attribute(&e, b"pindices").ok_or_else(|| {
                    Lib3mfError::Validation("multi missing pindices".to_string())
                })?;
                let pindices = pindices_str
                    .split_whitespace()
                    .map(|s| {
                        s.parse::<u32>()
                            .map_err(|_| Lib3mfError::Validation("Invalid pindex value".to_string()))
                    })
                    .collect::<Result<Vec<u32>>>()?;
                multis.push(Multi { pindices });
            }
            Event::End(e) if e.name().as_ref() == b"multiproperties" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in multiproperties".to_string(),
                ));
            }
            _ => {}
        }
    }

    Ok(MultiProperties {
        id,
        pids,
        blend_methods,
        multis,
    })
}

pub fn parse_base_materials<R: BufRead>(
    parser: &mut XmlParser<R>,
    id: ResourceId,
) -> Result<BaseMaterialsGroup> {
    let mut materials = Vec::new();

    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"base" => {
                let name = get_attribute(&e, b"name").ok_or_else(|| {
                    Lib3mfError::Validation("base element missing 'name' attribute".to_string())
                })?;
                let color_hex = get_attribute(&e, b"displaycolor").ok_or_else(|| {
                    Lib3mfError::Validation(
                        "base element missing 'displaycolor' attribute".to_string(),
                    )
                })?;
                let display_color = Color::from_hex(&color_hex).ok_or_else(|| {
                    Lib3mfError::Validation(format!("Invalid color format: {}", color_hex))
                })?;

                materials.push(BaseMaterial {
                    name: name.into_owned(),
                    display_color,
                });
            }
            Event::End(e) if e.name().as_ref() == b"basematerials" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in basematerials".to_string(),
                ));
            }
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
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"color" => {
                let color_hex = get_attribute(&e, b"color").ok_or_else(|| {
                    Lib3mfError::Validation("color element missing 'color' attribute".to_string())
                })?;
                let color = Color::from_hex(&color_hex).ok_or_else(|| {
                    Lib3mfError::Validation(format!("Invalid color format: {}", color_hex))
                })?;
                colors.push(color);
            }
            Event::End(e) if e.name().as_ref() == b"colorgroup" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in colorgroup".to_string(),
                ));
            }
            _ => {}
        }
    }

    Ok(ColorGroup { id, colors })
}

