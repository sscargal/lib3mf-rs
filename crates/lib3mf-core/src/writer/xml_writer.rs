use crate::error::{Lib3mfError, Result};
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::io::Write;

/// A low-level XML writer providing indented output for 3MF XML serialization.
pub struct XmlWriter<W: Write> {
    writer: Writer<W>,
}

impl<W: Write> XmlWriter<W> {
    /// Creates a new `XmlWriter` wrapping the given writer with 2-space indentation.
    pub fn new(inner: W) -> Self {
        Self {
            writer: Writer::new_with_indent(inner, b' ', 2),
        }
    }

    /// Writes the XML declaration (`<?xml version="1.0" encoding="UTF-8"?>`).
    pub fn write_declaration(&mut self) -> Result<()> {
        let decl = BytesDecl::new("1.0", Some("UTF-8"), None);
        self.writer
            .write_event(Event::Decl(decl))
            .map_err(|e| Lib3mfError::Validation(e.to_string()))
    }

    /// Returns an `ElementBuilder` for constructing and writing a start or empty element.
    pub fn start_element(&mut self, name: &str) -> ElementBuilder<'_, W> {
        ElementBuilder {
            writer: self,
            name: name.to_string(),
            attributes: Vec::new(),
        }
    }

    /// Writes a closing tag for the given element name.
    pub fn end_element(&mut self, name: &str) -> Result<()> {
        self.writer
            .write_event(Event::End(BytesEnd::new(name)))
            .map_err(|e| Lib3mfError::Validation(e.to_string()))
    }

    /// Writes a text node with the given content.
    pub fn write_text(&mut self, text: &str) -> Result<()> {
        self.writer
            .write_event(Event::Text(BytesText::new(text)))
            .map_err(|e| Lib3mfError::Validation(e.to_string()))
    }
}

/// A builder for constructing XML elements with attributes before writing.
pub struct ElementBuilder<'a, W: Write> {
    writer: &'a mut XmlWriter<W>,
    name: String,
    attributes: Vec<(String, String)>,
}

impl<'a, W: Write> ElementBuilder<'a, W> {
    /// Adds a required attribute to the element.
    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.attributes.push((key.to_string(), value.to_string()));
        self
    }

    /// Adds an optional attribute to the element, only if `value` is `Some`.
    pub fn optional_attr(mut self, key: &str, value: Option<&str>) -> Self {
        if let Some(v) = value {
            self.attributes.push((key.to_string(), v.to_string()));
        }
        self
    }

    /// Writes the element as a self-closing empty element (e.g., `<vertex x="1" />`).
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

    /// Writes the element as an opening tag (e.g., `<model xmlns="...">`) with child content to follow.
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
