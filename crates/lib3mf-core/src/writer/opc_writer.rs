use crate::error::Result;
use crate::writer::xml_writer::XmlWriter;
use std::io::Write;

pub fn write_content_types<W: Write>(writer: W) -> Result<()> {
    let mut xml = XmlWriter::new(writer);
    xml.write_declaration()?;

    xml.start_element("Types")
        .attr(
            "xmlns",
            "http://schemas.openxmlformats.org/package/2006/content-types",
        )
        .write_start()?;

    // Defaults
    xml.start_element("Default")
        .attr("Extension", "rels")
        .attr(
            "ContentType",
            "application/vnd.openxmlformats-package.relationships+xml",
        )
        .write_empty()?;
    xml.start_element("Default")
        .attr("Extension", "model")
        .attr(
            "ContentType",
            "application/vnd.ms-package.3dmanufacturing-3dmodel+xml",
        )
        .write_empty()?;
    xml.start_element("Default")
        .attr("Extension", "png")
        .attr("ContentType", "image/png")
        .write_empty()?;

    // Don't enforce Override for 3D/3dmodel.model if extension match works,
    // but spec usually recommends explicit override for parts.
    // For now, minimal valid set.

    xml.end_element("Types")?;
    Ok(())
}

pub fn write_relationships<W: Write>(
    writer: W,
    model_part: &str,
    thumbnail_part: Option<&str>,
) -> Result<()> {
    let mut xml = XmlWriter::new(writer);
    xml.write_declaration()?;

    xml.start_element("Relationships")
        .attr(
            "xmlns",
            "http://schemas.openxmlformats.org/package/2006/relationships",
        )
        .write_start()?;

    xml.start_element("Relationship")
        .attr("Target", model_part)
        .attr("Id", "rel0")
        .attr(
            "Type",
            "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel",
        )
        .write_empty()?;

    if let Some(thumb) = thumbnail_part {
        xml.start_element("Relationship")
            .attr("Target", thumb)
            .attr("Id", "rel1")
            .attr(
                "Type",
                "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail",
            )
            .write_empty()?;
    }

    xml.end_element("Relationships")?;
    Ok(())
}
