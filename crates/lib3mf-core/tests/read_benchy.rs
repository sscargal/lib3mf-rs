use lib3mf_core::archive::{
    ArchiveReader, ZipArchiver, find_model_path,
    opc::{ContentType, parse_content_types},
};
use std::fs::File;
use std::path::PathBuf;

#[test]
fn test_read_benchy() {
    // Locate the test file
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop(); // crates
    d.pop(); // lib3mf-rs
    d.push("models");
    d.push("Benchy.3mf");

    // Check if models/Benchy.3mf exists. If not, we might be in CI environment without the file.
    // Ideally we should have it committed or downloaded.
    // For this test, we skip if not found, but print a warning.
    if !d.exists() {
        eprintln!("Test file not found at {:?}. Skipping integration test.", d);
        return;
    }

    let file = File::open(&d).expect("Failed to open Benchy.3mf");
    let mut archiver = ZipArchiver::new(file).expect("Failed to create ZipArchiver");

    // 1. Verify file listing
    let entries = archiver.list_entries().expect("Failed to list entries");
    assert!(!entries.is_empty(), "Archive should not be empty");
    println!("Entries found: {:?}", entries);

    // 2. Find Start Part via Relationships
    let model_path = find_model_path(&mut archiver).expect("Failed to find 3D model path");
    println!("Found model path: {}", model_path);
    assert!(!model_path.is_empty(), "Model path should not be empty");
    assert!(
        archiver.entry_exists(&model_path),
        "Model file referenced in _rels should exist"
    );

    // 3. Read Content Types
    if archiver.entry_exists("[Content_Types].xml") {
        let ct_data = archiver
            .read_entry("[Content_Types].xml")
            .expect("Failed to read [Content_Types].xml");
        let content_types = parse_content_types(&ct_data).expect("Failed to parse content types");
        println!("Content Types: {:?}", content_types);

        let has_3d_model_type = content_types.iter().any(|ct| match ct {
            ContentType::Default { content_type, .. } => {
                content_type == "application/vnd.ms-package.3dmanufacturing-3dmodel+xml"
            }
            ContentType::Override { content_type, .. } => {
                content_type == "application/vnd.ms-package.3dmanufacturing-3dmodel+xml"
            }
        });

        assert!(has_3d_model_type, "Should contain 3D Model content type");
    } else {
        panic!("[Content_Types].xml is missing (required by OPC)");
    }
}
