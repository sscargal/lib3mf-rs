pub mod model_locator;
pub mod opc;
pub mod zip_archive;

pub use model_locator::*;
pub use opc::*;
pub use zip_archive::*;

use crate::error::Result;
use std::io::{Read, Seek};

/// Trait for reading entries from an archive (ZIP).
pub trait ArchiveReader: Read + Seek {
    /// Read the content of an entry by name.
    fn read_entry(&mut self, name: &str) -> Result<Vec<u8>>;

    /// Check if an entry exists.
    fn entry_exists(&mut self, name: &str) -> bool;

    /// List all entries in the archive.
    fn list_entries(&mut self) -> Result<Vec<String>>;
}

/// Trait for writing entries to an archive.
pub trait ArchiveWriter {
    /// Write data to an entry.
    fn write_entry(&mut self, name: &str, data: &[u8]) -> Result<()>;
}
