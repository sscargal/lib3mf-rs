use crate::error::{Lib3mfError, Result};
use crate::model::{Beam, BuildItem, CapMode, DisplacementTriangle, ResourceId};
use crate::parser::material_parser::{parse_base_materials, parse_color_group};
use crate::parser::visitor::ModelVisitor;
use crate::parser::xml_parser::{XmlParser, get_attribute, get_attribute_f32, get_attribute_u32};
use glam::Mat4;
use quick_xml::events::Event;
use std::io::BufRead;

/// Parses a 3MF model from an XML reader in a streaming fashion,
/// emitting events to the provided visitor.
pub fn parse_model_streaming<R: BufRead, V: ModelVisitor>(
    reader: R,
    visitor: &mut V,
) -> Result<()> {
    let mut parser = XmlParser::new(reader);

    visitor.on_start_model()?;

    loop {
        match parser.read_next_event()? {
            Event::Start(e) => {
                match e.name().as_ref() {
                    b"model" => {
                        // Attributes like unit/lang handled here if needed?
                    }
                    b"metadata" => {
                        let name = get_attribute(&e, b"name")
                            .ok_or(Lib3mfError::Validation("Metadata missing name".to_string()))?
                            .into_owned();
                        let content = parser.read_text_content()?;
                        visitor.on_metadata(&name, &content)?;
                    }
                    b"resources" => {
                        visitor.on_start_resources()?;
                        parse_resources_streaming(&mut parser, visitor)?;
                        visitor.on_end_resources()?;
                    }
                    b"build" => {
                        visitor.on_start_build()?;
                        parse_build_streaming(&mut parser, visitor)?;
                        visitor.on_end_build()?;
                    }
                    _ => {}
                }
            }
            Event::Empty(e) => {
                if e.name().as_ref() == b"metadata" {
                    let name = get_attribute(&e, b"name")
                        .ok_or(Lib3mfError::Validation("Metadata missing name".to_string()))?;
                    visitor.on_metadata(name.as_ref(), "")?;
                }
            }
            Event::End(e) if e.name().as_ref() == b"model" => break,
            Event::Eof => break,
            _ => {}
        }
    }

    visitor.on_end_model()?;
    Ok(())
}

fn parse_resources_streaming<R: BufRead, V: ModelVisitor>(
    parser: &mut XmlParser<R>,
    visitor: &mut V,
) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"object" => {
                        let id = ResourceId(get_attribute_u32(&e, b"id")?);
                        parse_object_content_streaming(parser, visitor, id)?;
                    }
                    b"basematerials" => {
                        let id = ResourceId(get_attribute_u32(&e, b"id")?);
                        let group = parse_base_materials(parser, id)?;
                        visitor.on_base_materials(id, &group)?;
                    }
                    b"colorgroup" => {
                        let id = ResourceId(get_attribute_u32(&e, b"id")?);
                        let group = parse_color_group(parser, id)?;
                        visitor.on_color_group(id, &group)?;
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

fn parse_object_content_streaming<R: BufRead, V: ModelVisitor>(
    parser: &mut XmlParser<R>,
    visitor: &mut V,
    object_id: ResourceId,
) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) => match e.local_name().as_ref() {
                b"mesh" => {
                    visitor.on_start_mesh(object_id)?;
                    parse_mesh_streaming(parser, visitor, object_id)?;
                    visitor.on_end_mesh()?;
                }
                b"components" => {
                    // Skipping components for now
                }
                b"displacementmesh" => {
                    visitor.on_start_displacement_mesh(object_id)?;
                    parse_displacement_mesh_streaming(parser, visitor)?;
                    visitor.on_end_displacement_mesh()?;
                }
                _ => {}
            },
            Event::End(e) if e.local_name().as_ref() == b"object" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in object".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_mesh_streaming<R: BufRead, V: ModelVisitor>(
    parser: &mut XmlParser<R>,
    visitor: &mut V,
    object_id: ResourceId,
) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) => match e.local_name().as_ref() {
                b"vertices" => parse_vertices_streaming(parser, visitor)?,
                b"triangles" => parse_triangles_streaming(parser, visitor)?,
                b"beamlattice" => {
                    let default_radius = get_attribute_f32(&e, b"radius").unwrap_or(0.0);
                    visitor.on_start_beam_lattice(object_id)?;
                    parse_beam_lattice_streaming(parser, visitor, default_radius)?;
                    visitor.on_end_beam_lattice()?;
                }
                _ => {}
            },
            Event::End(e) if e.local_name().as_ref() == b"mesh" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in mesh".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_vertices_streaming<R: BufRead, V: ModelVisitor>(
    parser: &mut XmlParser<R>,
    visitor: &mut V,
) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) => {
                if e.name().as_ref() == b"vertex" {
                    let x = get_attribute_f32(&e, b"x")?;
                    let y = get_attribute_f32(&e, b"y")?;
                    let z = get_attribute_f32(&e, b"z")?;
                    visitor.on_vertex(x, y, z)?;
                }
            }
            Event::End(e) if e.name().as_ref() == b"vertices" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in vertices".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_triangles_streaming<R: BufRead, V: ModelVisitor>(
    parser: &mut XmlParser<R>,
    visitor: &mut V,
) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) => {
                if e.name().as_ref() == b"triangle" {
                    let v1 = get_attribute_u32(&e, b"v1")?;
                    let v2 = get_attribute_u32(&e, b"v2")?;
                    let v3 = get_attribute_u32(&e, b"v3")?;
                    visitor.on_triangle(v1, v2, v3)?;
                }
            }
            Event::End(e) if e.name().as_ref() == b"triangles" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in triangles".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_beam_lattice_streaming<R: BufRead, V: ModelVisitor>(
    parser: &mut XmlParser<R>,
    visitor: &mut V,
    default_radius: f32,
) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) => match e.local_name().as_ref() {
                b"beams" => parse_beams_streaming(parser, visitor, default_radius)?,
                b"beamsets" => {
                    // BeamSets reference beams by index and require all beams to be
                    // known first — incompatible with streaming semantics. Skip silently.
                    parser.read_to_end(b"beamsets")?;
                }
                _ => {}
            },
            Event::End(e) if e.local_name().as_ref() == b"beamlattice" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in beamlattice".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_beams_streaming<R: BufRead, V: ModelVisitor>(
    parser: &mut XmlParser<R>,
    visitor: &mut V,
    default_radius: f32,
) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == b"beam" => {
                let v1 = get_attribute_u32(&e, b"v1")?;
                let v2 = get_attribute_u32(&e, b"v2")?;
                let r1 = get_attribute_f32(&e, b"r1").unwrap_or(default_radius);
                let r2 = get_attribute_f32(&e, b"r2").unwrap_or(r1);
                let p1 = get_attribute_u32(&e, b"p1").ok();
                let p2 = get_attribute_u32(&e, b"p2").ok();
                let cap_mode = if let Some(s) = get_attribute(&e, b"cap") {
                    match s.as_ref() {
                        "sphere" => CapMode::Sphere,
                        "hemisphere" => CapMode::Hemisphere,
                        "butt" => CapMode::Butt,
                        _ => CapMode::Sphere,
                    }
                } else {
                    CapMode::Sphere
                };
                visitor.on_beam(&Beam {
                    v1,
                    v2,
                    r1,
                    r2,
                    p1,
                    p2,
                    cap_mode,
                })?;
            }
            Event::End(e) if e.local_name().as_ref() == b"beams" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in beams".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_displacement_mesh_streaming<R: BufRead, V: ModelVisitor>(
    parser: &mut XmlParser<R>,
    visitor: &mut V,
) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) => match e.local_name().as_ref() {
                b"vertices" => parse_displacement_vertices_streaming(parser, visitor)?,
                b"triangles" => parse_displacement_triangles_streaming(parser, visitor)?,
                b"normvectors" => parse_displacement_normals_streaming(parser, visitor)?,
                b"disp2dgroups" => {
                    // Gradient vectors have two-level nesting (disp2dgroups > disp2dgroup > gradient).
                    // They are small secondary data and incompatible with single-pass streaming.
                    // Skip silently. Use DOM mode if gradient data is required.
                    parser.read_to_end(b"disp2dgroups")?;
                }
                _ => {}
            },
            Event::End(e) if e.local_name().as_ref() == b"displacementmesh" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in displacementmesh".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_displacement_vertices_streaming<R: BufRead, V: ModelVisitor>(
    parser: &mut XmlParser<R>,
    visitor: &mut V,
) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == b"vertex" => {
                let x = get_attribute_f32(&e, b"x")?;
                let y = get_attribute_f32(&e, b"y")?;
                let z = get_attribute_f32(&e, b"z")?;
                visitor.on_displacement_vertex(x, y, z)?;
            }
            Event::End(e) if e.local_name().as_ref() == b"vertices" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in displacement vertices".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_displacement_triangles_streaming<R: BufRead, V: ModelVisitor>(
    parser: &mut XmlParser<R>,
    visitor: &mut V,
) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == b"triangle" => {
                let triangle = DisplacementTriangle {
                    v1: get_attribute_u32(&e, b"v1")?,
                    v2: get_attribute_u32(&e, b"v2")?,
                    v3: get_attribute_u32(&e, b"v3")?,
                    d1: get_attribute_u32(&e, b"d1").ok(),
                    d2: get_attribute_u32(&e, b"d2").ok(),
                    d3: get_attribute_u32(&e, b"d3").ok(),
                    p1: get_attribute_u32(&e, b"p1").ok(),
                    p2: get_attribute_u32(&e, b"p2").ok(),
                    p3: get_attribute_u32(&e, b"p3").ok(),
                    pid: get_attribute_u32(&e, b"pid").ok(),
                };
                visitor.on_displacement_triangle(&triangle)?;
            }
            Event::End(e) if e.local_name().as_ref() == b"triangles" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in displacement triangles".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_displacement_normals_streaming<R: BufRead, V: ModelVisitor>(
    parser: &mut XmlParser<R>,
    visitor: &mut V,
) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == b"normvector" => {
                let nx = get_attribute_f32(&e, b"nx")?;
                let ny = get_attribute_f32(&e, b"ny")?;
                let nz = get_attribute_f32(&e, b"nz")?;
                visitor.on_displacement_normal(nx, ny, nz)?;
            }
            Event::End(e) if e.local_name().as_ref() == b"normvectors" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in displacement normals".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_build_streaming<R: BufRead, V: ModelVisitor>(
    parser: &mut XmlParser<R>,
    visitor: &mut V,
) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) => {
                if e.name().as_ref() == b"item" {
                    let object_id = ResourceId(get_attribute_u32(&e, b"objectid")?);
                    let item = BuildItem {
                        object_id,
                        transform: Mat4::IDENTITY,
                        part_number: None,
                        uuid: None,
                        path: None,
                        printable: None,
                    };
                    visitor.on_build_item(&item)?;
                }
            }
            Event::End(e) if e.name().as_ref() == b"build" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in build".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}
