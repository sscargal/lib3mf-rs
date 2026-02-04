//! XML-to-Model parsing pipeline for 3MF files.
//!
//! This module converts XML content from a 3MF archive into the in-memory [`Model`](crate::model::Model)
//! structure. It handles the Core 3MF specification and all major extensions.
//!
//! ## Two Parsing Modes
//!
//! The parser provides two modes optimized for different use cases:
//!
//! ### DOM Mode (Default)
//!
//! The [`parse_model`] function loads the entire XML document into memory and constructs the complete
//! [`Model`](crate::model::Model) structure. This is:
//!
//! - **Fast**: Single-pass parsing with efficient XML handling
//! - **Simple**: Returns a complete model ready to use
//! - **Suitable for**: Files under 100MB (typical use case)
//!
//! ```no_run
//! use lib3mf_core::parser::parse_model;
//! use std::io::Cursor;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let xml_data = b"<model xmlns='http://schemas.microsoft.com/3dmanufacturing/core/2015/02'>...</model>";
//! let model = parse_model(Cursor::new(xml_data))?;
//! println!("Loaded {} objects", model.resources.iter_objects().count());
//! # Ok(())
//! # }
//! ```
//!
//! ### SAX/Streaming Mode (For Large Files)
//!
//! The [`streaming`] module provides event-based parsing with constant memory usage via the
//! [`ModelVisitor`](visitor::ModelVisitor) trait. This is:
//!
//! - **Memory-efficient**: Constant memory regardless of file size
//! - **Suitable for**: Files over 100MB, or memory-constrained environments
//! - **More complex**: Requires implementing visitor callbacks
//!
//! See [`streaming`] module documentation for details.
//!
//! ## Parser Architecture
//!
//! The parser is organized into specialized modules:
//!
//! ### Core Parsers
//!
//! - [`model_parser`]: Orchestrates parsing of the `<model>` root element
//! - [`mesh_parser`]: Parses `<mesh>` geometry (vertices, triangles)
//! - [`material_parser`]: Parses material resources (base materials, colors, textures, composites)
//! - [`build_parser`]: Parses `<build>` section (what to print and where)
//! - [`component_parser`]: Parses `<components>` (object references and transformations)
//!
//! ### Extension Parsers
//!
//! - [`beamlattice_parser`]: Beam Lattice Extension (structural lattices)
//! - [`slice_parser`]: Slice Extension (2D layer-based geometry for DLP/SLA)
//! - [`volumetric_parser`]: Volumetric Extension (voxel data)
//! - [`boolean_parser`]: Boolean Operations Extension (CSG operations)
//! - [`displacement_parser`]: Displacement Extension (texture-driven surface modification)
//! - [`crypto_parser`]: Digital signature metadata (always available, parses XML only)
//! - [`secure_content_parser`]: Secure Content Extension (encryption/decryption, requires `crypto` feature)
//!
//! ### Vendor Extensions
//!
//! - [`bambu_config`]: Bambu Studio project files (plates, filament data, print times)
//!
//! ## Crypto vs Secure Content
//!
//! The parser distinguishes between two related modules:
//!
//! - **`crypto_parser`**: Always available. Parses XML structure of `<Signature>` elements but doesn't
//!   verify them. This allows reading signed files without crypto dependencies.
//! - **`secure_content_parser`**: Behind `crypto` feature. Handles encryption/decryption and signature
//!   verification. Requires base64, aes-gcm, rsa, sha1, sha2, x509-parser dependencies.
//!
//! ## Error Handling
//!
//! All parsing functions return [`Result<T, Lib3mfError>`](crate::error::Result):
//!
//! - **InvalidStructure**: Malformed XML, missing required attributes, spec violations
//! - **Io**: File reading errors, ZIP errors
//! - **ResourceNotFound**: References to non-existent resource IDs
//! - **FeatureNotEnabled**: Trying to use `secure_content_parser` without `crypto` feature
//!
//! The parser never panics on invalid input.

pub mod bambu_config;
pub mod beamlattice_parser;
pub mod boolean_parser;
pub mod build_parser;
pub mod component_parser;
pub mod crypto_parser;
pub mod displacement_parser;
pub mod material_parser;
pub mod mesh_parser;
pub mod model_parser;
#[cfg(feature = "crypto")]
pub mod secure_content_parser;
pub mod slice_parser;
pub mod streaming;
pub mod visitor;
pub mod volumetric_parser;
pub mod xml_parser;

pub use bambu_config::parse_model_settings;
pub use crypto_parser::parse_signature;
/// Primary entry point for parsing 3MF model XML.
///
/// Converts XML content into an in-memory [`Model`](crate::model::Model) structure with all resources,
/// build instructions, and extension data.
///
/// # Parameters
///
/// - `reader`: Any type implementing `Read` (e.g., `File`, `Cursor<Vec<u8>>`, `&[u8]`)
///
/// # Returns
///
/// A complete [`Model`](crate::model::Model) with all parsed data.
///
/// # Errors
///
/// - [`Lib3mfError::InvalidStructure`](crate::error::Lib3mfError::InvalidStructure): Malformed XML or spec violations
/// - [`Lib3mfError::Io`](crate::error::Lib3mfError::Io): I/O errors reading the stream
///
/// # Examples
///
/// ```no_run
/// use lib3mf_core::parser::parse_model;
/// use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
/// use std::fs::File;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let file = File::open("model.3mf")?;
/// let mut archive = ZipArchiver::new(file)?;
/// let model_path = find_model_path(&mut archive)?;
/// let xml_data = archive.read_entry(&model_path)?;
///
/// let model = parse_model(std::io::Cursor::new(xml_data))?;
/// println!("Parsed model with {} objects", model.resources.iter_objects().count());
/// # Ok(())
/// # }
/// ```
pub use model_parser::parse_model;
pub use xml_parser::XmlParser;
