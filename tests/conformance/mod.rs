//! Conformance tests using official 3MF Consortium test suite
//!
//! This module contains tests that verify compliance with the 3MF specification
//! using the official test suite from https://github.com/3MFConsortium/3mf-samples

pub mod mustpass;
pub mod mustfail;

use std::path::PathBuf;

/// Get the path to the conformance test samples directory
pub fn samples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("conformance")
        .join("3mf-samples")
        .join("validation tests")
        .join("_archive")
        .join("3mf-Verify")
}

/// Get the path to a MUSTPASS test file
pub fn mustpass_path(filename: &str) -> PathBuf {
    samples_dir().join("MUSTPASS").join(filename)
}

/// Get the path to a MUSTFAIL test file
pub fn mustfail_path(filename: &str) -> PathBuf {
    samples_dir().join("MUSTFAIL").join(filename)
}
