//! Wavefront OBJ format import and export.
//!
//! This module provides conversion between OBJ files and 3MF [`Model`] structures.
//!
//! ## OBJ Format
//!
//! The Wavefront OBJ format is a text-based 3D geometry format. This implementation supports:
//!
//! **Supported features:**
//! - `v` - Vertex positions (x, y, z)
//! - `f` - Faces (vertex indices, with automatic fan triangulation for polygons)
//! - `g` / `o` - Group/object directives (each creates a separate 3MF Object)
//! - `usemtl` - Material assignment (maps to per-triangle `pid`/`p1`/`p2`/`p3`)
//! - `mtllib` - Material library file reference (parsed via [`mtl`](crate::mtl) module)
//!
//! **Ignored features:**
//! - `vt` - Texture coordinates
//! - `vn` - Vertex normals
//!
//! ## Material Import
//!
//! When using [`ObjImporter::read_from_path`], the importer resolves `mtllib` directives
//! relative to the OBJ file's directory. MTL `Kd` (diffuse color) maps to 3MF
//! [`BaseMaterial`] display colors. Materials are collected into a single
//! [`BaseMaterialsGroup`] resource.
//!
//! When using [`ObjImporter::read`], no MTL resolution is possible and materials
//! are not imported (geometry-only mode for backward compatibility).
//!
//! ## Examples
//!
//! ### Importing OBJ with materials
//!
//! ```no_run
//! use lib3mf_converters::obj::ObjImporter;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let model = ObjImporter::read_from_path(Path::new("model.obj"))?;
//! println!("Imported model with {} build items", model.build.items.len());
//! # Ok(())
//! # }
//! ```
//!
//! ### Importing OBJ (geometry only)
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
//! [`BaseMaterial`]: lib3mf_core::model::BaseMaterial
//! [`BaseMaterialsGroup`]: lib3mf_core::model::BaseMaterialsGroup

use crate::mtl;
use lib3mf_core::error::{Lib3mfError, Result};
use lib3mf_core::model::resources::ResourceId;
use lib3mf_core::model::{
    BaseMaterial, BaseMaterialsGroup, BuildItem, Color, Mesh, Model, Object, ObjectType, Triangle,
    Vertex,
};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;

/// Default gray color for undefined materials.
const DEFAULT_GRAY: Color = Color {
    r: 128,
    g: 128,
    b: 128,
    a: 255,
};

// ---------------------------------------------------------------------------
// Intermediate representation for OBJ parsing
// ---------------------------------------------------------------------------

/// A face parsed from OBJ, storing global 0-based vertex indices.
struct ObjFace {
    indices: Vec<u32>,
    material_name: Option<String>,
}

/// A group/object parsed from OBJ.
struct ObjGroup {
    name: Option<String>,
    faces: Vec<ObjFace>,
}

/// Complete intermediate representation of a parsed OBJ file.
struct ObjIntermediate {
    global_vertices: Vec<(f32, f32, f32)>,
    groups: Vec<ObjGroup>,
    mtllib: Option<String>,
    had_explicit_group: bool,
}

/// Imports Wavefront OBJ files into 3MF [`Model`] structures.
///
/// Supports two modes:
/// - [`read_from_path`](Self::read_from_path): Full import with MTL material resolution
/// - [`read`](Self::read): Geometry-only import (backward compatible, no materials)
///
/// [`Model`]: lib3mf_core::model::Model
pub struct ObjImporter;

impl ObjImporter {
    /// Reads an OBJ file from a filesystem path, resolving MTL files relative to it.
    ///
    /// This is the recommended entry point for OBJ import. It resolves `mtllib`
    /// directives relative to the OBJ file's parent directory, parses materials,
    /// splits groups/objects into separate 3MF Objects, and assigns per-triangle
    /// material properties.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the OBJ file on disk
    ///
    /// # Returns
    ///
    /// A [`Model`] containing:
    /// - One [`BaseMaterialsGroup`] (if materials are referenced)
    /// - One [`Object`] per OBJ group/object (or single Object if no groups)
    /// - Per-triangle `pid`/`p1`/`p2`/`p3` material assignment
    /// - One [`BuildItem`] per Object
    ///
    /// # Errors
    ///
    /// Returns errors for I/O failures or invalid OBJ syntax (see [`read`](Self::read)).
    ///
    /// [`Model`]: lib3mf_core::model::Model
    pub fn read_from_path(path: &Path) -> Result<Model> {
        let dir = path.parent().unwrap_or(Path::new("."));
        let file = std::fs::File::open(path).map_err(Lib3mfError::Io)?;
        let intermediate = Self::parse_obj(BufReader::new(file))?;

        // Resolve MTL file
        let materials = if let Some(ref mtl_filename) = intermediate.mtllib {
            let mtl_path = dir.join(mtl_filename);
            mtl::parse_mtl_file(&mtl_path)
        } else {
            HashMap::new()
        };

        Self::build_model(intermediate, &materials)
    }

    /// Reads an OBJ file and converts it to a 3MF [`Model`].
    ///
    /// This is the backward-compatible entry point. No MTL file resolution is
    /// performed -- material directives (`usemtl`, `mtllib`) are ignored.
    /// Group directives (`g`, `o`) are also ignored for full backward compatibility.
    ///
    /// For material-aware import, use [`read_from_path`](Self::read_from_path).
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
        let intermediate = Self::parse_obj(BufReader::new(reader))?;
        // Backward-compatible: no materials, collapse all groups into one object
        Self::build_model_compat(intermediate)
    }

    /// Parse OBJ text into the intermediate representation.
    fn parse_obj<R: BufRead>(mut reader: R) -> Result<ObjIntermediate> {
        let mut global_vertices: Vec<(f32, f32, f32)> = Vec::new();
        let mut groups: Vec<ObjGroup> = Vec::new();
        let mut current_group = ObjGroup {
            name: None,
            faces: Vec::new(),
        };
        let mut current_material: Option<String> = None;
        let mut mtllib: Option<String> = None;
        let mut had_explicit_group = false;

        let mut line = String::new();

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
                        return Err(Lib3mfError::Validation(
                            "Invalid OBJ vertex".to_string(),
                        ));
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
                    global_vertices.push((x, y, z));
                }
                "f" => {
                    if parts.len() < 4 {
                        // Skip point/line elements
                        line.clear();
                        continue;
                    }

                    let mut indices = Vec::new();
                    for part in &parts[1..] {
                        // Format: v, v/vt, v/vt/vn, v//vn
                        let subparts: Vec<&str> = part.split('/').collect();
                        let v_idx = subparts[0]
                            .parse::<i32>()
                            .map_err(|_| Lib3mfError::Validation("Invalid index".to_string()))?;

                        let idx = if v_idx > 0 {
                            (v_idx - 1) as u32
                        } else {
                            return Err(Lib3mfError::Validation(
                                "Relative OBJ indices not supported yet".to_string(),
                            ));
                        };
                        indices.push(idx);
                    }

                    // Fan-triangulate
                    if indices.len() >= 3 {
                        for i in 1..indices.len() - 1 {
                            current_group.faces.push(ObjFace {
                                indices: vec![indices[0], indices[i], indices[i + 1]],
                                material_name: current_material.clone(),
                            });
                        }
                    }
                }
                "g" | "o" => {
                    had_explicit_group = true;
                    // Flush current group if it has faces
                    if !current_group.faces.is_empty() {
                        groups.push(current_group);
                    }
                    let name = if parts.len() >= 2 {
                        Some(parts[1..].join(" "))
                    } else {
                        None
                    };
                    current_group = ObjGroup {
                        name,
                        faces: Vec::new(),
                    };
                }
                "usemtl" => {
                    if parts.len() >= 2 {
                        current_material = Some(parts[1..].join(" "));
                    }
                }
                "mtllib" => {
                    if parts.len() >= 2 {
                        mtllib = Some(parts[1..].join(" "));
                    }
                }
                _ => {} // Ignore vt, vn, comments, etc.
            }

            line.clear();
        }

        // Flush last group
        if !current_group.faces.is_empty() {
            groups.push(current_group);
        }

        Ok(ObjIntermediate {
            global_vertices,
            groups,
            mtllib,
            had_explicit_group,
        })
    }

    /// Build a 3MF Model from the intermediate representation with material support.
    fn build_model(
        intermediate: ObjIntermediate,
        materials_map: &HashMap<String, mtl::MtlMaterial>,
    ) -> Result<Model> {
        let mut model = Model::default();

        if intermediate.groups.is_empty() {
            return Ok(model);
        }

        // Collect all referenced material names across all groups
        let mut referenced_materials: Vec<String> = Vec::new();
        let mut material_seen: HashMap<String, u32> = HashMap::new();
        for group in &intermediate.groups {
            for face in &group.faces {
                if let Some(ref mat_name) = face.material_name
                    && !material_seen.contains_key(mat_name)
                {
                    let idx = referenced_materials.len() as u32;
                    material_seen.insert(mat_name.clone(), idx);
                    referenced_materials.push(mat_name.clone());
                }
            }
        }

        let has_materials = !referenced_materials.is_empty();
        let mut next_id: u32 = 1;

        // Create BaseMaterialsGroup if materials are referenced
        let materials_group_id = if has_materials {
            let group_id = ResourceId(next_id);
            next_id += 1;

            let mut base_materials = Vec::new();
            for mat_name in &referenced_materials {
                let base_mat = if let Some(mtl_mat) = materials_map.get(mat_name) {
                    BaseMaterial {
                        name: mtl_mat.name.clone(),
                        display_color: mtl_mat.display_color,
                    }
                } else {
                    // Undefined material -- warn and use gray
                    eprintln!(
                        "Warning: undefined material '{}', using default gray",
                        mat_name
                    );
                    BaseMaterial {
                        name: mat_name.clone(),
                        display_color: DEFAULT_GRAY,
                    }
                };
                base_materials.push(base_mat);
            }

            model.resources.add_base_materials(BaseMaterialsGroup {
                id: group_id,
                materials: base_materials,
            })?;

            Some(group_id)
        } else {
            None
        };

        // Determine if we're in single-object backward-compat mode
        let single_object_mode =
            !intermediate.had_explicit_group && intermediate.groups.len() == 1;

        if single_object_mode && !has_materials {
            // Full backward compatibility: single object with ResourceId(1), no materials
            let group = &intermediate.groups[0];
            let mesh = Self::build_mesh_full(&intermediate.global_vertices, group, None, None);

            let resource_id = ResourceId(next_id);
            let object = Object {
                id: resource_id,
                object_type: ObjectType::Model,
                name: Some("OBJ Import".to_string()),
                part_number: None,
                uuid: None,
                pid: None,
                pindex: None,
                thumbnail: None,
                geometry: lib3mf_core::model::Geometry::Mesh(mesh),
            };
            model.resources.add_object(object)?;
            model.build.items.push(BuildItem {
                object_id: resource_id,
                transform: glam::Mat4::IDENTITY,
                part_number: None,
                uuid: None,
                path: None,
                printable: None,
            });
        } else {
            // Multi-object or single-object-with-materials path
            for group in &intermediate.groups {
                let obj_id = ResourceId(next_id);
                next_id += 1;

                let mesh = Self::build_mesh_full(
                    &intermediate.global_vertices,
                    group,
                    materials_group_id,
                    Some(&material_seen),
                );

                let name = group
                    .name
                    .clone()
                    .unwrap_or_else(|| "OBJ Import".to_string());

                let object = Object {
                    id: obj_id,
                    object_type: ObjectType::Model,
                    name: Some(name),
                    part_number: None,
                    uuid: None,
                    pid: None,
                    pindex: None,
                    thumbnail: None,
                    geometry: lib3mf_core::model::Geometry::Mesh(mesh),
                };
                model.resources.add_object(object)?;
                model.build.items.push(BuildItem {
                    object_id: obj_id,
                    transform: glam::Mat4::IDENTITY,
                    part_number: None,
                    uuid: None,
                    path: None,
                    printable: None,
                });
            }
        }

        Ok(model)
    }

    /// Build a 3MF Model in backward-compatible mode (no materials, no group splitting).
    fn build_model_compat(intermediate: ObjIntermediate) -> Result<Model> {
        let mut model = Model::default();

        // Collapse all groups into a single mesh
        let mut mesh = Mesh::default();
        for &(x, y, z) in &intermediate.global_vertices {
            mesh.add_vertex(x, y, z);
        }
        for group in &intermediate.groups {
            for face in &group.faces {
                if face.indices.len() == 3 {
                    mesh.triangles.push(Triangle {
                        v1: face.indices[0],
                        v2: face.indices[1],
                        v3: face.indices[2],
                        ..Default::default()
                    });
                }
            }
        }

        if mesh.vertices.is_empty() && mesh.triangles.is_empty() {
            // Still return a model with a single empty object for backward compat
        }

        let resource_id = ResourceId(1);
        let object = Object {
            id: resource_id,
            object_type: ObjectType::Model,
            name: Some("OBJ Import".to_string()),
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: lib3mf_core::model::Geometry::Mesh(mesh),
        };
        let _ = model.resources.add_object(object);
        model.build.items.push(BuildItem {
            object_id: resource_id,
            transform: glam::Mat4::IDENTITY,
            part_number: None,
            uuid: None,
            path: None,
            printable: None,
        });

        Ok(model)
    }

    /// Build a Mesh for a single OBJ group, remapping vertices to local indices.
    fn build_mesh_full(
        global_vertices: &[(f32, f32, f32)],
        group: &ObjGroup,
        materials_group_id: Option<ResourceId>,
        material_index_map: Option<&HashMap<String, u32>>,
    ) -> Mesh {
        let mut mesh = Mesh::default();
        let mut local_map: HashMap<u32, u32> = HashMap::new();

        for face in &group.faces {
            // Remap vertex indices to local
            let mut local_indices = Vec::with_capacity(face.indices.len());
            for &global_idx in &face.indices {
                let local_idx = if let Some(&li) = local_map.get(&global_idx) {
                    li
                } else {
                    let li = mesh.vertices.len() as u32;
                    local_map.insert(global_idx, li);
                    let (x, y, z) = global_vertices[global_idx as usize];
                    mesh.vertices.push(Vertex { x, y, z });
                    li
                };
                local_indices.push(local_idx);
            }

            // Build triangle with material assignment
            if local_indices.len() == 3 {
                let (pid, p1, p2, p3) =
                    if let (Some(group_id), Some(index_map), Some(mat_name)) =
                        (materials_group_id, material_index_map, &face.material_name)
                    {
                        if let Some(&mat_idx) = index_map.get(mat_name.as_str()) {
                            (Some(group_id.0), Some(mat_idx), Some(mat_idx), Some(mat_idx))
                        } else {
                            (None, None, None, None)
                        }
                    } else {
                        (None, None, None, None)
                    };

                mesh.triangles.push(Triangle {
                    v1: local_indices[0],
                    v2: local_indices[1],
                    v3: local_indices[2],
                    pid,
                    p1,
                    p2,
                    p3,
                });
            }
        }

        mesh
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

#[cfg(test)]
mod tests {
    use super::*;
    use lib3mf_core::model::Geometry;

    /// Helper: create a simple OBJ string with a triangle.
    fn bare_triangle_obj() -> &'static [u8] {
        b"v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3\n"
    }

    #[test]
    fn test_backward_compat_bare_obj() {
        // A bare OBJ (no groups, no materials) must produce identical output to the
        // original importer: single object with ResourceId(1), name "OBJ Import",
        // no BaseMaterialsGroup, no pid/p1/p2/p3.
        let model = ObjImporter::read(&bare_triangle_obj()[..]).unwrap();

        // Single object with ID 1
        assert_eq!(model.build.items.len(), 1);
        assert_eq!(model.build.items[0].object_id, ResourceId(1));

        let obj = model.resources.get_object(ResourceId(1)).unwrap();
        assert_eq!(obj.name.as_deref(), Some("OBJ Import"));
        assert_eq!(obj.object_type, ObjectType::Model);

        if let Geometry::Mesh(mesh) = &obj.geometry {
            assert_eq!(mesh.vertices.len(), 3);
            assert_eq!(mesh.triangles.len(), 1);
            // No material assignment
            assert!(mesh.triangles[0].pid.is_none());
            assert!(mesh.triangles[0].p1.is_none());
            assert!(mesh.triangles[0].p2.is_none());
            assert!(mesh.triangles[0].p3.is_none());
        } else {
            panic!("Expected mesh geometry");
        }

        // No base materials
        assert_eq!(model.resources.base_material_groups_count(), 0);
    }

    #[test]
    fn test_single_group_with_material() {
        // OBJ with mtllib+usemtl but parsed via build_model (simulating read_from_path)
        let obj_data = b"mtllib test.mtl\nv 0 0 0\nv 1 0 0\nv 0 1 0\nusemtl Red\nf 1 2 3\n";
        let intermediate = ObjImporter::parse_obj(BufReader::new(&obj_data[..])).unwrap();

        // Simulate material data
        let mut materials = HashMap::new();
        materials.insert(
            "Red".to_string(),
            mtl::MtlMaterial {
                name: "Red".to_string(),
                display_color: Color::new(255, 0, 0, 255),
            },
        );

        let model = ObjImporter::build_model(intermediate, &materials).unwrap();

        // Should have BaseMaterialsGroup with ID 1
        assert_eq!(model.resources.base_material_groups_count(), 1);
        let bmg = model.resources.get_base_materials(ResourceId(1)).unwrap();
        assert_eq!(bmg.materials.len(), 1);
        assert_eq!(bmg.materials[0].name, "Red");
        assert_eq!(bmg.materials[0].display_color, Color::new(255, 0, 0, 255));

        // One object (single group, no explicit g/o directive)
        assert_eq!(model.build.items.len(), 1);
        let obj = model
            .resources
            .get_object(model.build.items[0].object_id)
            .unwrap();

        if let Geometry::Mesh(mesh) = &obj.geometry {
            assert_eq!(mesh.triangles.len(), 1);
            let tri = &mesh.triangles[0];
            assert_eq!(tri.pid, Some(1)); // BaseMaterialsGroup ID
            assert_eq!(tri.p1, Some(0)); // Index 0 in materials array
            assert_eq!(tri.p2, Some(0));
            assert_eq!(tri.p3, Some(0));
        } else {
            panic!("Expected mesh geometry");
        }
    }

    #[test]
    fn test_multiple_groups_creates_separate_objects() {
        let obj_data = b"v 0 0 0\nv 1 0 0\nv 0 1 0\nv 2 0 0\nv 2 1 0\nv 3 0 0\ng GroupA\nf 1 2 3\ng GroupB\nf 4 5 6\n";
        let intermediate = ObjImporter::parse_obj(BufReader::new(&obj_data[..])).unwrap();
        let model = ObjImporter::build_model(intermediate, &HashMap::new()).unwrap();

        // Two objects (two groups)
        assert_eq!(model.build.items.len(), 2);

        // First object: GroupA
        let obj_a = model
            .resources
            .get_object(model.build.items[0].object_id)
            .unwrap();
        assert_eq!(obj_a.name.as_deref(), Some("GroupA"));

        // Second object: GroupB
        let obj_b = model
            .resources
            .get_object(model.build.items[1].object_id)
            .unwrap();
        assert_eq!(obj_b.name.as_deref(), Some("GroupB"));
    }

    #[test]
    fn test_vertex_remapping_per_group() {
        // Group A uses vertices 1,2,3 (0-based: 0,1,2)
        // Group B uses vertices 4,5,6 (0-based: 3,4,5)
        // After remapping, each group should have local indices 0,1,2
        let obj_data = b"v 0 0 0\nv 1 0 0\nv 0 1 0\nv 10 0 0\nv 11 0 0\nv 10 1 0\ng A\nf 1 2 3\ng B\nf 4 5 6\n";
        let intermediate = ObjImporter::parse_obj(BufReader::new(&obj_data[..])).unwrap();
        let model = ObjImporter::build_model(intermediate, &HashMap::new()).unwrap();

        // Group A
        let obj_a = model
            .resources
            .get_object(model.build.items[0].object_id)
            .unwrap();
        if let Geometry::Mesh(mesh) = &obj_a.geometry {
            assert_eq!(mesh.vertices.len(), 3);
            assert_eq!(mesh.triangles[0].v1, 0);
            assert_eq!(mesh.triangles[0].v2, 1);
            assert_eq!(mesh.triangles[0].v3, 2);
            // Check actual coordinates
            assert!((mesh.vertices[0].x - 0.0).abs() < f32::EPSILON);
            assert!((mesh.vertices[1].x - 1.0).abs() < f32::EPSILON);
        } else {
            panic!("Expected mesh");
        }

        // Group B
        let obj_b = model
            .resources
            .get_object(model.build.items[1].object_id)
            .unwrap();
        if let Geometry::Mesh(mesh) = &obj_b.geometry {
            assert_eq!(mesh.vertices.len(), 3);
            assert_eq!(mesh.triangles[0].v1, 0);
            assert_eq!(mesh.triangles[0].v2, 1);
            assert_eq!(mesh.triangles[0].v3, 2);
            // Check actual coordinates -- should be the 10,11,10 vertices
            assert!((mesh.vertices[0].x - 10.0).abs() < f32::EPSILON);
            assert!((mesh.vertices[1].x - 11.0).abs() < f32::EPSILON);
        } else {
            panic!("Expected mesh");
        }
    }

    #[test]
    fn test_empty_groups_are_skipped() {
        // Group "Empty" has no faces between it and GroupB
        let obj_data =
            b"v 0 0 0\nv 1 0 0\nv 0 1 0\ng Empty\ng HasFaces\nf 1 2 3\n";
        let intermediate = ObjImporter::parse_obj(BufReader::new(&obj_data[..])).unwrap();
        let model = ObjImporter::build_model(intermediate, &HashMap::new()).unwrap();

        // Only one object (the empty group is skipped)
        assert_eq!(model.build.items.len(), 1);
        let obj = model
            .resources
            .get_object(model.build.items[0].object_id)
            .unwrap();
        assert_eq!(obj.name.as_deref(), Some("HasFaces"));
    }

    #[test]
    fn test_undefined_material_gets_gray() {
        let obj_data =
            b"v 0 0 0\nv 1 0 0\nv 0 1 0\nusemtl Unknown\nf 1 2 3\n";
        let intermediate = ObjImporter::parse_obj(BufReader::new(&obj_data[..])).unwrap();

        // Empty materials map -- "Unknown" is not defined
        let model = ObjImporter::build_model(intermediate, &HashMap::new()).unwrap();

        // Should still have a BaseMaterialsGroup with gray fallback
        assert_eq!(model.resources.base_material_groups_count(), 1);
        let bmg = model.resources.get_base_materials(ResourceId(1)).unwrap();
        assert_eq!(bmg.materials.len(), 1);
        assert_eq!(bmg.materials[0].name, "Unknown");
        assert_eq!(
            bmg.materials[0].display_color,
            Color::new(128, 128, 128, 255)
        );
    }

    #[test]
    fn test_polygon_fan_triangulation() {
        // A quad face (4 vertices) should produce 2 triangles
        let obj_data = b"v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
        let model = ObjImporter::read(&obj_data[..]).unwrap();
        let obj = model.resources.get_object(ResourceId(1)).unwrap();
        if let Geometry::Mesh(mesh) = &obj.geometry {
            assert_eq!(mesh.triangles.len(), 2);
            // First triangle: 0,1,2
            assert_eq!(mesh.triangles[0].v1, 0);
            assert_eq!(mesh.triangles[0].v2, 1);
            assert_eq!(mesh.triangles[0].v3, 2);
            // Second triangle: 0,2,3
            assert_eq!(mesh.triangles[1].v1, 0);
            assert_eq!(mesh.triangles[1].v2, 2);
            assert_eq!(mesh.triangles[1].v3, 3);
        } else {
            panic!("Expected mesh");
        }
    }

    #[test]
    fn test_face_with_vt_vn_format() {
        // v/vt/vn format -- should extract vertex index only
        let obj_data =
            b"v 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nf 1/1/1 2/2/1 3/3/1\n";
        let model = ObjImporter::read(&obj_data[..]).unwrap();
        let obj = model.resources.get_object(ResourceId(1)).unwrap();
        if let Geometry::Mesh(mesh) = &obj.geometry {
            assert_eq!(mesh.triangles.len(), 1);
            assert_eq!(mesh.triangles[0].v1, 0);
            assert_eq!(mesh.triangles[0].v2, 1);
            assert_eq!(mesh.triangles[0].v3, 2);
        } else {
            panic!("Expected mesh");
        }
    }

    #[test]
    fn test_multiple_materials_in_one_group() {
        // A single group with two usemtl directives
        let obj_data = b"v 0 0 0\nv 1 0 0\nv 0 1 0\nv 2 0 0\nv 2 1 0\nv 3 0 0\nusemtl Red\nf 1 2 3\nusemtl Blue\nf 4 5 6\n";
        let intermediate = ObjImporter::parse_obj(BufReader::new(&obj_data[..])).unwrap();

        let mut materials = HashMap::new();
        materials.insert(
            "Red".to_string(),
            mtl::MtlMaterial {
                name: "Red".to_string(),
                display_color: Color::new(255, 0, 0, 255),
            },
        );
        materials.insert(
            "Blue".to_string(),
            mtl::MtlMaterial {
                name: "Blue".to_string(),
                display_color: Color::new(0, 0, 255, 255),
            },
        );

        let model = ObjImporter::build_model(intermediate, &materials).unwrap();

        // One BaseMaterialsGroup with 2 materials
        let bmg = model.resources.get_base_materials(ResourceId(1)).unwrap();
        assert_eq!(bmg.materials.len(), 2);

        // One object (no explicit group directive)
        assert_eq!(model.build.items.len(), 1);
        let obj = model
            .resources
            .get_object(model.build.items[0].object_id)
            .unwrap();

        if let Geometry::Mesh(mesh) = &obj.geometry {
            assert_eq!(mesh.triangles.len(), 2);
            // First triangle: Red (index 0)
            assert_eq!(mesh.triangles[0].pid, Some(1));
            assert_eq!(mesh.triangles[0].p1, Some(0));
            // Second triangle: Blue (index 1)
            assert_eq!(mesh.triangles[1].pid, Some(1));
            assert_eq!(mesh.triangles[1].p1, Some(1));
        } else {
            panic!("Expected mesh");
        }
    }

    #[test]
    fn test_faces_before_first_group() {
        // Faces before any g/o directive go into default group
        let obj_data =
            b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\ng Named\nv 2 0 0\nv 2 1 0\nv 3 0 0\nf 4 5 6\n";
        let intermediate = ObjImporter::parse_obj(BufReader::new(&obj_data[..])).unwrap();
        let model = ObjImporter::build_model(intermediate, &HashMap::new()).unwrap();

        assert_eq!(model.build.items.len(), 2);
        // First group: default (unnamed)
        let obj0 = model
            .resources
            .get_object(model.build.items[0].object_id)
            .unwrap();
        assert_eq!(obj0.name.as_deref(), Some("OBJ Import")); // unnamed defaults

        // Second group: Named
        let obj1 = model
            .resources
            .get_object(model.build.items[1].object_id)
            .unwrap();
        assert_eq!(obj1.name.as_deref(), Some("Named"));
    }

    #[test]
    fn test_o_directive_treated_like_g() {
        let obj_data = b"v 0 0 0\nv 1 0 0\nv 0 1 0\no MyObject\nf 1 2 3\n";
        let intermediate = ObjImporter::parse_obj(BufReader::new(&obj_data[..])).unwrap();
        let model = ObjImporter::build_model(intermediate, &HashMap::new()).unwrap();

        assert_eq!(model.build.items.len(), 1);
        let obj = model
            .resources
            .get_object(model.build.items[0].object_id)
            .unwrap();
        assert_eq!(obj.name.as_deref(), Some("MyObject"));
    }
}
