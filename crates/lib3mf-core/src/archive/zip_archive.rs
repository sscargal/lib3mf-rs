use crate::archive::ArchiveReader;
use crate::error::{Lib3mfError, Result};
use std::io::{Read, Seek};
use zip::ZipArchive;

#[derive(Debug)]
pub struct ZipArchiver<R> {
    archive: ZipArchive<R>,
}

impl<R: Read + Seek> ZipArchiver<R> {
    pub fn new(reader: R) -> Result<Self> {
        Ok(Self {
            archive: ZipArchive::new(reader).map_err(|e| Lib3mfError::Io(e.into()))?,
        })
    }
}

impl<R: Read + Seek> Read for ZipArchiver<R> {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        // ZipArchiver itself doesn't implement Read in a meaningful way for the whole archive,
        // but we need to satisfy the trait bounds if we passed it around.
        // For now, this is a placeholder or we might remove Read from ArchiveReader if not strictly needed.
        // However, ArchiveReader inherits Read + Seek to allow flexibility.
        // A better design might be to separate the "Opener" from the "Reader".
        // Let's implement dummy Read/Seek for the Archiver wrapper or rethink the trait.

        // Actually, looking at the design, ArchiveReader requires Read+Seek.
        // This implies the *underlying* reader has it, but the Archiver *is* the manager.
        // Let's implement pass-through if we have access, or just return 0.
        Ok(0)
    }
}

impl<R: Read + Seek> Seek for ZipArchiver<R> {
    fn seek(&mut self, _pos: std::io::SeekFrom) -> std::io::Result<u64> {
        Ok(0)
    }
}

impl<R: Read + Seek> ArchiveReader for ZipArchiver<R> {
    fn read_entry(&mut self, name: &str) -> Result<Vec<u8>> {
        let name = name.trim_start_matches('/');
        let mut file = self.archive.by_name(name).map_err(|_| {
            Lib3mfError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, name))
        })?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn entry_exists(&mut self, name: &str) -> bool {
        let name = name.trim_start_matches('/');
        self.archive.by_name(name).is_ok()
    }

    fn list_entries(&mut self) -> Result<Vec<String>> {
        Ok(self.archive.file_names().map(|s| s.to_string()).collect())
    }
}
