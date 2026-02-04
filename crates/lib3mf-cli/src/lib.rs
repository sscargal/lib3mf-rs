//! # lib3mf-cli
//!
//! Command-line tool for analyzing, validating, converting, and repairing 3MF files.
//!
//! ## Overview
//!
//! This crate provides the `3mf` command-line tool for working with 3D Manufacturing Format files.
//! While primarily a binary crate, it exposes its command implementations as a library to enable
//! programmatic usage and testing.
//!
//! ## CLI Commands
//!
//! The `3mf` binary supports the following commands:
//!
//! - **`stats`**: Report statistics and metadata (unit, geometry counts, materials)
//! - **`list`**: List all entries in the 3MF archive (flat or tree view)
//! - **`rels`**: Inspect OPC relationships and content types
//! - **`dump`**: Dump the raw parsed model structure for debugging
//! - **`extract`**: Extract a file from the archive by path or resource ID
//! - **`copy`**: Copy and re-package a 3MF file (verifies read/write cycle)
//! - **`convert`**: Convert between 3MF, STL, and OBJ formats
//! - **`validate`**: Validate a 3MF file at various strictness levels
//! - **`repair`**: Repair mesh geometry (stitch vertices, remove degenerates, harmonize orientations)
//! - **`sign`**: Sign a 3MF file using an RSA key (not yet implemented)
//! - **`verify`**: Verify digital signatures in a 3MF file (requires `crypto` feature)
//! - **`encrypt`**: Encrypt a 3MF file (not yet implemented)
//! - **`decrypt`**: Decrypt a 3MF file (not yet implemented)
//! - **`benchmark`**: Benchmark loading and parsing speed
//! - **`diff`**: Compare two 3MF files structurally
//! - **`thumbnails`**: Manage thumbnails (extract, inject, list)
//!
//! ## Installation
//!
//! ```bash
//! cargo install lib3mf-cli
//! ```
//!
//! ## CLI Usage Examples
//!
//! ```bash
//! # Report model statistics
//! 3mf stats model.3mf
//!
//! # Validate at strict level
//! 3mf validate model.3mf --level strict
//!
//! # Convert STL to 3MF
//! 3mf convert mesh.stl model.3mf
//!
//! # Repair a mesh with vertex stitching
//! 3mf repair broken.3mf fixed.3mf --epsilon 0.001
//! ```
//!
//! ## Programmatic Usage
//!
//! The command implementations are exposed as public functions in the [`commands`] module:
//!
//! ```no_run
//! use lib3mf_cli::commands::{OutputFormat, stats, validate};
//! use std::path::PathBuf;
//!
//! # fn main() -> anyhow::Result<()> {
//! // Generate statistics programmatically
//! stats(PathBuf::from("model.3mf"), OutputFormat::Json)?;
//!
//! // Validate a model
//! validate(PathBuf::from("model.3mf"), "standard".to_string())?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! - `crypto` (default): Enables signature verification via `lib3mf-core/crypto`
//! - `parallel` (default): Enables parallel mesh processing via `lib3mf-core/parallel`
//!
//! ## Cross-Reference
//!
//! This CLI tool is built on top of [`lib3mf_core`], which provides the underlying
//! 3MF parsing, validation, and serialization functionality. For detailed information
//! about the 3MF format and library architecture, see the `lib3mf-core` documentation.

pub mod commands;
