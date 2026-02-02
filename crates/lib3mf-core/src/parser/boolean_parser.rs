use crate::error::{Lib3mfError, Result};
use crate::model::{BooleanOperation, BooleanOperationType, BooleanShape, ResourceId};
use crate::parser::component_parser::parse_transform;
use crate::parser::xml_parser::{get_attribute, get_attribute_u32, XmlParser};
use quick_xml::events::Event;
use std::io::BufRead;

/// Parse a <booleanshape> element into a BooleanShape structure.
///
/// A booleanshape defines geometry through constructive solid geometry (CSG)
/// operations on referenced objects.
///
/// Per Boolean Operations Extension v1.1.1:
/// - Requires base objectid attribute (parsed by caller)
/// - Optional transform on base (parsed by caller)
/// - Optional p:path for external references (parsed by caller)
/// - Contains nested <boolean> elements defining operations
pub fn parse_boolean_shape<R: BufRead>(
    parser: &mut XmlParser<R>,
    base_object_id: ResourceId,
    base_transform: glam::Mat4,
    base_path: Option<String>,
) -> Result<BooleanShape> {
    let mut operations = Vec::new();

    // Parse nested <boolean> elements
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) => {
                let local_name = e.local_name();
                if local_name.as_ref() == b"boolean" {
                    let operation = parse_boolean_operation(&e)?;
                    operations.push(operation);
                }
            }
            Event::End(e) => {
                let local_name = e.local_name();
                if local_name.as_ref() == b"booleanshape" {
                    break;
                }
            }
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in booleanshape".to_string(),
                ));
            }
            _ => {}
        }
    }

    Ok(BooleanShape {
        base_object_id,
        base_transform,
        base_path,
        operations,
    })
}

/// Parse a single <boolean> operation element.
fn parse_boolean_operation(elem: &quick_xml::events::BytesStart) -> Result<BooleanOperation> {
    let object_id = ResourceId(get_attribute_u32(elem, b"objectid")?);

    // Parse operation type (defaults to "union" per spec)
    let operation_type = if let Some(op_str) = get_attribute(elem, b"operation") {
        match op_str.as_ref() {
            "union" => BooleanOperationType::Union,
            "difference" => BooleanOperationType::Difference,
            "intersection" => BooleanOperationType::Intersection,
            _ => BooleanOperationType::Union, // Default on unknown
        }
    } else {
        BooleanOperationType::Union
    };

    let transform = if let Some(s) = get_attribute(elem, b"transform") {
        parse_transform(&s)?
    } else {
        glam::Mat4::IDENTITY
    };

    let path = get_attribute(elem, b"path")
        .or_else(|| get_attribute(elem, b"p:path"))
        .map(|s| s.into_owned());

    Ok(BooleanOperation {
        operation_type,
        object_id,
        transform,
        path,
    })
}
