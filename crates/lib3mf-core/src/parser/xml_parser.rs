use crate::error::{Lib3mfError, Result};
use lexical_core;
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::borrow::Cow;
use std::io::BufRead;

pub struct XmlParser<R: BufRead> {
    pub reader: Reader<R>,
    pub buf: Vec<u8>,
}

impl<R: BufRead> XmlParser<R> {
    pub fn new(reader: R) -> Self {
        let mut reader = Reader::from_reader(reader);
        reader.config_mut().trim_text(true);
        reader.config_mut().expand_empty_elements = true;
        Self {
            reader,
            buf: Vec::new(),
        }
    }

    pub fn read_next_event(&mut self) -> Result<Event<'_>> {
        self.buf.clear();
        self.reader
            .read_event_into(&mut self.buf)
            .map_err(|e| Lib3mfError::Validation(e.to_string()))
    }

    pub fn read_text_content(&mut self) -> Result<String> {
        let mut text = String::new();
        let mut depth = 0;

        loop {
            match self.read_next_event()? {
                Event::Text(e) => text.push_str(&String::from_utf8_lossy(e.as_ref())),
                Event::CData(e) => text.push_str(&String::from_utf8_lossy(e.into_inner().as_ref())),
                Event::Start(_) => depth += 1,
                Event::End(_) => {
                    if depth > 0 {
                        depth -= 1;
                    } else {
                        return Ok(text);
                    }
                }
                Event::Eof => {
                    return Err(Lib3mfError::Validation(
                        "Unexpected EOF in text content".to_string(),
                    ));
                }
                _ => {}
            }
        }
    }

    pub fn read_to_end(&mut self, end: &[u8]) -> Result<()> {
        // read_to_end_into expects QName
        self.reader
            .read_to_end_into(quick_xml::name::QName(end), &mut self.buf)
            .map_err(|e| Lib3mfError::Validation(e.to_string()))?;
        Ok(())
    }
}

// Helper functions for attribute parsing
pub fn get_attribute<'a>(e: &'a BytesStart, name: &[u8]) -> Option<Cow<'a, str>> {
    e.try_get_attribute(name).ok().flatten().map(|a| {
        a.unescape_value()
            .unwrap_or_else(|_| String::from_utf8_lossy(&a.value).into_owned().into())
    })
}

pub fn get_attribute_f32(e: &BytesStart, name: &[u8]) -> Result<f32> {
    let attr = e.try_get_attribute(name).ok().flatten().ok_or_else(|| {
        Lib3mfError::Validation(format!(
            "Missing attribute: {}",
            String::from_utf8_lossy(name)
        ))
    })?;
    lexical_core::parse::<f32>(attr.value.as_ref()).map_err(|_| {
        Lib3mfError::Validation(format!(
            "Invalid float for attribute {}: {}",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(&attr.value)
        ))
    })
}

pub fn get_attribute_u32(e: &BytesStart, name: &[u8]) -> Result<u32> {
    let attr = e.try_get_attribute(name).ok().flatten().ok_or_else(|| {
        Lib3mfError::Validation(format!(
            "Missing attribute: {}",
            String::from_utf8_lossy(name)
        ))
    })?;
    lexical_core::parse::<u32>(attr.value.as_ref()).map_err(|_| {
        Lib3mfError::Validation(format!(
            "Invalid integer for attribute {}: {}",
            String::from_utf8_lossy(name),
            String::from_utf8_lossy(&attr.value)
        ))
    })
}

pub fn get_attribute_u32_opt(e: &BytesStart, name: &[u8]) -> Result<Option<u32>> {
    match e.try_get_attribute(name).ok().flatten() {
        Some(attr) => lexical_core::parse::<u32>(attr.value.as_ref())
            .map(Some)
            .map_err(|_| {
                Lib3mfError::Validation(format!(
                    "Invalid integer: {}",
                    String::from_utf8_lossy(&attr.value)
                ))
            }),
        None => Ok(None),
    }
}

pub fn get_attribute_uuid(e: &BytesStart) -> Result<Option<uuid::Uuid>> {
    // Try "uuid" then "p:uuid"
    let val = get_attribute(e, b"uuid").or_else(|| get_attribute(e, b"p:uuid"));

    match val {
        Some(s) => uuid::Uuid::parse_str(&s)
            .map(Some)
            .map_err(|_| Lib3mfError::Validation(format!("Invalid UUID: {}", s))),
        None => Ok(None),
    }
}
