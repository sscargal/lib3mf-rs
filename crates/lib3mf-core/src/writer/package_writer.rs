use crate::error::{Lib3mfError, Result};
use crate::model::Package;
use crate::writer::opc_writer::{write_content_types, write_relationships};
use std::io::{Seek, Write};
use zip::ZipWriter;
use zip::write::FileOptions;

/// A writer that orchestrates the creation of a 3MF package (ZIP archive).
pub struct PackageWriter<W: Write + Seek> {
    zip: ZipWriter<W>,
    options: FileOptions<'static, ()>,
}

impl<W: Write + Seek> PackageWriter<W> {
    pub fn new(writer: W) -> Self {
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        Self {
            zip: ZipWriter::new(writer),
            options,
        }
    }

    pub fn write(mut self, package: &Package) -> Result<()> {
        // 1. Write Attachments (Textures, Thumbnails) from the main model
        // (In a true multi-part, attachments might be shared or part-specific,
        // but for now we aggregate them in the main model or handle them simply).
        for (path, data) in &package.main_model.attachments {
            let zip_path = path.trim_start_matches('/');
            self.zip
                .start_file(zip_path, self.options)
                .map_err(|e| Lib3mfError::Io(e.into()))?;
            self.zip.write_all(data).map_err(Lib3mfError::Io)?;
        }

        // 2. Write 3D Model parts
        let main_path = "3D/3dmodel.model";
        self.zip
            .start_file(main_path, self.options)
            .map_err(|e| Lib3mfError::Io(e.into()))?;
        package.main_model.write_xml(&mut self.zip)?;

        for (path, model) in &package.parts {
            self.zip
                .start_file(path.trim_start_matches('/'), self.options)
                .map_err(|e| Lib3mfError::Io(e.into()))?;
            model.write_xml(&mut self.zip)?;
        }

        // 3. Write Relationships (_rels/.rels and model relationships)
        // Global Relationships
        self.zip
            .start_file("_rels/.rels", self.options)
            .map_err(|e| Lib3mfError::Io(e.into()))?;
        write_relationships(&mut self.zip, &format!("/{}", main_path))?;

        // Model Relationships (e.g. 3D/_rels/3dmodel.model.rels)
        let mut model_rels = Vec::new();
        for path in package.main_model.attachments.keys() {
            if path.starts_with("3D/Textures/") || path.starts_with("/3D/Textures/") {
                let target = if path.starts_with('/') {
                    path.to_string()
                } else {
                    format!("/{}", path)
                };

                let id = format!("rel_tex_{}", model_rels.len());
                model_rels.push(crate::archive::opc::Relationship {
                    id,
                    rel_type: "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel/relationship/texture".to_string(),
                    target,
                    target_mode: "Internal".to_string(),
                });
            }
        }

        if !model_rels.is_empty() {
            self.zip
                .start_file("3D/_rels/3dmodel.model.rels", self.options)
                .map_err(|e| Lib3mfError::Io(e.into()))?;

            let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
            xml.push_str("<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n");
            for rel in model_rels {
                xml.push_str(&format!(
                    "  <Relationship Target=\"{}\" Id=\"{}\" Type=\"{}\" />\n",
                    rel.target, rel.id, rel.rel_type
                ));
            }
            xml.push_str("</Relationships>");

            self.zip
                .write_all(xml.as_bytes())
                .map_err(Lib3mfError::Io)?;
        }

        // 4. Write Content Types
        self.zip
            .start_file("[Content_Types].xml", self.options)
            .map_err(|e| Lib3mfError::Io(e.into()))?;
        write_content_types(&mut self.zip)?;

        self.zip.finish().map_err(|e| Lib3mfError::Io(e.into()))?;
        Ok(())
    }
}
