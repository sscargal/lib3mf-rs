use crate::error::Result;
use crate::model::{Model, Package};
use std::io::{Seek, Write};

impl Model {
    /// Serializes the model to a complete 3MF package (ZIP archive) using the given writer.
    pub fn write<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let package_writer = crate::writer::package_writer::PackageWriter::new(writer);
        let package = Package::new(self.clone());
        package_writer.write(&package)
    }
}
