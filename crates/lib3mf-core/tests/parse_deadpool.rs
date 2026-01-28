use lib3mf_core::archive::{find_model_path, ArchiveReader, ZipArchiver};
use lib3mf_core::parser::parse_model;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

#[test]
fn test_parse_deadpool() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop();
    d.pop();
    d.push("models");
    d.push("Deadpool_3_Mask.3mf");

    if !d.exists() {
         eprintln!("Skipping test_parse_deadpool: File not found at {:?}", d);
        return;
    }

    let file = File::open(&d).expect("Failed to open Deadpool_3_Mask.3mf");
    let mut archiver = ZipArchiver::new(file).expect("Failed to create archiver");
    
    // Find Start Part
    let model_path = find_model_path(&mut archiver).expect("Failed to find model path");
    let model_data = archiver.read_entry(&model_path).expect("Failed to read model");

    let model = parse_model(Cursor::new(model_data)).expect("Failed to parse model XML");

    // 1. Verify Metadata
    assert_eq!(
         model.metadata.get("Title").map(|s| s.as_str()),
         Some("Deadpool 3 Movie Helmet Textured")
    );

    // 2. Verify Multi-Object Structure
    // Deadpool 3MF has multiple root objects (for different plates)
    let objects: Vec<_> = model.resources.iter_objects().collect();
    println!("Found {} objects", objects.len());
    
    // Based on unzip output, we expect IDs 2, 4, 6, 8, 10, 12, 14, 15
    assert!(objects.len() >= 8, "Should have at least 8 objects");
    
    // 3. Verify Components
    // Check object 2 (Mask front?)
    let obj2 = objects.iter().find(|o| o.id.0 == 2).expect("Object 2 not found");
    if let lib3mf_core::Geometry::Components(comps) = &obj2.geometry {
        assert_eq!(comps.components.len(), 1, "Object 2 should have 1 component");
    } else {
        panic!("Object 2 should be components");
    }
}
