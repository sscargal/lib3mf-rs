use crate::archive::AsyncArchiveReader;
use async_trait::async_trait;
use async_zip::StoredZipEntry;
use async_zip::tokio::read::seek::ZipFileReader;
use futures_lite::io::AsyncReadExt;
use lib3mf_core::error::{Lib3mfError, Result};
use tokio::io::{AsyncRead, AsyncSeek, BufReader};
use tokio_util::compat::TokioAsyncReadCompatExt;

/// Async ZIP archive reader.
pub struct AsyncZipArchive<R: AsyncRead + AsyncSeek + Unpin> {
    // ZipFileReader from tokio module is likely an alias: ZipFileReader<R> = Base<Compat<R>>.
    // So we pass BufReader<R> here, and it expands to Base<Compat<BufReader<R>>>.
    reader: ZipFileReader<BufReader<R>>,
}

impl<R: AsyncRead + AsyncSeek + Unpin> AsyncZipArchive<R> {
    /// Opens a ZIP archive from a reader.
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
