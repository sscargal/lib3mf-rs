//! Async archive reader abstraction layer.
//!
//! This module provides the [`AsyncArchiveReader`] trait, which defines the interface for
//! asynchronously reading entries from archive containers (ZIP files). It mirrors the synchronous
//! [`lib3mf_core::archive::ArchiveReader`] trait but with async methods.
//!
//! ## Design
//!
//! The trait abstracts over different async archive implementations, allowing the 3MF loader
//! to work with any async archive backend. Currently, [`AsyncZipArchive`] is the primary
//! implementation using the async-zip crate.
//!
//! ## Examples
//!
//! ```no_run
//! use lib3mf_async::archive::AsyncArchiveReader;
//! use lib3mf_async::zip::AsyncZipArchive;
//! use tokio::fs::File;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let file = File::open("model.3mf").await?;
//!     let mut archive = AsyncZipArchive::new(file).await?;
//!
//!     // Check if an entry exists
//!     if archive.entry_exists("_rels/.rels").await {
//!         // Read the entry
//!         let data = archive.read_entry("_rels/.rels").await?;
//!         println!("Read {} bytes", data.len());
//!     }
//!
//!     // List all entries
//!     let entries = archive.list_entries().await?;
//!     println!("Archive contains {} entries", entries.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! [`AsyncZipArchive`]: crate::zip::AsyncZipArchive

use async_trait::async_trait;
use lib3mf_core::error::Result;

/// Trait for reading entries from an archive asynchronously.
///
/// This trait provides async methods for reading archive contents without blocking the executor.
/// It is analogous to [`lib3mf_core::archive::ArchiveReader`] but with async/await semantics.
///
/// # Trait Bounds
///
/// Implementors must be `Send + Sync` to allow the archive reader to be used across async task
/// boundaries. This is required because async functions may be sent between threads in the tokio
/// runtime.
///
/// # Implementors
///
/// - [`AsyncZipArchive`]: ZIP archive reader using async-zip
///
/// [`AsyncZipArchive`]: crate::zip::AsyncZipArchive
#[async_trait]
pub trait AsyncArchiveReader: Send + Sync {
    /// Reads the content of an archive entry by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The entry path within the archive (e.g., "_rels/.rels", "3D/3dmodel.model")
    ///
    /// # Returns
    ///
    /// The full content of the entry as a `Vec<u8>`.
    ///
    /// # Errors
    ///
    /// Returns [`Lib3mfError::ResourceNotFound`] if the entry does not exist.
    ///
    /// Returns [`Lib3mfError::Io`] if reading fails.
    ///
    /// [`Lib3mfError::ResourceNotFound`]: lib3mf_core::error::Lib3mfError::ResourceNotFound
    /// [`Lib3mfError::Io`]: lib3mf_core::error::Lib3mfError::Io
    async fn read_entry(&mut self, name: &str) -> Result<Vec<u8>>;

    /// Checks if an archive entry exists.
    ///
    /// # Arguments
    ///
    /// * `name` - The entry path within the archive
    ///
    /// # Returns
    ///
    /// `true` if the entry exists, `false` otherwise.
    async fn entry_exists(&mut self, name: &str) -> bool;

    /// Lists all entry names in the archive.
    ///
    /// # Returns
    ///
    /// A vector of all entry path names in the archive.
    ///
    /// # Errors
    ///
    /// Returns [`Lib3mfError::Io`] if the archive cannot be read.
    ///
    /// [`Lib3mfError::Io`]: lib3mf_core::error::Lib3mfError::Io
    async fn list_entries(&mut self) -> Result<Vec<String>>;
}
