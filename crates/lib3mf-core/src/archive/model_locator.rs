use crate::archive::{ArchiveReader, opc::parse_relationships};
use crate::error::{Lib3mfError, Result};

/// Locates the path of the 3D model file within the archive.
pub fn find_model_path<R: ArchiveReader>(archive: &mut R) -> Result<String> {
    // 1. Read _rels/.rels
    if !archive.entry_exists("_rels/.rels") {
        return Err(Lib3mfError::InvalidStructure(
            "Missing _rels/.rels".to_string(),
        ));
    }

    let rels_data = archive.read_entry("_rels/.rels")?;
    let rels = parse_relationships(&rels_data)?;

    // 2. Validate relationships and find the 3D Model
    // 3MF Core Spec: http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel
    let model_rel_type = "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel";
    let thumbnail_rel_type =
        "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail";
    let print_ticket_rel_type = "http://schemas.microsoft.com/3dmanufacturing/2013/01/printticket";

    let mut model_path: Option<String> = None;
    let mut model_count = 0;
    let mut print_ticket_count = 0;

    for rel in rels {
        // Check for external relationships - these are not allowed in 3MF
        if rel.target_mode.to_lowercase() == "external" {
            return Err(Lib3mfError::Validation(format!(
                "External relationships are not allowed in 3MF packages. Found external relationship with target '{}'",
                rel.target
            )));
        }

        if rel.rel_type == model_rel_type {
            model_count += 1;
            if model_count > 1 {
                return Err(Lib3mfError::Validation(
                    "Multiple 3D model part relationships found. Only one 3D model part is allowed per package".to_string(),
                ));
            }

            // Target paths in OPC are often relative to the root if they start with /
            // or relative to the relations file location.
            // For root relationships, they are usually relative to root.
            let path = rel.target.clone();
            model_path = Some(if path.starts_with('/') {
                path.trim_start_matches('/').to_string()
            } else {
                path
            });
        } else if rel.rel_type == thumbnail_rel_type {
            // Validate that thumbnail file exists
            let thumb_path = if rel.target.starts_with('/') {
                rel.target.trim_start_matches('/').to_string()
            } else {
                rel.target.clone()
            };
            if !archive.entry_exists(&thumb_path) {
                return Err(Lib3mfError::Validation(format!(
                    "Thumbnail file '{}' referenced in relationships does not exist in package",
                    thumb_path
                )));
            }
        } else if rel.rel_type == print_ticket_rel_type {
            print_ticket_count += 1;
            if print_ticket_count > 1 {
                return Err(Lib3mfError::Validation(
                    "Multiple print ticket relationships found. Only one print ticket is allowed per package".to_string(),
                ));
            }
        }
    }

    model_path.ok_or_else(|| Lib3mfError::Validation("No 3D model relationship found".to_string()))
}
