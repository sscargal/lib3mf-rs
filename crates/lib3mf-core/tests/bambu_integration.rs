//! Integration tests for Bambu Studio 3MF file parsing.
//!
//! Tests against real Bambu Studio files in tmp/models/.
//! Each test skips gracefully if the test file is not available,
//! since these files live in tmp/models/ and may not be present in CI.

use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path};
use lib3mf_core::parser::parse_model;
use std::fs::File;
use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn model_path(filename: &str) -> std::path::PathBuf {
    repo_root().join("tmp/models").join(filename)
}

fn parse_bambu_file(
    filename: &str,
) -> (
    lib3mf_core::model::Model,
    lib3mf_core::model::stats::ModelStats,
) {
    let path = model_path(filename);

    if !path.exists() {
        panic!(
            "Test file not found: {}. Download Bambu test files to tmp/models/",
            path.display()
        );
    }

    let file = File::open(&path).expect("Failed to open test file");
    let mut archiver = ZipArchiver::new(file).expect("Failed to open ZIP");
    let model_path_str = find_model_path(&mut archiver).expect("Failed to find model path");
    let model_data = archiver
        .read_entry(&model_path_str)
        .expect("Failed to read model");
    let model = parse_model(std::io::Cursor::new(model_data)).expect("Failed to parse model");

    // Re-open for stats (archiver consumed by read)
    let file2 = File::open(&path).expect("Failed to reopen");
    let mut archiver2 = ZipArchiver::new(file2).expect("Failed to reopen ZIP");
    let stats = model
        .compute_stats(&mut archiver2)
        .expect("Failed to compute stats");

    (model, stats)
}

#[test]
fn test_benchy_bambu_printable_attribute() {
    let path = model_path("3DBenchy_PLA.3mf");
    if !path.exists() {
        return;
    }

    let (model, _) = parse_bambu_file("3DBenchy_PLA.3mf");

    // Build items should have printable attribute
    assert!(!model.build.items.is_empty(), "Build should have items");
    for item in &model.build.items {
        assert!(
            item.printable.is_some(),
            "Bambu build items should have printable attribute"
        );
    }
}

#[test]
fn test_benchy_bambu_namespace() {
    let path = model_path("3DBenchy_PLA.3mf");
    if !path.exists() {
        return;
    }

    let (model, _) = parse_bambu_file("3DBenchy_PLA.3mf");

    // Should have BambuStudio namespace
    assert!(
        model.extra_namespaces.contains_key("BambuStudio"),
        "BambuStudio namespace should be preserved"
    );
}

#[test]
fn test_benchy_vendor_data() {
    let path = model_path("3DBenchy_PLA.3mf");
    if !path.exists() {
        return;
    }

    let (_, stats) = parse_bambu_file("3DBenchy_PLA.3mf");

    // Slicer version should be populated
    assert!(
        stats.vendor.slicer_version.is_some(),
        "Slicer version should be populated"
    );

    // Should have filaments
    assert!(
        !stats.vendor.filaments.is_empty(),
        "Should have filament info"
    );
    let f = &stats.vendor.filaments[0];
    assert_eq!(f.type_, "PLA");
    assert!(f.used_m.is_some(), "Filament should have used_m");
    assert!(f.used_g.is_some(), "Filament should have used_g");

    // Should have print time
    assert!(
        stats.vendor.print_time_estimate.is_some(),
        "Should have print time estimate"
    );

    // Should have plates with items
    assert!(!stats.vendor.plates.is_empty(), "Should have plates");
    assert!(
        !stats.vendor.plates[0].items.is_empty(),
        "Plate should have model instances"
    );

    // Should have object metadata
    assert!(
        !stats.vendor.object_metadata.is_empty(),
        "Should have object metadata"
    );

    // Should have project settings
    assert!(
        stats.vendor.project_settings.is_some(),
        "Should have project settings"
    );

    // Printer model should be populated
    assert!(
        stats.vendor.printer_model.is_some(),
        "Printer model should be populated"
    );
}

#[test]
fn test_cube_profile_mini() {
    let path = model_path("Cube02ProfileMini.3mf");
    if !path.exists() {
        return;
    }

    let (model, stats) = parse_bambu_file("Cube02ProfileMini.3mf");

    // Basic sanity
    assert!(!model.build.items.is_empty());
    // Should have vendor data
    assert!(
        stats.vendor.slicer_version.is_some() || !stats.vendor.plates.is_empty(),
        "Cube profile should have some Bambu data"
    );
}

#[test]
fn test_simple_pyramid_minimal() {
    let path = model_path("SimplePyramid.3mf");
    if !path.exists() {
        return;
    }

    let (model, _stats) = parse_bambu_file("SimplePyramid.3mf");

    // SimplePyramid has minimal Bambu data (no plates in slice_info, no profile configs)
    // Should still parse without error
    assert!(!model.build.items.is_empty());

    // Main assertion: no panics, graceful handling of minimal file
}

#[test]
fn test_format_duration_via_print_time() {
    // 3DBenchy has prediction=1895 seconds => "31m 35s"
    let path = model_path("3DBenchy_PLA.3mf");
    if !path.exists() {
        return;
    }

    let (_, stats) = parse_bambu_file("3DBenchy_PLA.3mf");
    let time = stats
        .vendor
        .print_time_estimate
        .as_deref()
        .expect("Should have print time");
    // Verify human-friendly format (not raw seconds)
    assert!(
        !time.chars().all(|c| c.is_ascii_digit()),
        "Print time should be human-friendly, not raw seconds"
    );
    assert!(
        time.contains('m') || time.contains('h'),
        "Print time should contain 'm' or 'h' unit"
    );
}
