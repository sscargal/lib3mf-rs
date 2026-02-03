use crate::error::{Lib3mfError, Result};
use crate::model::{
    Channel, Displacement2D, DisplacementMesh, DisplacementTriangle, FilterMode, GradientVector,
    NormalVector, ResourceId, TileStyle, Vertex,
};
use crate::parser::xml_parser::{get_attribute_f32, get_attribute_u32, XmlParser};
use quick_xml::events::Event;
use std::io::BufRead;

pub fn parse_displacement_mesh<R: BufRead>(parser: &mut XmlParser<R>) -> Result<DisplacementMesh> {
    let mut vertices = Vec::new();
    let mut triangles = Vec::new();
    let mut normals = Vec::new();
    let mut gradients = Vec::new();

    loop {
        match parser.read_next_event()? {
            Event::Start(e) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"vertices" => {
                        vertices = parse_displacement_vertices(parser)?;
                    }
                    b"triangles" => {
                        triangles = parse_displacement_triangles(parser)?;
                    }
                    b"normvectors" => {
                        normals = parse_normal_vectors(parser)?;
                    }
                    b"disp2dgroups" => {
                        // Parse displacement 2D groups which contain gradient vectors
                        gradients = parse_disp2d_groups(parser)?;
                    }
                    _ => {}
                }
            }
            Event::End(e) if e.local_name().as_ref() == b"displacementmesh" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in displacementmesh".to_string(),
                ));
            }
            _ => {}
        }
    }

    Ok(DisplacementMesh {
        vertices,
        triangles,
        normals,
        gradients: if gradients.is_empty() {
            None
        } else {
            Some(gradients)
        },
    })
}

fn parse_displacement_vertices<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Vec<Vertex>> {
    let mut vertices = Vec::new();
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == b"vertex" => {
                let x = get_attribute_f32(&e, b"x")?;
                let y = get_attribute_f32(&e, b"y")?;
                let z = get_attribute_f32(&e, b"z")?;
                vertices.push(Vertex { x, y, z });
            }
            Event::End(e) if e.local_name().as_ref() == b"vertices" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in vertices".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(vertices)
}

fn parse_displacement_triangles<R: BufRead>(
    parser: &mut XmlParser<R>,
) -> Result<Vec<DisplacementTriangle>> {
    let mut triangles = Vec::new();
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == b"triangle" => {
                let v1 = get_attribute_u32(&e, b"v1")?;
                let v2 = get_attribute_u32(&e, b"v2")?;
                let v3 = get_attribute_u32(&e, b"v3")?;
                let d1 = get_attribute_u32(&e, b"d1").ok();
                let d2 = get_attribute_u32(&e, b"d2").ok();
                let d3 = get_attribute_u32(&e, b"d3").ok();
                let p1 = get_attribute_u32(&e, b"p1").ok();
                let p2 = get_attribute_u32(&e, b"p2").ok();
                let p3 = get_attribute_u32(&e, b"p3").ok();
                let pid = get_attribute_u32(&e, b"pid").ok();

                triangles.push(DisplacementTriangle {
                    v1,
                    v2,
                    v3,
                    d1,
                    d2,
                    d3,
                    p1,
                    p2,
                    p3,
                    pid,
                });
            }
            Event::End(e) if e.local_name().as_ref() == b"triangles" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in triangles".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(triangles)
}

fn parse_normal_vectors<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Vec<NormalVector>> {
    let mut normals = Vec::new();
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == b"normvector" => {
                let nx = get_attribute_f32(&e, b"nx")?;
                let ny = get_attribute_f32(&e, b"ny")?;
                let nz = get_attribute_f32(&e, b"nz")?;
                normals.push(NormalVector { nx, ny, nz });
            }
            Event::End(e) if e.local_name().as_ref() == b"normvectors" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in normvectors".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(normals)
}

fn parse_disp2d_groups<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Vec<GradientVector>> {
    let mut gradients = Vec::new();
    loop {
        match parser.read_next_event()? {
            Event::Start(e) => {
                if e.local_name().as_ref() == b"disp2dgroup" {
                    // Parse gradient vectors within the group
                    loop {
                        match parser.read_next_event()? {
                            Event::Start(grad_e) | Event::Empty(grad_e)
                                if grad_e.local_name().as_ref() == b"gradient" =>
                            {
                                let gu = get_attribute_f32(&grad_e, b"gu")?;
                                let gv = get_attribute_f32(&grad_e, b"gv")?;
                                gradients.push(GradientVector { gu, gv });
                            }
                            Event::End(end_e) if end_e.local_name().as_ref() == b"disp2dgroup" => {
                                break
                            }
                            Event::Eof => {
                                return Err(Lib3mfError::Validation(
                                    "Unexpected EOF in disp2dgroup".to_string(),
                                ));
                            }
                            _ => {}
                        }
                    }
                }
            }
            Event::End(e) if e.local_name().as_ref() == b"disp2dgroups" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in disp2dgroups".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(gradients)
}

pub fn parse_displacement_2d<R: BufRead>(
    parser: &mut XmlParser<R>,
    id: ResourceId,
    path: String,
    channel: Channel,
    tile_style: TileStyle,
    filter: FilterMode,
    height: f32,
    offset: f32,
) -> Result<Displacement2D> {
    // Skip to closing tag (displacement2d is typically empty element)
    loop {
        match parser.read_next_event()? {
            Event::End(e) if e.local_name().as_ref() == b"displacement2d" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in displacement2d".to_string(),
                ))
            }
            _ => {}
        }
    }

    Ok(Displacement2D {
        id,
        path,
        channel,
        tile_style,
        filter,
        height,
        offset,
    })
}
