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

    // 2. Find the relationship with type 3D Model
    // 3MF Core Spec: http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel
    let model_rel_type = "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel";

    for rel in rels {
        if rel.rel_type == model_rel_type {
            // Target paths in OPC are often relative to the root if they start with /
            // or relative to the relations file location.
            // For root relationships, they are usually relative to root.
            let path = rel.target;
            if path.starts_with('/') {
                return Ok(path.trim_start_matches('/').to_string());
            }
            return Ok(path);
        }
    }

    Err(Lib3mfError::Validation(
        "No 3D model relationship found".to_string(),
    ))
}
