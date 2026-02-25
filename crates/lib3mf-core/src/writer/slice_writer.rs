use crate::error::{Lib3mfError, Result};
use crate::model::{Polygon, Segment, Slice, SliceRef, SliceStack, Vertex2D};
use crate::writer::xml_writer::XmlWriter;
use std::io::Write;

/// Controls how slice data is written to the 3MF archive.
///
/// - `PreserveOriginal`: Writes inline slices as `<slice>` elements and slice
///   references as `<sliceref>` elements, preserving the original structure
///   as parsed. This is the only mode fully implemented in Phase 13.
/// - `Inline`: Would convert all external slicerefs to inline slices.
///   Not yet implemented.
/// - `External`: Would move inline slices to external files with slicerefs.
///   Not yet implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SliceMode {
    #[default]
    PreserveOriginal,
    Inline,
    External,
}

/// Options controlling slice extension serialization behavior.
pub struct SliceWriteOptions {
    /// How slice data is laid out in the archive.
    pub slice_mode: SliceMode,
    /// When true, empty slice stacks cause an error instead of a warning.
    pub strict: bool,
}

impl Default for SliceWriteOptions {
    fn default() -> Self {
        Self {
            slice_mode: SliceMode::PreserveOriginal,
            strict: false,
        }
    }
}

/// Writes a `<slicestack>` element with its child slices and slicerefs.
///
/// Element names are unqualified (no namespace prefix) because the slice
/// namespace URI is declared on the root model element as `xmlns:s`.
///
/// # Degenerate handling
///
/// - Empty slice stacks (zero slices AND zero refs): emits a warning and writes
///   an empty `<slicestack>` element. If `opts.strict` is true, returns an error.
/// - Orphan vertices (vertices present but no polygons in a slice): skips the
///   entire slice with a warning.
/// - Degenerate polygons (zero segments): emits a warning but writes faithfully.
pub fn write_slice_stack<W: Write>(
    writer: &mut XmlWriter<W>,
    stack: &SliceStack,
    opts: &SliceWriteOptions,
) -> Result<()> {
    // Check slice mode first
    match opts.slice_mode {
        SliceMode::PreserveOriginal => {}
        SliceMode::Inline => {
            return Err(Lib3mfError::Validation(
                "SliceMode::Inline is not yet implemented".to_string(),
            ));
        }
        SliceMode::External => {
            return Err(Lib3mfError::Validation(
                "SliceMode::External is not yet implemented".to_string(),
            ));
        }
    }

    // Warn on empty slice stacks
    if stack.slices.is_empty() && stack.refs.is_empty() {
        eprintln!(
            "Warning: slice stack id={} has no slices and no slicerefs",
            stack.id.0
        );
        if opts.strict {
            return Err(Lib3mfError::Validation(format!(
                "Empty slice stack id={}",
                stack.id.0
            )));
        }
    }

    writer
        .start_element("slicestack")
        .attr("id", &stack.id.0.to_string())
        .attr("zbottom", &stack.z_bottom.to_string())
        .write_start()?;

    // Write inline slices first
    for slice in &stack.slices {
        write_slice(writer, slice)?;
    }

    // Write slicerefs after inline slices
    for sref in &stack.refs {
        write_sliceref(writer, sref)?;
    }

    writer.end_element("slicestack")?;
    Ok(())
}

/// Writes a single `<slice>` element with its vertices, polygons, and segments.
///
/// Skips the entire slice (with a warning) if it has vertices but no polygons
/// (orphan vertices).
fn write_slice<W: Write>(writer: &mut XmlWriter<W>, slice: &Slice) -> Result<()> {
    // Orphan vertices: skip the slice entirely
    if !slice.vertices.is_empty() && slice.polygons.is_empty() {
        eprintln!(
            "Warning: slice ztop={} has {} vertices but no polygons, skipping",
            slice.z_top,
            slice.vertices.len()
        );
        return Ok(());
    }

    writer
        .start_element("slice")
        .attr("ztop", &slice.z_top.to_string())
        .write_start()?;

    // Write vertices section (only if there are vertices)
    if !slice.vertices.is_empty() {
        writer.start_element("vertices").write_start()?;
        for v in &slice.vertices {
            write_vertex(writer, v)?;
        }
        writer.end_element("vertices")?;
    }

    // Write polygons
    for polygon in &slice.polygons {
        write_polygon(writer, polygon)?;
    }

    writer.end_element("slice")?;
    Ok(())
}

/// Writes a `<vertex>` element.
fn write_vertex<W: Write>(writer: &mut XmlWriter<W>, v: &Vertex2D) -> Result<()> {
    writer
        .start_element("vertex")
        .attr("x", &v.x.to_string())
        .attr("y", &v.y.to_string())
        .write_empty()?;
    Ok(())
}

/// Writes a `<polygon>` element with its child segments.
///
/// Emits a warning for degenerate polygons (zero segments) but writes them
/// faithfully rather than skipping.
fn write_polygon<W: Write>(writer: &mut XmlWriter<W>, polygon: &Polygon) -> Result<()> {
    if polygon.segments.is_empty() {
        eprintln!(
            "Warning: polygon start={} has zero segments (degenerate)",
            polygon.start_segment
        );
    }

    writer
        .start_element("polygon")
        .attr("start", &polygon.start_segment.to_string())
        .write_start()?;

    for segment in &polygon.segments {
        write_segment(writer, segment)?;
    }

    writer.end_element("polygon")?;
    Ok(())
}

/// Writes a `<segment>` element with required v2 and optional p1/p2/pid attributes.
fn write_segment<W: Write>(writer: &mut XmlWriter<W>, segment: &Segment) -> Result<()> {
    let mut elem = writer
        .start_element("segment")
        .attr("v2", &segment.v2.to_string());

    if let Some(p1) = segment.p1 {
        elem = elem.attr("p1", &p1.to_string());
    }
    if let Some(p2) = segment.p2 {
        elem = elem.attr("p2", &p2.to_string());
    }
    if let Some(pid) = segment.pid {
        elem = elem.attr("pid", &pid.0.to_string());
    }

    elem.write_empty()?;
    Ok(())
}

/// Writes a `<sliceref>` element referencing an external slice stack.
fn write_sliceref<W: Write>(writer: &mut XmlWriter<W>, sref: &SliceRef) -> Result<()> {
    writer
        .start_element("sliceref")
        .attr("slicestackid", &sref.slice_stack_id.0.to_string())
        .attr("slicepath", &sref.slice_path)
        .write_empty()?;
    Ok(())
}
