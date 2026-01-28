use crate::error::{Lib3mfError, Result};
use crate::model::crypto::*;
use crate::parser::xml_parser::{get_attribute, XmlParser};
use quick_xml::events::Event;
use std::io::BufRead;

pub fn parse_signature<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Signature> {
    let mut signature = Signature::default();
    
    loop {
        // Read event separately to limit scope of borrow
        let evt_info = match parser.read_next_event()? {
            Event::Start(e) => Some((e.local_name().as_ref().to_vec(), e.name().as_ref().to_vec())),
            Event::End(e) if e.local_name().as_ref() == b"Signature" => return Ok(signature),
            Event::Eof => return Err(Lib3mfError::Validation("Unexpected EOF in Signature".to_string())),
            _ => None,
        };

        if let Some((local_name, raw_name)) = evt_info {
            match local_name.as_slice() {
                b"SignedInfo" => {
                    signature.signed_info = parse_signed_info(parser)?;
                }
                b"SignatureValue" => {
                    let val = parser.read_text_content()?;
                    signature.signature_value = SignatureValue { value: val };
                }
                b"KeyInfo" => {
                    signature.key_info = Some(parse_key_info(parser)?);
                }
                _ => {
                    parser.read_to_end(&raw_name)?;
                }
            }
        }
    }
}

fn parse_signed_info<R: BufRead>(parser: &mut XmlParser<R>) -> Result<SignedInfo> {
    let mut info = SignedInfo::default();
    
    loop {
        let evt_info = match parser.read_next_event()? {
            Event::Start(e) => {
                let alg = get_attribute(&e, b"Algorithm"); // Capture attribute if present
                let uri = get_attribute(&e, b"URI"); // For Reference
                Some((e.local_name().as_ref().to_vec(), e.name().as_ref().to_vec(), alg, uri, false))
            },
            Event::Empty(e) => {
                 let alg = get_attribute(&e, b"Algorithm");
                 let uri = get_attribute(&e, b"URI");
                 Some((e.local_name().as_ref().to_vec(), e.name().as_ref().to_vec(), alg, uri, true))
            },
            Event::End(e) if e.local_name().as_ref() == b"SignedInfo" => return Ok(info),
            Event::Eof => return Err(Lib3mfError::Validation("Unexpected EOF in SignedInfo".to_string())),
            _ => None,
        };

        if let Some((local_name, raw_name, alg, uri, is_empty)) = evt_info {
            match local_name.as_slice() {
                b"CanonicalizationMethod" => {
                    info.canonicalization_method = CanonicalizationMethod { algorithm: alg.unwrap_or_default() };
                    if !is_empty { parser.read_to_end(&raw_name)?; }
                }
                b"SignatureMethod" => {
                    info.signature_method = SignatureMethod { algorithm: alg.unwrap_or_default() };
                    if !is_empty { parser.read_to_end(&raw_name)?; }
                }
                b"Reference" => {
                    // Logic for Reference: if empty, it's invalid usually, but we handle it.
                    // If Start, we need to recurse.
                    if !is_empty {
                         // We need to pass the URI we extracted.
                         // parse_reference logic assumes it gets parser.
                         // But parse_reference usually expects to handle Start event attributes...
                         // Since we consumed the Start event, we must handle attributes here or pass them.
                         // I will modify parse_reference to take URI and assume start event consumed.
                         let r = parse_reference_content(parser, uri.unwrap_or_default())?;
                         info.references.push(r);
                    } else {
                        // Empty Reference? Not really valid but...
                         let mut r = Reference::default();
                         r.uri = uri.unwrap_or_default();
                         info.references.push(r);
                    }
                }
                _ => {
                    if !is_empty { parser.read_to_end(&raw_name)?; }
                }
            }
        }
    }
}

// Renamed from parse_reference to parse_reference_content as it assumes Start event consumed
fn parse_reference_content<R: BufRead>(parser: &mut XmlParser<R>, uri: String) -> Result<Reference> {
    let mut r = Reference::default();
    r.uri = uri;
    
    loop {
         let evt_info = match parser.read_next_event()? {
            Event::Start(e) => {
                 let alg = get_attribute(&e, b"Algorithm");
                 Some((e.local_name().as_ref().to_vec(), e.name().as_ref().to_vec(), alg, false))
            },
            Event::Empty(e) => {
                 let alg = get_attribute(&e, b"Algorithm");
                 Some((e.local_name().as_ref().to_vec(), e.name().as_ref().to_vec(), alg, true))
            },
            Event::End(e) if e.local_name().as_ref() == b"Reference" => return Ok(r),
             Event::Eof => return Err(Lib3mfError::Validation("Unexpected EOF in Reference".to_string())),
            _ => None,
        };
        
        if let Some((local_name, raw_name, alg, is_empty)) = evt_info {
            match local_name.as_slice() {
                b"DigestMethod" => {
                      r.digest_method = DigestMethod { algorithm: alg.unwrap_or_default() };
                      if !is_empty { parser.read_to_end(&raw_name)?; }
                }
                b"DigestValue" => {
                    // Start element, read text content
                    if !is_empty {
                         let val = parser.read_text_content()?;
                         r.digest_value = DigestValue { value: val };
                    }
                }
                b"Transforms" => {
                     if !is_empty {
                          let transforms = parse_transforms_content(parser)?;
                          r.transforms = Some(transforms);
                     }
                }
                _ => {
                     if !is_empty { parser.read_to_end(&raw_name)?; }
                }
            }
        }
    }
}

fn parse_transforms_content<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Vec<Transform>> {
     let mut transforms = Vec::new();
     loop {
          let evt_info = match parser.read_next_event()? {
             Event::Start(e) => {
                 let alg = get_attribute(&e, b"Algorithm");
                 Some((e.local_name().as_ref().to_vec(), e.name().as_ref().to_vec(), alg, false))
             },
             Event::Empty(e) => {
                 let alg = get_attribute(&e, b"Algorithm");
                 Some((e.local_name().as_ref().to_vec(), e.name().as_ref().to_vec(), alg, true))
             },
             Event::End(e) if e.local_name().as_ref() == b"Transforms" => return Ok(transforms),
              Event::Eof => return Err(Lib3mfError::Validation("Unexpected EOF in Transforms".to_string())),
             _ => None,
         };
         
         if let Some((local_name, raw_name, alg, is_empty)) = evt_info {
             match local_name.as_slice() {
                 b"Transform" => {
                      transforms.push(Transform { algorithm: alg.unwrap_or_default() });
                      if !is_empty { parser.read_to_end(&raw_name)?; }
                 }
                 _ => {
                      if !is_empty { parser.read_to_end(&raw_name)?; }
                 }
             }
         }
     }
}

fn parse_key_info<R: BufRead>(parser: &mut XmlParser<R>) -> Result<KeyInfo> {
    let mut info = KeyInfo::default();
    
    loop {
         let evt_info = match parser.read_next_event()? {
             Event::Start(e) => Some((e.local_name().as_ref().to_vec(), e.name().as_ref().to_vec())),
             Event::End(e) if e.local_name().as_ref() == b"KeyInfo" => return Ok(info),
              Event::Eof => return Err(Lib3mfError::Validation("Unexpected EOF in KeyInfo".to_string())),
             _ => None,
         };
         
         if let Some((local_name, raw_name)) = evt_info {
             match local_name.as_slice() {
                 b"KeyName" => {
                     let val = parser.read_text_content()?;
                     info.key_name = Some(val);
                 }
                 b"KeyValue" => {
                     info.key_value = Some(parse_key_value(parser)?);
                 }
                 _ => { parser.read_to_end(&raw_name)?; }
             }
         }
    }
}

fn parse_key_value<R: BufRead>(parser: &mut XmlParser<R>) -> Result<KeyValue> {
    let mut val = KeyValue::default();
    loop {
         let evt_info = match parser.read_next_event()? {
             Event::Start(e) => Some((e.local_name().as_ref().to_vec(), e.name().as_ref().to_vec())),
             Event::End(e) if e.local_name().as_ref() == b"KeyValue" => return Ok(val),
              Event::Eof => return Err(Lib3mfError::Validation("Unexpected EOF in KeyValue".to_string())),
             _ => None,
         };
         
         if let Some((local_name, raw_name)) = evt_info {
             match local_name.as_slice() {
                 b"RSAKeyValue" => {
                      val.rsa_key_value = Some(parse_rsa_key_value(parser)?);
                 }
                  _ => { parser.read_to_end(&raw_name)?; }
             }
         }
    }
}

fn parse_rsa_key_value<R: BufRead>(parser: &mut XmlParser<R>) -> Result<RSAKeyValue> {
    let mut modulus = String::new();
    let mut exponent = String::new();
    
    loop {
        let evt_info = match parser.read_next_event()? {
            Event::Start(e) => Some((e.local_name().as_ref().to_vec(), e.name().as_ref().to_vec())),
            Event::End(e) if e.local_name().as_ref() == b"RSAKeyValue" => return Ok(RSAKeyValue { modulus, exponent }),
             Event::Eof => return Err(Lib3mfError::Validation("Unexpected EOF in RSAKeyValue".to_string())),
            _ => None,
        };
        
        if let Some((local_name, raw_name)) = evt_info {
            match local_name.as_slice() {
                b"Modulus" => modulus = parser.read_text_content()?,
                b"Exponent" => exponent = parser.read_text_content()?,
                 _ => { parser.read_to_end(&raw_name)?; }
            }
        }
    }
}
