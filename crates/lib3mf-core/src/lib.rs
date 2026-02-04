//! # lib3mf-core
//!
//! Pure Rust implementation of the 3D Manufacturing Format (3MF) specification.
//!
//! ## Overview
//!
//! 3MF is an XML-based file format for additive manufacturing (3D printing), developed by the 3MF Consortium.
//! It stores geometry, materials, colors, textures, and metadata in an OPC (Open Packaging Conventions) ZIP container.
//!
//! This crate provides a complete, memory-safe implementation of the 3MF Core Specification v1.4.0 and all major
//! extensions, including:
//!
//! - **Beam Lattice Extension**: Structural lattices with cylindrical beams
//! - **Boolean Operations Extension**: CSG operations on meshes
//! - **Displacement Extension**: Texture-driven surface modification
//! - **Materials and Properties Extension**: Base materials, color groups, textures, and composites
//! - **Production Extension**: UUIDs, part numbers, manufacturing metadata
//! - **Secure Content Extension**: Digital signatures and encryption (behind `crypto` feature)
//! - **Slice Extension**: 2D layer-based geometry for DLP/SLA printing
//! - **Volumetric Extension**: Voxel data representation
//!
//! ## Quick Start
//!
//! ```no_run
//! use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
//! use lib3mf_core::parser::parse_model;
//! use lib3mf_core::validation::ValidationLevel;
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Open the 3MF file (ZIP archive)
//! let file = File::open("model.3mf")?;
//! let mut archiver = ZipArchiver::new(file)?;
//!
//! // Locate the main model XML via OPC relationships
//! let model_path = find_model_path(&mut archiver)?;
//!
//! // Read and parse the model
//! let model_data = archiver.read_entry(&model_path)?;
//! let model = parse_model(std::io::Cursor::new(model_data))?;
//!
//! // Validate the model
//! let report = model.validate(ValidationLevel::Standard);
//! if report.has_errors() {
//!     for item in &report.items {
//!         eprintln!("Validation issue: {}", item.message);
//!     }
//! }
//!
//! // Access model data
//! println!("Unit: {:?}", model.unit);
//! println!("Build items: {}", model.build.items.len());
//! println!("Objects: {}", model.resources.iter_objects().count());
//! # Ok(())
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! By default, `lib3mf-core` is built with minimal dependencies (`default = []`). Optional features can be enabled
//! to add functionality at the cost of additional dependencies:
//!
//! | Feature | Description | Dependency Impact |
//! |---------|-------------|-------------------|
//! | `crypto` | Enables Secure Content Extension (digital signatures, encryption) | ~300 crates (rsa, aes-gcm, sha1, sha2, x509-parser, base64) |
//! | `parallel` | Enables multi-threaded mesh processing using Rayon | +1 crate |
//! | `png-validation` | Enables PNG texture validation | +1 crate |
//! | `full` | Enables all features: `crypto`, `parallel`, `png-validation` | All of the above |
//!
//! **Minimal build** (no features): ~154 crates
//! **Full build** (`--all-features`): ~300 crates
//!
//! ```toml
//! # Cargo.toml - minimal build (no crypto, no parallel)
//! [dependencies]
//! lib3mf-core = "0.1"
//!
//! # Cargo.toml - with crypto support
//! [dependencies]
//! lib3mf-core = { version = "0.1", features = ["crypto"] }
//!
//! # Cargo.toml - full-featured build
//! [dependencies]
//! lib3mf-core = { version = "0.1", features = ["full"] }
//! ```
//!
//! ## Modules
//!
//! - [`archive`]: OPC (Open Packaging Conventions) container and ZIP archive handling. Provides the [`ArchiveReader`]
//!   trait for abstracting over archive backends and [`find_model_path`] for discovering the main model XML.
//! - [`parser`]: XML-to-Model parsing pipeline. The primary entry point is [`parse_model`], which converts XML to an
//!   in-memory [`Model`] structure. For large files (>100MB), see [`parser::streaming`] for event-based parsing.
//! - [`model`]: Core data model structures: [`Model`], [`Object`], [`Mesh`], [`ResourceCollection`], [`Build`], [`BuildItem`].
//!   Follows an immutable-by-default design philosophy for thread safety and predictability.
//! - [`validation`]: Progressive validation system with four levels ([`ValidationLevel`]): Minimal (structure),
//!   Standard (reference integrity), Strict (spec compliance), and Paranoid (geometry analysis with BVH acceleration).
//! - [`writer`]: Model-to-XML-to-ZIP serialization pipeline. Mirrors the parser module structure but in reverse.
//! - [`crypto`] (feature gated): Secure Content Extension support for digital signatures and encryption. Requires
//!   `features = ["crypto"]` to enable.
//! - [`error`]: Error handling types. All library functions return [`Result<T>`][`crate::Result`] with [`Lib3mfError`]
//!   for failures. The library never panics on user input.
//!
//! ## Architecture
//!
//! The library follows a layered architecture:
//!
//! ```text
//! Archive Layer (ZIP/OPC) → Parser Layer (XML) → Model Layer (Immutable) → Validation → Writer Layer
//! ```
//!
//! 1. **Archive layer** ([`ZipArchiver`]) opens the 3MF file (ZIP container)
//! 2. **OPC parser** reads `_rels/.rels` to locate the main model XML via [`find_model_path`]
//! 3. **XML parser** ([`parse_model`]) converts XML to an in-memory [`Model`] structure
//! 4. **Model** contains resources (objects, materials, textures) and build instructions
//! 5. **Validation** applies progressive checks at different levels
//! 6. **Writer** serializes the model back to XML and ZIP
//!
//! ## Design Principles
//!
//! - **Immutable-by-default**: Model structures use Clone semantics. Mutation happens via explicit repair operations.
//! - **No panics**: All errors are returned as `Result<T, Lib3mfError>`. Invalid input never panics.
//! - **Trait-based abstraction**: [`ArchiveReader`], [`ArchiveWriter`], and other traits decouple implementation
//!   from interface.
//! - **Progressive validation**: Choose the validation level that matches your performance/correctness tradeoff.
//! - **Extension-first**: Extensions like Beam Lattice, Slice, and Boolean Operations are first-class citizens
//!   integrated into core structures.
//!
//! [`archive`]: crate::archive
//! [`parser`]: crate::parser
//! [`model`]: crate::model
//! [`validation`]: crate::validation
//! [`writer`]: crate::writer
//! [`crypto`]: crate::crypto
//! [`error`]: crate::error
//! [`ArchiveReader`]: crate::archive::ArchiveReader
//! [`ArchiveWriter`]: crate::archive::ArchiveWriter
//! [`find_model_path`]: crate::archive::find_model_path
//! [`parse_model`]: crate::parser::parse_model
//! [`Model`]: crate::model::Model
//! [`Object`]: crate::model::Object
//! [`Mesh`]: crate::model::Mesh
//! [`ResourceCollection`]: crate::model::ResourceCollection
//! [`Build`]: crate::model::Build
//! [`BuildItem`]: crate::model::BuildItem
//! [`ValidationLevel`]: crate::validation::ValidationLevel
//! [`Lib3mfError`]: crate::error::Lib3mfError
//! [`ZipArchiver`]: crate::archive::ZipArchiver

pub mod archive;
#[cfg(feature = "crypto")]
pub mod crypto;
pub mod error;
pub mod model;
pub mod parser;
pub mod utils;
pub mod validation;
pub mod writer;

pub use error::{Lib3mfError, Result};
pub use model::*;

#[cfg(test)]
mod tests {
    use super::*;
    // use glam::Vec3; // Removed unused import

    #[test]
    fn test_model_default() {
        let model = Model::default();
        assert_eq!(model.unit, Unit::Millimeter);
        assert!(model.metadata.is_empty());
    }

    #[test]
    fn test_mesh_construction() {
        let mut mesh = Mesh::new();
        let v1 = mesh.add_vertex(0.0, 0.0, 0.0);
        let v2 = mesh.add_vertex(1.0, 0.0, 0.0);
        let v3 = mesh.add_vertex(0.0, 1.0, 0.0);

        mesh.add_triangle(v1, v2, v3);

        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.triangles.len(), 1);

        let t = &mesh.triangles[0];
        assert_eq!(t.v1, 0);
        assert_eq!(t.v2, 1);
        assert_eq!(t.v3, 2);
    }

    #[test]
    fn test_resource_collection() {
        let mut resources = ResourceCollection::new();
        let mesh = Mesh::new();
        let object = Object {
            id: ResourceId(1),
            object_type: ObjectType::Model,
            name: Some("Test Object".to_string()),
            part_number: None,
            uuid: None,
            pid: None,
            thumbnail: None,
            pindex: None,
            geometry: Geometry::Mesh(mesh),
        };

        assert!(resources.add_object(object.clone()).is_ok());
        assert!(resources.add_object(object).is_err()); // Duplicate ID

        assert!(resources.get_object(ResourceId(1)).is_some());
        assert!(resources.get_object(ResourceId(2)).is_none());
    }
}
