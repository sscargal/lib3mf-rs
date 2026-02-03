use crate::error::{Lib3mfError, Result};
use crate::model::{AccessRight, Consumer, KeyStore, ResourceDataGroup};
use crate::parser::xml_parser::{XmlParser, get_attribute};
use base64::prelude::*;
use quick_xml::events::Event;
use std::borrow::Cow;
use std::io::BufRead;

pub fn parse_keystore_content<R: BufRead>(
    parser: &mut XmlParser<R>,
    uuid: uuid::Uuid,
) -> Result<KeyStore> {
    let mut store = KeyStore {
        uuid,
        ..Default::default()
    };

    loop {
        match parser.read_next_event()? {
            Event::Start(e) => {
                let local_name = e.local_name();
                match local_name.as_ref() {
                    b"consumer" => {
                        let id = get_attribute(&e, b"consumerid")
                            .ok_or(Lib3mfError::Validation("Missing consumerid".to_string()))?
                            .into_owned();
                        let key_id = get_attribute(&e, b"keyid").map(|s: Cow<str>| s.into_owned());
                        let key_value =
                            get_attribute(&e, b"keyvalue").map(|s: Cow<str>| s.into_owned());

                        store.consumers.push(Consumer {
                            id,
                            key_id,
                            key_value,
                        });

                        let end_tag = e.name().as_ref().to_vec();
                        parser.read_to_end(&end_tag)?;
                    }
                    b"resourcedatagroup" => {
                        let key_uuid_str = get_attribute(&e, b"keyuuid")
                            .ok_or(Lib3mfError::Validation("Missing keyuuid".to_string()))?;
                        let key_uuid = uuid::Uuid::parse_str(&key_uuid_str).map_err(|_| {
                            Lib3mfError::Validation(format!("Invalid keyuuid: {}", key_uuid_str))
                        })?;
                        let group = parse_resource_data_group(parser, key_uuid)?;
                        store.resource_data_groups.push(group);
                    }
                    _ => {
                        let end_tag = e.name().as_ref().to_vec();
                        parser.read_to_end(&end_tag)?;
                    }
                }
            }
            Event::End(e) if e.local_name().as_ref() == b"keystore" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in keystore".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(store)
}

fn parse_resource_data_group<R: BufRead>(
    parser: &mut XmlParser<R>,
    key_uuid: uuid::Uuid,
) -> Result<ResourceDataGroup> {
    let mut group = ResourceDataGroup {
        key_uuid,
        access_rights: Vec::new(),
    };

    loop {
        match parser.read_next_event()? {
            Event::Start(e) => {
                match e.local_name().as_ref() {
                    b"accessright" => {
                        let consumer_id = get_attribute(&e, b"consumerid")
                            .ok_or(Lib3mfError::Validation("Missing consumerid".to_string()))?
                            .into_owned();
                        // Parse children (wrappedkey)
                        let (wrapped_key, algorithm) = parse_access_right_content(parser)?;

                        group.access_rights.push(AccessRight {
                            consumer_id,
                            algorithm,
                            wrapped_key,
                        });
                    }
                    _ => {
                        let end_tag = e.name().as_ref().to_vec();
                        parser.read_to_end(&end_tag)?;
                    }
                }
            }
            Event::End(e) if e.local_name().as_ref() == b"resourcedatagroup" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in resourcedatagroup".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(group)
}

fn parse_access_right_content<R: BufRead>(parser: &mut XmlParser<R>) -> Result<(Vec<u8>, String)> {
    let mut wrapped_key = Vec::new();
    let mut algorithm = "RSA-OAEP".to_string(); // Default or extracted

    loop {
        match parser.read_next_event()? {
            Event::Start(e) => {
                match e.local_name().as_ref() {
                    b"wrappedkey" => {
                        // Attribute "algorithm" might be here?
                        if let Some(alg) = get_attribute(&e, b"encryptionalgorithm") {
                            algorithm = alg.into_owned();
                        }

                        let text = parser.read_text_content()?;
                        // Decode base64 to actual encrypted key bytes
                        wrapped_key = BASE64_STANDARD.decode(text.as_bytes()).map_err(|e| {
                            Lib3mfError::Validation(format!("Invalid base64 wrapped key: {}", e))
                        })?;
                    }
                    _ => {
                        let end_tag = e.name().as_ref().to_vec();
                        parser.read_to_end(&end_tag)?;
                    }
                }
            }
            Event::End(e) if e.local_name().as_ref() == b"accessright" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in accessright".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok((wrapped_key, algorithm))
}
