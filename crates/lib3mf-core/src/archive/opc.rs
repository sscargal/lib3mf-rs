use crate::error::{Lib3mfError, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};

/// Represents an OPC Relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub rel_type: String,
    pub target: String,
}

/// Represents an OPC Content Type override or default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Default { extension: String, content_type: String },
    Override { part_name: String, content_type: String },
}

/// Parses relationship file (e.g., _rels/.rels).
pub fn parse_relationships(xml_content: &[u8]) -> Result<Vec<Relationship>> {
    let mut reader = Reader::from_reader(xml_content);
    reader.config_mut().trim_text(true);

    let mut rels = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                if e.name().as_ref() == b"Relationship" {
                    let mut id = String::new();
                    let mut rel_type = String::new();
                    let mut target = String::new();

                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| Lib3mfError::Validation(e.to_string()))?;
                        match attr.key.as_ref() {
                            b"Id" => id = String::from_utf8_lossy(&attr.value).to_string(),
                            b"Type" => rel_type = String::from_utf8_lossy(&attr.value).to_string(),
                            b"Target" => target = String::from_utf8_lossy(&attr.value).to_string(),
                            _ => {}
                        }
                    }
                    rels.push(Relationship { id, rel_type, target });
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Lib3mfError::Validation(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    Ok(rels)
}

/// Parses [Content_Types].xml.
pub fn parse_content_types(xml_content: &[u8]) -> Result<Vec<ContentType>> {
    let mut reader = Reader::from_reader(xml_content);
    reader.config_mut().trim_text(true);

    let mut types = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"Default" => {
                        let mut extension = String::new();
                        let mut content_type = String::new();
                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| Lib3mfError::Validation(e.to_string()))?;
                            match attr.key.as_ref() {
                                b"Extension" => extension = String::from_utf8_lossy(&attr.value).to_string(),
                                b"ContentType" => content_type = String::from_utf8_lossy(&attr.value).to_string(),
                                _ => {}
                            }
                        }
                        types.push(ContentType::Default { extension, content_type });
                    }
                    b"Override" => {
                        let mut part_name = String::new();
                        let mut content_type = String::new();
                        for attr in e.attributes() {
                            let attr = attr.map_err(|e| Lib3mfError::Validation(e.to_string()))?;
                            match attr.key.as_ref() {
                                b"PartName" => part_name = String::from_utf8_lossy(&attr.value).to_string(),
                                b"ContentType" => content_type = String::from_utf8_lossy(&attr.value).to_string(),
                                _ => {}
                            }
                        }
                        types.push(ContentType::Override { part_name, content_type });
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Lib3mfError::Validation(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    Ok(types)
}
