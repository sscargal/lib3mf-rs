use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path};
use lib3mf_core::parser::parse_model;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

#[test]
fn test_stats_deadpool() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.pop();
    d.pop();
    d.push("models");
    d.push("Deadpool_3_Mask.3mf");

    let file = File::open(&d).expect("Failed to open Deadpool_3_Mask.3mf");
    let mut archiver = ZipArchiver::new(file).expect("Failed to create archiver");

    // Parse Model
    let model_path = find_model_path(&mut archiver).expect("Failed to find model path");
    let model_data = archiver
        .read_entry(&model_path)
        .expect("Failed to read model");
    let model = parse_model(Cursor::new(model_data)).expect("Failed to parse model XML");

    // Compute Stats
    let stats = model
        .compute_stats(&mut archiver)
        .expect("Failed to compute stats");

    println!("Deadpool Stats: {:?}", stats);

    // Verify Vendor Data - Plates
    assert!(
        stats.vendor.plates.len() >= 3,
        "Should have at least 3 plates"
    );

    let main_plate = stats
        .vendor
        .plates
        .iter()
        .find(|p| p.name.as_deref() == Some("Main"));
    assert!(main_plate.is_some(), "Should have 'Main' plate");

    let back_plate = stats
        .vendor
        .plates
        .iter()
        .find(|p| p.name.as_deref() == Some("Back"));
    assert!(back_plate.is_some(), "Should have 'Back' plate");

    // Verify generator
    assert!(
        stats.generator.as_ref().unwrap().contains("Bambu"),
        "Should be Bambu Studio"
    );
}
