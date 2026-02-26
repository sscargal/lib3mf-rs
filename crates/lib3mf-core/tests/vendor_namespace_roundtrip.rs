//! Integration test for vendor namespace (extra_namespaces) roundtrip fidelity.
//!
//! Verifies that namespace declarations added to `Model::extra_namespaces`
//! survive a write-parse cycle: they appear in the serialized XML and are
//! reconstructed in the parsed model.
//!
//! This is a pure synthetic test — no dependency on external .3mf files.

use lib3mf_core::model::Model;
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

/// Verify that multiple vendor namespaces inserted into `model.extra_namespaces`
/// are preserved through a full write-parse roundtrip.
#[test]
fn test_vendor_namespace_roundtrip() {
    let mut model = Model::default();

    // Insert two distinct vendor namespaces
    model.extra_namespaces.insert(
        "myvendor".to_string(),
        "http://example.com/myvendor/2024".to_string(),
    );
    model.extra_namespaces.insert(
        "custom".to_string(),
        "http://example.com/custom/ns".to_string(),
    );

    // Serialize to XML
    let mut buf: Vec<u8> = Vec::new();
    model.write_xml(&mut buf, None).expect("write_xml failed");
    let xml_str = String::from_utf8(buf.clone()).expect("non-UTF-8 output from write_xml");

    // The namespace declarations must appear in the serialized XML
    assert!(
        xml_str.contains(r#"xmlns:myvendor="http://example.com/myvendor/2024""#),
        "Expected xmlns:myvendor declaration in XML output.\nGot:\n{}",
        &xml_str[..xml_str.len().min(800)]
    );
    assert!(
        xml_str.contains(r#"xmlns:custom="http://example.com/custom/ns""#),
        "Expected xmlns:custom declaration in XML output.\nGot:\n{}",
        &xml_str[..xml_str.len().min(800)]
    );

    // Parse back and verify extra_namespaces are reconstructed
    let parsed = parse_model(Cursor::new(buf)).expect("parse_model failed after roundtrip");

    assert_eq!(
        parsed.extra_namespaces.get("myvendor"),
        Some(&"http://example.com/myvendor/2024".to_string()),
        "myvendor namespace not preserved after roundtrip"
    );
    assert_eq!(
        parsed.extra_namespaces.get("custom"),
        Some(&"http://example.com/custom/ns".to_string()),
        "custom namespace not preserved after roundtrip"
    );
}
