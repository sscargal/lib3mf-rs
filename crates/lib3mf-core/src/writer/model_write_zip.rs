use crate::error::Result;
use crate::model::{Model, Package};
use std::io::{Seek, Write};

impl Model {
    pub fn write<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let package_writer = crate::writer::package_writer::PackageWriter::new(writer);
        let package = Package::new(self.clone());
        package_writer.write(&package)
    }
}
