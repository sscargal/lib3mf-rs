//! Async ZIP archive implementation.
//!
//! This module provides [`AsyncZipArchive`], an async implementation of the [`AsyncArchiveReader`]
//! trait using the async-zip crate for non-blocking ZIP file access.
//!
//! ## Implementation Details
//!
//! - Uses `async-zip` with tokio compatibility layer (`tokio-util::compat`)
//! - Wraps readers in `BufReader` for efficient I/O
//! - Converts between tokio's `AsyncRead` and futures' `AsyncRead` traits
//!
//! ## Examples
//!
//! ```no_run
//! use lib3mf_async::zip::AsyncZipArchive;
//! use lib3mf_async::archive::AsyncArchiveReader;
//! use tokio::fs::File;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Open a 3MF file (ZIP archive)
//!     let file = File::open("model.3mf").await?;
//!     let mut archive = AsyncZipArchive::new(file).await?;
//!
//!     // Read an entry
//!     let model_rels = archive.read_entry("_rels/.rels").await?;
//!     println!("Relationships XML: {} bytes", model_rels.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! [`AsyncArchiveReader`]: crate::archive::AsyncArchiveReader

use crate::archive::AsyncArchiveReader;
use async_trait::async_trait;
use async_zip::StoredZipEntry;
use async_zip::tokio::read::seek::ZipFileReader;
use futures_lite::io::AsyncReadExt;
use lib3mf_core::error::{Lib3mfError, Result};
use tokio::io::{AsyncRead, AsyncSeek, BufReader};
use tokio_util::compat::TokioAsyncReadCompatExt;

/// Async ZIP archive reader implementing [`AsyncArchiveReader`].
///
/// This type wraps the async-zip crate's `ZipFileReader` and provides async access to ZIP
/// archive entries without blocking the tokio runtime.
///
/// # Type Parameters
///
/// * `R` - The underlying reader type, must implement:
///   - [`AsyncRead`]: For reading data
///   - [`AsyncSeek`]: For random access to ZIP entries
///   - [`Unpin`]: Required by tokio's async traits
///
/// Common types that satisfy these bounds:
/// - `tokio::fs::File`
/// - `std::io::Cursor<Vec<u8>>`
/// - `tokio::io::BufReader<File>`
///
/// [`AsyncArchiveReader`]: crate::archive::AsyncArchiveReader
pub struct AsyncZipArchive<R: AsyncRead + AsyncSeek + Unpin> {
    // ZipFileReader from tokio module is likely an alias: ZipFileReader<R> = Base<Compat<R>>.
    // So we pass BufReader<R> here, and it expands to Base<Compat<BufReader<R>>>.
    reader: ZipFileReader<BufReader<R>>,
}

impl<R: AsyncRead + AsyncSeek + Unpin> AsyncZipArchive<R> {
    /// Creates a new async ZIP archive reader.
    ///
    /// # Arguments
    ///
    /// * `reader` - Any async reader implementing `AsyncRead + AsyncSeek + Unpin`
    ///
    /// # Returns
    ///
    /// An initialized `AsyncZipArchive` ready to read entries.
    ///
    /// # Errors
    ///
    /// Returns [`Lib3mfError::Io`] if:
    /// - The ZIP file header cannot be read
    /// - The ZIP central directory is corrupt or missing
    /// - The file is not a valid ZIP archive
    ///
    /// # Implementation Notes
    ///
    /// - Wraps the reader in a `BufReader` for efficient I/O
    /// - Uses `tokio_util::compat` to bridge tokio and futures AsyncRead traits
    /// - Reads the ZIP central directory during construction
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lib3mf_async::zip::AsyncZipArchive;
    /// use tokio::fs::File;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let file = File::open("model.3mf").await?;
    ///     let archive = AsyncZipArchive::new(file).await?;
    ///     // Archive is ready to read entries
    ///     Ok(())
    /// }
    /// ```
    ///
    /// [`Lib3mfError::Io`]: lib3mf_core::error::Lib3mfError::Io
    pub async fn new(reader: R) -> Result<Self> {
        let buf_reader = BufReader::new(reader);
        let compat_reader = buf_reader.compat();
        // Construct the Base reader. since we use the Alias type to define the field,
        // we might need to cast or construct carefully.
        // Actually, if ZipFileReader is an alias, ZipFileReader::new is Base::new.
        // Base::new takes the inner reader (Compat<Buf>).
        // And returns Base<Compat<Buf>>.
        // This matches the Alias<Buf>.
        let zip = ZipFileReader::new(compat_reader)
            .await
            .map_err(|e| Lib3mfError::Io(std::io::Error::other(e.to_string())))?;
        Ok(Self { reader: zip })
    }
}

#[async_trait]
impl<R: AsyncRead + AsyncSeek + Unpin + Send + Sync> AsyncArchiveReader for AsyncZipArchive<R> {
    async fn read_entry(&mut self, name: &str) -> Result<Vec<u8>> {
        let entries = self.reader.file().entries();
        let index = entries
            .iter()
            .position(|e: &StoredZipEntry| e.filename().as_str().ok() == Some(name))
            .ok_or(Lib3mfError::ResourceNotFound(0))?;

        let mut reader = self
            .reader
            .reader_with_entry(index)
            .await
            .map_err(|e| Lib3mfError::Io(std::io::Error::other(e.to_string())))?;

        let mut buffer = Vec::new();
        // reader implements futures::io::AsyncRead.
        // We imported futures_lite::io::AsyncReadExt so read_to_end should work.
        reader
            .read_to_end(&mut buffer)
            .await
            .map_err(|e| Lib3mfError::Io(std::io::Error::other(e.to_string())))?;

        Ok(buffer)
    }

    async fn entry_exists(&mut self, name: &str) -> bool {
        self.reader
            .file()
            .entries()
            .iter()
            .any(|e: &StoredZipEntry| e.filename().as_str().ok() == Some(name))
    }

    async fn list_entries(&mut self) -> Result<Vec<String>> {
        let names = self
            .reader
            .file()
            .entries()
            .iter()
            .filter_map(|e: &StoredZipEntry| e.filename().as_str().map(|s| s.to_string()).ok())
            .collect();
        Ok(names)
    }
}
