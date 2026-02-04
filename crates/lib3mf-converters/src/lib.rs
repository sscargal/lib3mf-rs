//! # lib3mf-converters
//!
//! Format converters for converting between 3MF and other 3D file formats.
//!
//! ## Overview
//!
//! This crate provides importers and exporters for converting STL and OBJ files to and from the 3MF format.
//! It builds on [`lib3mf_core`] to provide bi-directional conversion between these common 3D formats and the
//! full 3MF [`Model`] representation.
//!
//! **Supported formats:**
//! - **STL**: Binary STL only (80-byte header + triangle records)
//! - **OBJ**: Basic geometry only (vertices and faces, no materials/textures)
//!
//! ## Quick Start
//!
//! ```no_run
//! use lib3mf_converters::stl::StlImporter;
//! use lib3mf_core::validation::ValidationLevel;
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Import an STL file
//! let file = File::open("model.stl")?;
//! let model = StlImporter::read(file)?;
//!
//! // Validate the imported geometry
//! let report = model.validate(ValidationLevel::Standard);
//! if report.has_errors() {
//!     eprintln!("Imported model has validation errors");
//! }
//!
//! // Access the mesh data
//! println!("Imported {} objects", model.resources.iter_objects().count());
//! # Ok(())
//! # }
//! ```
//!
//! ## Modules
//!
//! - [`stl`]: Binary STL import and export
//! - [`obj`]: Wavefront OBJ import and export
//!
//! ## Limitations
//!
//! - **STL**: Only binary STL format is supported (ASCII STL is not supported)
//! - **OBJ**: Materials (mtllib/usemtl), texture coordinates (vt), and normals (vn) are ignored during import
//! - **OBJ**: Export does not include materials or textures, only geometry
//! - Vertex deduplication in STL import uses bitwise float comparison (exact match required)
//!
//! ## Cross-References
//!
//! This crate works with types from [`lib3mf_core`], particularly:
//! - [`lib3mf_core::model::Model`]: The primary 3MF model structure
//! - [`lib3mf_core::model::Mesh`]: Triangle mesh geometry
//! - [`lib3mf_core::error::Lib3mfError`]: Error type for conversion operations
//!
//! [`lib3mf_core`]: https://docs.rs/lib3mf-core
//! [`Model`]: https://docs.rs/lib3mf-core/latest/lib3mf_core/model/struct.Model.html
//! [`lib3mf_core::model::Model`]: https://docs.rs/lib3mf-core/latest/lib3mf_core/model/struct.Model.html
//! [`lib3mf_core::model::Mesh`]: https://docs.rs/lib3mf-core/latest/lib3mf_core/model/struct.Mesh.html
//! [`lib3mf_core::error::Lib3mfError`]: https://docs.rs/lib3mf-core/latest/lib3mf_core/error/enum.Lib3mfError.html

pub mod obj;
pub mod stl;
