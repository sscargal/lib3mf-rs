use crate::error::{Lib3mfError, Result};
use crate::model::{Build, BuildItem};
use crate::parser::component_parser::parse_transform;
use crate::parser::xml_parser::{XmlParser, get_attribute, get_attribute_u32};

use glam::Mat4;
use quick_xml::events::Event;
use std::io::BufRead;

pub fn parse_build<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Build> {
    let mut build = Build::default();

    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"item" => {
                let object_id = crate::model::ResourceId(get_attribute_u32(&e, b"objectid")?);
                let transform = if let Some(s) = get_attribute(&e, b"transform") {
                    parse_transform(&s)?
                } else {
                    Mat4::IDENTITY
                };

                let part_number = get_attribute(&e, b"partnumber");
                let uuid = crate::parser::xml_parser::get_attribute_uuid(&e)?;
                // Try "path" or "p:path"
                let path = get_attribute(&e, b"path").or_else(|| get_attribute(&e, b"p:path"));

                build.items.push(BuildItem {
                    object_id,
                    part_number,
                    uuid,
                    path,
                    transform,
                });
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

    Ok(build)
}
