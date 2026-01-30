use crate::archive::AsyncArchiveReader;
use crate::zip::AsyncZipArchive;
use lib3mf_core::archive::opc::{Relationship, parse_relationships};
use lib3mf_core::error::{Lib3mfError, Result};
use lib3mf_core::model::Model;
use lib3mf_core::parser::model_parser::parse_model;
use std::io::Cursor;
use std::path::Path;
use tokio::fs::File;

/// Asynchronously loads a 3MF model from a file.
pub async fn load_model_async<P: AsRef<Path>>(path: P) -> Result<Model> {
    let file = File::open(path).await.map_err(Lib3mfError::Io)?;
    let mut archive = AsyncZipArchive::new(file).await?;

    // 1. Read [Content_Types].xml (Optional but good for robustness)
    // For simplicity of this Phase, we might follow strict 3MF discovery via _rels

    // 2. Read _rels/.rels to find the Start Part (3D Model)
    let rels_path = "_rels/.rels";
    let rels_data = archive.read_entry(rels_path).await?;
    let rels = parse_rels(&rels_data)?;

    // Find the 3D Model part (Type = http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel)
    let model_rel = rels
        .iter()
        .find(|r| {
            r.target_mode == "Internal"
                && r.rel_type == "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"
        })
        .ok_or(Lib3mfError::InvalidStructure(
            "No 3D Model part found in .rels".to_string(),
        ))?;

    let model_path = clean_path(&model_rel.target);

    // 3. Read Model Part
    let model_data = archive.read_entry(&model_path).await?;

    // 4. Parse Model (Synchronous - CPU bound but usually fast enough or run via spawn_blocking if needed)
    // For Buffer & Parse strategy, we stick to current thread for now unless blocking is an issue.
    // Parsing ~100MB XML might block for seconds. For true async app, spawn_blocking is better.
    // But keeping it simple for now.

    // Use spawn_blocking for CPU bound parsing
    let model = tokio::task::spawn_blocking(move || {
        let cursor = Cursor::new(model_data);
        parse_model(cursor)
    })
    .await
    .map_err(|e| Lib3mfError::Validation(format!("Join error: {}", e)))??; // JoinError + Parse Result

    Ok(model)
}

fn parse_rels(data: &[u8]) -> Result<Vec<Relationship>> {
    parse_relationships(data)
}

fn clean_path(path: &str) -> String {
    let p = path.trim_start_matches('/');
    p.to_string()
}
