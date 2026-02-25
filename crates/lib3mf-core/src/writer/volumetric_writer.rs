use crate::error::Result;
use crate::model::{VolumetricLayer, VolumetricRef, VolumetricStack};
use crate::writer::xml_writer::XmlWriter;
use std::io::Write;

/// Writes a `<volumetricstack>` element with its child layers and volumetricrefs.
///
/// Element names are unqualified (no namespace prefix) because the volumetric
/// namespace URI is declared on the root model element as `xmlns:v`.
///
/// Note: The `version` field of `VolumetricStack` is intentionally NOT emitted.
/// The parser never reads it from XML, so writing it would break roundtrip symmetry.
pub fn write_volumetric_stack<W: Write>(
    writer: &mut XmlWriter<W>,
    stack: &VolumetricStack,
) -> Result<()> {
    writer
        .start_element("volumetricstack")
        .attr("id", &stack.id.0.to_string())
        .write_start()?;

    // Write layers first, then refs (matches parser storage order)
    for layer in &stack.layers {
        write_layer(writer, layer)?;
    }

    for vref in &stack.refs {
        write_volumetricref(writer, vref)?;
    }

    writer.end_element("volumetricstack")?;
    Ok(())
}

/// Writes a `<layer>` element with z and path attributes.
fn write_layer<W: Write>(writer: &mut XmlWriter<W>, layer: &VolumetricLayer) -> Result<()> {
    writer
        .start_element("layer")
        .attr("z", &layer.z_height.to_string())
        .attr("path", &layer.content_path)
        .write_empty()?;
    Ok(())
}

/// Writes a `<volumetricref>` element with volumetricstackid and path attributes.
fn write_volumetricref<W: Write>(writer: &mut XmlWriter<W>, vref: &VolumetricRef) -> Result<()> {
    writer
        .start_element("volumetricref")
        .attr("volumetricstackid", &vref.stack_id.0.to_string())
        .attr("path", &vref.path)
        .write_empty()?;
    Ok(())
}
