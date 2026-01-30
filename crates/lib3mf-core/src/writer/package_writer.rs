use crate::error::{Lib3mfError, Result};
use crate::model::Model;
use crate::writer::opc_writer::{write_content_types, write_relationships};
use std::io::{Seek, Write};
use zip::ZipWriter;
use zip::write::FileOptions;

/// A writer that orchestrates the creation of a 3MF package (ZIP archive).
///
/// It handles:
/// - Writing the model stream(s)
/// - Writing attachments (Textures, Thumbnails)
/// - Generatirng the correct OPC relationships (.rels)
/// - Generating [Content_Types].xml
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

    pub fn write(mut self, model: &Model) -> Result<()> {
        // 1. Write Attachments (Textures, Thumbnails)
        for (path, data) in &model.attachments {
            // Ensure path doesn't start with / for zip crate consistency if needed,
            // but 3MF paths generally don't in ZIP.
            let zip_path = path.trim_start_matches('/');
            self.zip
                .start_file(zip_path, self.options)
                .map_err(|e| Lib3mfError::Io(e.into()))?;
            self.zip.write_all(data).map_err(Lib3mfError::Io)?;
        }

        // 2. Write 3D Model
        // TODO: Support multi-part if model has split flag or partitioning logic.
        // For now, always write to the standard root path.
        let model_path = "3D/3dmodel.model";
        self.zip
            .start_file(model_path, self.options)
            .map_err(|e| Lib3mfError::Io(e.into()))?;
        model.write_xml(&mut self.zip)?;

        // 3. Write Relationships (_rels/.rels and model relationships)
        // Global Relationships
        self.zip
            .start_file("_rels/.rels", self.options)
            .map_err(|e| Lib3mfError::Io(e.into()))?;
        // Root points to the 3D Model
        write_relationships(&mut self.zip, &format!("/{}", model_path))?;

        // Model Relationships (e.g. 3D/_rels/3dmodel.model.rels)
        // If we had textures, we would need to link them here.
        // Placeholder for phase 26 logic expansion
        // Model Relationships (e.g. 3D/_rels/3dmodel.model.rels)
        // If we have attachments that are resources like Textures, we need to link them.
        // For now, assume anything in 3D/Textures/ is a texture relationship.
        let mut model_rels = Vec::new();
        for path in model.attachments.keys() {
            // Very basic heuristic for now: if path starts with 3D/Textures or Metadata/thumbnail
            if path.starts_with("3D/Textures/") || path.starts_with("/3D/Textures/") {
                // Determine ID (filename?) or generate one.
                // 3MF usually uses implicit relationships or explicit texture references in XML.
                // But the relationship file connects the model part to the texture part.
                let target = if path.starts_with('/') {
                    path.to_string()
                } else {
                    format!("/{}", path)
                };

                // Add texture relationship
                // Schema: http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel/relationship/texture
                // ID: Arbitrary, maybe "rel1", "rel2", etc.
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
            // We need a helper to write a custom list of relationships
            // Reusing opc_writer logic but passing the list.
            // For now, let's manually write or expose a helper in opc_writer.
            // Let's manually write simplest XML for now or check if opc_writer exposes a generic writer.
            // Checking opc_writer... assumes hardcoded "StartPart" usually.

            // Simplest: Write raw XML for these specific rels
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
