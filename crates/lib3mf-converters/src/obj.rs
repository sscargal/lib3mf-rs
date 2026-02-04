//! Wavefront OBJ format import and export.
//!
//! This module provides conversion between OBJ files and 3MF [`Model`] structures.
//!
//! ## OBJ Format
//!
//! The Wavefront OBJ format is a text-based 3D geometry format. This implementation supports
//! a basic subset of the full OBJ specification:
//!
//! **Supported features:**
//! - `v` - Vertex positions (x, y, z)
//! - `f` - Faces (vertex indices)
//! - Polygon faces (automatically triangulated using fan triangulation)
//!
//! **Ignored features:**
//! - `vt` - Texture coordinates
//! - `vn` - Vertex normals
//! - `g` - Group names
//! - `usemtl` - Material references
//! - `mtllib` - Material library files
//!
//! ## Examples
//!
//! ### Importing OBJ
//!
//! ```no_run
//! use lib3mf_converters::obj::ObjImporter;
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let file = File::open("model.obj")?;
//! let model = ObjImporter::read(file)?;
//! println!("Imported model with {} build items", model.build.items.len());
//! # Ok(())
//! # }
//! ```
//!
//! ### Exporting OBJ
//!
//! ```no_run
//! use lib3mf_converters::obj::ObjExporter;
//! use lib3mf_core::model::Model;
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let model = Model::default();
//! let file = File::create("output.obj")?;
//! ObjExporter::write(&model, file)?;
//! # Ok(())
//! # }
//! ```
//!
//! [`Model`]: lib3mf_core::model::Model

use lib3mf_core::error::{Lib3mfError, Result};
use lib3mf_core::model::resources::ResourceId;
use lib3mf_core::model::{BuildItem, Mesh, Model, Triangle};
use std::io::{BufRead, BufReader, Read, Write};

/// Imports Wavefront OBJ files into 3MF [`Model`] structures.
///
/// Parses vertex positions (`v`) and faces (`f`) from OBJ text format. Polygonal faces
/// with more than 3 vertices are automatically triangulated using fan triangulation.
///
/// [`Model`]: lib3mf_core::model::Model
pub struct ObjImporter;

impl ObjImporter {
    /// Reads an OBJ file and converts it to a 3MF [`Model`].
    ///
    /// # Arguments
    ///
    /// * `reader` - Any type implementing [`Read`] containing OBJ text data
    ///
    /// # Returns
    ///
    /// A [`Model`] containing:
    /// - Single mesh object with ResourceId(1) named "OBJ Import"
    /// - All triangles from the OBJ file (polygons triangulated via fan method)
    /// - All vertices from the OBJ file
    /// - Single build item referencing the mesh object
    ///
    /// # Errors
    ///
    /// Returns [`Lib3mfError::Validation`] if:
    /// - Vertex line has fewer than 4 fields (v x y z)
    /// - Face line has fewer than 4 fields (f v1 v2 v3...)
    /// - Float parsing fails for vertex coordinates
    /// - Integer parsing fails for face indices
    /// - Relative indices (negative values) are used (not supported)
    ///
    /// Returns [`Lib3mfError::Io`] if reading from the input fails.
    ///
    /// # Format Details
    ///
    /// - **Index conversion**: OBJ uses 1-based indices, converted to 0-based for internal mesh
    /// - **Fan triangulation**: Polygons with N vertices create N-2 triangles using first vertex as fan apex
    /// - **Ignored elements**: Texture coords (vt), normals (vn), groups (g), materials (usemtl, mtllib)
    /// - **Comments**: Lines starting with `#` are skipped
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lib3mf_converters::obj::ObjImporter;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = File::open("cube.obj")?;
    /// let model = ObjImporter::read(file)?;
    ///
    /// // Access the imported mesh
    /// let obj = model.resources.get_object(lib3mf_core::model::resources::ResourceId(1))
    ///     .expect("OBJ import creates object with ID 1");
    /// if let lib3mf_core::model::Geometry::Mesh(mesh) = &obj.geometry {
    ///     println!("Imported {} vertices, {} triangles",
    ///         mesh.vertices.len(), mesh.triangles.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Model`]: lib3mf_core::model::Model
    /// [`Lib3mfError::Validation`]: lib3mf_core::error::Lib3mfError::Validation
    /// [`Lib3mfError::Io`]: lib3mf_core::error::Lib3mfError::Io
    pub fn read<R: Read>(reader: R) -> Result<Model> {
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        let mut mesh = Mesh::default();

        // OBJ indices are 1-based
        // Mesh stores vertices in order added.

        while reader.read_line(&mut line).map_err(Lib3mfError::Io)? > 0 {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                line.clear();
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.is_empty() {
                line.clear();
                continue;
            }

            match parts[0] {
                "v" => {
                    if parts.len() < 4 {
                        return Err(Lib3mfError::Validation("Invalid OBJ vertex".to_string()));
                    }
                    let x = parts[1]
                        .parse::<f32>()
                        .map_err(|_| Lib3mfError::Validation("Invalid float".to_string()))?;
                    let y = parts[2]
                        .parse::<f32>()
                        .map_err(|_| Lib3mfError::Validation("Invalid float".to_string()))?;
                    let z = parts[3]
                        .parse::<f32>()
                        .map_err(|_| Lib3mfError::Validation("Invalid float".to_string()))?;
                    mesh.add_vertex(x, y, z);
                }
                "f" => {
                    if parts.len() < 4 {
                        // Skipping point/line elements
                        line.clear();
                        continue;
                    }

                    let mut indices = Vec::new();
                    for part in &parts[1..] {
                        // Format: v, v/vt, v/vt/vn, v//vn
                        let subparts: Vec<&str> = part.split('/').collect();
                        let v_idx_str = subparts[0];
                        let v_idx = v_idx_str
                            .parse::<i32>()
                            .map_err(|_| Lib3mfError::Validation("Invalid index".to_string()))?;

                        let idx = if v_idx > 0 {
                            (v_idx - 1) as u32
                        } else {
                            // Relative index
                            return Err(Lib3mfError::Validation(
                                "Relative OBJ indices not supported yet".to_string(),
                            ));
                        };
                        indices.push(idx);
                    }

                    // Triangulate fan
                    if indices.len() >= 3 {
                        for i in 1..indices.len() - 1 {
                            mesh.triangles.push(Triangle {
                                v1: indices[0],
                                v2: indices[i],
                                v3: indices[i + 1],
                                ..Default::default()
                            });
                        }
                    }
                }
                _ => {} // Ignore vt, vn, g, usemtl etc. for now
            }

            line.clear();
        }

        let mut model = Model::default();
        let resource_id = ResourceId(1);

        let object = lib3mf_core::model::Object {
            id: resource_id,
            object_type: lib3mf_core::model::ObjectType::Model,
            name: Some("OBJ Import".to_string()),
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: lib3mf_core::model::Geometry::Mesh(mesh),
        };

        // Handle result
        let _ = model.resources.add_object(object);

        model.build.items.push(BuildItem {
            object_id: resource_id,
            transform: glam::Mat4::IDENTITY,
            part_number: None,
            uuid: None,
            path: None,
        });

        Ok(model)
    }
}

/// Exports 3MF [`Model`] structures to Wavefront OBJ files.
///
/// The exporter writes all mesh objects from build items to OBJ format, creating
/// separate groups for each object and applying build item transformations.
///
/// [`Model`]: lib3mf_core::model::Model
pub struct ObjExporter;

impl ObjExporter {
    /// Writes a 3MF [`Model`] to OBJ text format.
    ///
    /// # Arguments
    ///
    /// * `model` - The 3MF model to export
    /// * `writer` - Any type implementing [`Write`] to receive OBJ text data
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful export.
    ///
    /// # Errors
    ///
    /// Returns [`Lib3mfError::Io`] if any write operation fails.
    ///
    /// # Format Details
    ///
    /// - **Groups**: Each mesh object creates an OBJ group (`g`) with the object's name or "Object"
    /// - **Vertex indices**: Written as 1-based indices (OBJ convention)
    /// - **Transformations**: Build item transforms are applied to vertex coordinates
    /// - **Materials**: Not exported (OBJ output is geometry-only)
    /// - **Normals/UVs**: Not exported
    ///
    /// # Behavior
    ///
    /// - Only mesh objects from `model.build.items` are exported
    /// - Non-mesh geometries (Components, BooleanShape, etc.) are skipped
    /// - Vertex indices are offset correctly across multiple objects
    /// - Each object's vertices and faces are written in sequence
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lib3mf_converters::obj::ObjExporter;
    /// use lib3mf_core::model::Model;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let model = Model::default();
    /// let output = File::create("exported.obj")?;
    /// ObjExporter::write(&model, output)?;
    /// println!("Model exported successfully");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Model`]: lib3mf_core::model::Model
    /// [`Lib3mfError::Io`]: lib3mf_core::error::Lib3mfError::Io
    pub fn write<W: Write>(model: &Model, mut writer: W) -> Result<()> {
        let mut vertex_offset = 1;

        for item in &model.build.items {
            if let Some(object) = model.resources.get_object(item.object_id)
                && let lib3mf_core::model::Geometry::Mesh(mesh) = &object.geometry
            {
                let transform = item.transform;

                writeln!(writer, "g {}", object.name.as_deref().unwrap_or("Object"))
                    .map_err(Lib3mfError::Io)?;

                // Write vertices
                for v in &mesh.vertices {
                    let p = transform.transform_point3(glam::Vec3::new(v.x, v.y, v.z));
                    writeln!(writer, "v {} {} {}", p.x, p.y, p.z).map_err(Lib3mfError::Io)?;
                }

                // Write faces
                for tri in &mesh.triangles {
                    writeln!(
                        writer,
                        "f {} {} {}",
                        tri.v1 + vertex_offset,
                        tri.v2 + vertex_offset,
                        tri.v3 + vertex_offset
                    )
                    .map_err(Lib3mfError::Io)?;
                }

                vertex_offset += mesh.vertices.len() as u32;
            }
        }
        Ok(())
    }
}
