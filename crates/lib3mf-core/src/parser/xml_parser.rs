use crate::error::{Lib3mfError, Result};
use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;
use std::io::BufRead;
use std::str::FromStr;

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

    pub fn next(&mut self) -> Result<Event<'_>> {
        self.buf.clear();
        self.reader
            .read_event_into(&mut self.buf)
            .map_err(|e| Lib3mfError::Validation(e.to_string()))
    }

    pub fn read_text_content(&mut self) -> Result<String> {
        let mut text = String::new();
        let mut depth = 0;
        
        loop {
            match self.next()? {
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
                Event::Eof => return Err(Lib3mfError::Validation("Unexpected EOF in text content".to_string())),
                _ => {}
            }
        }
    }

    pub fn read_to_end(&mut self, end: &[u8]) -> Result<()> {
         // read_to_end_into expects QName
         self.reader.read_to_end_into(quick_xml::name::QName(end), &mut self.buf)
            .map_err(|e| Lib3mfError::Validation(e.to_string()))?;
         Ok(())
    }
}

// Helper functions for attribute parsing
pub fn get_attribute(e: &BytesStart, name: &[u8]) -> Option<String> {
    e.attributes()
        .find(|a| a.as_ref().map(|a| a.key.as_ref() == name).unwrap_or(false))
        .map(|a| {
            let a = a.unwrap();
            String::from_utf8_lossy(&a.value).to_string()
        })
}

pub fn get_attribute_f32(e: &BytesStart, name: &[u8]) -> Result<f32> {
    let val = get_attribute(e, name).ok_or_else(|| {
        Lib3mfError::Validation(format!("Missing attribute: {}", String::from_utf8_lossy(name)))
    })?;
    f32::from_str(&val).map_err(|_| {
        Lib3mfError::Validation(format!(
            "Invalid float for attribute {}: {}",
            String::from_utf8_lossy(name),
            val
        ))
    })
}

pub fn get_attribute_u32(e: &BytesStart, name: &[u8]) -> Result<u32> {
    let val = get_attribute(e, name).ok_or_else(|| {
        Lib3mfError::Validation(format!("Missing attribute: {}", String::from_utf8_lossy(name)))
    })?;
    u32::from_str(&val).map_err(|_| {
        Lib3mfError::Validation(format!(
            "Invalid integer for attribute {}: {}",
            String::from_utf8_lossy(name),
            val
        ))
    })
}

pub fn get_attribute_u32_opt(e: &BytesStart, name: &[u8]) -> Result<Option<u32>> {
    match get_attribute(e, name) {
        Some(val) => u32::from_str(&val)
            .map(Some)
            .map_err(|_| Lib3mfError::Validation(format!("Invalid integer: {}", val))),
        None => Ok(None),
    }
}

pub fn get_attribute_uuid(e: &BytesStart) -> Result<Option<uuid::Uuid>> {
    // Try "uuid" then "p:uuid"
    let val = get_attribute(e, b"uuid")
        .or_else(|| get_attribute(e, b"p:uuid"));
        
    match val {
        Some(s) => uuid::Uuid::parse_str(&s)
            .map(Some)
            .map_err(|_| Lib3mfError::Validation(format!("Invalid UUID: {}", s))),
        None => Ok(None),
    }
}
