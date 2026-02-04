use crate::error::{Lib3mfError, Result};
use crate::model::{Geometry, Model, Object, Unit};
use crate::parser::boolean_parser::parse_boolean_shape;
use crate::parser::build_parser::parse_build;
use crate::parser::component_parser::parse_components;
use crate::parser::displacement_parser::{parse_displacement_2d, parse_displacement_mesh};
use crate::parser::material_parser::{
    parse_base_materials, parse_color_group, parse_composite_materials, parse_multi_properties,
    parse_texture_2d_group,
};
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

                        let object_type = match get_attribute(&e, b"type") {
                            Some(type_str) => match type_str.as_ref() {
                                "model" => crate::model::ObjectType::Model,
                                "support" => crate::model::ObjectType::Support,
                                "solidsupport" => crate::model::ObjectType::SolidSupport,
                                "surface" => crate::model::ObjectType::Surface,
                                "other" => crate::model::ObjectType::Other,
                                unknown => {
                                    eprintln!(
                                        "Warning: Unknown object type '{}', defaulting to 'model'",
                                        unknown
                                    );
                                    crate::model::ObjectType::Model
                                }
                            },
                            None => crate::model::ObjectType::Model,
                        };

                        let thumbnail = get_attribute(&e, b"thumbnail").map(|s| s.into_owned());

                        let geometry_content = parse_object_geometry(parser)?;

                        let geometry = if let Some(ssid) = slice_stack_id {
                            if geometry_content.has_content() {
                                eprintln!(
                                    "Warning: Object {} has slicestackid but also contains geometry content; geometry will be ignored",
                                    id.0
                                );
                            }
                            crate::model::Geometry::SliceStack(ssid)
                        } else if let Some(vsid) = vol_stack_id {
                            if geometry_content.has_content() {
                                eprintln!(
                                    "Warning: Object {} has volumetricstackid but also contains geometry content; geometry will be ignored",
                                    id.0
                                );
                            }
                            crate::model::Geometry::VolumetricStack(vsid)
                        } else {
                            geometry_content
                        };

                        let object = Object {
                            id,
                            object_type,
                            name,
                            part_number,
                            uuid,
                            pid,
                            pindex,
                            thumbnail,
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
                    b"texture2dgroup" => {
                        let id = crate::model::ResourceId(get_attribute_u32(&e, b"id")?);
                        let texid = crate::model::ResourceId(get_attribute_u32(&e, b"texid")?);
                        let group = parse_texture_2d_group(parser, id, texid)?;
                        model.resources.add_texture_2d_group(group)?;
                    }
                    b"compositematerials" => {
                        let id = crate::model::ResourceId(get_attribute_u32(&e, b"id")?);
                        let matid = crate::model::ResourceId(get_attribute_u32(&e, b"matid")?);
                        let matindices_str = get_attribute(&e, b"matindices").ok_or_else(|| {
                            Lib3mfError::Validation(
                                "compositematerials missing matindices".to_string(),
                            )
                        })?;
                        let indices = matindices_str
                            .split_whitespace()
                            .map(|s| {
                                s.parse::<u32>().map_err(|_| {
                                    Lib3mfError::Validation("Invalid matindices value".to_string())
                                })
                            })
                            .collect::<Result<Vec<u32>>>()?;
                        let group = parse_composite_materials(parser, id, matid, indices)?;
                        model.resources.add_composite_materials(group)?;
                    }
                    b"multiproperties" => {
                        let id = crate::model::ResourceId(get_attribute_u32(&e, b"id")?);
                        let pids_str = get_attribute(&e, b"pids").ok_or_else(|| {
                            Lib3mfError::Validation("multiproperties missing pids".to_string())
                        })?;
                        let pids = pids_str
                            .split_whitespace()
                            .map(|s| {
                                s.parse::<u32>()
                                    .map_err(|_| {
                                        Lib3mfError::Validation("Invalid pid value".to_string())
                                    })
                                    .map(crate::model::ResourceId)
                            })
                            .collect::<Result<Vec<crate::model::ResourceId>>>()?;

                        let blendmethods_str =
                            get_attribute(&e, b"blendmethods").ok_or_else(|| {
                                Lib3mfError::Validation(
                                    "multiproperties missing blendmethods".to_string(),
                                )
                            })?;
                        let blend_methods = blendmethods_str
                            .split_whitespace()
                            .map(|s| match s {
                                "mix" => Ok(crate::model::BlendMethod::Mix),
                                "multiply" => Ok(crate::model::BlendMethod::Multiply),
                                _ => Err(Lib3mfError::Validation(format!(
                                    "Invalid blend method: {}",
                                    s
                                ))),
                            })
                            .collect::<Result<Vec<crate::model::BlendMethod>>>()?;

                        let group = parse_multi_properties(parser, id, pids, blend_methods)?;
                        model.resources.add_multi_properties(group)?;
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
                    b"booleanshape" => {
                        let id = crate::model::ResourceId(get_attribute_u32(&e, b"id")?);
                        let base_object_id =
                            crate::model::ResourceId(get_attribute_u32(&e, b"objectid")?);
                        let base_transform = if let Some(s) = get_attribute(&e, b"transform") {
                            crate::parser::component_parser::parse_transform(&s)?
                        } else {
                            glam::Mat4::IDENTITY
                        };
                        let base_path = get_attribute(&e, b"path")
                            .or_else(|| get_attribute(&e, b"p:path"))
                            .map(|s| s.into_owned());

                        let bool_shape =
                            parse_boolean_shape(parser, base_object_id, base_transform, base_path)?;

                        // Per spec, booleanshape is a model-type object
                        let object = Object {
                            id,
                            object_type: crate::model::ObjectType::Model,
                            name: None,
                            part_number: None,
                            uuid: None,
                            pid: None,
                            pindex: None,
                            thumbnail: None,
                            geometry: Geometry::BooleanShape(bool_shape),
                        };
                        model.resources.add_object(object)?;
                    }
                    b"displacement2d" => {
                        let id = crate::model::ResourceId(get_attribute_u32(&e, b"id")?);
                        let path = get_attribute(&e, b"path")
                            .ok_or_else(|| {
                                Lib3mfError::Validation(
                                    "displacement2d missing path attribute".to_string(),
                                )
                            })?
                            .into_owned();

                        let channel = if let Some(ch_str) = get_attribute(&e, b"channel") {
                            match ch_str.as_ref() {
                                "R" => crate::model::Channel::R,
                                "G" => crate::model::Channel::G,
                                "B" => crate::model::Channel::B,
                                "A" => crate::model::Channel::A,
                                _ => crate::model::Channel::G,
                            }
                        } else {
                            crate::model::Channel::G
                        };

                        let tile_style = if let Some(ts_str) = get_attribute(&e, b"tilestyle") {
                            match ts_str.to_lowercase().as_str() {
                                "wrap" => crate::model::TileStyle::Wrap,
                                "mirror" => crate::model::TileStyle::Mirror,
                                "clamp" => crate::model::TileStyle::Clamp,
                                "none" => crate::model::TileStyle::None,
                                _ => crate::model::TileStyle::Wrap,
                            }
                        } else {
                            crate::model::TileStyle::Wrap
                        };

                        let filter = if let Some(f_str) = get_attribute(&e, b"filter") {
                            match f_str.to_lowercase().as_str() {
                                "linear" => crate::model::FilterMode::Linear,
                                "nearest" => crate::model::FilterMode::Nearest,
                                _ => crate::model::FilterMode::Linear,
                            }
                        } else {
                            crate::model::FilterMode::Linear
                        };

                        let height = get_attribute_f32(&e, b"height")?;
                        let offset = get_attribute_f32(&e, b"offset").unwrap_or(0.0);

                        let displacement = parse_displacement_2d(
                            parser, id, path, channel, tile_style, filter, height, offset,
                        )?;
                        model.resources.add_displacement_2d(displacement)?;
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
            Event::Start(e) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"mesh" => {
                        geometry = Geometry::Mesh(parse_mesh(parser)?);
                    }
                    b"components" => {
                        geometry = Geometry::Components(parse_components(parser)?);
                    }
                    b"displacementmesh" => {
                        geometry = Geometry::DisplacementMesh(parse_displacement_mesh(parser)?);
                    }
                    _ => {}
                }
            }
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
