//! Official 3MF Consortium conformance test suite integration
//!
//! This integration test validates compliance with the 3MF specification using the
//! official test suite from: https://github.com/3MFConsortium/3mf-samples
//!
//! The tests are organized into two categories:
//! - mustpass: Tests that valid 3MF files parse successfully (13 tests)
//! - mustfail: Tests that invalid 3MF files fail appropriately (38 tests)

use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path};
use lib3mf_core::parser::parse_model;
use lib3mf_core::validation::ValidationLevel;
use std::fs::File;
use std::path::PathBuf;

/// Get the path to the conformance test samples directory
fn samples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap() // crates/
        .parent()
        .unwrap() // workspace root
        .join("tests")
        .join("conformance")
        .join("3mf-samples")
        .join("validation tests")
        .join("_archive")
        .join("3mf-Verify")
}

/// Get the path to a MUSTPASS test file
fn mustpass_path(filename: &str) -> PathBuf {
    samples_dir().join("MUSTPASS").join(filename)
}

/// Get the path to a MUSTFAIL test file
fn mustfail_path(filename: &str) -> PathBuf {
    samples_dir().join("MUSTFAIL").join(filename)
}

// =============================================================================
// MUSTPASS Tests - Valid files should parse successfully
// =============================================================================

/// Helper function to test a MUSTPASS file
fn test_mustpass_file(filename: &str) {
    let path = mustpass_path(filename);

    // Open the 3MF file
    let file = File::open(&path).unwrap_or_else(|e| panic!("Failed to open {}: {}", filename, e));

    // Parse the archive
    let mut archiver = ZipArchiver::new(file)
        .unwrap_or_else(|e| panic!("Failed to create archiver for {}: {}", filename, e));

    // Find the main model path
    let model_path = find_model_path(&mut archiver)
        .unwrap_or_else(|e| panic!("Failed to find model path in {}: {}", filename, e));

    // Read and parse the model
    let model_data = archiver
        .read_entry(&model_path)
        .unwrap_or_else(|e| panic!("Failed to read model entry in {}: {}", filename, e));

    let model = parse_model(std::io::Cursor::new(model_data))
        .unwrap_or_else(|e| panic!("Failed to parse model in {}: {}", filename, e));

    // Validate at Standard level
    let report = model.validate(ValidationLevel::Standard);

    // MUSTPASS files should have no errors
    assert!(
        !report.has_errors(),
        "{} should not have validation errors, but got: {:?}",
        filename,
        report
            .items
            .iter()
            .filter(|i| i.severity == lib3mf_core::validation::ValidationSeverity::Error)
            .collect::<Vec<_>>()
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
    test_mustpass_file(
        "MUSTPASS_Chapter5.1b_MaterialResources_MultiObjects_CompositeAndMultiProperties.3mf",
    );
}

#[test]
fn test_mustpass_chapter5_1c_material_resources_srgb_rgb_colors() {
    test_mustpass_file("MUSTPASS_Chapter5.1c_MaterialResources_sRGB_RGB_Colors.3mf");
}

// =============================================================================
// MUSTFAIL Tests - Invalid files should fail to parse or produce validation errors
// =============================================================================

/// Helper function to test a MUSTFAIL file
/// Returns true if the file failed to parse OR produced validation errors
fn test_mustfail_file(filename: &str) -> bool {
    let path = mustfail_path(filename);

    // Try to open the 3MF file
    let file = match File::open(&path) {
        Ok(f) => f,
        Err(_) => {
            // File doesn't exist - this is an error in the test setup
            panic!("Test file not found: {}", filename);
        }
    };

    // Try to parse the archive
    let mut archiver = match ZipArchiver::new(file) {
        Ok(a) => a,
        Err(_) => return true, // Failed to parse archive - that's a valid failure
    };

    // Try to find the main model path
    let model_path = match find_model_path(&mut archiver) {
        Ok(p) => p,
        Err(_) => return true, // Failed to find model path - that's a valid failure
    };

    // Try to read the model data
    let model_data = match archiver.read_entry(&model_path) {
        Ok(d) => d,
        Err(_) => return true, // Failed to read entry - that's a valid failure
    };

    // Try to parse the model
    let model = match parse_model(std::io::Cursor::new(model_data)) {
        Ok(m) => m,
        Err(_) => return true, // Failed to parse model - that's a valid failure
    };

    // If parsing succeeded, validate at Strict level
    let report = model.validate(ValidationLevel::Strict);

    // Check if validation found errors
    report.has_errors()
}

macro_rules! mustfail_test {
    ($name:ident, $filename:expr) => {
        #[test]
        fn $name() {
            assert!(
                test_mustfail_file($filename),
                "{} should fail to parse or produce validation errors",
                $filename
            );
        }
    };
}

// Extension tests - Materials and Properties
mustfail_test!(
    test_mustfail_extension_chapter2_duplicated_color_group_id,
    "MUSTFAIL_3MF100_Extension_Chapter2_DuplicatedColorGroupId.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter2_duplicated_id_across_group,
    "MUSTFAIL_3MF100_Extension_Chapter2_DuplicatedIdAcrossGroup.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter2_missing_color_value,
    "MUSTFAIL_3MF100_Extension_Chapter2_MissingColorValue.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter3_duplicated_texture2d_group,
    "MUSTFAIL_3MF100_Extension_Chapter3_DuplicatedTexture2DGroup.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter3_missing_tex_id,
    "MUSTFAIL_3MF100_Extension_Chapter3_MissingTexId.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter4a_duplicated_composite_materials,
    "MUSTFAIL_3MF100_Extension_Chapter4a_DuplicatedCompositeMaterials.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter4b_mat_id_is_not_base_materials,
    "MUSTFAIL_3MF100_Extension_Chapter4b_MatId_IsNotBaseMaterials.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter4c_missing_matid_mat_indices,
    "MUSTFAIL_3MF100_Extension_Chapter4c_MissingMatidMatIndices.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter4d_missing_mat_indices,
    "MUSTFAIL_3MF100_Extension_Chapter4d_MissingMatIndices.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter5a_missing_pids,
    "MUSTFAIL_3MF100_Extension_Chapter5a_MissingPIDs.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter5b_multiple_reference_to_base_and_composite_materials,
    "MUSTFAIL_3MF100_Extension_Chapter5b_MultipleReferenceToBaseAndCompositeMatterials.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter5b_multiple_reference_to_base_materials,
    "MUSTFAIL_3MF100_Extension_Chapter5b_MultipleReferenceToBaseMatterials.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter5b_refer_to_another_multi_properties,
    "MUSTFAIL_3MF100_Extension_Chapter5b_ReferToAnotherMultiProperties.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter5c_multiple_reference_to_colorgroup,
    "MUSTFAIL_3MF100_Extension_Chapter5c_MultipleReferenceToColorgroup.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter5d_multiple_reference_to_composite_materials,
    "MUSTFAIL_3MF100_Extension_Chapter5d_MultipleReferenceToCompositeMaterials.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter5e_reference_to_multi_properties,
    "MUSTFAIL_3MF100_Extension_Chapter5e_ReferenceToMultiProperties.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter6_2d_texture_invalid_content_type,
    "MUSTFAIL_3MF100_Extension_Chapter6_2DTexture_InvalidContentType.3mf"
);

mustfail_test!(
    test_mustfail_extension_chapter6_2d_texture_missing_path,
    "MUSTFAIL_3MF100_Extension_Chapter6_2DTexture_MissingPath.3mf"
);

// Core specification tests
mustfail_test!(
    test_mustfail_chapter2_1_1a_parts_relationships_non_exist_3d_model_part,
    "MUSTFAIL_Chapter2.1.1a_PartsRelationships_NonExist3DModelPart.3mf"
);

mustfail_test!(
    test_mustfail_chapter2_1_1b_parts_relationships_link_to_external,
    "MUSTFAIL_Chapter2.1.1b_PartsRelationships_LinkToExternal.3mf"
);

mustfail_test!(
    test_mustfail_chapter2_1_2_parts_relationships_more_than_one_3d_model_part,
    "MUSTFAIL_Chapter2.1.2_PartsRelationships_MoreThanOne3DModelPart.3mf"
);

mustfail_test!(
    test_mustfail_chapter2_1_3_parts_relationships_non_exist_thumbnail_part,
    "MUSTFAIL_Chapter2.1.3_PartsRelationships_NonExistThumbnailPart.3mf"
);

mustfail_test!(
    test_mustfail_chapter2_1_4_parts_relationships_more_than_one_print_ticket,
    "MUSTFAIL_Chapter2.1.4_PartsRelationships_MoreThanOnePrintTicket.3mf"
);

mustfail_test!(
    test_mustfail_chapter2_3_2a_non_utf_encoding,
    "MUSTFAIL_Chapter2.3.2a_NonUTF_Encoding.3mf"
);

mustfail_test!(
    test_mustfail_chapter2_3_2b_data_type_definition_in_xml_markup,
    "MUSTFAIL_Chapter2.3.2b_DataTypeDefinitionInXMLMarkup.3mf"
);

mustfail_test!(
    test_mustfail_chapter2_3_2c_undefined_namespace_in_xds,
    "MUSTFAIL_Chapter2.3.2c_UndefinedNameSpaceInXDS.3mf"
);

mustfail_test!(
    test_mustfail_chapter2_3_2d_undefined_xmlxsi_in_xsd_schema,
    "MUSTFAIL_Chapter2.3.2d_UndefinedXMLXSIInXSDSchema.3mf"
);

mustfail_test!(
    test_mustfail_chapter2_3_4_whitespace,
    "MUSTFAIL_Chapter2.3.4_WhiteSpace.3mf"
);

mustfail_test!(
    test_mustfail_chapter3_4_1a_missing_metadata_name,
    "MUSTFAIL_Chapter3.4.1a_MissingMetadataName.3mf"
);

mustfail_test!(
    test_mustfail_chapter3_4_1b_duplicated_metadata_name,
    "MUSTFAIL_Chapter3.4.1b_DuplicatedMetadataName.3mf"
);

mustfail_test!(
    test_mustfail_chapter3_4_3_1a_not_matching_object_id_in_build_item,
    "MUSTFAIL_Chapter3.4.3.1a_NotMatchingObjectIdInBuildItem.3mf"
);

mustfail_test!(
    test_mustfail_chapter3_4_3_1b_build_item_refer_to_object_of_type_other,
    "MUSTFAIL_Chapter3.4.3.1b_BuildItemReferToObjectOfTypeOther.3mf"
);

mustfail_test!(
    test_mustfail_chapter3_4a_more_than_one_model,
    "MUSTFAIL_Chapter3.4a_MoreThanOneModel.3mf"
);

mustfail_test!(
    test_mustfail_chapter3_4b_missing_build_element,
    "MUSTFAIL_Chapter3.4b_MissingBuildElement.3mf"
);

mustfail_test!(
    test_mustfail_chapter3_4c_missing_resource_element,
    "MUSTFAIL_Chapter3.4c_MissingResourceElement.3mf"
);

mustfail_test!(
    test_mustfail_chapter4a_invalid_object_type,
    "MUSTFAIL_Chapter4a_InvalidObjectType.3mf"
);

mustfail_test!(
    test_mustfail_chapter4b_pid_is_not_specified,
    "MUSTFAIL_Chapter4b_PID_IsNotSpecified.3mf"
);

mustfail_test!(
    test_mustfail_chapter5a_duplicated_base_materials,
    "MUSTFAIL_Chapter5a_DuplicatedBaseMaterials.3mf"
);
