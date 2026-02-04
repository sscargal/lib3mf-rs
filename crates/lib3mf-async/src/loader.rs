//! High-level async 3MF model loading.
//!
//! This module provides the [`load_model_async`] function, which orchestrates the complete
//! async loading pipeline from file path to parsed [`Model`].
//!
//! ## Loading Pipeline
//!
//! The function performs these steps:
//!
//! 1. **Async file open**: Opens the 3MF file using `tokio::fs::File::open()`
//! 2. **ZIP initialization**: Creates an [`AsyncZipArchive`] and reads the central directory
//! 3. **OPC relationship parsing**: Reads `_rels/.rels` to find the main model part
//! 4. **Async entry reading**: Reads the model XML data from the archive
//! 5. **Spawn-blocked parsing**: Offloads CPU-bound XML parsing to a blocking thread pool
//!
//! ## Performance Characteristics
//!
//! - **I/O operations**: Non-blocking, multiple files can be loaded concurrently
//! - **XML parsing**: Runs on blocking thread pool via `tokio::task::spawn_blocking`
//! - **Memory**: Entire model XML is loaded into memory before parsing
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```no_run
//! use lib3mf_async::loader::load_model_async;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let model = load_model_async("cube.3mf").await?;
//!     println!("Loaded {} build items", model.build.items.len());
//!     Ok(())
//! }
//! ```
//!
//! ### Concurrent Loading
//!
//! ```no_run
//! use lib3mf_async::loader::load_model_async;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load multiple files concurrently
//!     let (model1, model2, model3) = tokio::try_join!(
//!         load_model_async("file1.3mf"),
//!         load_model_async("file2.3mf"),
//!         load_model_async("file3.3mf"),
//!     )?;
//!
//!     println!("Loaded {} models concurrently", 3);
//!     Ok(())
//! }
//! ```
//!
//! [`Model`]: lib3mf_core::model::Model
//! [`AsyncZipArchive`]: crate::zip::AsyncZipArchive

use crate::archive::AsyncArchiveReader;
use crate::zip::AsyncZipArchive;
use lib3mf_core::archive::opc::{Relationship, parse_relationships};
use lib3mf_core::error::{Lib3mfError, Result};
use lib3mf_core::model::Model;
use lib3mf_core::parser::model_parser::parse_model;
use std::io::Cursor;
use std::path::Path;
use tokio::fs::File;

/// Asynchronously loads a 3MF model from a file path.
///
/// This is the primary entry point for async 3MF loading. It handles all aspects of
/// loading: file I/O, ZIP archive access, OPC relationship parsing, and XML model parsing.
///
/// # Arguments
///
/// * `path` - Path to the 3MF file (any type implementing `AsRef<Path>`)
///
/// # Returns
///
/// A fully parsed [`Model`] containing all resources, build items, and metadata.
///
/// # Errors
///
/// Returns [`Lib3mfError::Io`] if:
/// - File cannot be opened
/// - ZIP archive cannot be read
/// - Archive entries cannot be read
///
/// Returns [`Lib3mfError::InvalidStructure`] if:
/// - `_rels/.rels` file is missing or malformed
/// - No 3D model relationship is found (must have type `http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel`)
/// - Model part path is invalid
///
/// Returns parsing errors from [`parse_model`] if the model XML is malformed.
///
/// # Implementation Details
///
/// ## Async vs Blocking
///
/// - **Async operations**: File open, ZIP directory reading, entry reading
/// - **Blocking operations**: XML parsing (offloaded to `tokio::task::spawn_blocking`)
///
/// The function uses `spawn_blocking` for XML parsing because it's CPU-bound work that would
/// otherwise block the tokio executor. This allows other async tasks to progress while
/// parsing happens on a dedicated thread pool.
///
/// ## OPC Discovery
///
/// The function follows the Open Packaging Conventions (OPC) standard to discover the
/// main model part:
///
/// 1. Reads `_rels/.rels` (package-level relationships)
/// 2. Finds relationship with type ending in `/3dmodel`
/// 3. Reads the target model part (typically `/3D/3dmodel.model`)
///
/// # Examples
///
/// ```no_run
/// use lib3mf_async::loader::load_model_async;
/// use lib3mf_core::validation::ValidationLevel;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Load a 3MF file
///     let model = load_model_async("complex_model.3mf").await?;
///
///     // Validate the loaded model
///     let report = model.validate(ValidationLevel::Standard);
///     if report.has_errors() {
///         eprintln!("Model has validation errors:");
///         for item in &report.items {
///             eprintln!("  - {}", item.message);
///         }
///     }
///
///     // Process the model
///     println!("Objects: {}", model.resources.iter_objects().count());
///     println!("Build items: {}", model.build.items.len());
///
///     Ok(())
/// }
/// ```
///
/// [`Model`]: lib3mf_core::model::Model
/// [`Lib3mfError::Io`]: lib3mf_core::error::Lib3mfError::Io
/// [`Lib3mfError::InvalidStructure`]: lib3mf_core::error::Lib3mfError::InvalidStructure
/// [`parse_model`]: lib3mf_core::parser::model_parser::parse_model
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
