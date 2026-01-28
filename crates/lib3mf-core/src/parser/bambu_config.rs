use crate::error::Result;
use crate::model::stats::PlateInfo;
use crate::parser::xml_parser::{get_attribute, XmlParser};
use quick_xml::events::Event;
use std::io::Cursor;

pub fn parse_model_settings(content: &[u8]) -> Result<Vec<PlateInfo>> {
    let mut parser = XmlParser::new(Cursor::new(content));
    let mut plates = Vec::new();

    loop {
        match parser.next()? {
            Event::Start(e) => {
                if e.name().as_ref() == b"plate" {
                    let mut id = 0;
                    let mut name = None;
                    
                    // The Bambu model_settings.config has <plate> as a container.
                    // The ID and Name are inside <metadata> tags WITHIN the plate tag.
                    // Example:
                    // <plate>
                    //   <metadata key="plater_id" value="1"/>
                    //   <metadata key="plater_name" value="Main"/>
                    // </plate>
                    
                    // We need to parse children of <plate>
                    loop {
                         match parser.next()? {
                             Event::Empty(child) | Event::Start(child) => {
                                 if child.name().as_ref() == b"metadata" {
                                     let key = get_attribute(&child, b"key");
                                     let value = get_attribute(&child, b"value");
                                     
                                     if let (Some(k), Some(v)) = (key, value) {
                                         match k.as_str() {
                                             "plater_id" => {
                                                 if let Ok(pid) = v.parse::<u32>() {
                                                     id = pid;
                                                 }
                                             }
                                             "plater_name" => name = Some(v),
                                             _ => {}
                                         }
                                     }
                                 }
                             }
                             Event::End(end) if end.name().as_ref() == b"plate" => break,
                             Event::Eof => break,
                             _ => {}
                         }
                    }
                    
                    if id != 0 {
                        plates.push(PlateInfo { id, name });
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(plates)
}
