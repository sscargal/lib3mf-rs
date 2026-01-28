use crate::error::{Lib3mfError, Result};
use crate::model::{Component, Components};
use crate::parser::xml_parser::{get_attribute, get_attribute_u32, XmlParser};
use glam::Mat4;
use quick_xml::events::Event;
use std::io::BufRead;
use std::str::FromStr;

pub fn parse_components<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Components> {
    let mut components = Vec::new();

    loop {
        match parser.next()? {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"component" => {
                let object_id = crate::model::ResourceId(get_attribute_u32(&e, b"objectid")?);
                let uuid = crate::parser::xml_parser::get_attribute_uuid(&e)?;
                let transform = if let Some(s) = get_attribute(&e, b"transform") {
                    parse_transform(&s)?
                } else {
                    Mat4::IDENTITY
                };
                components.push(Component {
                    object_id,
                    uuid,
                    transform,
                });
            }
            Event::End(e) if e.name().as_ref() == b"components" => break,
            Event::Eof => return Err(Lib3mfError::Validation("Unexpected EOF in components".to_string())),
            _ => {}
        }
    }

    Ok(Components { components })
}

pub fn parse_transform(s: &str) -> Result<Mat4> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 12 {
        return Err(Lib3mfError::Validation(format!(
            "Invalid transform matrix: expected 12 values, got {}",
            parts.len()
        )));
    }

    let p: Result<Vec<f32>> = parts
        .iter()
        .map(|v| {
            f32::from_str(v).map_err(|_| Lib3mfError::Validation(format!("Invalid float in transform: {}", v)))
        })
        .collect();
    let p = p?;

    // 3MF uses column-major order 4x3 matrix, last column is 0,0,0,1
    Ok(Mat4::from_cols_array(&[
        p[0], p[1], p[2], 0.0,
        p[3], p[4], p[5], 0.0,
        p[6], p[7], p[8], 0.0,
        p[9], p[10], p[11], 1.0,
    ]))
}
