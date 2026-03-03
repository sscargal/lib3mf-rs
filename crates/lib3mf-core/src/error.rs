//! Error handling for lib3mf-core.
//!
//! This module defines the error types returned by all fallible operations in the library.
//!
//! ## Design Philosophy
//!
//! The library follows a strict **no-panic** policy:
//!
//! - All errors are returned as `Result<T, Lib3mfError>`, never panicked
//! - Invalid user input (malformed 3MF files, bad parameters) produces errors, not panics
//! - Internal consistency violations are also returned as errors (via `InvalidStructure`)
//!
//! This makes the library safe to use in production environments where panics are unacceptable.
//!
//! ## Error Types
//!
//! [`Lib3mfError`] is the main error enum, with variants covering different failure modes:
//!
//! - **Io**: File system errors, network errors, ZIP reading failures
//! - **Validation**: Model failed validation checks (see [`crate::validation`])
//! - **ResourceNotFound**: Referenced resource ID doesn't exist in the model
//! - **InvalidStructure**: Malformed XML, missing required elements, spec violations
//! - **EncryptionError**: Cryptographic operations failed (wrong key, tampered data)
//! - **FeatureNotEnabled**: Operation requires a cargo feature that wasn't enabled
//!
//! ## Usage
//!
//! The [`Result`] type alias is provided for convenience:
//!
//! ```
//! use lib3mf_core::error::{Lib3mfError, Result};
//!
//! fn parse_something() -> Result<String> {
//!     // Use the ? operator to propagate errors
//!     Ok("success".to_string())
//! }
//! ```
//!
//! ## Error Context
//!
//! Most error variants include a `String` message with context about what failed:
//!
//! ```
//! use lib3mf_core::error::Lib3mfError;
//!
//! let error = Lib3mfError::InvalidStructure(
//!     "Missing required 'unit' attribute on <model> element".to_string()
//! );
//!
//! // Error messages are human-readable
//! assert_eq!(
//!     error.to_string(),
//!     "Invalid 3MF structure: Missing required 'unit' attribute on <model> element"
//! );
//! ```

use thiserror::Error;

/// The main error type for all fallible operations in lib3mf-core.
#[derive(Error, Debug)]
pub enum Lib3mfError {
    /// An I/O error occurred (file not found, permission denied, ZIP read failure, etc.).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The model failed a validation check at the requested level.
    #[error("Validation failed: {0}")]
    Validation(String),

    /// A referenced resource ID does not exist in the model.
    #[error("Resource not found: {0}")]
    ResourceNotFound(u32),

    /// The 3MF structure is malformed (missing required elements, spec violations, etc.).
    #[error("Invalid 3MF structure: {0}")]
    InvalidStructure(String),

    /// A cryptographic operation failed (wrong key, tampered data, unsupported algorithm).
    #[error("Encryption error: {0}")]
    EncryptionError(String),

    /// The requested operation requires a cargo feature that was not enabled at compile time.
    #[error("Feature not enabled: {0}. Rebuild with `cargo build --features {1}`")]
    FeatureNotEnabled(String, String),
}

/// Convenience type alias for `Result<T, Lib3mfError>`.
pub type Result<T> = std::result::Result<T, Lib3mfError>;
