use crate::error::{Lib3mfError, Result};
use crate::model::{Beam, BeamLattice, BeamSet, CapMode, ClippingMode};
use crate::parser::xml_parser::{XmlParser, get_attribute, get_attribute_f32, get_attribute_u32};
use quick_xml::events::Event;
use std::io::BufRead;
// parse_beam_lattice removed

// Rewriting above thinking:
// I will define `pub fn parse_beam_lattice_content<R>(parser, min_length, precision, clipping) -> Result<BeamLattice>`
// And the caller will handle the attributes.

pub fn parse_beam_lattice_content<R: BufRead>(
    parser: &mut XmlParser<R>,
    min_length: f32,
    precision: f32,
    clipping_mode: ClippingMode,
) -> Result<BeamLattice> {
    let mut beams = Vec::new();
    let mut beam_sets = Vec::new();

    loop {
        match parser.read_next_event()? {
            Event::Start(e) => match e.name().as_ref() {
                b"beams" => {
                    beams = parse_beams(parser)?;
                }
                b"beamsets" => {
                    beam_sets = parse_beam_sets(parser)?;
                }
                _ => {}
            },
            Event::End(e) if e.name().as_ref() == b"beamlattice" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in beamlattice".to_string(),
                ));
            }
            _ => {}
        }
    }

    Ok(BeamLattice {
        min_length,
        precision,
        clipping_mode,
        beams,
        beam_sets,
    })
}

fn parse_beams<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Vec<Beam>> {
    let mut beams = Vec::new();
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"beam" => {
                let v1 = get_attribute_u32(&e, b"v1")?;
                let v2 = get_attribute_u32(&e, b"v2")?;
                let r1 = get_attribute_f32(&e, b"r1")?;
                let r2 = get_attribute_f32(&e, b"r2").unwrap_or(r1);
                let p1 = get_attribute_u32(&e, b"p1").ok();
                let p2 = get_attribute_u32(&e, b"p2").ok();

                let cap_mode = if let Some(s) = get_attribute(&e, b"cap") {
                    match s.as_str() {
                        "sphere" => CapMode::Sphere,
                        "hemisphere" => CapMode::Hemisphere,
                        "butt" => CapMode::Butt,
                        _ => CapMode::Sphere,
                    }
                } else {
                    CapMode::Sphere
                };

                beams.push(Beam {
                    v1,
                    v2,
                    r1,
                    r2,
                    p1,
                    p2,
                    cap_mode,
                });
            }
            Event::End(e) if e.name().as_ref() == b"beams" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in beams".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(beams)
}

fn parse_beam_sets<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Vec<BeamSet>> {
    let mut sets = Vec::new();
    loop {
        let event = parser.read_next_event()?;
        match event {
            Event::Start(e) if e.name().as_ref() == b"beamset" => {
                let name = get_attribute(&e, b"name");
                let identifier = get_attribute(&e, b"identifier");
                let refs = parse_refs(parser)?;
                sets.push(BeamSet {
                    name,
                    identifier,
                    refs,
                });
            }
            Event::Empty(e) if e.name().as_ref() == b"beamset" => {
                let name = get_attribute(&e, b"name");
                let identifier = get_attribute(&e, b"identifier");
                sets.push(BeamSet {
                    name,
                    identifier,
                    refs: Vec::new(),
                });
            }
            Event::End(e) if e.name().as_ref() == b"beamsets" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in beamsets".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(sets)
}

fn parse_refs<R: BufRead>(parser: &mut XmlParser<R>) -> Result<Vec<u32>> {
    let mut refs = Vec::new();
    loop {
        match parser.read_next_event()? {
            Event::Start(e) | Event::Empty(e) if e.name().as_ref() == b"ref" => {
                let idx = get_attribute_u32(&e, b"index")?;
                refs.push(idx);
            }
            Event::End(e) if e.name().as_ref() == b"beamset" => break,
            Event::Eof => {
                return Err(Lib3mfError::Validation(
                    "Unexpected EOF in beamset".to_string(),
                ));
            }
            _ => {}
        }
    }
    Ok(refs)
}
