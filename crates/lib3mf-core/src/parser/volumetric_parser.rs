use crate::error::{Lib3mfError, Result};
use crate::model::{ResourceId, VolumetricLayer, VolumetricRef, VolumetricStack};
use crate::parser::xml_parser::{XmlParser, get_attribute, get_attribute_f32, get_attribute_u32};
use quick_xml::events::Event;
use std::io::BufRead;

pub fn parse_volumetric_stack_content<R: BufRead>(
    parser: &mut XmlParser<R>,
    id: ResourceId,
    _z_bottom: f32, // Unused in stack def usually? Or can be base. SliceStack uses it.
) -> Result<VolumetricStack> {
    let mut stack = VolumetricStack {
        id,
        ..Default::default()
    };

    loop {
        match parser.read_next_event()? {
            Event::Start(e) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"layer" => {
                        let z = get_attribute_f32(&e, b"z").unwrap_or(0.0);
                        // Content path: usually "path" or "src" attribute
                        // Or implicitly the content? Spec usually uses "path".
                        let path = get_attribute(&e, b"path").unwrap_or_default();

                        let end_tag = e.name().as_ref().to_vec();
                        stack.layers.push(VolumetricLayer {
                            z_height: z,
                            content_path: path.into_owned(),
                        });
                        parser.read_to_end(&end_tag)?;
                    }
                    b"volumetricref" => {
                        let stack_id =
                            get_attribute_u32(&e, b"volumetricstackid").map(ResourceId)?;
                        let path = get_attribute(&e, b"path").unwrap_or_default();
                        let end_tag = e.name().as_ref().to_vec();
                        stack.refs.push(VolumetricRef {
                            stack_id,
                            path: path.into_owned(),
                        });
                        parser.read_to_end(&end_tag)?;
                    }
                    _ => {}
                }
            }
            Event::End(e) if e.local_name().as_ref() == b"volumetricstack" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in volumetricstack".to_string(),
                ));
            }
            _ => {}
        }
    }

    Ok(stack)
}
