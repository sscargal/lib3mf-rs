//! # lib3mf-async
//!
//! Asynchronous I/O support for loading 3MF files using tokio.
//!
//! ## Overview
//!
//! This crate provides async/non-blocking 3MF file loading for applications that require
//! concurrent I/O operations without blocking threads. It builds on [`lib3mf_core`] and uses
//! tokio for async runtime and async-zip for ZIP archive handling.
//!
//! **When to use this crate:**
//! - Web servers handling multiple concurrent requests
//! - Applications with UI threads that must remain responsive during file loading
//! - Systems processing multiple 3MF files concurrently
//! - Any async Rust application using tokio
//!
//! **When to use lib3mf-core instead:**
//! - Simple CLI tools or scripts
//! - Single-threaded applications
//! - Batch processing where blocking I/O is acceptable
//! - When you don't already have a tokio runtime
//!
//! ## Quick Start
//!
//! ```no_run
//! use lib3mf_async::loader::load_model_async;
//! use lib3mf_core::validation::ValidationLevel;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load a 3MF file asynchronously
//!     let model = load_model_async("model.3mf").await?;
//!
//!     // Validate and use the model
//!     let report = model.validate(ValidationLevel::Standard);
//!     println!("Loaded {} objects", model.resources.iter_objects().count());
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The async loading pipeline consists of:
//!
//! 1. **Async file open**: `tokio::fs::File::open()` - non-blocking file system access
//! 2. **Async ZIP reading**: [`AsyncZipArchive`] reads archive entries without blocking
//! 3. **Async OPC parsing**: Relationship discovery happens with async I/O
//! 4. **Spawn-blocked XML parsing**: CPU-bound XML parsing offloaded via `tokio::task::spawn_blocking`
//!
//! This hybrid approach keeps the tokio runtime responsive during I/O while leveraging blocking
//! threads for CPU-intensive parsing work.
//!
//! ## Modules
//!
//! - [`archive`]: Async archive reader trait ([`AsyncArchiveReader`]) and trait definition
//! - [`zip`]: Async ZIP implementation ([`AsyncZipArchive`]) using async-zip
//! - [`loader`]: High-level model loading function ([`load_model_async`])
//!
//! ## Runtime Requirements
//!
//! This crate requires a tokio runtime with both I/O and time drivers enabled. The examples
//! use `#[tokio::main]` which provides this automatically.
//!
//! ```toml
//! [dependencies]
//! lib3mf-async = "0.1"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! ## Cross-References
//!
//! This crate is the async counterpart to [`lib3mf_core`]. The [`Model`] type and all
//! validation/processing operations come from the core crate.
//!
//! [`lib3mf_core`]: https://docs.rs/lib3mf-core
//! [`Model`]: https://docs.rs/lib3mf-core/latest/lib3mf_core/model/struct.Model.html
//! [`AsyncArchiveReader`]: archive::AsyncArchiveReader
//! [`AsyncZipArchive`]: zip::AsyncZipArchive
//! [`load_model_async`]: loader::load_model_async

pub mod archive;
pub mod loader;
pub mod zip;
