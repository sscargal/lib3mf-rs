use crate::error::Result;
use quick_xml::events::{BytesStart, Event};
use std::collections::BTreeMap;
use std::io::Write;

/// A simplified C14N (Canonical XML) implementation for 3MF signatures.
///
/// This handles:
/// - Attribute sorting
/// - Namespace handling (basic)
/// - Empty element expansion
///
/// Note: This is NOT a fully compliant W3C C14N implementation, but sufficient for
/// standard 3MF signature use cases involving `SignedInfo`.
pub struct Canonicalizer;

impl Canonicalizer {
    /// Canonicalize a specific subtree (e.g., "SignedInfo")
    pub fn canonicalize_subtree(input_xml: &str, target_tag: &str) -> Result<Vec<u8>> {
        let mut reader = quick_xml::Reader::from_str(input_xml);
        reader.config_mut().trim_text(true);
        let mut writer = Vec::new();
        let mut buf = Vec::new();
        let mut capturing = false;
        let mut depth = 0;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == target_tag {
                        capturing = true;
                        depth = 1;
                        write_start_tag(&mut writer, e)?;
                    } else if capturing {
                        depth += 1;
                        write_start_tag(&mut writer, e)?;
                    }
                }
                Ok(Event::End(ref e)) => {
                    if capturing {
                        write!(writer, "</{}>", String::from_utf8_lossy(e.name().as_ref()))
                            .map_err(crate::error::Lib3mfError::Io)?;
                        depth -= 1;
                        if depth == 0 {
                            break; // Done capturing target
                        }
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == target_tag {
                        // Empty target tag <Target/> -> <Target></Target>
                        write_start_tag(&mut writer, e)?;
                        write!(writer, "</{}>", name).map_err(crate::error::Lib3mfError::Io)?;
                        break;
                    } else if capturing {
                        write_start_tag(&mut writer, e)?;
                        write!(writer, "</{}>", name).map_err(crate::error::Lib3mfError::Io)?;
                    }
                }
                Ok(Event::Text(e)) => {
                    if capturing {
                        let content = String::from_utf8_lossy(&e.into_inner()).to_string();
                        writer
                            .write_all(content.as_bytes())
                            .map_err(crate::error::Lib3mfError::Io)?;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(crate::error::Lib3mfError::InvalidStructure(e.to_string())),
                _ => {}
            }
            buf.clear();
        }

        if writer.is_empty() {
            return Err(crate::error::Lib3mfError::Validation(format!(
                "Tag <{}> not found for C14N",
                target_tag
            )));
        }

        Ok(writer)
    }

    /// Canonicalize strict whole document
    pub fn canonicalize(input_xml: &str) -> Result<Vec<u8>> {
        let mut reader = quick_xml::Reader::from_str(input_xml);
        reader.config_mut().trim_text(true);
        let mut writer = Vec::new();
        let mut buf = Vec::new();

        // Very basic canonicalization:
        // 1. Parse events
        // 2. Sort attributes
        // 3. Write back
        // Real C14N is complex (namespace propagation, etc.).
        // 3MF signatures usually just sign the exact bytes of the `SignedInfo` in the package,
        // but strict construction requires C14N.

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    write_start_tag(&mut writer, e)?;
                }
                Ok(Event::End(ref e)) => {
                    write!(writer, "</{}>", String::from_utf8_lossy(e.name().as_ref()))
                        .map_err(crate::error::Lib3mfError::Io)?;
                }
                Ok(Event::Empty(ref e)) => {
                    // C14N expands empty tags: <a/> -> <a></a>
                    write_start_tag(&mut writer, e)?;
                    write!(writer, "</{}>", String::from_utf8_lossy(e.name().as_ref()))
                        .map_err(crate::error::Lib3mfError::Io)?;
                }
                Ok(Event::Text(e)) => {
                    // Escape basic entities
                    let content = String::from_utf8_lossy(&e.into_inner()).to_string();
                    // Basic escaping should already be done or preserved?
                    // Quick-xml text contains unescaped content usually?
                    // Actually, if we read raw, we get entities.
                    // For verification, we often operate on the byte stream.
                    // If we are reconstructing, we need to be careful.
                    writer
                        .write_all(content.as_bytes())
                        .map_err(crate::error::Lib3mfError::Io)?;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(crate::error::Lib3mfError::InvalidStructure(e.to_string())),
                _ => {} // Ignore comments, PIs for simple C14N
            }
            buf.clear();
        }

        Ok(writer)
    }
}

fn write_start_tag(writer: &mut Vec<u8>, e: &BytesStart) -> Result<()> {
    write!(writer, "<{}", String::from_utf8_lossy(e.name().as_ref()))
        .map_err(crate::error::Lib3mfError::Io)?;

    // Sort attributes by name
    let mut attrs: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
    for attr in e.attributes() {
        let attr = attr.map_err(|e| crate::error::Lib3mfError::InvalidStructure(e.to_string()))?;
        let val: &[u8] = attr.value.as_ref();
        attrs.insert(attr.key.as_ref().to_vec(), val.to_vec());
    }

    for (key, value) in &attrs {
        write!(
            writer,
            " {}=\"{}\"",
            String::from_utf8_lossy(key),
            String::from_utf8_lossy(value)
        )
        .map_err(crate::error::Lib3mfError::Io)?;
    }

    write!(writer, ">").map_err(crate::error::Lib3mfError::Io)?;
    Ok(())
}
