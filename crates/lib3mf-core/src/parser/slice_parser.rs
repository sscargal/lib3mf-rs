use crate::error::{Lib3mfError, Result};
use crate::model::{Polygon, Segment, Slice, SliceRef, SliceStack, Vertex2D};
use crate::parser::xml_parser::{XmlParser, get_attribute, get_attribute_f32, get_attribute_u32};
use quick_xml::events::Event;
use std::borrow::Cow;
use std::io::BufRead;

// parse_slice_stack removed (used content version directly)

pub fn parse_slice_stack_content<R: BufRead>(
    parser: &mut XmlParser<R>,
    id: crate::model::ResourceId,
    z_bottom: f32,
) -> Result<SliceStack> {
    let mut slices = Vec::new();
    let mut refs = Vec::new();

    loop {
        match parser.read_next_event()? {
            Event::Start(e) => match e.local_name().as_ref() {
                b"slice" => {
                    let z_top = get_attribute_f32(&e, b"ztop")?;
                    let slice = parse_slice(parser, z_top)?;
                    slices.push(slice);
                }
                b"sliceref" => {
                    // Usually empty element <sliceref ... />?
                    // Start or Empty.
                    let stack_id =
                        crate::model::ResourceId(get_attribute_u32(&e, b"slicestackid")?);
                    let path = get_attribute(&e, b"slicepath").map(|s: Cow<str>| s.into_owned()).unwrap_or_default();
                    refs.push(SliceRef {
                        slice_stack_id: stack_id,
                        slice_path: path,
                    });
                }
                _ => {}
            },
            Event::Empty(e) => {
                if e.local_name().as_ref() == b"sliceref" {
                    let stack_id =
                        crate::model::ResourceId(get_attribute_u32(&e, b"slicestackid")?);
                    let path = get_attribute(&e, b"slicepath").map(|s: Cow<str>| s.into_owned()).unwrap_or_default();
                    refs.push(SliceRef {
                        slice_stack_id: stack_id,
                        slice_path: path,
                    });
                }
            }
            Event::End(e) if e.local_name().as_ref() == b"slicestack" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in slicestack".to_string(),
                ));
            }
            _ => {}
        }
    }

    Ok(SliceStack {
        id,
        z_bottom,
        slices,
        refs,
    })
}

fn parse_slice<R: BufRead>(parser: &mut XmlParser<R>, z_top: f32) -> Result<Slice> {
    let mut vertices = Vec::new();
    let mut polygons = Vec::new();

    loop {
        match parser.read_next_event()? {
            Event::Start(e) => match e.local_name().as_ref() {
                b"vertices" => {
                    vertices = parse_slice_vertices(parser)?;
                }
                b"polygon" => {
                    // Polygon has attributes? "start"
                    // Wait, polygon has `start` attribute indicating start vertex index?
                    // "startsegment" is my field name. Spec says "start".
                    let start = get_attribute_u32(&e, b"start").unwrap_or(0);
                    let segments = parse_polygon_segments(parser)?;
                    polygons.push(Polygon {
                        start_segment: start,
                        segments,
                    });
                }
                _ => {}
            },
            Event::End(e) if e.local_name().as_ref() == b"slice" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in slice".to_string(),
                ));
            }
            _ => {}
        }
    }

    Ok(Slice {
        z_top,
        vertices,
        polygons,
    })
}

fn parse_slice_vertices<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Vec<Vertex2D>> {
    let mut vertices = Vec::new();
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == b"vertex" => {
                let x = get_attribute_f32(&e, b"x")?;
                let y = get_attribute_f32(&e, b"y")?;
                vertices.push(Vertex2D { x, y });
            }
            Event::End(e) if e.local_name().as_ref() == b"vertices" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in slice vertices".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(vertices)
}

fn parse_polygon_segments<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Vec<Segment>> {
    let mut segments = Vec::new();
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.local_name().as_ref() == b"segment" => {
                let v2 = get_attribute_u32(&e, b"v2")?;
                let p1 = get_attribute_u32(&e, b"p1").ok();
                let p2 = get_attribute_u32(&e, b"p2").ok();
                let pid = get_attribute_u32(&e, b"pid")
                    .map(crate::model::ResourceId)
                    .ok();

                segments.push(Segment { v2, p1, p2, pid });
            }
            Event::End(e) if e.local_name().as_ref() == b"polygon" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in polygon segments".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(segments)
}
