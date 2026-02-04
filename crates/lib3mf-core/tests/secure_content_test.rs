#![cfg(feature = "crypto")]

use base64::prelude::*;
use lib3mf_core::parser::secure_content_parser::parse_keystore_content;
use lib3mf_core::parser::xml_parser::XmlParser;
use std::io::Cursor;
use uuid::Uuid;

#[test]
fn test_parse_secure_content() -> anyhow::Result<()> {
    let xml = r##"
        <keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/secure_content/2019/04/keystore">
            <consumer consumerid="consumer-1" keyid="key-1" keyvalue="val" />
            <resourcedatagroup keyuuid="123e4567-e89b-12d3-a456-426614174000">
                <accessright consumerid="consumer-1">
                    <wrappedkey encryptionalgorithm="RSA-OAEP">Base64EncodedKeyData</wrappedkey>
                </accessright>
            </resourcedatagroup>
        </keystore>
    "##;

    let mut parser = XmlParser::new(Cursor::new(xml));

    while let Ok(event) = parser.read_next_event() {
        if let quick_xml::events::Event::Start(e) = event
            && e.local_name().as_ref() == b"keystore"
        {
            let ks = parse_keystore_content(&mut parser, Uuid::new_v4())?;

            assert_eq!(ks.consumers.len(), 1, "Expected 1 consumer");
            assert_eq!(ks.consumers[0].id, "consumer-1", "Consumer ID mismatch");

            assert_eq!(
                ks.resource_data_groups.len(),
                1,
                "Expected 1 resource data group"
            );
            let grp = &ks.resource_data_groups[0];
            assert_eq!(
                grp.key_uuid,
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000")?,
                "Key UUID mismatch"
            );

            assert_eq!(grp.access_rights.len(), 1, "Expected 1 access right");
            assert_eq!(
                grp.access_rights[0].consumer_id, "consumer-1",
                "Access right consumer ID mismatch"
            );
            let expected_key = BASE64_STANDARD.decode("Base64EncodedKeyData")?;
            assert_eq!(
                grp.access_rights[0].wrapped_key, expected_key,
                "Wrapped key mismatch"
            );
            break;
        }
    }

    Ok(())
}

#[test]
fn test_multiple_consumers_with_different_attributes() -> anyhow::Result<()> {
    let xml = r##"
        <keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/secure_content/2019/04/keystore">
            <consumer consumerid="alice@example.com" keyid="key-001" keyvalue="value-001" />
            <consumer consumerid="bob@example.com" />
            <consumer consumerid="charlie@example.com" keyid="key-003" />
        </keystore>
    "##;

    let mut parser = XmlParser::new(Cursor::new(xml));

    while let Ok(event) = parser.read_next_event() {
        if let quick_xml::events::Event::Start(e) = event
            && e.local_name().as_ref() == b"keystore"
        {
            let ks = parse_keystore_content(&mut parser, Uuid::new_v4())?;

            assert_eq!(ks.consumers.len(), 3, "Expected 3 consumers");

            // Consumer with all attributes
            assert_eq!(
                ks.consumers[0].id, "alice@example.com",
                "Consumer 1 ID mismatch"
            );
            assert_eq!(
                ks.consumers[0].key_id,
                Some("key-001".to_string()),
                "Consumer 1 key_id mismatch"
            );
            assert_eq!(
                ks.consumers[0].key_value,
                Some("value-001".to_string()),
                "Consumer 1 key_value mismatch"
            );

            // Consumer with only required attribute
            assert_eq!(
                ks.consumers[1].id, "bob@example.com",
                "Consumer 2 ID mismatch"
            );
            assert_eq!(
                ks.consumers[1].key_id, None,
                "Consumer 2 key_id should be None"
            );
            assert_eq!(
                ks.consumers[1].key_value, None,
                "Consumer 2 key_value should be None"
            );

            // Consumer with partial optional attributes
            assert_eq!(
                ks.consumers[2].id, "charlie@example.com",
                "Consumer 3 ID mismatch"
            );
            assert_eq!(
                ks.consumers[2].key_id,
                Some("key-003".to_string()),
                "Consumer 3 key_id mismatch"
            );
            assert_eq!(
                ks.consumers[2].key_value, None,
                "Consumer 3 key_value should be None"
            );

            break;
        }
    }

    Ok(())
}

#[test]
fn test_multiple_resource_data_groups() -> anyhow::Result<()> {
    let xml = r##"
        <keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/secure_content/2019/04/keystore">
            <consumer consumerid="alice@example.com" />
            <consumer consumerid="bob@example.com" />
            <resourcedatagroup keyuuid="11111111-1111-1111-1111-111111111111">
                <accessright consumerid="alice@example.com">
                    <wrappedkey encryptionalgorithm="RSA-OAEP">YWxpY2VrZXk=</wrappedkey>
                </accessright>
            </resourcedatagroup>
            <resourcedatagroup keyuuid="22222222-2222-2222-2222-222222222222">
                <accessright consumerid="alice@example.com">
                    <wrappedkey encryptionalgorithm="RSA-OAEP">YWxpY2VrZXky</wrappedkey>
                </accessright>
                <accessright consumerid="bob@example.com">
                    <wrappedkey encryptionalgorithm="RSA-OAEP">Ym9ia2V5</wrappedkey>
                </accessright>
            </resourcedatagroup>
        </keystore>
    "##;

    let mut parser = XmlParser::new(Cursor::new(xml));

    while let Ok(event) = parser.read_next_event() {
        if let quick_xml::events::Event::Start(e) = event
            && e.local_name().as_ref() == b"keystore"
        {
            let ks = parse_keystore_content(&mut parser, Uuid::new_v4())?;

            assert_eq!(ks.consumers.len(), 2, "Expected 2 consumers");
            assert_eq!(
                ks.resource_data_groups.len(),
                2,
                "Expected 2 resource data groups"
            );

            // First group - single access right
            let grp1 = &ks.resource_data_groups[0];
            assert_eq!(
                grp1.key_uuid,
                Uuid::parse_str("11111111-1111-1111-1111-111111111111")?,
                "Group 1 UUID mismatch"
            );
            assert_eq!(
                grp1.access_rights.len(),
                1,
                "Group 1 should have 1 access right"
            );
            assert_eq!(
                grp1.access_rights[0].consumer_id, "alice@example.com",
                "Group 1 consumer mismatch"
            );

            // Second group - two access rights
            let grp2 = &ks.resource_data_groups[1];
            assert_eq!(
                grp2.key_uuid,
                Uuid::parse_str("22222222-2222-2222-2222-222222222222")?,
                "Group 2 UUID mismatch"
            );
            assert_eq!(
                grp2.access_rights.len(),
                2,
                "Group 2 should have 2 access rights"
            );
            assert_eq!(
                grp2.access_rights[0].consumer_id, "alice@example.com",
                "Group 2 first consumer mismatch"
            );
            assert_eq!(
                grp2.access_rights[1].consumer_id, "bob@example.com",
                "Group 2 second consumer mismatch"
            );

            break;
        }
    }

    Ok(())
}

#[test]
fn test_encryption_algorithms() -> anyhow::Result<()> {
    let xml = r##"
        <keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/secure_content/2019/04/keystore">
            <consumer consumerid="consumer-1" />
            <resourcedatagroup keyuuid="123e4567-e89b-12d3-a456-426614174000">
                <accessright consumerid="consumer-1">
                    <wrappedkey encryptionalgorithm="RSA-OAEP">dGVzdGRhdGEx</wrappedkey>
                </accessright>
            </resourcedatagroup>
            <resourcedatagroup keyuuid="223e4567-e89b-12d3-a456-426614174000">
                <accessright consumerid="consumer-1">
                    <wrappedkey encryptionalgorithm="AES-KW">dGVzdGRhdGEy</wrappedkey>
                </accessright>
            </resourcedatagroup>
            <resourcedatagroup keyuuid="323e4567-e89b-12d3-a456-426614174000">
                <accessright consumerid="consumer-1">
                    <wrappedkey encryptionalgorithm="UNKNOWN-ALGORITHM">dGVzdGRhdGEz</wrappedkey>
                </accessright>
            </resourcedatagroup>
        </keystore>
    "##;

    let mut parser = XmlParser::new(Cursor::new(xml));

    while let Ok(event) = parser.read_next_event() {
        if let quick_xml::events::Event::Start(e) = event
            && e.local_name().as_ref() == b"keystore"
        {
            let ks = parse_keystore_content(&mut parser, Uuid::new_v4())?;

            assert_eq!(
                ks.resource_data_groups.len(),
                3,
                "Expected 3 resource data groups"
            );

            // RSA-OAEP algorithm
            assert_eq!(
                ks.resource_data_groups[0].access_rights[0].algorithm, "RSA-OAEP",
                "Algorithm 1 mismatch"
            );

            // AES-KW algorithm
            assert_eq!(
                ks.resource_data_groups[1].access_rights[0].algorithm, "AES-KW",
                "Algorithm 2 mismatch"
            );

            // Unknown algorithm (should parse and preserve)
            assert_eq!(
                ks.resource_data_groups[2].access_rights[0].algorithm, "UNKNOWN-ALGORITHM",
                "Algorithm 3 mismatch"
            );

            break;
        }
    }

    Ok(())
}

#[test]
fn test_base64_wrapped_key_decoding() -> anyhow::Result<()> {
    let xml = r##"
        <keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/secure_content/2019/04/keystore">
            <consumer consumerid="consumer-1" />
            <resourcedatagroup keyuuid="123e4567-e89b-12d3-a456-426614174000">
                <accessright consumerid="consumer-1">
                    <wrappedkey encryptionalgorithm="RSA-OAEP">SGVsbG8sIFdvcmxkIQ==</wrappedkey>
                </accessright>
            </resourcedatagroup>
        </keystore>
    "##;

    let mut parser = XmlParser::new(Cursor::new(xml));

    while let Ok(event) = parser.read_next_event() {
        if let quick_xml::events::Event::Start(e) = event
            && e.local_name().as_ref() == b"keystore"
        {
            let ks = parse_keystore_content(&mut parser, Uuid::new_v4())?;

            let wrapped_key = &ks.resource_data_groups[0].access_rights[0].wrapped_key;
            let expected = b"Hello, World!";

            assert_eq!(
                wrapped_key.as_slice(),
                expected,
                "Base64 decoded key should match expected value"
            );

            break;
        }
    }

    Ok(())
}

#[test]
fn test_invalid_uuid_format() -> anyhow::Result<()> {
    let xml = r##"
        <keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/secure_content/2019/04/keystore">
            <consumer consumerid="consumer-1" />
            <resourcedatagroup keyuuid="not-a-valid-uuid">
                <accessright consumerid="consumer-1">
                    <wrappedkey>dGVzdA==</wrappedkey>
                </accessright>
            </resourcedatagroup>
        </keystore>
    "##;

    let mut parser = XmlParser::new(Cursor::new(xml));

    while let Ok(event) = parser.read_next_event() {
        if let quick_xml::events::Event::Start(e) = event
            && e.local_name().as_ref() == b"keystore"
        {
            let result = parse_keystore_content(&mut parser, Uuid::new_v4());
            assert!(result.is_err(), "Should fail with invalid UUID");

            if let Err(e) = result {
                let err_msg = format!("{}", e);
                assert!(
                    err_msg.contains("Invalid keyuuid") || err_msg.contains("uuid"),
                    "Error message should mention invalid UUID: {}",
                    err_msg
                );
            }

            break;
        }
    }

    Ok(())
}

#[test]
fn test_missing_consumerid_in_consumer() -> anyhow::Result<()> {
    let xml = r##"
        <keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/secure_content/2019/04/keystore">
            <consumer keyid="key-1" keyvalue="value-1" />
        </keystore>
    "##;

    let mut parser = XmlParser::new(Cursor::new(xml));

    while let Ok(event) = parser.read_next_event() {
        if let quick_xml::events::Event::Start(e) = event
            && e.local_name().as_ref() == b"keystore"
        {
            let result = parse_keystore_content(&mut parser, Uuid::new_v4());
            assert!(result.is_err(), "Should fail with missing consumerid");

            if let Err(e) = result {
                let err_msg = format!("{}", e);
                assert!(
                    err_msg.contains("Missing consumerid"),
                    "Error message should mention missing consumerid: {}",
                    err_msg
                );
            }

            break;
        }
    }

    Ok(())
}

#[test]
fn test_missing_keyuuid_in_resourcedatagroup() -> anyhow::Result<()> {
    let xml = r##"
        <keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/secure_content/2019/04/keystore">
            <consumer consumerid="consumer-1" />
            <resourcedatagroup>
                <accessright consumerid="consumer-1">
                    <wrappedkey>dGVzdA==</wrappedkey>
                </accessright>
            </resourcedatagroup>
        </keystore>
    "##;

    let mut parser = XmlParser::new(Cursor::new(xml));

    while let Ok(event) = parser.read_next_event() {
        if let quick_xml::events::Event::Start(e) = event
            && e.local_name().as_ref() == b"keystore"
        {
            let result = parse_keystore_content(&mut parser, Uuid::new_v4());
            assert!(result.is_err(), "Should fail with missing keyuuid");

            if let Err(e) = result {
                let err_msg = format!("{}", e);
                assert!(
                    err_msg.contains("Missing keyuuid"),
                    "Error message should mention missing keyuuid: {}",
                    err_msg
                );
            }

            break;
        }
    }

    Ok(())
}

#[test]
fn test_missing_consumerid_in_accessright() -> anyhow::Result<()> {
    let xml = r##"
        <keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/secure_content/2019/04/keystore">
            <consumer consumerid="consumer-1" />
            <resourcedatagroup keyuuid="123e4567-e89b-12d3-a456-426614174000">
                <accessright>
                    <wrappedkey>dGVzdA==</wrappedkey>
                </accessright>
            </resourcedatagroup>
        </keystore>
    "##;

    let mut parser = XmlParser::new(Cursor::new(xml));

    while let Ok(event) = parser.read_next_event() {
        if let quick_xml::events::Event::Start(e) = event
            && e.local_name().as_ref() == b"keystore"
        {
            let result = parse_keystore_content(&mut parser, Uuid::new_v4());
            assert!(
                result.is_err(),
                "Should fail with missing consumerid in accessright"
            );

            if let Err(e) = result {
                let err_msg = format!("{}", e);
                assert!(
                    err_msg.contains("Missing consumerid"),
                    "Error message should mention missing consumerid: {}",
                    err_msg
                );
            }

            break;
        }
    }

    Ok(())
}

#[test]
fn test_empty_keystore() -> anyhow::Result<()> {
    let xml = r##"
        <keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/secure_content/2019/04/keystore">
        </keystore>
    "##;

    let mut parser = XmlParser::new(Cursor::new(xml));

    while let Ok(event) = parser.read_next_event() {
        if let quick_xml::events::Event::Start(e) = event
            && e.local_name().as_ref() == b"keystore"
        {
            let ks = parse_keystore_content(&mut parser, Uuid::new_v4())?;

            assert_eq!(
                ks.consumers.len(),
                0,
                "Empty keystore should have no consumers"
            );
            assert_eq!(
                ks.resource_data_groups.len(),
                0,
                "Empty keystore should have no resource data groups"
            );

            break;
        }
    }

    Ok(())
}

#[test]
fn test_truncated_xml_eof_handling() -> anyhow::Result<()> {
    let xml = r##"
        <keystore xmlns="http://schemas.microsoft.com/3dmanufacturing/secure_content/2019/04/keystore">
            <consumer consumerid="consumer-1" />
            <resourcedatagroup keyuuid="123e4567-e89b-12d3-a456-426614174000">
    "##;

    let mut parser = XmlParser::new(Cursor::new(xml));

    while let Ok(event) = parser.read_next_event() {
        if let quick_xml::events::Event::Start(e) = event
            && e.local_name().as_ref() == b"keystore"
        {
            let result = parse_keystore_content(&mut parser, Uuid::new_v4());
            assert!(result.is_err(), "Should fail with truncated XML");

            if let Err(e) = result {
                let err_msg = format!("{}", e);
                assert!(
                    err_msg.contains("EOF") || err_msg.contains("Unexpected"),
                    "Error message should mention EOF or unexpected end: {}",
                    err_msg
                );
            }

            break;
        }
    }

    Ok(())
}
