use crate::error::{Lib3mfError, Result};
use crate::model::Model;
use crate::writer::opc_writer::{write_content_types, write_relationships};
use std::io::{Seek, Write};
use zip::ZipWriter;
use zip::write::FileOptions;

impl Model {
    pub fn write<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let mut zip = ZipWriter::new(writer);

        let options: FileOptions<()> = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        // 1. [Content_Types].xml
        zip.start_file("[Content_Types].xml", options)
            .map_err(|e| Lib3mfError::Io(e.into()))?;
        write_content_types(&mut zip)?;

        // 2. _rels/.rels
        zip.start_file("_rels/.rels", options)
            .map_err(|e| Lib3mfError::Io(e.into()))?;
        write_relationships(&mut zip, "/3D/3dmodel.model")?;

        // 3. 3D/3dmodel.model
        zip.start_file("3D/3dmodel.model", options)
            .map_err(|e| Lib3mfError::Io(e.into()))?;
        self.write_xml(&mut zip)?;

        zip.finish().map_err(|e| Lib3mfError::Io(e.into()))?;

        Ok(())
    }
}
