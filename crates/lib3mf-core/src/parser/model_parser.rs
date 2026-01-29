use crate::error::{Lib3mfError, Result};
use crate::model::{Geometry, Model, Object, Unit};
use crate::parser::build_parser::parse_build;
use crate::parser::component_parser::parse_components;
use crate::parser::material_parser::{parse_base_materials, parse_color_group};
use crate::parser::mesh_parser::parse_mesh;
use crate::parser::slice_parser::parse_slice_stack_content;
use crate::parser::volumetric_parser::parse_volumetric_stack_content;
use crate::parser::xml_parser::{XmlParser, get_attribute, get_attribute_f32, get_attribute_u32};
use quick_xml::events::Event;
use std::io::BufRead;

pub fn parse_model<R: BufRead>(reader: R) -> Result<Model> {
    let mut parser = XmlParser::new(reader);
    let mut model = Model::default();

    loop {
        match parser.read_next_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"model" => {
                    if let Some(unit_str) = get_attribute(&e, b"unit") {
                        model.unit = match unit_str.as_ref() {
                            "micron" => Unit::Micron,
                            "millimeter" => Unit::Millimeter,
                            "centimeter" => Unit::Centimeter,
                            "inch" => Unit::Inch,
                            "foot" => Unit::Foot,
                            "meter" => Unit::Meter,
                            _ => Unit::Millimeter, // Default or warn?
                        };
                    }
                    model.language = get_attribute(&e, b"xml:lang").map(|s| s.into_owned());
                }
                b"metadata" => {
                    let name = get_attribute(&e, b"name")
                        .ok_or(Lib3mfError::Validation("Metadata missing name".to_string()))?
                        .into_owned();
                    let content = parser.read_text_content()?;
                    model.metadata.insert(name, content);
                }
                b"resources" => parse_resources(&mut parser, &mut model)?,
                b"build" => {
                    model.build = parse_build(&mut parser)?;
                }
                _ => {}
            },
            Event::Empty(e) => {
                if e.name().as_ref() == b"metadata" {
                    let name = get_attribute(&e, b"name")
                        .ok_or(Lib3mfError::Validation("Metadata missing name".to_string()))?;
                    model.metadata.insert(name.into_owned(), String::new());
                }
            }
            Event::End(e) if e.name().as_ref() == b"model" => break,
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(model)
}

fn parse_resources<R: BufRead>(parser: &mut XmlParser<R>, model: &mut Model) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"object" => {
                        let id = crate::model::ResourceId(get_attribute_u32(&e, b"id")?);
                        let name = get_attribute(&e, b"name").map(|s| s.into_owned());
                        let part_number = get_attribute(&e, b"partnumber").map(|s| s.into_owned());
                        let pid = get_attribute_u32(&e, b"pid")
                            .map(crate::model::ResourceId)
                            .ok();
                        let pindex = get_attribute_u32(&e, b"pindex").ok();
                        let uuid = crate::parser::xml_parser::get_attribute_uuid(&e)?;

                        // Check for slicestackid (default or prefixed)
                        let slice_stack_id = get_attribute_u32(&e, b"slicestackid")
                            .or_else(|_| get_attribute_u32(&e, b"s:slicestackid"))
                            .map(crate::model::ResourceId)
                            .ok();

                        // Check for volumetricstackid (hypothetical prefix v:)
                        let vol_stack_id = get_attribute_u32(&e, b"volumetricstackid")
                            .or_else(|_| get_attribute_u32(&e, b"v:volumetricstackid"))
                            .map(crate::model::ResourceId)
                            .ok();

                        let _obj_type =
                            get_attribute(&e, b"type").unwrap_or_else(|| "model".into());

                        let geometry_content = parse_object_geometry(parser)?;

                        let geometry = if let Some(ssid) = slice_stack_id {
                            // TODO: Warn if geometry_content is not empty?
                            crate::model::Geometry::SliceStack(ssid)
                        } else if let Some(vsid) = vol_stack_id {
                            crate::model::Geometry::VolumetricStack(vsid)
                        } else {
                            geometry_content
                        };

                        let object = Object {
                            id,
                            name,
                            part_number,
                            uuid,
                            pid,
                            pindex,
                            geometry,
                        };
                        model.resources.add_object(object)?;
                    }
                    b"basematerials" => {
                        let id = crate::model::ResourceId(get_attribute_u32(&e, b"id")?);
                        let group = parse_base_materials(parser, id)?;
                        model.resources.add_base_materials(group)?;
                    }
                    b"colorgroup" => {
                        let id = crate::model::ResourceId(get_attribute_u32(&e, b"id")?);
                        let group = parse_color_group(parser, id)?;
                        model.resources.add_color_group(group)?;
                    }
                    b"slicestack" => {
                        let id = crate::model::ResourceId(get_attribute_u32(&e, b"id")?);
                        let z_bottom = get_attribute_f32(&e, b"zbottom").unwrap_or(0.0);
                        let stack = parse_slice_stack_content(parser, id, z_bottom)?;
                        model.resources.add_slice_stack(stack)?;
                    }
                    b"volumetricstack" => {
                        let id = crate::model::ResourceId(get_attribute_u32(&e, b"id")?);
                        let stack = parse_volumetric_stack_content(parser, id, 0.0)?;
                        model.resources.add_volumetric_stack(stack)?;
                    }
                    _ => {}
                }
            }
            Event::End(e) if e.name().as_ref() == b"resources" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in resources".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_object_geometry<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Geometry> {
    // We are inside <object> tag. We expect either <mesh> or <components> next.
    // NOTE: object is open. We read until </object>.

    // Actually, parse_object_geometry needs to look for mesh/components.
    // If <object> was Empty, we wouldn't be here (logic above needs check).
    // The previous match Event::Start(object) means it has content.

    let mut geometry = Geometry::Mesh(crate::model::Mesh::default()); // Default fallback? Or Option/Result?

    loop {
        match parser.read_next_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"mesh" => {
                    geometry = Geometry::Mesh(parse_mesh(parser)?);
                }
                b"components" => {
                    geometry = Geometry::Components(parse_components(parser)?);
                }
                _ => {}
            },
            Event::End(e) if e.name().as_ref() == b"object" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in object".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(geometry)
}
