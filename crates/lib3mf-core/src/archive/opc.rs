use crate::error::{Lib3mfError, Result};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::{Deserialize, Serialize};

/// Bambu Studio OPC relationship type constants.
///
/// These URIs appear in `_rels/*.rels` files within Bambu Studio 3MF archives
/// to identify vendor-specific relationships to thumbnails and embedded G-code.
///
/// # Example
///
/// ```ignore
/// use lib3mf_core::archive::opc::bambu_rel_types;
///
/// let is_bambu_thumbnail = rel.rel_type == bambu_rel_types::COVER_THUMBNAIL_MIDDLE;
/// ```
pub mod bambu_rel_types {
    /// Relationship type for the medium-size cover thumbnail image.
    ///
    /// Targets a PNG or similar image file used as the model's display thumbnail
    /// in Bambu Studio's file browser.
    pub const COVER_THUMBNAIL_MIDDLE: &str =
        "http://schemas.bambulab.com/package/2021/cover-thumbnail-middle";

    /// Relationship type for the small cover thumbnail image.
    ///
    /// Targets a small PNG image suitable for grid/icon views in Bambu Studio.
    pub const COVER_THUMBNAIL_SMALL: &str =
        "http://schemas.bambulab.com/package/2021/cover-thumbnail-small";

    /// Relationship type for embedded G-code.
    ///
    /// Targets a `.gcode` file embedded in the archive. When present, the file
    /// contains pre-sliced G-code that can be sent directly to a Bambu printer.
    pub const GCODE: &str = "http://schemas.bambulab.com/package/2021/gcode";
}

/// Represents an OPC Relationship.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Unique identifier for this relationship within the package.
    pub id: String,
    /// The relationship type URI (e.g., `http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel`).
    pub rel_type: String,
    /// Target part path (e.g., `/3D/3dmodel.model`).
    pub target: String,
    /// Target mode: `"Internal"` for package-relative paths, `"External"` for absolute URIs.
    pub target_mode: String,
}

/// Represents an OPC Content Type override or default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    /// A default content type mapped to a file extension.
    Default {
        /// File extension (without the leading dot, e.g., `"model"`).
        extension: String,
        /// MIME content type string (e.g., `"application/vnd.ms-package.3dmanufacturing-3dmodel+xml"`).
        content_type: String,
    },
    /// An explicit content type override for a specific part path.
    Override {
        /// Package-relative path of the part (e.g., `/3D/3dmodel.model`).
        part_name: String,
        /// MIME content type string for this specific part.
        content_type: String,
    },
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
                    let mut target_mode = "Internal".to_string(); // Default

                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| Lib3mfError::Validation(e.to_string()))?;
                        match attr.key.as_ref() {
                            b"Id" => id = String::from_utf8_lossy(&attr.value).to_string(),
                            b"Type" => rel_type = String::from_utf8_lossy(&attr.value).to_string(),
                            b"Target" => target = String::from_utf8_lossy(&attr.value).to_string(),
                            b"TargetMode" => {
                                target_mode = String::from_utf8_lossy(&attr.value).to_string()
                            }
                            _ => {}
                        }
                    }
                    rels.push(Relationship {
                        id,
                        rel_type,
                        target,
                        target_mode,
                    });
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

/// Parses `[Content_Types].xml`.
pub fn parse_content_types(xml_content: &[u8]) -> Result<Vec<ContentType>> {
    let mut reader = Reader::from_reader(xml_content);
    reader.config_mut().trim_text(true);

    let mut types = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Default" => {
                    let mut extension = String::new();
                    let mut content_type = String::new();
                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| Lib3mfError::Validation(e.to_string()))?;
                        match attr.key.as_ref() {
                            b"Extension" => {
                                extension = String::from_utf8_lossy(&attr.value).to_string()
                            }
                            b"ContentType" => {
                                content_type = String::from_utf8_lossy(&attr.value).to_string()
                            }
                            _ => {}
                        }
                    }
                    types.push(ContentType::Default {
                        extension,
                        content_type,
                    });
                }
                b"Override" => {
                    let mut part_name = String::new();
                    let mut content_type = String::new();
                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| Lib3mfError::Validation(e.to_string()))?;
                        match attr.key.as_ref() {
                            b"PartName" => {
                                part_name = String::from_utf8_lossy(&attr.value).to_string()
                            }
                            b"ContentType" => {
                                content_type = String::from_utf8_lossy(&attr.value).to_string()
                            }
                            _ => {}
                        }
                    }
                    types.push(ContentType::Override {
                        part_name,
                        content_type,
                    });
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(Lib3mfError::Validation(e.to_string())),
            _ => {}
        }
        buf.clear();
    }

    Ok(types)
}
