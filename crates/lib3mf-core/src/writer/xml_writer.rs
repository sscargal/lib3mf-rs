use crate::error::{Lib3mfError, Result};
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::io::Write;

pub struct XmlWriter<W: Write> {
    writer: Writer<W>,
}

impl<W: Write> XmlWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            writer: Writer::new_with_indent(inner, b' ', 2),
        }
    }

    pub fn write_declaration(&mut self) -> Result<()> {
        let decl = BytesDecl::new("1.0", Some("UTF-8"), None);
        self.writer
            .write_event(Event::Decl(decl))
            .map_err(|e| Lib3mfError::Validation(e.to_string()))
    }

    pub fn start_element(&mut self, name: &str) -> ElementBuilder<'_, W> {
        ElementBuilder {
            writer: self,
            name: name.to_string(),
            attributes: Vec::new(),
        }
    }

    pub fn end_element(&mut self, name: &str) -> Result<()> {
        self.writer
            .write_event(Event::End(BytesEnd::new(name)))
            .map_err(|e| Lib3mfError::Validation(e.to_string()))
    }

    pub fn write_text(&mut self, text: &str) -> Result<()> {
        self.writer
            .write_event(Event::Text(BytesText::new(text)))
            .map_err(|e| Lib3mfError::Validation(e.to_string()))
    }
}

pub struct ElementBuilder<'a, W: Write> {
    writer: &'a mut XmlWriter<W>,
    name: String,
    attributes: Vec<(String, String)>,
}

impl<'a, W: Write> ElementBuilder<'a, W> {
    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.attributes.push((key.to_string(), value.to_string()));
        self
    }

    pub fn optional_attr(mut self, key: &str, value: Option<&str>) -> Self {
        if let Some(v) = value {
            self.attributes.push((key.to_string(), v.to_string()));
        }
        self
    }

    pub fn write_empty(self) -> Result<()> {
        let mut elem = BytesStart::new(&self.name);
        for (k, v) in &self.attributes {
            elem.push_attribute((k.as_str(), v.as_str()));
        }
        self.writer
            .writer
            .write_event(Event::Empty(elem))
            .map_err(|e| Lib3mfError::Validation(e.to_string()))
    }

    pub fn write_start(self) -> Result<()> {
        let mut elem = BytesStart::new(&self.name);
        for (k, v) in &self.attributes {
            elem.push_attribute((k.as_str(), v.as_str()));
        }
        self.writer
            .writer
            .write_event(Event::Start(elem))
            .map_err(|e| Lib3mfError::Validation(e.to_string()))
    }
}
