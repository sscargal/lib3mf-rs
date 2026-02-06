//! MUSTPASS conformance tests
//!
//! These tests verify that valid 3MF files are parsed successfully.
//! All files should parse without errors at Standard validation level.

use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
use lib3mf_core::parser::parse_model;
use lib3mf_core::validation::ValidationLevel;
use std::fs::File;
use super::mustpass_path;

/// Helper function to test a MUSTPASS file
fn test_mustpass_file(filename: &str) {
    let path = mustpass_path(filename);

    // Open the 3MF file
    let file = File::open(&path)
        .unwrap_or_else(|e| panic!("Failed to open {}: {}", filename, e));

    // Parse the archive
    let mut archiver = ZipArchiver::new(file)
        .unwrap_or_else(|e| panic!("Failed to create archiver for {}: {}", filename, e));

    // Find the main model path
    let model_path = find_model_path(&mut archiver)
        .unwrap_or_else(|e| panic!("Failed to find model path in {}: {}", filename, e));

    // Read and parse the model
    let model_data = archiver.read_entry(&model_path)
        .unwrap_or_else(|e| panic!("Failed to read model entry in {}: {}", filename, e));

    let model = parse_model(std::io::Cursor::new(model_data))
        .unwrap_or_else(|e| panic!("Failed to parse model in {}: {}", filename, e));

    // Validate at Standard level
    let report = model.validate(ValidationLevel::Standard)
        .unwrap_or_else(|e| panic!("Failed to validate {}: {}", filename, e));

    // MUSTPASS files should have no errors
    assert!(
        !report.has_errors(),
        "{} should not have validation errors, but got: {:?}",
        filename,
        report.errors()
    );
}

#[test]
fn test_mustpass_chapter2_1_parts_relationships() {
    test_mustpass_file("MUSTPASS_Chapter2.1_PartsRelationships.3mf");
}

#[test]
fn test_mustpass_chapter2_2_part_naming() {
    test_mustpass_file("MUSTPASS_Chapter2.2_PartNaming.3mf");
}

#[test]
fn test_mustpass_chapter2_3a_ignorable_markup() {
    test_mustpass_file("MUSTPASS_Chapter2.3a_IgnorableMarkup.3mf");
}

#[test]
fn test_mustpass_chapter3_2b_units_measurement_mm() {
    test_mustpass_file("MUSTPASS_Chapter3.2b_UnitsMeasurementMM.3mf");
}

#[test]
fn test_mustpass_chapter3_2c_multiple_items_transform() {
    test_mustpass_file("MUSTPASS_Chapter3.2c_MultipleItemsTransform.3mf");
}

#[test]
fn test_mustpass_chapter3_4_1c_must_ignore_undefined_metadata_name() {
    test_mustpass_file("MUSTPASS_Chapter3.4.1c_MustIgnoreUndefinedMetadataName.3mf");
}

#[test]
fn test_mustpass_chapter3_4_2_metadata_resources_build() {
    test_mustpass_file("MUSTPASS_Chapter3.4.2_MetaData_Resources_Build.3mf");
}

#[test]
fn test_mustpass_chapter3_4_3a_must_not_output_non_referenced_objects() {
    test_mustpass_file("MUSTPASS_Chapter3.4.3a_MustNotOutputNonReferencedObjects.3mf");
}

#[test]
fn test_mustpass_chapter4_1_explicit_support() {
    test_mustpass_file("MUSTPASS_Chapter4.1_ExplicitSupport.3mf");
}

#[test]
fn test_mustpass_chapter4_2_components() {
    test_mustpass_file("MUSTPASS_Chapter4.2_Components.3mf");
}

#[test]
fn test_mustpass_chapter5_1a_material_resources_composite_and_multi_properties() {
    test_mustpass_file("MUSTPASS_Chapter5.1a_MaterialResources_CompositeAndMultiProperties.3mf");
}

#[test]
fn test_mustpass_chapter5_1b_material_resources_multi_objects_composite_and_multi_properties() {
    test_mustpass_file("MUSTPASS_Chapter5.1b_MaterialResources_MultiObjects_CompositeAndMultiProperties.3mf");
}

#[test]
fn test_mustpass_chapter5_1c_material_resources_srgb_rgb_colors() {
    test_mustpass_file("MUSTPASS_Chapter5.1c_MaterialResources_sRGB_RGB_Colors.3mf");
}
