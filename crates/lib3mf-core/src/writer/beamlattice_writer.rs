use crate::error::Result;
use crate::model::{BeamLattice, CapMode, ClippingMode};
use crate::writer::xml_writer::XmlWriter;
use std::io::Write;

/// Writes a `<beamlattice>` element inside a `<mesh>` element.
///
/// The beam lattice is written as a child element of `<mesh>`, after the
/// `<triangles>` section, per the Beam Lattice Extension v1.2.0 specification.
///
/// Element names are unqualified (no namespace prefix) because the parser
/// matches them by local name. The namespace URI is declared on the root
/// model element as `xmlns:bl`.
pub fn write_beam_lattice<W: Write>(
    writer: &mut XmlWriter<W>,
    lattice: &BeamLattice,
) -> Result<()> {
    let mut bl_elem = writer
        .start_element("beamlattice")
        .attr("minlength", &lattice.min_length.to_string())
        .attr("precision", &lattice.precision.to_string());

    // clippingmode: omit when None (default) to match spec conventions
    if lattice.clipping_mode != ClippingMode::None {
        bl_elem = bl_elem.attr("clippingmode", clipping_mode_to_str(lattice.clipping_mode));
    }

    bl_elem.write_start()?;

    // Write beams section
    writer.start_element("beams").write_start()?;
    for beam in &lattice.beams {
        let mut b = writer
            .start_element("beam")
            .attr("v1", &beam.v1.to_string())
            .attr("v2", &beam.v2.to_string())
            .attr("r1", &beam.r1.to_string())
            .attr("r2", &beam.r2.to_string());

        // cap: omit for default (Sphere) to match spec conventions
        if beam.cap_mode != CapMode::Sphere {
            b = b.attr("cap", cap_mode_to_str(beam.cap_mode));
        }

        if let Some(p1) = beam.p1 {
            b = b.attr("p1", &p1.to_string());
        }
        if let Some(p2) = beam.p2 {
            b = b.attr("p2", &p2.to_string());
        }

        b.write_empty()?;
    }
    writer.end_element("beams")?;

    // Write beam sets section (only when non-empty)
    if !lattice.beam_sets.is_empty() {
        writer.start_element("beamsets").write_start()?;
        for beam_set in &lattice.beam_sets {
            let mut bs = writer.start_element("beamset");
            if let Some(name) = &beam_set.name {
                bs = bs.attr("name", name);
            }
            if let Some(identifier) = &beam_set.identifier {
                bs = bs.attr("identifier", identifier);
            }

            if beam_set.refs.is_empty() {
                bs.write_empty()?;
            } else {
                bs.write_start()?;
                for &ref_index in &beam_set.refs {
                    writer
                        .start_element("ref")
                        .attr("index", &ref_index.to_string())
                        .write_empty()?;
                }
                writer.end_element("beamset")?;
            }
        }
        writer.end_element("beamsets")?;
    }

    writer.end_element("beamlattice")?;
    Ok(())
}

/// Converts a `CapMode` enum to its XML attribute string value.
fn cap_mode_to_str(c: CapMode) -> &'static str {
    match c {
        CapMode::Sphere => "sphere",
        CapMode::Hemisphere => "hemisphere",
        CapMode::Butt => "butt",
    }
}

/// Converts a `ClippingMode` enum to its XML attribute string value.
fn clipping_mode_to_str(cm: ClippingMode) -> &'static str {
    match cm {
        ClippingMode::None => "none",
        ClippingMode::Inside => "inside",
        ClippingMode::Outside => "outside",
    }
}
