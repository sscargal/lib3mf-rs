//! MUSTFAIL conformance tests
//!
//! These tests verify that invalid 3MF files either fail to parse or produce validation errors.
//! Success for these tests means the parser correctly detects the error.

use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
use lib3mf_core::parser::parse_model;
use lib3mf_core::validation::ValidationLevel;
use std::fs::File;
use super::mustfail_path;

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
    let report = match model.validate(ValidationLevel::Strict) {
        Ok(r) => r,
        Err(_) => return true, // Validation itself failed - that's a valid failure
    };

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
mustfail_test!(test_mustfail_extension_chapter2_duplicated_color_group_id,
    "MUSTFAIL_3MF100_Extension_Chapter2_DuplicatedColorGroupId.3mf");

mustfail_test!(test_mustfail_extension_chapter2_duplicated_id_across_group,
    "MUSTFAIL_3MF100_Extension_Chapter2_DuplicatedIdAcrossGroup.3mf");

mustfail_test!(test_mustfail_extension_chapter2_missing_color_value,
    "MUSTFAIL_3MF100_Extension_Chapter2_MissingColorValue.3mf");

mustfail_test!(test_mustfail_extension_chapter3_duplicated_texture2d_group,
    "MUSTFAIL_3MF100_Extension_Chapter3_DuplicatedTexture2DGroup.3mf");

mustfail_test!(test_mustfail_extension_chapter3_missing_tex_id,
    "MUSTFAIL_3MF100_Extension_Chapter3_MissingTexId.3mf");

mustfail_test!(test_mustfail_extension_chapter4a_duplicated_composite_materials,
    "MUSTFAIL_3MF100_Extension_Chapter4a_DuplicatedCompositeMaterials.3mf");

mustfail_test!(test_mustfail_extension_chapter4b_mat_id_is_not_base_materials,
    "MUSTFAIL_3MF100_Extension_Chapter4b_MatId_IsNotBaseMaterials.3mf");

mustfail_test!(test_mustfail_extension_chapter4c_missing_matid_mat_indices,
    "MUSTFAIL_3MF100_Extension_Chapter4c_MissingMatidMatIndices.3mf");

mustfail_test!(test_mustfail_extension_chapter4d_missing_mat_indices,
    "MUSTFAIL_3MF100_Extension_Chapter4d_MissingMatIndices.3mf");

mustfail_test!(test_mustfail_extension_chapter5a_missing_pids,
    "MUSTFAIL_3MF100_Extension_Chapter5a_MissingPIDs.3mf");

mustfail_test!(test_mustfail_extension_chapter5b_multiple_reference_to_base_and_composite_materials,
    "MUSTFAIL_3MF100_Extension_Chapter5b_MultipleReferenceToBaseAndCompositeMatterials.3mf");

mustfail_test!(test_mustfail_extension_chapter5b_multiple_reference_to_base_materials,
    "MUSTFAIL_3MF100_Extension_Chapter5b_MultipleReferenceToBaseMatterials.3mf");

mustfail_test!(test_mustfail_extension_chapter5b_refer_to_another_multi_properties,
    "MUSTFAIL_3MF100_Extension_Chapter5b_ReferToAnotherMultiProperties.3mf");

mustfail_test!(test_mustfail_extension_chapter5c_multiple_reference_to_colorgroup,
    "MUSTFAIL_3MF100_Extension_Chapter5c_MultipleReferenceToColorgroup.3mf");

mustfail_test!(test_mustfail_extension_chapter5d_multiple_reference_to_composite_materials,
    "MUSTFAIL_3MF100_Extension_Chapter5d_MultipleReferenceToCompositeMaterials.3mf");

mustfail_test!(test_mustfail_extension_chapter5e_reference_to_multi_properties,
    "MUSTFAIL_3MF100_Extension_Chapter5e_ReferenceToMultiProperties.3mf");

mustfail_test!(test_mustfail_extension_chapter6_2d_texture_invalid_content_type,
    "MUSTFAIL_3MF100_Extension_Chapter6_2DTexture_InvalidContentType.3mf");

mustfail_test!(test_mustfail_extension_chapter6_2d_texture_missing_path,
    "MUSTFAIL_3MF100_Extension_Chapter6_2DTexture_MissingPath.3mf");

// Core specification tests
mustfail_test!(test_mustfail_chapter2_1_1a_parts_relationships_non_exist_3d_model_part,
    "MUSTFAIL_Chapter2.1.1a_PartsRelationships_NonExist3DModelPart.3mf");

mustfail_test!(test_mustfail_chapter2_1_1b_parts_relationships_link_to_external,
    "MUSTFAIL_Chapter2.1.1b_PartsRelationships_LinkToExternal.3mf");

mustfail_test!(test_mustfail_chapter2_1_2_parts_relationships_more_than_one_3d_model_part,
    "MUSTFAIL_Chapter2.1.2_PartsRelationships_MoreThanOne3DModelPart.3mf");

mustfail_test!(test_mustfail_chapter2_1_3_parts_relationships_non_exist_thumbnail_part,
    "MUSTFAIL_Chapter2.1.3_PartsRelationships_NonExistThumbnailPart.3mf");

mustfail_test!(test_mustfail_chapter2_1_4_parts_relationships_more_than_one_print_ticket,
    "MUSTFAIL_Chapter2.1.4_PartsRelationships_MoreThanOnePrintTicket.3mf");

mustfail_test!(test_mustfail_chapter2_3_2a_non_utf_encoding,
    "MUSTFAIL_Chapter2.3.2a_NonUTF_Encoding.3mf");

mustfail_test!(test_mustfail_chapter2_3_2b_data_type_definition_in_xml_markup,
    "MUSTFAIL_Chapter2.3.2b_DataTypeDefinitionInXMLMarkup.3mf");

mustfail_test!(test_mustfail_chapter2_3_2c_undefined_namespace_in_xds,
    "MUSTFAIL_Chapter2.3.2c_UndefinedNameSpaceInXDS.3mf");

mustfail_test!(test_mustfail_chapter2_3_2d_undefined_xmlxsi_in_xsd_schema,
    "MUSTFAIL_Chapter2.3.2d_UndefinedXMLXSIInXSDSchema.3mf");

mustfail_test!(test_mustfail_chapter2_3_4_whitespace,
    "MUSTFAIL_Chapter2.3.4_WhiteSpace.3mf");

mustfail_test!(test_mustfail_chapter3_4_1a_missing_metadata_name,
    "MUSTFAIL_Chapter3.4.1a_MissingMetadataName.3mf");

mustfail_test!(test_mustfail_chapter3_4_1b_duplicated_metadata_name,
    "MUSTFAIL_Chapter3.4.1b_DuplicatedMetadataName.3mf");

mustfail_test!(test_mustfail_chapter3_4_3_1a_not_matching_object_id_in_build_item,
    "MUSTFAIL_Chapter3.4.3.1a_NotMatchingObjectIdInBuildItem.3mf");

mustfail_test!(test_mustfail_chapter3_4_3_1b_build_item_refer_to_object_of_type_other,
    "MUSTFAIL_Chapter3.4.3.1b_BuildItemReferToObjectOfTypeOther.3mf");

mustfail_test!(test_mustfail_chapter3_4a_more_than_one_model,
    "MUSTFAIL_Chapter3.4a_MoreThanOneModel.3mf");

mustfail_test!(test_mustfail_chapter3_4b_missing_build_element,
    "MUSTFAIL_Chapter3.4b_MissingBuildElement.3mf");

mustfail_test!(test_mustfail_chapter3_4c_missing_resource_element,
    "MUSTFAIL_Chapter3.4c_MissingResourceElement.3mf");

mustfail_test!(test_mustfail_chapter4a_invalid_object_type,
    "MUSTFAIL_Chapter4a_InvalidObjectType.3mf");

mustfail_test!(test_mustfail_chapter4b_pid_is_not_specified,
    "MUSTFAIL_Chapter4b_PID_IsNotSpecified.3mf");

mustfail_test!(test_mustfail_chapter5a_duplicated_base_materials,
    "MUSTFAIL_Chapter5a_DuplicatedBaseMaterials.3mf");
