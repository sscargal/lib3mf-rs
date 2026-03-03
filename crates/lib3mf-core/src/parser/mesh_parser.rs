use crate::error::{Lib3mfError, Result};
use crate::model::{ClippingMode, Mesh, Triangle, Vertex};
use crate::parser::beamlattice_parser::parse_beam_lattice_content;
use crate::parser::xml_parser::{XmlParser, get_attribute, get_attribute_f32, get_attribute_u32};
use quick_xml::events::Event;
use std::io::BufRead;

/// Parses a `<mesh>` element (vertices and triangles) into a `Mesh`.
pub fn parse_mesh<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Mesh> {
    let mut mesh = Mesh::default();

    loop {
        match parser.read_next_event()? {
            Event::Start(e) => match e.local_name().as_ref() {
                b"vertices" => parse_vertices(parser, &mut mesh)?,
                b"triangles" => parse_triangles(parser, &mut mesh)?,
                b"beamlattice" => {
                    let radius = get_attribute_f32(&e, b"radius").ok();
                    let min_length = get_attribute_f32(&e, b"minlength").unwrap_or(0.0);
                    let precision = get_attribute_f32(&e, b"precision").unwrap_or(0.0);
                    // Handle both "clippingmode" (spec) and "clipping" (some implementations)
                    let clipping_str = get_attribute(&e, b"clippingmode")
                        .or_else(|| get_attribute(&e, b"clipping"));
                    let clipping_mode = if let Some(s) = clipping_str {
                        match s.as_ref() {
                            "inside" => ClippingMode::Inside,
                            "outside" => ClippingMode::Outside,
                            _ => ClippingMode::None,
                        }
                    } else {
                        ClippingMode::None
                    };

                    let lattice = parse_beam_lattice_content(
                        parser,
                        radius,
                        min_length,
                        precision,
                        clipping_mode,
                    )?;
                    mesh.beam_lattice = Some(lattice);
                }
                _ => {} // Ignore headers/metadata inside mesh for now
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

    Ok(mesh)
}

fn parse_vertices<R: BufRead>(parser: &mut XmlParser<R>, mesh: &mut Mesh) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"vertex" => {
                let x = get_attribute_f32(&e, b"x")?;
                let y = get_attribute_f32(&e, b"y")?;
                let z = get_attribute_f32(&e, b"z")?;
                mesh.vertices.push(Vertex { x, y, z });
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

fn parse_triangles<R: BufRead>(parser: &mut XmlParser<R>, mesh: &mut Mesh) -> Result<()> {
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"triangle" => {
                let v1 = get_attribute_u32(&e, b"v1")?;
                let v2 = get_attribute_u32(&e, b"v2")?;
                let v3 = get_attribute_u32(&e, b"v3")?;
                let p1 = get_attribute_u32(&e, b"p1").ok();
                let p2 = get_attribute_u32(&e, b"p2").ok();
                let p3 = get_attribute_u32(&e, b"p3").ok();
                let pid = get_attribute_u32(&e, b"pid").ok();

                mesh.triangles.push(Triangle {
                    v1,
                    v2,
                    v3,
                    p1,
                    p2,
                    p3,
                    pid,
                });
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
