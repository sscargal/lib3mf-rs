use crate::error::Result;
use crate::model::Model;
use std::io::{Seek, Write};

impl Model {
    pub fn write<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let package_writer = crate::writer::package_writer::PackageWriter::new(writer);
        package_writer.write(self)
    }
}
