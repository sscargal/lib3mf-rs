//! Archive layer for reading and writing 3MF container files.
//!
//! This module implements the OPC (Open Packaging Conventions) container layer that 3MF is built on.
//! 3MF files are ZIP archives containing XML model files, textures, thumbnails, and metadata, organized
//! according to OPC relationship and content type conventions.
//!
//! ## Architecture
//!
//! The archive layer provides:
//!
//! 1. **Trait-based abstraction**: [`ArchiveReader`] and [`ArchiveWriter`] traits decouple the parser
//!    from the underlying ZIP implementation, allowing different backends (file, memory, async, etc.)
//! 2. **OPC relationship discovery**: The [`find_model_path`] function traverses `_rels/.rels` files
//!    to locate the main 3D model XML file within the archive.
//! 3. **Default ZIP implementation**: [`ZipArchiver`] provides a standard file-based ZIP backend.
//!
//! ## Typical Usage
//!
//! Opening and reading a 3MF file:
//!
//! ```no_run
//! use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Open the 3MF ZIP archive
//! let file = File::open("model.3mf")?;
//! let mut archiver = ZipArchiver::new(file)?;
//!
//! // Discover the main model XML path via OPC relationships
//! let model_path = find_model_path(&mut archiver)?;
//! // Typically returns "3D/3dmodel.model" or similar
//!
//! // Read the model XML content
//! let model_xml = archiver.read_entry(&model_path)?;
//!
//! // Read attachments (textures, thumbnails, etc.)
//! if archiver.entry_exists("Metadata/thumbnail.png") {
//!     let thumbnail = archiver.read_entry("Metadata/thumbnail.png")?;
//! }
//!
//! // List all entries
//! let entries = archiver.list_entries()?;
//! for entry in entries {
//!     println!("Archive contains: {}", entry);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## OPC Relationship Discovery
//!
//! The [`find_model_path`] function implements the OPC discovery algorithm:
//!
//! 1. Read `_rels/.rels` (package-level relationships)
//! 2. Find relationship with type `http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel`
//! 3. Extract target path (e.g., `/3D/3dmodel.model`)
//! 4. Normalize path (remove leading `/`)
//!
//! This allows 3MF files to have different internal structures while remaining conformant to the spec.
//!
//! ## ArchiveReader Trait
//!
//! The [`ArchiveReader`] trait provides three core operations:
//!
//! - [`read_entry`](ArchiveReader::read_entry): Read file content by path
//! - [`entry_exists`](ArchiveReader::entry_exists): Check if a path exists
//! - [`list_entries`](ArchiveReader::list_entries): Enumerate all archive contents
//!
//! Implementations must also satisfy `Read + Seek` for compatibility with the ZIP crate.
//!
//! ## ArchiveWriter Trait
//!
//! The [`ArchiveWriter`] trait provides a single operation:
//!
//! - [`write_entry`](ArchiveWriter::write_entry): Write data to a path in the archive
//!
//! Implementations handle compression, content type registration, and relationship generation.

pub mod model_locator;
pub mod opc;
pub mod zip_archive;

pub use model_locator::*;
// pub use opc::*; // Clippy says unused
pub use zip_archive::*;

use crate::error::Result;
use std::io::{Read, Seek};

/// Trait for reading entries from an archive (ZIP).
///
/// This trait abstracts over different ZIP backend implementations, allowing the parser to work
/// with files, in-memory buffers, async I/O, or custom storage.
///
/// # Requirements
///
/// Implementations must also implement `Read + Seek` for compatibility with the underlying ZIP library.
///
/// # Examples
///
/// ```no_run
/// use lib3mf_core::archive::{ArchiveReader, ZipArchiver};
/// use std::fs::File;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let file = File::open("model.3mf")?;
/// let mut archive = ZipArchiver::new(file)?;
///
/// // Check if entry exists before reading
/// if archive.entry_exists("3D/3dmodel.model") {
///     let content = archive.read_entry("3D/3dmodel.model")?;
///     println!("Model XML size: {} bytes", content.len());
/// }
///
/// // List all entries
/// for entry in archive.list_entries()? {
///     println!("Found: {}", entry);
/// }
/// # Ok(())
/// # }
/// ```
pub trait ArchiveReader: Read + Seek {
    /// Read the content of an entry by name.
    ///
    /// # Parameters
    ///
    /// - `name`: Path to the entry within the archive (e.g., `"3D/3dmodel.model"`, `"Metadata/thumbnail.png"`)
    ///
    /// # Returns
    ///
    /// The binary content of the entry as a `Vec<u8>`.
    ///
    /// # Errors
    ///
    /// Returns [`Lib3mfError::Io`](crate::error::Lib3mfError::Io) if the entry doesn't exist or can't be read.
    fn read_entry(&mut self, name: &str) -> Result<Vec<u8>>;

    /// Check if an entry exists.
    ///
    /// # Parameters
    ///
    /// - `name`: Path to the entry within the archive
    ///
    /// # Returns
    ///
    /// `true` if the entry exists, `false` otherwise.
    fn entry_exists(&mut self, name: &str) -> bool;

    /// List all entries in the archive.
    ///
    /// # Returns
    ///
    /// A vector of entry paths (e.g., `["3D/3dmodel.model", "Metadata/thumbnail.png", "_rels/.rels"]`)
    ///
    /// # Errors
    ///
    /// Returns [`Lib3mfError::Io`](crate::error::Lib3mfError::Io) if the archive can't be read.
    fn list_entries(&mut self) -> Result<Vec<String>>;
}

/// Trait for writing entries to an archive.
///
/// This trait abstracts over different ZIP backend implementations for creating 3MF files.
pub trait ArchiveWriter {
    /// Write data to an entry.
    ///
    /// # Parameters
    ///
    /// - `name`: Path for the entry within the archive (e.g., `"3D/3dmodel.model"`)
    /// - `data`: Binary content to write
    ///
    /// # Errors
    ///
    /// Returns [`Lib3mfError::Io`](crate::error::Lib3mfError::Io) if the entry can't be written.
    fn write_entry(&mut self, name: &str, data: &[u8]) -> Result<()>;
}
