//! Model-to-XML-to-ZIP serialization pipeline for writing 3MF files.
//!
//! This module provides the inverse of the parser: it converts an in-memory [`Model`](crate::model::Model)
//! structure back into a valid 3MF archive (ZIP container with XML and attachments).
//!
//! ## Architecture
//!
//! The writer mirrors the parser module structure but in reverse:
//!
//! ```text
//! Model → XML Writer → OPC Writer → Package Writer → ZIP Archive
//! ```
//!
//! 1. **Model writer** ([`model_writer`]): Serializes the [`Model`](crate::model::Model) to XML
//! 2. **Mesh writer** ([`mesh_writer`]): Writes `<mesh>` elements with vertices and triangles
//! 3. **OPC writer** ([`opc_writer`]): Generates `_rels/.rels` and `[Content_Types].xml`
//! 4. **Package writer** ([`package_writer`]): Orchestrates full 3MF package creation
//! 5. **ZIP archive**: Compresses and writes the final `.3mf` file
//!
//! ## Usage
//!
//! The primary entry point is the [`write`](crate::model::Model::write) method on [`Model`](crate::model::Model):
//!
//! ```no_run
//! use lib3mf_core::Model;
//! use std::fs::File;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create or load a model
//! let model = Model::default();
//!
//! // Write to a 3MF file
//! let output = File::create("output.3mf")?;
//! model.write(output)?;
//!
//! println!("Model written to output.3mf");
//! # Ok(())
//! # }
//! ```
//!
//! ## Writer Modules
//!
//! ### Core Writers
//!
//! - [`model_writer`]: Writes the `<model>` root element and resources
//! - [`mesh_writer`]: Writes `<mesh>` geometry (vertices, triangles, properties)
//! - [`opc_writer`]: Writes OPC metadata (`_rels/.rels`, `[Content_Types].xml`)
//! - [`package_writer`]: Orchestrates writing of complete 3MF package
//! - [`xml_writer`]: Low-level XML writing utilities
//!
//! ### Extension Writers
//!
//! - [`displacement_writer`]: Writes Displacement Extension data
//! - **Beam Lattice**: Not yet implemented (partial support)
//! - **Slice**: Partial support (slicestackid attribute only)
//! - **Boolean Operations**: Fully supported in `model_writer`
//! - **Volumetric**: Fully supported in `model_writer`
//!
//! ## Known Limitations
//!
//! The writer has some gaps compared to the parser:
//!
//! - **Beam lattice writer**: Not implemented. Models with beam lattices can be read but not written back.
//! - **Slice writer**: Only writes `slicestackid` attribute, doesn't serialize full slice geometry.
//! - **Namespace optimization**: Always emits all namespace declarations rather than only needed ones.
//!
//! These limitations don't affect core 3MF functionality but prevent full roundtrip testing for these extensions.
//!
//! ## Roundtrip Testing
//!
//! For supported features, the writer produces output that can be parsed back to an equivalent model:
//!
//! ```no_run
//! use lib3mf_core::Model;
//! use std::io::Cursor;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let original_model = Model::default();
//!
//! // Write to memory
//! let mut buffer = Cursor::new(Vec::new());
//! original_model.write(&mut buffer)?;
//!
//! // Read back (would need to extract XML from ZIP in practice)
//! // let parsed_model = parse_model(buffer)?;
//! // assert_eq!(original_model, parsed_model);
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All writing functions return [`Result<(), Lib3mfError>`](crate::error::Result):
//!
//! - **Io**: File system errors, ZIP writing errors, out of disk space
//! - **InvalidStructure**: Model contains invalid data that can't be serialized
//!
//! The writer never panics on invalid input, though it may produce a 3MF file that fails validation.

pub mod displacement_writer;
pub mod mesh_writer;
pub mod model_write_zip;
pub mod model_writer;
pub mod opc_writer;
pub mod package_writer;
pub mod xml_writer;
