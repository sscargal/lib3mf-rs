//! Binary STL format import and export.
//!
//! This module provides conversion between binary STL files and 3MF [`Model`] structures.
//!
//! ## STL Format
//!
//! The binary STL format consists of:
//! - 80-byte header (typically unused, set to zeros)
//! - 4-byte little-endian unsigned integer triangle count
//! - For each triangle:
//!   - 12 bytes: normal vector (3 × f32, often ignored by importers)
//!   - 12 bytes: vertex 1 (x, y, z as f32)
//!   - 12 bytes: vertex 2 (x, y, z as f32)
//!   - 12 bytes: vertex 3 (x, y, z as f32)
//!   - 2 bytes: attribute byte count (typically 0)
//!
//! **Note:** ASCII STL format is not supported.
//!
//! ## Examples
//!
//! ### Importing STL
//!
//! ```no_run
//! use lib3mf_converters::stl::StlImporter;
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let file = File::open("model.stl")?;
//! let model = StlImporter::read(file)?;
//! println!("Imported {} vertices",
//!     model.resources.iter_objects()
//!         .filter_map(|obj| match &obj.geometry {
//!             lib3mf_core::model::Geometry::Mesh(m) => Some(m.vertices.len()),
//!             _ => None,
//!         })
//!         .sum::<usize>()
//! );
//! # Ok(())
//! # }
//! ```
//!
//! ### Exporting STL
//!
//! ```no_run
//! use lib3mf_converters::stl::StlExporter;
//! use lib3mf_core::model::Model;
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let model = Model::default();
//! let file = File::create("output.stl")?;
//! StlExporter::write(&model, file)?;
//! # Ok(())
//! # }
//! ```
//!
//! [`Model`]: lib3mf_core::model::Model

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use lib3mf_core::error::{Lib3mfError, Result};
use lib3mf_core::model::resources::ResourceId;
use lib3mf_core::model::{BuildItem, Mesh, Model, Triangle, Vertex};
use std::io::{Read, Write};

/// Imports binary STL files into 3MF [`Model`] structures.
///
/// The importer reads binary STL format and creates a single mesh object with ResourceId(1).
/// Vertices are deduplicated using bitwise float comparison during import.
///
/// [`Model`]: lib3mf_core::model::Model
pub struct StlImporter;

impl Default for StlImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl StlImporter {
    /// Creates a new STL importer instance.
    pub fn new() -> Self {
        Self
    }

    /// Reads a binary STL file and converts it to a 3MF [`Model`].
    ///
    /// # Arguments
    ///
    /// * `reader` - Any type implementing [`Read`] containing binary STL data
    ///
    /// # Returns
    ///
    /// A [`Model`] containing:
    /// - Single mesh object with ResourceId(1) named "STL Import"
    /// - All triangles from the STL file
    /// - Deduplicated vertices (using bitwise float comparison)
    /// - Single build item referencing the mesh object
    ///
    /// # Errors
    ///
    /// Returns [`Lib3mfError::Io`] if:
    /// - Cannot read 80-byte header
    /// - Cannot read triangle count
    /// - Cannot read triangle data (normals, vertices, attribute bytes)
    ///
    /// Returns [`Lib3mfError::Validation`] if triangle count field cannot be parsed.
    ///
    /// # Format Details
    ///
    /// - **Vertex deduplication**: Uses HashMap with bitwise float comparison `[x.to_bits(), y.to_bits(), z.to_bits()]`
    ///   as key. Only exactly identical vertices (bitwise) are merged.
    /// - **Normal vectors**: Read from STL but ignored (not stored in Model).
    /// - **Attribute bytes**: Read but ignored (2-byte field after each triangle).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lib3mf_converters::stl::StlImporter;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = File::open("cube.stl")?;
    /// let model = StlImporter::read(file)?;
    ///
    /// // Access the imported mesh
    /// let obj = model.resources.get_object(lib3mf_core::model::resources::ResourceId(1))
    ///     .expect("STL import creates object with ID 1");
    /// if let lib3mf_core::model::Geometry::Mesh(mesh) = &obj.geometry {
    ///     println!("Imported {} vertices, {} triangles",
    ///         mesh.vertices.len(), mesh.triangles.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Model`]: lib3mf_core::model::Model
    /// [`Lib3mfError::Io`]: lib3mf_core::error::Lib3mfError::Io
    /// [`Lib3mfError::Validation`]: lib3mf_core::error::Lib3mfError::Validation
    pub fn read<R: Read>(mut reader: R) -> Result<Model> {
        // STL Format:
        // 80 bytes header
        // 4 bytes triangle info (u32)
        // Triangles...

        let mut header = [0u8; 80];
        reader.read_exact(&mut header).map_err(Lib3mfError::Io)?;

        let triangle_count = reader.read_u32::<LittleEndian>().map_err(|_| {
            Lib3mfError::Validation("Failed to read STL triangle count".to_string())
        })?;

        let mut mesh = Mesh::default();
        use std::collections::HashMap;
        let mut vert_map: HashMap<[u32; 3], u32> = HashMap::new();

        for _ in 0..triangle_count {
            // Normal (3 floats) - Ignored
            let _nx = reader.read_f32::<LittleEndian>().map_err(Lib3mfError::Io)?;
            let _ny = reader.read_f32::<LittleEndian>().map_err(Lib3mfError::Io)?;
            let _nz = reader.read_f32::<LittleEndian>().map_err(Lib3mfError::Io)?;

            let mut indices = [0u32; 3];

            for index in &mut indices {
                let x = reader.read_f32::<LittleEndian>().map_err(Lib3mfError::Io)?;
                let y = reader.read_f32::<LittleEndian>().map_err(Lib3mfError::Io)?;
                let z = reader.read_f32::<LittleEndian>().map_err(Lib3mfError::Io)?;

                let key = [x.to_bits(), y.to_bits(), z.to_bits()];

                let idx = *vert_map.entry(key).or_insert_with(|| {
                    let new_idx = mesh.vertices.len() as u32;
                    mesh.vertices.push(Vertex { x, y, z });
                    new_idx
                });
                *index = idx;
            }

            let _attr_byte_count = reader.read_u16::<LittleEndian>().map_err(Lib3mfError::Io)?;

            mesh.triangles.push(Triangle {
                v1: indices[0],
                v2: indices[1],
                v3: indices[2],
                ..Default::default()
            });
        }

        let mut model = Model::default();
        let resource_id = ResourceId(1); // Default ID

        let object = lib3mf_core::model::Object {
            id: resource_id,
            object_type: lib3mf_core::model::ObjectType::Model,
            name: Some("STL Import".to_string()),
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
            uuid: None, // Generate one?
            path: None,
        });

        Ok(model)
    }
}

/// Exports 3MF [`Model`] structures to binary STL files.
///
/// The exporter flattens all mesh objects referenced in build items into a single STL file,
/// applying build item transformations to vertex coordinates.
///
/// [`Model`]: lib3mf_core::model::Model
pub struct StlExporter;

impl StlExporter {
    /// Writes a 3MF [`Model`] to binary STL format.
    ///
    /// # Arguments
    ///
    /// * `model` - The 3MF model to export
    /// * `writer` - Any type implementing [`Write`] to receive STL data
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
    /// - **Header**: 80 zero bytes (standard for most STL files)
    /// - **Normals**: Written as (0, 0, 0) - viewers must compute face normals
    /// - **Transformations**: Build item transforms are applied to vertex coordinates
    /// - **Attribute bytes**: Written as 0 (no extended attributes)
    ///
    /// # Behavior
    ///
    /// - Only mesh objects from `model.build.items` are exported
    /// - Non-mesh geometries (Components, BooleanShape, etc.) are skipped
    /// - Each build item's transformation matrix is applied to its mesh vertices
    /// - All triangles from all build items are combined into a single STL file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lib3mf_converters::stl::StlExporter;
    /// use lib3mf_core::model::Model;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let model = Model::default();
    /// let output = File::create("exported.stl")?;
    /// StlExporter::write(&model, output)?;
    /// println!("Model exported successfully");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Model`]: lib3mf_core::model::Model
    /// [`Lib3mfError::Io`]: lib3mf_core::error::Lib3mfError::Io
    pub fn write<W: Write>(model: &Model, mut writer: W) -> Result<()> {
        // 1. Collect all triangles from all build items
        let mut triangles: Vec<(glam::Vec3, glam::Vec3, glam::Vec3)> = Vec::new(); // v1, v2, v3

        for item in &model.build.items {
            #[allow(clippy::collapsible_if)]
            if let Some(object) = model.resources.get_object(item.object_id) {
                if let lib3mf_core::model::Geometry::Mesh(mesh) = &object.geometry {
                    let transform = item.transform;

                    for tri in &mesh.triangles {
                        let v1_local = mesh.vertices[tri.v1 as usize];
                        let v2_local = mesh.vertices[tri.v2 as usize];
                        let v3_local = mesh.vertices[tri.v3 as usize];

                        let v1 = transform
                            .transform_point3(glam::Vec3::new(v1_local.x, v1_local.y, v1_local.z));
                        let v2 = transform
                            .transform_point3(glam::Vec3::new(v2_local.x, v2_local.y, v2_local.z));
                        let v3 = transform
                            .transform_point3(glam::Vec3::new(v3_local.x, v3_local.y, v3_local.z));

                        triangles.push((v1, v2, v3));
                    }
                }
            }
        }

        // 2. Write Header (80 bytes)
        let header = [0u8; 80];
        writer.write_all(&header).map_err(Lib3mfError::Io)?;

        // 3. Write Count
        writer
            .write_u32::<LittleEndian>(triangles.len() as u32)
            .map_err(Lib3mfError::Io)?;

        // 4. Write Triangles
        for (v1, v2, v3) in triangles {
            // Normal (0,0,0) - let viewer calculate
            writer
                .write_f32::<LittleEndian>(0.0)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(0.0)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(0.0)
                .map_err(Lib3mfError::Io)?;

            // v1
            writer
                .write_f32::<LittleEndian>(v1.x)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v1.y)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v1.z)
                .map_err(Lib3mfError::Io)?;

            // v2
            writer
                .write_f32::<LittleEndian>(v2.x)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v2.y)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v2.z)
                .map_err(Lib3mfError::Io)?;

            // v3
            writer
                .write_f32::<LittleEndian>(v3.x)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v3.y)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v3.z)
                .map_err(Lib3mfError::Io)?;

            // Attribute byte count (0)
            writer
                .write_u16::<LittleEndian>(0)
                .map_err(Lib3mfError::Io)?;
        }

        Ok(())
    }

    /// Writes a 3MF [`Model`] to binary STL format with support for multi-part 3MF files.
    ///
    /// This method extends [`write`] by recursively resolving component references and external
    /// model parts using a [`PartResolver`]. This is necessary for 3MF files with the Production
    /// Extension that reference objects from external model parts.
    ///
    /// # Arguments
    ///
    /// * `model` - The root 3MF model to export
    /// * `resolver` - A [`PartResolver`] for loading external model parts from the 3MF archive
    /// * `writer` - Any type implementing [`Write`] to receive STL data
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful export.
    ///
    /// # Errors
    ///
    /// Returns [`Lib3mfError::Io`] if any write operation fails.
    ///
    /// Returns errors from the resolver if external parts cannot be loaded.
    ///
    /// # Behavior
    ///
    /// - Recursively resolves component hierarchies using the PartResolver
    /// - Follows external references via component `path` attributes
    /// - Applies accumulated transformations through the component tree
    /// - Flattens all resolved meshes into a single STL file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lib3mf_converters::stl::StlExporter;
    /// use lib3mf_core::archive::ZipArchiver;
    /// use lib3mf_core::model::resolver::PartResolver;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let model = lib3mf_core::model::Model::default();
    /// let archive_file = File::open("multipart.3mf")?;
    /// let mut archiver = ZipArchiver::new(archive_file)?;
    /// let resolver = PartResolver::new(&mut archiver, model.clone());
    ///
    /// let output = File::create("output.stl")?;
    /// StlExporter::write_with_resolver(&model, resolver, output)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`write`]: StlExporter::write
    /// [`Model`]: lib3mf_core::model::Model
    /// [`PartResolver`]: lib3mf_core::model::resolver::PartResolver
    /// [`Lib3mfError::Io`]: lib3mf_core::error::Lib3mfError::Io
    pub fn write_with_resolver<W: Write, A: lib3mf_core::archive::ArchiveReader>(
        model: &Model,
        mut resolver: lib3mf_core::model::resolver::PartResolver<A>,
        mut writer: W,
    ) -> Result<()> {
        // 1. Collect all triangles from all build items (recursively)
        let mut triangles: Vec<(glam::Vec3, glam::Vec3, glam::Vec3)> = Vec::new();

        for item in &model.build.items {
            collect_triangles(
                &mut resolver,
                item.object_id,
                item.transform,
                None, // Start with root path (None)
                &mut triangles,
            )?;
        }

        // 2. Write Header (80 bytes)
        let header = [0u8; 80];
        writer.write_all(&header).map_err(Lib3mfError::Io)?;

        // 3. Write Count
        writer
            .write_u32::<LittleEndian>(triangles.len() as u32)
            .map_err(Lib3mfError::Io)?;

        // 4. Write Triangles
        for (v1, v2, v3) in triangles {
            // Normal (0,0,0) - let viewer calculate
            writer
                .write_f32::<LittleEndian>(0.0)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(0.0)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(0.0)
                .map_err(Lib3mfError::Io)?;

            // v1
            writer
                .write_f32::<LittleEndian>(v1.x)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v1.y)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v1.z)
                .map_err(Lib3mfError::Io)?;

            // v2
            writer
                .write_f32::<LittleEndian>(v2.x)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v2.y)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v2.z)
                .map_err(Lib3mfError::Io)?;

            // v3
            writer
                .write_f32::<LittleEndian>(v3.x)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v3.y)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v3.z)
                .map_err(Lib3mfError::Io)?;

            // Attribute byte count (0)
            writer
                .write_u16::<LittleEndian>(0)
                .map_err(Lib3mfError::Io)?;
        }

        Ok(())
    }
}

fn collect_triangles<A: lib3mf_core::archive::ArchiveReader>(
    resolver: &mut lib3mf_core::model::resolver::PartResolver<A>,
    object_id: ResourceId,
    transform: glam::Mat4,
    path: Option<&str>,
    triangles: &mut Vec<(glam::Vec3, glam::Vec3, glam::Vec3)>,
) -> Result<()> {
    // Resolve geometry
    // Note: We need to clone the geometry or handle the borrow of resolver carefully.
    // resolving returns a reference to Model and Object, which borrows from resolver.
    // We can't keep that borrow while recursing (mutably borrowing resolver).
    // So we need to clone the relevant data (Geometry) or restructure.
    // Cloning Geometry (which contains Mesh) might be expensive but safe.
    // OR: resolve_object returns reference, we inspect it, then drop reference before recursing.

    let geometry = {
        let res = resolver.resolve_object(object_id, path)?;
        if let Some((_, obj)) = res {
            Some(obj.geometry.clone()) // Cloning geometry to release borrow
        } else {
            None
        }
    };

    if let Some(geo) = geometry {
        match geo {
            lib3mf_core::model::Geometry::Mesh(mesh) => {
                for tri in &mesh.triangles {
                    let v1_local = mesh.vertices[tri.v1 as usize];
                    let v2_local = mesh.vertices[tri.v2 as usize];
                    let v3_local = mesh.vertices[tri.v3 as usize];

                    let v1 = transform
                        .transform_point3(glam::Vec3::new(v1_local.x, v1_local.y, v1_local.z));
                    let v2 = transform
                        .transform_point3(glam::Vec3::new(v2_local.x, v2_local.y, v2_local.z));
                    let v3 = transform
                        .transform_point3(glam::Vec3::new(v3_local.x, v3_local.y, v3_local.z));

                    triangles.push((v1, v2, v3));
                }
            }
            lib3mf_core::model::Geometry::Components(comps) => {
                for comp in comps.components {
                    let new_transform = transform * comp.transform;
                    // Path handling: If component references a different path/part, update it?
                    // Currently lib3mf-core resolver handles relative paths if we pass them.
                    // The component struct has a 'path' field? (Checking core definition...)
                    // Assuming Component has 'path' field which is Option<String>
                    // Wait, I need to check lib3mf_core::model::Component definition.
                    // Assuming it does based on previous experience, but let's be safe.
                    // Actually, looking at `print_tree` in commands.rs, it uses `comp.path`.

                    let next_path_store = comp.path.clone();
                    // If component has path, it overrides/is relative to current?
                    // Usually in 3MF, if path represents a model part.
                    // resolver logic: "if path is provided, look there".

                    let next_path = next_path_store.as_deref().or(path);

                    collect_triangles(
                        resolver,
                        comp.object_id,
                        new_transform,
                        next_path,
                        triangles,
                    )?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
