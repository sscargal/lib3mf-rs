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

        // 2. Prepare Relationships (Textures, Thumbnails) for 3D Model
        // We do this BEFORE writing XML because objects need the Relationship ID for the 'thumbnail' attribute.
        let mut model_rels = Vec::new();
        let mut path_to_rel_id = std::collections::HashMap::new();

        // A. Collect Textures from Attachments
        for path in package.main_model.attachments.keys() {
            if path.starts_with("3D/Textures/") || path.starts_with("/3D/Textures/") {
                let target = if path.starts_with('/') {
                    path.to_string()
                } else {
                    format!("/{}", path)
                };

                // Deduplicate? For now, we assume 1:1 path to rel or just create distinct rels per path
                path_to_rel_id.entry(target.clone()).or_insert_with(|| {
                    let id = format!("rel_tex_{}", model_rels.len());
                    model_rels.push(crate::archive::opc::Relationship {
                        id: id.clone(),
                        rel_type: "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel/relationship/texture".to_string(),
                        target: target.clone(),
                        target_mode: "Internal".to_string(),
                    });
                    id
                });
            }
        }

        // B. Collect Object Thumbnails
        for obj in package.main_model.resources.iter_objects() {
            if let Some(thumb_path) = &obj.thumbnail {
                let target = if thumb_path.starts_with('/') {
                    thumb_path.clone()
                } else {
                    format!("/{}", thumb_path)
                };

                path_to_rel_id.entry(target.clone()).or_insert_with(|| {
                    let id = format!("rel_thumb_{}", model_rels.len());
                    model_rels.push(crate::archive::opc::Relationship {
                        id: id.clone(),
                        rel_type: "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel/relationship/thumbnail".to_string(),
                        target: target.clone(),
                        target_mode: "Internal".to_string(),
                    });
                    id
                });
            }
        }

        // 3. Write 3D Model parts
        let main_path = "3D/3dmodel.model";
        self.zip
            .start_file(main_path, self.options)
            .map_err(|e| Lib3mfError::Io(e.into()))?;

        // Pass the relationship map to write_xml so it can write attributes
        package
            .main_model
            .write_xml(&mut self.zip, Some(&path_to_rel_id))?;

        for (path, model) in &package.parts {
            self.zip
                .start_file(path.trim_start_matches('/'), self.options)
                .map_err(|e| Lib3mfError::Io(e.into()))?;
            // TODO: Support relationships for other parts if they have their own thumbnails
            model.write_xml(&mut self.zip, None)?;
        }

        // 4. Write Relationships (_rels/.rels and model relationships)
        // Global Relationships
        self.zip
            .start_file("_rels/.rels", self.options)
            .map_err(|e| Lib3mfError::Io(e.into()))?;

        let package_thumb = package
            .main_model
            .attachments
            .keys()
            .find(|k| k == &"Metadata/thumbnail.png" || k == &"/Metadata/thumbnail.png")
            .map(|k| {
                if k.starts_with('/') {
                    k.clone()
                } else {
                    format!("/{}", k)
                }
            });

        write_relationships(
            &mut self.zip,
            &format!("/{}", main_path),
            package_thumb.as_deref(),
        )?;

        // Model Relationships (e.g. 3D/_rels/3dmodel.model.rels)
        // Merge existing relationships with new texture/thumbnail relationships
        let model_rels_path = "3D/_rels/3dmodel.model.rels";

        // Start with existing relationships if available
        let mut all_model_rels = package
            .main_model
            .existing_relationships
            .get(model_rels_path)
            .cloned()
            .unwrap_or_default();

        // Add new texture/thumbnail relationships
        // Use a HashSet to track existing IDs to avoid duplicates
        let existing_ids: std::collections::HashSet<String> =
            all_model_rels.iter().map(|r| r.id.clone()).collect();

        for rel in model_rels {
            if !existing_ids.contains(&rel.id) {
                all_model_rels.push(rel);
            }
        }

        // Write merged relationships if any exist
        if !all_model_rels.is_empty() {
            self.zip
                .start_file(model_rels_path, self.options)
                .map_err(|e| Lib3mfError::Io(e.into()))?;

            let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
            xml.push_str("<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n");
            for rel in all_model_rels {
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
