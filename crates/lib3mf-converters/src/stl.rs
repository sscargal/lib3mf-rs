//! STL format import and export (binary and ASCII).
//!
//! This module provides conversion between STL files and 3MF [`Model`] structures.
//! Both binary and ASCII STL formats are supported.
//!
//! ## STL Formats
//!
//! ### Binary STL
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
//! ### ASCII STL
//!
//! The ASCII STL format is a text-based format with keyword-delimited geometry:
//! - Keywords are case-insensitive (real-world files use both uppercase and lowercase)
//! - Multiple solids per file are supported (each becomes a separate object)
//! - Solid names with spaces are supported
//!
//! ## Auto-Detection
//!
//! [`StlImporter::read()`] automatically detects the format using the file size formula.
//! Use [`StlImporter::read_binary()`] or [`StlImporter::read_ascii()`] for explicit format selection.
//!
//! ## Examples
//!
//! ### Importing STL (auto-detect)
//!
//! ```no_run
//! use lib3mf_converters::stl::StlImporter;
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let file = File::open("model.stl")?;
//! let model = StlImporter::read(file)?;
//! println!("Imported {} objects", model.build.items.len());
//! # Ok(())
//! # }
//! ```
//!
//! ### Exporting Binary STL
//!
//! ```no_run
//! use lib3mf_converters::stl::BinaryStlExporter;
//! use lib3mf_core::model::Model;
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let model = Model::default();
//! let file = File::create("output.stl")?;
//! BinaryStlExporter::write(&model, file)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Exporting ASCII STL
//!
//! ```no_run
//! use lib3mf_converters::stl::AsciiStlExporter;
//! use lib3mf_core::model::Model;
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let model = Model::default();
//! let file = File::create("output.stl")?;
//! AsciiStlExporter::write(&model, file)?;
//! # Ok(())
//! # }
//! ```
//!
//! [`Model`]: lib3mf_core::model::Model

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use lib3mf_core::error::{Lib3mfError, Result};
use lib3mf_core::model::resources::ResourceId;
use lib3mf_core::model::{BuildItem, Mesh, Model, Triangle, Vertex};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};

/// Detected STL file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StlFormat {
    /// Binary STL — compact 80-byte header followed by packed triangle records.
    Binary,
    /// ASCII STL — human-readable `solid`/`facet normal`/`vertex` text format.
    Ascii,
}

/// Detects whether an STL file is binary or ASCII.
///
/// Uses the reliable size-formula check to disambiguate binary files whose headers
/// begin with the ASCII text "solid" (a common case with many CAD tools).
///
/// # Arguments
///
/// * `reader` - Any type implementing [`Read`] + [`Seek`]
///
/// # Returns
///
/// - [`StlFormat::Binary`] if the file matches the binary STL size formula, or if the
///   first 5 bytes are not `solid` (case-insensitive).
/// - [`StlFormat::Ascii`] otherwise.
///
/// After return, the reader position is reset to 0.
pub fn detect_stl_format<R: Read + Seek>(reader: &mut R) -> Result<StlFormat> {
    let mut buf = [0u8; 84];
    let n = reader.read(&mut buf).map_err(Lib3mfError::Io)?;
    reader.seek(SeekFrom::Start(0)).map_err(Lib3mfError::Io)?;

    if n < 5 {
        // Very small file — treat as ASCII
        return Ok(StlFormat::Ascii);
    }

    if !buf[..5].eq_ignore_ascii_case(b"solid") {
        return Ok(StlFormat::Binary);
    }

    // First 5 bytes are "solid" — could be ASCII or binary with "solid" in header.
    // Use size formula: binary STL = 84 + triangle_count * 50 bytes.
    if n >= 84 {
        let tri_count = u32::from_le_bytes([buf[80], buf[81], buf[82], buf[83]]);
        let expected_binary_size = 84u64 + tri_count as u64 * 50;
        let file_size = reader.seek(SeekFrom::End(0)).map_err(Lib3mfError::Io)?;
        reader.seek(SeekFrom::Start(0)).map_err(Lib3mfError::Io)?;
        if file_size == expected_binary_size {
            return Ok(StlFormat::Binary);
        }
    }

    Ok(StlFormat::Ascii)
}

/// Imports STL files (binary or ASCII) into 3MF [`Model`] structures.
///
/// The importer supports both binary and ASCII STL formats:
///
/// - [`read()`]: Auto-detects the format using the file size formula, then dispatches
///   to the appropriate parser. Requires `Read + Seek`.
/// - [`read_binary()`]: Explicit binary-format parser. Requires only `Read`.
/// - [`read_ascii()`]: Explicit ASCII-format parser. Requires only `Read`.
///
/// Vertices are deduplicated using bitwise float comparison during import.
///
/// [`read()`]: StlImporter::read
/// [`read_binary()`]: StlImporter::read_binary
/// [`read_ascii()`]: StlImporter::read_ascii
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

    /// Reads an STL file, auto-detecting binary vs ASCII format.
    ///
    /// Uses the file size formula to distinguish binary files (even those whose headers
    /// begin with "solid") from ASCII files.
    ///
    /// # Arguments
    ///
    /// * `reader` - Any type implementing [`Read`] + [`Seek`] containing STL data
    ///
    /// # Returns
    ///
    /// A [`Model`] containing the parsed geometry. Binary STL produces a single object;
    /// ASCII STL produces one object per solid.
    ///
    /// # Errors
    ///
    /// Returns [`Lib3mfError::Io`] if reading or seeking fails.
    /// Returns [`Lib3mfError::Validation`] or [`Lib3mfError::InvalidStructure`] if parsing fails.
    ///
    /// [`Model`]: lib3mf_core::model::Model
    /// [`Lib3mfError::Io`]: lib3mf_core::error::Lib3mfError::Io
    pub fn read<R: Read + Seek>(mut reader: R) -> Result<Model> {
        let format = detect_stl_format(&mut reader)?;
        match format {
            StlFormat::Binary => Self::read_binary(reader),
            StlFormat::Ascii => Self::read_ascii(reader),
        }
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
    /// let model = StlImporter::read_binary(file)?;
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
    pub fn read_binary<R: Read>(mut reader: R) -> Result<Model> {
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
            uuid: None,
            path: None,
            printable: None,
        });

        Ok(model)
    }

    /// Reads an ASCII STL file and converts it to a 3MF [`Model`].
    ///
    /// Parses one or more `solid ... endsolid` blocks. Each solid becomes a separate
    /// [`Object`] with its own [`ResourceId`] and [`BuildItem`].
    ///
    /// # Arguments
    ///
    /// * `reader` - Any type implementing [`Read`] containing ASCII STL text
    ///
    /// # Returns
    ///
    /// A [`Model`] containing:
    /// - One mesh object per solid, with ResourceIds starting at 1
    /// - Object names taken from the solid name (if any)
    /// - Deduplicated vertices per solid (using bitwise float comparison)
    /// - One build item per solid with identity transform
    ///
    /// # Errors
    ///
    /// Returns [`Lib3mfError::Io`] if reading fails.
    /// Returns [`Lib3mfError::InvalidStructure`] if vertex coordinates cannot be parsed.
    ///
    /// # Behavior
    ///
    /// - Keywords are matched case-insensitively (`SOLID`, `Facet`, `VERTEX`, etc.)
    /// - Solid names with spaces are supported (`solid My Cool Part`)
    /// - `endsolid` name is not validated (may differ from `solid` name or be absent)
    /// - Files that end without `endsolid` are handled leniently
    /// - Normal vectors from `facet normal` lines are read and discarded
    ///
    /// [`Model`]: lib3mf_core::model::Model
    /// [`Object`]: lib3mf_core::model::Object
    /// [`ResourceId`]: lib3mf_core::model::resources::ResourceId
    /// [`BuildItem`]: lib3mf_core::model::BuildItem
    pub fn read_ascii<R: Read>(reader: R) -> Result<Model> {
        let buf_reader = BufReader::new(reader);
        let mut model = Model::default();
        let mut next_id = 1u32;

        let mut current_mesh: Option<(Mesh, String)> = None;
        let mut vert_map: HashMap<[u32; 3], u32> = HashMap::new();
        // Buffer for the 3 vertices of the current facet
        let mut facet_verts: Vec<(f32, f32, f32)> = Vec::with_capacity(3);

        for line_res in buf_reader.lines() {
            let line = line_res.map_err(Lib3mfError::Io)?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0].to_ascii_lowercase().as_str() {
                "solid" => {
                    // Name is everything after "solid" (joined with spaces)
                    let name = if parts.len() > 1 {
                        parts[1..].join(" ")
                    } else {
                        String::new()
                    };
                    current_mesh = Some((Mesh::default(), name));
                    vert_map.clear();
                    facet_verts.clear();
                }
                "facet" => {
                    // facet normal nx ny nz — read and discard normals
                    facet_verts.clear();
                }
                "vertex" => {
                    if parts.len() >= 4 {
                        let x = parts[1].parse::<f32>().map_err(|_| {
                            Lib3mfError::InvalidStructure("Invalid STL vertex x coordinate".into())
                        })?;
                        let y = parts[2].parse::<f32>().map_err(|_| {
                            Lib3mfError::InvalidStructure("Invalid STL vertex y coordinate".into())
                        })?;
                        let z = parts[3].parse::<f32>().map_err(|_| {
                            Lib3mfError::InvalidStructure("Invalid STL vertex z coordinate".into())
                        })?;
                        facet_verts.push((x, y, z));
                    }
                }
                "endfacet" => {
                    if facet_verts.len() != 3 {
                        return Err(Lib3mfError::InvalidStructure(format!(
                            "STL facet must have exactly 3 vertices, found {}",
                            facet_verts.len()
                        )));
                    }
                    if let Some((ref mut mesh, _)) = current_mesh {
                        let mut indices = [0u32; 3];
                        for (i, &(x, y, z)) in facet_verts.iter().enumerate() {
                            let key = [x.to_bits(), y.to_bits(), z.to_bits()];
                            let idx = *vert_map.entry(key).or_insert_with(|| {
                                let new_idx = mesh.vertices.len() as u32;
                                mesh.vertices.push(Vertex { x, y, z });
                                new_idx
                            });
                            indices[i] = idx;
                        }
                        mesh.triangles.push(Triangle {
                            v1: indices[0],
                            v2: indices[1],
                            v3: indices[2],
                            ..Default::default()
                        });
                    }
                    facet_verts.clear();
                }
                "endsolid" => {
                    if let Some((mesh, name)) = current_mesh.take() {
                        finalize_solid(&mut model, mesh, name, next_id);
                        next_id += 1;
                    }
                }
                // Skip: "outer", "loop", "endloop", and any other unrecognized lines
                _ => {}
            }
        }

        // Handle file that ended without endsolid (lenient parsing)
        if let Some((mesh, name)) = current_mesh.take() {
            finalize_solid(&mut model, mesh, name, next_id);
        }

        Ok(model)
    }
}

/// Finalizes a parsed solid into a Model object and build item.
fn finalize_solid(model: &mut Model, mesh: Mesh, name: String, id: u32) {
    let resource_id = ResourceId(id);
    let object_name = if name.is_empty() { None } else { Some(name) };

    let object = lib3mf_core::model::Object {
        id: resource_id,
        object_type: lib3mf_core::model::ObjectType::Model,
        name: object_name,
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
}

/// Computes a face normal from three vertices using the cross product.
///
/// Returns a zero vector for degenerate (zero-area) triangles.
fn compute_face_normal(v1: glam::Vec3, v2: glam::Vec3, v3: glam::Vec3) -> glam::Vec3 {
    let edge1 = v2 - v1;
    let edge2 = v3 - v1;
    edge1.cross(edge2).normalize_or_zero()
}

/// Exports 3MF [`Model`] structures to binary STL files.
///
/// The exporter flattens all mesh objects referenced in build items into a single STL file,
/// applying build item transformations to vertex coordinates.
///
/// This was previously named `StlExporter`. If you were using `StlExporter`, update your
/// code to use `BinaryStlExporter`.
///
/// [`Model`]: lib3mf_core::model::Model
pub struct BinaryStlExporter;

impl BinaryStlExporter {
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
    /// use lib3mf_converters::stl::BinaryStlExporter;
    /// use lib3mf_core::model::Model;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let model = Model::default();
    /// let output = File::create("exported.stl")?;
    /// BinaryStlExporter::write(&model, output)?;
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
    /// use lib3mf_converters::stl::BinaryStlExporter;
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
    /// BinaryStlExporter::write_with_resolver(&model, resolver, output)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`write`]: BinaryStlExporter::write
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

/// Exports 3MF [`Model`] structures to ASCII STL files.
///
/// Each mesh object in the model's build items is written as a separate ASCII STL solid.
/// Face normals are computed from the triangle edges using the cross product.
///
/// [`Model`]: lib3mf_core::model::Model
pub struct AsciiStlExporter;

impl AsciiStlExporter {
    /// Writes a 3MF [`Model`] to ASCII STL format.
    ///
    /// Each mesh object referenced in the model's build items is written as a separate
    /// `solid ... endsolid` block. The solid name is taken from the object's name field,
    /// or left empty if the object has no name.
    ///
    /// # Arguments
    ///
    /// * `model` - The 3MF model to export
    /// * `writer` - Any type implementing [`Write`] to receive ASCII STL text
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
    /// - **Normals**: Computed via cross product of triangle edges (`normalize_or_zero()`)
    /// - **Degenerate triangles**: Emit zero normal `(0 0 0)`, triangle is not skipped
    /// - **Normal format**: Scientific notation with 6 decimal places (`{:.6e}`)
    /// - **Vertex format**: Fixed-point with 6 decimal places (`{:.6}`)
    /// - **Transformations**: Build item transforms are applied to vertex coordinates
    /// - **Solid names**: Taken from `object.name`, empty string if `None`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lib3mf_converters::stl::AsciiStlExporter;
    /// use lib3mf_core::model::Model;
    /// use std::fs::File;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let model = Model::default();
    /// let output = File::create("exported.stl")?;
    /// AsciiStlExporter::write(&model, output)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// [`Model`]: lib3mf_core::model::Model
    /// [`Lib3mfError::Io`]: lib3mf_core::error::Lib3mfError::Io
    pub fn write<W: Write>(model: &Model, mut writer: W) -> Result<()> {
        for item in &model.build.items {
            #[allow(clippy::collapsible_if)]
            if let Some(object) = model.resources.get_object(item.object_id) {
                if let lib3mf_core::model::Geometry::Mesh(mesh) = &object.geometry {
                    let name = object.name.as_deref().unwrap_or("");
                    let transform = item.transform;

                    writeln!(writer, "solid {name}").map_err(Lib3mfError::Io)?;

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

                        let normal = compute_face_normal(v1, v2, v3);

                        writeln!(
                            writer,
                            "  facet normal {:.6e} {:.6e} {:.6e}",
                            normal.x, normal.y, normal.z
                        )
                        .map_err(Lib3mfError::Io)?;
                        writeln!(writer, "    outer loop").map_err(Lib3mfError::Io)?;
                        writeln!(writer, "      vertex {:.6} {:.6} {:.6}", v1.x, v1.y, v1.z)
                            .map_err(Lib3mfError::Io)?;
                        writeln!(writer, "      vertex {:.6} {:.6} {:.6}", v2.x, v2.y, v2.z)
                            .map_err(Lib3mfError::Io)?;
                        writeln!(writer, "      vertex {:.6} {:.6} {:.6}", v3.x, v3.y, v3.z)
                            .map_err(Lib3mfError::Io)?;
                        writeln!(writer, "    endloop").map_err(Lib3mfError::Io)?;
                        writeln!(writer, "  endfacet").map_err(Lib3mfError::Io)?;
                    }

                    writeln!(writer, "endsolid {name}").map_err(Lib3mfError::Io)?;
                }
            }
        }
        Ok(())
    }

    /// Writes a 3MF [`Model`] to ASCII STL format with support for multi-part 3MF files.
    ///
    /// Resolves component references and external model parts using a [`PartResolver`],
    /// then writes all collected triangles as a single ASCII STL solid.
    ///
    /// # Arguments
    ///
    /// * `model` - The root 3MF model to export
    /// * `resolver` - A [`PartResolver`] for loading external model parts from the 3MF archive
    /// * `writer` - Any type implementing [`Write`] to receive ASCII STL text
    ///
    /// # Returns
    ///
    /// `Ok(())` on successful export.
    ///
    /// # Errors
    ///
    /// Returns [`Lib3mfError::Io`] if any write operation fails.
    /// Returns errors from the resolver if external parts cannot be loaded.
    ///
    /// [`Model`]: lib3mf_core::model::Model
    /// [`PartResolver`]: lib3mf_core::model::resolver::PartResolver
    pub fn write_with_resolver<W: Write, A: lib3mf_core::archive::ArchiveReader>(
        model: &Model,
        mut resolver: lib3mf_core::model::resolver::PartResolver<A>,
        mut writer: W,
    ) -> Result<()> {
        // Collect all triangles from all build items (recursively)
        let mut triangles: Vec<(glam::Vec3, glam::Vec3, glam::Vec3)> = Vec::new();

        for item in &model.build.items {
            collect_triangles(
                &mut resolver,
                item.object_id,
                item.transform,
                None,
                &mut triangles,
            )?;
        }

        // Write as a single solid (resolver flattens all objects)
        writeln!(writer, "solid ").map_err(Lib3mfError::Io)?;

        for (v1, v2, v3) in triangles {
            let normal = compute_face_normal(v1, v2, v3);

            writeln!(
                writer,
                "  facet normal {:.6e} {:.6e} {:.6e}",
                normal.x, normal.y, normal.z
            )
            .map_err(Lib3mfError::Io)?;
            writeln!(writer, "    outer loop").map_err(Lib3mfError::Io)?;
            writeln!(writer, "      vertex {:.6} {:.6} {:.6}", v1.x, v1.y, v1.z)
                .map_err(Lib3mfError::Io)?;
            writeln!(writer, "      vertex {:.6} {:.6} {:.6}", v2.x, v2.y, v2.z)
                .map_err(Lib3mfError::Io)?;
            writeln!(writer, "      vertex {:.6} {:.6} {:.6}", v3.x, v3.y, v3.z)
                .map_err(Lib3mfError::Io)?;
            writeln!(writer, "    endloop").map_err(Lib3mfError::Io)?;
            writeln!(writer, "  endfacet").map_err(Lib3mfError::Io)?;
        }

        writeln!(writer, "endsolid ").map_err(Lib3mfError::Io)?;

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
                    let next_path_store = comp.path.clone();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // ===== Helper functions =====

    /// Build a minimal binary STL with the given triangles.
    fn make_binary_stl(
        header: &[u8; 80],
        triangles: &[(f32, f32, f32, f32, f32, f32, f32, f32, f32)],
    ) -> Vec<u8> {
        // Each element: (v1x, v1y, v1z, v2x, v2y, v2z, v3x, v3y, v3z)
        use byteorder::{LittleEndian, WriteBytesExt};
        let mut buf = Vec::new();
        buf.extend_from_slice(header);
        buf.write_u32::<LittleEndian>(triangles.len() as u32)
            .unwrap();
        for &(v1x, v1y, v1z, v2x, v2y, v2z, v3x, v3y, v3z) in triangles {
            // normal (0,0,0)
            buf.write_f32::<LittleEndian>(0.0).unwrap();
            buf.write_f32::<LittleEndian>(0.0).unwrap();
            buf.write_f32::<LittleEndian>(0.0).unwrap();
            // v1
            buf.write_f32::<LittleEndian>(v1x).unwrap();
            buf.write_f32::<LittleEndian>(v1y).unwrap();
            buf.write_f32::<LittleEndian>(v1z).unwrap();
            // v2
            buf.write_f32::<LittleEndian>(v2x).unwrap();
            buf.write_f32::<LittleEndian>(v2y).unwrap();
            buf.write_f32::<LittleEndian>(v2z).unwrap();
            // v3
            buf.write_f32::<LittleEndian>(v3x).unwrap();
            buf.write_f32::<LittleEndian>(v3y).unwrap();
            buf.write_f32::<LittleEndian>(v3z).unwrap();
            // attribute byte count
            buf.write_u16::<LittleEndian>(0).unwrap();
        }
        buf
    }

    /// Build a simple model with one mesh object containing the given vertices and triangles.
    fn make_simple_model(
        vertices: Vec<(f32, f32, f32)>,
        triangles: Vec<(u32, u32, u32)>,
        name: Option<&str>,
    ) -> Model {
        use lib3mf_core::model::{Geometry, Object, ObjectType};

        let mut mesh = Mesh::default();
        for (x, y, z) in vertices {
            mesh.vertices.push(Vertex { x, y, z });
        }
        for (v1, v2, v3) in triangles {
            mesh.triangles.push(Triangle {
                v1,
                v2,
                v3,
                ..Default::default()
            });
        }

        let resource_id = ResourceId(1);
        let object = Object {
            id: resource_id,
            object_type: ObjectType::Model,
            name: name.map(|s| s.to_string()),
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::Mesh(mesh),
        };

        let mut model = Model::default();
        let _ = model.resources.add_object(object);
        model.build.items.push(BuildItem {
            object_id: resource_id,
            transform: glam::Mat4::IDENTITY,
            part_number: None,
            uuid: None,
            path: None,
            printable: None,
        });
        model
    }

    // ===== Test 1: detect_stl_format returns Binary for binary STL =====

    #[test]
    fn test_detect_binary_format() {
        let header = [0u8; 80];
        let tris = vec![(0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0)];
        let data = make_binary_stl(&header, &tris);
        let mut cursor = Cursor::new(data);
        let fmt = detect_stl_format(&mut cursor).expect("detect should succeed");
        assert_eq!(fmt, StlFormat::Binary);
    }

    // ===== Test 2: detect_stl_format returns Ascii for ASCII STL =====

    #[test]
    fn test_detect_ascii_format() {
        let ascii = "solid test\nendsolid test\n";
        let mut cursor = Cursor::new(ascii.as_bytes().to_vec());
        let fmt = detect_stl_format(&mut cursor).expect("detect should succeed");
        assert_eq!(fmt, StlFormat::Ascii);
    }

    // ===== Test 3: detect returns Binary even when header starts with "solid" =====

    #[test]
    fn test_detect_binary_with_solid_header() {
        // Build binary STL whose 80-byte header starts with "solid"
        let mut header = [0u8; 80];
        let solid_bytes = b"solid binary_test_header";
        header[..solid_bytes.len()].copy_from_slice(solid_bytes);

        let tris = vec![(0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0)];
        let data = make_binary_stl(&header, &tris);

        // Verify the size formula: 84 + 1 * 50 = 134
        assert_eq!(data.len(), 134);

        let mut cursor = Cursor::new(data);
        let fmt = detect_stl_format(&mut cursor).expect("detect should succeed");
        assert_eq!(
            fmt,
            StlFormat::Binary,
            "Binary STL with 'solid' in header must be detected as Binary"
        );
    }

    // ===== Test 4: read_ascii parses a simple single-triangle STL =====

    #[test]
    fn test_read_ascii_simple_triangle() {
        let ascii = "\
solid triangle
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 0 1 0
    endloop
  endfacet
endsolid triangle
";
        let model = StlImporter::read_ascii(Cursor::new(ascii)).expect("parse should succeed");

        assert_eq!(model.build.items.len(), 1);
        let obj = model
            .resources
            .get_object(ResourceId(1))
            .expect("object 1 should exist");
        if let lib3mf_core::model::Geometry::Mesh(mesh) = &obj.geometry {
            assert_eq!(mesh.vertices.len(), 3, "should have 3 unique vertices");
            assert_eq!(mesh.triangles.len(), 1, "should have 1 triangle");
            // Check vertex coordinates
            assert!((mesh.vertices[0].x - 0.0).abs() < 1e-6);
            assert!((mesh.vertices[1].x - 1.0).abs() < 1e-6);
            assert!((mesh.vertices[2].y - 1.0).abs() < 1e-6);
        } else {
            panic!("expected Mesh geometry");
        }
    }

    // ===== Test 5: case-insensitive keywords =====

    #[test]
    fn test_read_ascii_case_insensitive() {
        let ascii = "\
SOLID uppercase
  FACET NORMAL 0 0 1
    OUTER LOOP
      VERTEX 0 0 0
      VERTEX 1 0 0
      VERTEX 0 1 0
    ENDLOOP
  ENDFACET
ENDSOLID uppercase
";
        let model = StlImporter::read_ascii(Cursor::new(ascii)).expect("parse should succeed");
        assert_eq!(model.build.items.len(), 1);
        let obj = model.resources.get_object(ResourceId(1)).unwrap();
        if let lib3mf_core::model::Geometry::Mesh(mesh) = &obj.geometry {
            assert_eq!(mesh.triangles.len(), 1);
        } else {
            panic!("expected Mesh geometry");
        }
    }

    // ===== Test 6: multi-solid creates multiple objects =====

    #[test]
    fn test_read_ascii_multi_solid() {
        let ascii = "\
solid part_a
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 0 1 0
    endloop
  endfacet
endsolid part_a
solid part_b
  facet normal 0 0 -1
    outer loop
      vertex 0 0 1
      vertex 1 0 1
      vertex 0 1 1
    endloop
  endfacet
endsolid part_b
";
        let model = StlImporter::read_ascii(Cursor::new(ascii)).expect("parse should succeed");
        assert_eq!(model.build.items.len(), 2, "should have 2 build items");

        let obj1 = model.resources.get_object(ResourceId(1)).expect("object 1");
        let obj2 = model.resources.get_object(ResourceId(2)).expect("object 2");

        if let lib3mf_core::model::Geometry::Mesh(m) = &obj1.geometry {
            assert_eq!(m.triangles.len(), 1);
        }
        if let lib3mf_core::model::Geometry::Mesh(m) = &obj2.geometry {
            assert_eq!(m.triangles.len(), 1);
        }
    }

    // ===== Test 7: solid name with spaces =====

    #[test]
    fn test_read_ascii_solid_name_with_spaces() {
        let ascii = "\
solid My Cool Part
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 0 1 0
    endloop
  endfacet
endsolid My Cool Part
";
        let model = StlImporter::read_ascii(Cursor::new(ascii)).expect("parse should succeed");
        let obj = model.resources.get_object(ResourceId(1)).expect("object 1");
        assert_eq!(obj.name, Some("My Cool Part".to_string()));
    }

    // ===== Test 8: vertex deduplication =====

    #[test]
    fn test_read_ascii_vertex_dedup() {
        // Two triangles sharing two vertices
        let ascii = "\
solid dedup
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 0 1 0
    endloop
  endfacet
  facet normal 0 0 1
    outer loop
      vertex 1 0 0
      vertex 1 1 0
      vertex 0 1 0
    endloop
  endfacet
endsolid dedup
";
        let model = StlImporter::read_ascii(Cursor::new(ascii)).expect("parse should succeed");
        let obj = model.resources.get_object(ResourceId(1)).expect("object 1");
        if let lib3mf_core::model::Geometry::Mesh(mesh) = &obj.geometry {
            assert_eq!(mesh.triangles.len(), 2);
            // 4 unique vertices: (0,0,0), (1,0,0), (0,1,0), (1,1,0)
            assert_eq!(
                mesh.vertices.len(),
                4,
                "shared vertices should be deduplicated"
            );
        } else {
            panic!("expected Mesh");
        }
    }

    // ===== Test 9: mismatched endsolid name is accepted =====

    #[test]
    fn test_read_ascii_mismatched_endsolid() {
        let ascii = "\
solid foo
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 0 1 0
    endloop
  endfacet
endsolid bar
";
        let model = StlImporter::read_ascii(Cursor::new(ascii)).expect("parse should succeed");
        assert_eq!(
            model.build.items.len(),
            1,
            "mismatched endsolid name should not cause error"
        );
    }

    // ===== Test 10: file ends without endsolid =====

    #[test]
    fn test_read_ascii_no_endsolid() {
        let ascii = "\
solid truncated
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 0 1 0
    endloop
  endfacet
";
        let model = StlImporter::read_ascii(Cursor::new(ascii)).expect("parse should succeed");
        assert_eq!(
            model.build.items.len(),
            1,
            "truncated file should still produce one object"
        );
        let obj = model.resources.get_object(ResourceId(1)).expect("object 1");
        if let lib3mf_core::model::Geometry::Mesh(mesh) = &obj.geometry {
            assert_eq!(mesh.triangles.len(), 1);
        }
    }

    // ===== Test 11: write_ascii_simple produces expected keywords =====

    #[test]
    fn test_write_ascii_simple() {
        // A single triangle in the XY plane with normal pointing +Z
        let model = make_simple_model(
            vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
            vec![(0, 1, 2)],
            Some("test"),
        );

        let mut output = Vec::new();
        AsciiStlExporter::write(&model, &mut output).expect("write should succeed");
        let text = String::from_utf8(output).expect("valid UTF-8");

        assert!(
            text.contains("solid test"),
            "should contain solid keyword with name"
        );
        assert!(text.contains("facet normal"), "should contain facet normal");
        assert!(text.contains("outer loop"), "should contain outer loop");
        assert!(text.contains("vertex"), "should contain vertex lines");
        assert!(text.contains("endloop"), "should contain endloop");
        assert!(text.contains("endfacet"), "should contain endfacet");
        assert!(
            text.contains("endsolid test"),
            "should contain endsolid keyword with name"
        );

        // Normal should be non-zero for a valid triangle
        // The normal should be (0, 0, 1) for the XY-plane triangle
        let has_nonzero_normal = text
            .lines()
            .filter(|l| l.trim().starts_with("facet normal"))
            .any(|l| {
                let parts: Vec<&str> = l.split_whitespace().collect();
                if parts.len() >= 5 {
                    let nz: f64 = parts[4].parse().unwrap_or(0.0);
                    nz.abs() > 0.5
                } else {
                    false
                }
            });
        assert!(
            has_nonzero_normal,
            "normal should be non-zero for valid triangle"
        );
    }

    // ===== Test 12: degenerate triangle emits zero normal =====

    #[test]
    fn test_write_ascii_degenerate_normal() {
        // All three vertices are collinear — degenerate triangle
        let model = make_simple_model(
            vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (2.0, 0.0, 0.0)],
            vec![(0, 1, 2)],
            None,
        );

        let mut output = Vec::new();
        AsciiStlExporter::write(&model, &mut output).expect("write should succeed");
        let text = String::from_utf8(output).expect("valid UTF-8");

        let normal_line = text
            .lines()
            .find(|l| l.trim().starts_with("facet normal"))
            .expect("should have facet normal line");

        let parts: Vec<&str> = normal_line.split_whitespace().collect();
        assert!(parts.len() >= 5, "facet normal line should have 5 parts");
        let nx: f64 = parts[2].parse().unwrap_or(f64::NAN);
        let ny: f64 = parts[3].parse().unwrap_or(f64::NAN);
        let nz: f64 = parts[4].parse().unwrap_or(f64::NAN);
        assert!(
            nx.abs() < 1e-6 && ny.abs() < 1e-6 && nz.abs() < 1e-6,
            "degenerate triangle normal should be (0,0,0), got ({nx}, {ny}, {nz})"
        );
    }

    // ===== Test 13: object name in solid/endsolid =====

    #[test]
    fn test_write_ascii_object_name() {
        let model = make_simple_model(
            vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
            vec![(0, 1, 2)],
            Some("MyPart"),
        );

        let mut output = Vec::new();
        AsciiStlExporter::write(&model, &mut output).expect("write should succeed");
        let text = String::from_utf8(output).expect("valid UTF-8");

        let first_line = text.lines().next().expect("should have lines");
        assert_eq!(
            first_line, "solid MyPart",
            "first line should be 'solid MyPart'"
        );

        let last_line = text
            .lines()
            .filter(|l| !l.is_empty())
            .last()
            .expect("should have lines");
        assert_eq!(
            last_line, "endsolid MyPart",
            "last line should be 'endsolid MyPart'"
        );
    }

    // ===== Test 14: roundtrip ASCII STL -> Model -> ASCII STL -> Model =====

    #[test]
    fn test_roundtrip_ascii() {
        // Create a simple model: 4 vertices, 2 triangles (a flat square)
        let model = make_simple_model(
            vec![
                (0.0, 0.0, 0.0),
                (1.0, 0.0, 0.0),
                (1.0, 1.0, 0.0),
                (0.0, 1.0, 0.0),
            ],
            vec![(0, 1, 2), (0, 2, 3)],
            Some("RoundtripTest"),
        );

        // Write to ASCII
        let mut buf1 = Vec::new();
        AsciiStlExporter::write(&model, &mut buf1).expect("first write should succeed");

        // Parse back
        let model2 =
            StlImporter::read_ascii(Cursor::new(&buf1)).expect("first re-read should succeed");

        // Write again
        let mut buf2 = Vec::new();
        AsciiStlExporter::write(&model2, &mut buf2).expect("second write should succeed");

        // Parse again
        let model3 =
            StlImporter::read_ascii(Cursor::new(&buf2)).expect("second re-read should succeed");

        // Compare: same vertex count, triangle count, and positions
        let get_mesh_info = |m: &Model| -> (usize, usize, Vec<(f32, f32, f32)>) {
            let obj = m.resources.get_object(ResourceId(1)).expect("object 1");
            if let lib3mf_core::model::Geometry::Mesh(mesh) = &obj.geometry {
                let verts: Vec<(f32, f32, f32)> =
                    mesh.vertices.iter().map(|v| (v.x, v.y, v.z)).collect();
                (mesh.vertices.len(), mesh.triangles.len(), verts)
            } else {
                panic!("expected Mesh");
            }
        };

        let (v_count2, t_count2, verts2) = get_mesh_info(&model2);
        let (v_count3, t_count3, verts3) = get_mesh_info(&model3);

        assert_eq!(
            v_count2, v_count3,
            "vertex count must be stable across roundtrips"
        );
        assert_eq!(
            t_count2, t_count3,
            "triangle count must be stable across roundtrips"
        );

        // Vertex positions should match within f32 formatting tolerance (1e-5)
        for (i, (&(x2, y2, z2), &(x3, y3, z3))) in verts2.iter().zip(verts3.iter()).enumerate() {
            assert!(
                (x2 - x3).abs() < 1e-5 && (y2 - y3).abs() < 1e-5 && (z2 - z3).abs() < 1e-5,
                "vertex {i} position mismatch: ({x2},{y2},{z2}) vs ({x3},{y3},{z3})"
            );
        }

        // Original model should also match second model structurally
        assert_eq!(v_count2, 4, "should have 4 vertices");
        assert_eq!(t_count2, 2, "should have 2 triangles");
    }

    // ===== Test 15: BinaryStlExporter::write produces correct binary output =====

    #[test]
    fn test_write_binary_simple() {
        use byteorder::{LittleEndian, ReadBytesExt};

        // One triangle: vertices at (0,0,0), (1,0,0), (0,1,0)
        let model = make_simple_model(
            vec![(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
            vec![(0, 1, 2)],
            None,
        );

        let mut buf = Vec::new();
        BinaryStlExporter::write(&model, Cursor::new(&mut buf)).expect("write should succeed");

        // Total: 80-byte header + 4-byte count + 1 * 50-byte triangle = 134
        assert_eq!(
            buf.len(),
            134,
            "binary STL size should be 134 bytes for 1 triangle"
        );

        // Triangle count at bytes 80..84
        let mut count_bytes = Cursor::new(&buf[80..84]);
        let tri_count = count_bytes.read_u32::<LittleEndian>().unwrap();
        assert_eq!(tri_count, 1, "triangle count should be 1");

        // Triangle data layout (per triangle):
        //   bytes 84..96:  normal (3 x f32 = 12 bytes)
        //   bytes 96..108: vertex 1 (3 x f32 = 12 bytes)
        //   bytes 108..120: vertex 2 (3 x f32 = 12 bytes)
        //   bytes 120..132: vertex 3 (3 x f32 = 12 bytes)
        //   bytes 132..134: attribute byte count (u16 = 2 bytes)
        let mut tri_cursor = Cursor::new(&buf[96..]);

        // Vertex 1: (0,0,0)
        let v1x = tri_cursor.read_f32::<LittleEndian>().unwrap();
        let v1y = tri_cursor.read_f32::<LittleEndian>().unwrap();
        let v1z = tri_cursor.read_f32::<LittleEndian>().unwrap();
        assert!((v1x - 0.0).abs() < 1e-6, "v1.x should be 0.0, got {v1x}");
        assert!((v1y - 0.0).abs() < 1e-6, "v1.y should be 0.0, got {v1y}");
        assert!((v1z - 0.0).abs() < 1e-6, "v1.z should be 0.0, got {v1z}");

        // Vertex 2: (1,0,0)
        let v2x = tri_cursor.read_f32::<LittleEndian>().unwrap();
        let v2y = tri_cursor.read_f32::<LittleEndian>().unwrap();
        let v2z = tri_cursor.read_f32::<LittleEndian>().unwrap();
        assert!((v2x - 1.0).abs() < 1e-6, "v2.x should be 1.0, got {v2x}");
        assert!((v2y - 0.0).abs() < 1e-6, "v2.y should be 0.0, got {v2y}");
        assert!((v2z - 0.0).abs() < 1e-6, "v2.z should be 0.0, got {v2z}");

        // Vertex 3: (0,1,0)
        let v3x = tri_cursor.read_f32::<LittleEndian>().unwrap();
        let v3y = tri_cursor.read_f32::<LittleEndian>().unwrap();
        let v3z = tri_cursor.read_f32::<LittleEndian>().unwrap();
        assert!((v3x - 0.0).abs() < 1e-6, "v3.x should be 0.0, got {v3x}");
        assert!((v3y - 1.0).abs() < 1e-6, "v3.y should be 1.0, got {v3y}");
        assert!((v3z - 0.0).abs() < 1e-6, "v3.z should be 0.0, got {v3z}");
    }

    // ===== Test 16: BinaryStlExporter::write roundtrip preserves triangle count =====

    #[test]
    fn test_roundtrip_binary() {
        // Create a flat square with 4 vertices and 2 triangles
        let model = make_simple_model(
            vec![
                (0.0, 0.0, 0.0),
                (1.0, 0.0, 0.0),
                (1.0, 1.0, 0.0),
                (0.0, 1.0, 0.0),
            ],
            vec![(0, 1, 2), (0, 2, 3)],
            None,
        );

        // Write to binary STL
        let mut buf = Vec::new();
        BinaryStlExporter::write(&model, Cursor::new(&mut buf)).expect("write should succeed");

        // Read back using StlImporter
        let model2 =
            StlImporter::read(Cursor::new(buf)).expect("binary roundtrip read should succeed");

        // Binary STL import deduplicates vertices, so only check triangle count
        let obj2 = model2
            .resources
            .get_object(ResourceId(1))
            .expect("object 1");
        if let lib3mf_core::model::Geometry::Mesh(mesh2) = &obj2.geometry {
            assert_eq!(
                mesh2.triangles.len(),
                2,
                "read-back should have 2 triangles"
            );
        } else {
            panic!("expected Mesh geometry");
        }
    }

    // ===== Test 17: BinaryStlExporter::write combines triangles from multiple build items =====

    #[test]
    fn test_write_binary_multi_object() {
        use byteorder::{LittleEndian, ReadBytesExt};
        use lib3mf_core::model::{Geometry, Object, ObjectType};

        // Object 1: 1 triangle
        let mut mesh1 = Mesh::default();
        mesh1.vertices.push(Vertex {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        mesh1.vertices.push(Vertex {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        });
        mesh1.vertices.push(Vertex {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        });
        mesh1.triangles.push(Triangle {
            v1: 0,
            v2: 1,
            v3: 2,
            ..Default::default()
        });

        let obj1 = Object {
            id: ResourceId(1),
            object_type: ObjectType::Model,
            name: None,
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::Mesh(mesh1),
        };

        // Object 2: 2 triangles
        let mut mesh2 = Mesh::default();
        mesh2.vertices.push(Vertex {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        });
        mesh2.vertices.push(Vertex {
            x: 1.0,
            y: 0.0,
            z: 1.0,
        });
        mesh2.vertices.push(Vertex {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        });
        mesh2.vertices.push(Vertex {
            x: 0.0,
            y: 1.0,
            z: 1.0,
        });
        mesh2.triangles.push(Triangle {
            v1: 0,
            v2: 1,
            v3: 2,
            ..Default::default()
        });
        mesh2.triangles.push(Triangle {
            v1: 0,
            v2: 2,
            v3: 3,
            ..Default::default()
        });

        let obj2 = Object {
            id: ResourceId(2),
            object_type: ObjectType::Model,
            name: None,
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::Mesh(mesh2),
        };

        let mut model = Model::default();
        let _ = model.resources.add_object(obj1);
        let _ = model.resources.add_object(obj2);
        model.build.items.push(BuildItem {
            object_id: ResourceId(1),
            transform: glam::Mat4::IDENTITY,
            part_number: None,
            uuid: None,
            path: None,
            printable: None,
        });
        model.build.items.push(BuildItem {
            object_id: ResourceId(2),
            transform: glam::Mat4::IDENTITY,
            part_number: None,
            uuid: None,
            path: None,
            printable: None,
        });

        let mut buf = Vec::new();
        BinaryStlExporter::write(&model, Cursor::new(&mut buf)).expect("write should succeed");

        // Total: 80 + 4 + 3 * 50 = 234 bytes
        assert_eq!(
            buf.len(),
            234,
            "binary STL size should be 234 bytes for 3 triangles"
        );

        // Triangle count at bytes 80..84 should be 3
        let mut count_cursor = Cursor::new(&buf[80..84]);
        let tri_count = count_cursor.read_u32::<LittleEndian>().unwrap();
        assert_eq!(tri_count, 3, "combined triangle count should be 3 (1 + 2)");
    }

    // ===== Test 18: auto-detect dispatches to binary parser =====

    #[test]
    fn test_auto_detect_read_binary() {
        let header = [0u8; 80];
        let tris = vec![(0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0)];
        let data = make_binary_stl(&header, &tris);
        let cursor = Cursor::new(data);
        let model = StlImporter::read(cursor).expect("auto-detect binary should succeed");

        assert_eq!(model.build.items.len(), 1);
        let obj = model.resources.get_object(ResourceId(1)).expect("object 1");
        if let lib3mf_core::model::Geometry::Mesh(mesh) = &obj.geometry {
            assert_eq!(mesh.triangles.len(), 1, "should have 1 triangle");
        } else {
            panic!("expected Mesh");
        }
    }

    // ===== Test 19: auto-detect dispatches to ASCII parser =====

    #[test]
    fn test_auto_detect_read_ascii() {
        let ascii = "\
solid autotest
  facet normal 0 0 1
    outer loop
      vertex 0 0 0
      vertex 1 0 0
      vertex 0 1 0
    endloop
  endfacet
endsolid autotest
";
        let cursor = Cursor::new(ascii.as_bytes().to_vec());
        let model = StlImporter::read(cursor).expect("auto-detect ASCII should succeed");

        assert_eq!(model.build.items.len(), 1);
        let obj = model.resources.get_object(ResourceId(1)).expect("object 1");
        assert_eq!(obj.name, Some("autotest".to_string()));
        if let lib3mf_core::model::Geometry::Mesh(mesh) = &obj.geometry {
            assert_eq!(mesh.triangles.len(), 1, "should have 1 triangle");
        } else {
            panic!("expected Mesh");
        }
    }
}
