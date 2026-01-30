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

    // consume generic start, assuming we are inside or root
    // parse_keystore_content parses LOOP until </keystore>
    // We need to advance to <keystore> first?
    // parse_keystore_content logic:
    // loop { next() ... }
    // If we start at root, next() -> <keystore>.
    // Then check for consumer etc.

    // BUT `parse_keystore_content` expects to be *inside*?
    // Let's check impl:
    // It loops next(). match Start(e).
    // If it sees <consumer>, parses consumer.
    // If it sees <keystore>... wait.
    // If called with parser at "StartDocument" state.
    // Next -> Start(keystore).
    // Match "keystore"? Impl matches `consumer`, `resourcedatagroup`.
    // "keystore" falls to `_`. `read_to_end`?
    // If `read_to_end` consumes content of <keystore>, we skip everything!
    // FIX: `parse_keystore_content` should assume it is CALLED when `keystore` starts, OR handle `keystore` explicitly.
    // Currently it takes `uuid` arg, implying attributes already read?
    // Typical pattern: caller reads <keystore uuid="..."> then calls parse_content.

    // So in test:
    // Advance to <keystore>.
    // Call parse_content.

    while let Ok(event) = parser.read_next_event() {
        if let quick_xml::events::Event::Start(e) = event
            && e.local_name().as_ref() == b"keystore"
        {
            // Start parsing content
            // No uuid in sample XML, use random
            let ks = parse_keystore_content(&mut parser, Uuid::new_v4())?;

            assert_eq!(ks.consumers.len(), 1);
            assert_eq!(ks.consumers[0].id, "consumer-1");

            assert_eq!(ks.resource_data_groups.len(), 1);
            let grp = &ks.resource_data_groups[0];
            assert_eq!(
                grp.key_uuid,
                Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000")?
            );

            assert_eq!(grp.access_rights.len(), 1);
            assert_eq!(grp.access_rights[0].consumer_id, "consumer-1");
            assert_eq!(grp.access_rights[0].wrapped_key, b"Base64EncodedKeyData");
            break;
        }
    }

    Ok(())
}
