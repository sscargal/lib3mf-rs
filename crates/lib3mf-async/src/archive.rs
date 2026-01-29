use async_trait::async_trait;
use lib3mf_core::error::Result;

/// Trait for reading entries from an archive asynchronously.
#[async_trait]
pub trait AsyncArchiveReader: Send + Sync {
    /// Read the content of an entry by name.
    /// Returns the full content as a Vec<u8>.
    async fn read_entry(&mut self, name: &str) -> Result<Vec<u8>>;

    /// Check if an entry exists.
    async fn entry_exists(&mut self, name: &str) -> bool;

    /// List all entries in the archive.
    async fn list_entries(&mut self) -> Result<Vec<String>>;
}
