use crate::error::Result;
use crate::model::{Channel, DisplacementMesh, Displacement2D, FilterMode, TileStyle};
use crate::writer::xml_writer::XmlWriter;
use std::io::Write;

/// Writes a DisplacementMesh element.
pub fn write_displacement_mesh<W: Write>(
    writer: &mut XmlWriter<W>,
    mesh: &DisplacementMesh,
) -> Result<()> {
    writer.start_element("d:displacementmesh").write_start()?;

    // Write vertices section
    writer.start_element("d:vertices").write_start()?;
    for v in &mesh.vertices {
        writer
            .start_element("d:vertex")
            .attr("x", &v.x.to_string())
            .attr("y", &v.y.to_string())
            .attr("z", &v.z.to_string())
            .write_empty()?;
    }
    writer.end_element("d:vertices")?;

    // Write triangles section
    writer.start_element("d:triangles").write_start()?;
    for t in &mesh.triangles {
        let mut builder = writer
            .start_element("d:triangle")
            .attr("v1", &t.v1.to_string())
            .attr("v2", &t.v2.to_string())
            .attr("v3", &t.v3.to_string());

        // Add displacement indices if present
        if let Some(d1) = t.d1 {
            builder = builder.attr("d1", &d1.to_string());
        }
        if let Some(d2) = t.d2 {
            builder = builder.attr("d2", &d2.to_string());
        }
        if let Some(d3) = t.d3 {
            builder = builder.attr("d3", &d3.to_string());
        }

        // Add material properties if present
        if let Some(p1) = t.p1 {
            builder = builder.attr("p1", &p1.to_string());
        }
        if let Some(p2) = t.p2 {
            builder = builder.attr("p2", &p2.to_string());
        }
        if let Some(p3) = t.p3 {
            builder = builder.attr("p3", &p3.to_string());
        }
        if let Some(pid) = t.pid {
            builder = builder.attr("pid", &pid.to_string());
        }

        builder.write_empty()?;
    }
    writer.end_element("d:triangles")?;

    // Write normal vectors section
    writer.start_element("d:normvectors").write_start()?;
    for n in &mesh.normals {
        writer
            .start_element("d:normvector")
            .attr("nx", &n.nx.to_string())
            .attr("ny", &n.ny.to_string())
            .attr("nz", &n.nz.to_string())
            .write_empty()?;
    }
    writer.end_element("d:normvectors")?;

    // Write gradient vectors if present
    if let Some(gradients) = &mesh.gradients {
        writer.start_element("d:disp2dgroups").write_start()?;
        writer.start_element("d:disp2dgroup").write_start()?;
        for g in gradients {
            writer
                .start_element("d:tex2dcoord")
                .attr("gu", &g.gu.to_string())
                .attr("gv", &g.gv.to_string())
                .write_empty()?;
        }
        writer.end_element("d:disp2dgroup")?;
        writer.end_element("d:disp2dgroups")?;
    }

    writer.end_element("d:displacementmesh")?;
    Ok(())
}

/// Writes a Displacement2D resource element.
pub fn write_displacement_2d<W: Write>(
    writer: &mut XmlWriter<W>,
    res: &Displacement2D,
) -> Result<()> {
    let mut builder = writer
        .start_element("d:displacement2d")
        .attr("id", &res.id.0.to_string())
        .attr("path", &res.path);

    // Channel: write only if not default (G)
    if res.channel != Channel::G {
        builder = builder.attr("channel", channel_to_str(res.channel));
    }

    // TileStyle: write only if not default (Wrap)
    if res.tile_style != TileStyle::Wrap {
        builder = builder.attr("tilestyle", tile_style_to_str(res.tile_style));
    }

    // FilterMode: write only if not default (Linear)
    if res.filter != FilterMode::Linear {
        builder = builder.attr("filter", filter_mode_to_str(res.filter));
    }

    // Height: always required
    builder = builder.attr("height", &res.height.to_string());

    // Offset: write only if non-zero
    if res.offset != 0.0 {
        builder = builder.attr("offset", &res.offset.to_string());
    }

    builder.write_empty()?;
    Ok(())
}

/// Converts a Channel enum to its string representation (lowercase).
fn channel_to_str(c: Channel) -> &'static str {
    match c {
        Channel::R => "r",
        Channel::G => "g",
        Channel::B => "b",
        Channel::A => "a",
    }
}

/// Converts a TileStyle enum to its string representation (lowercase).
fn tile_style_to_str(ts: TileStyle) -> &'static str {
    match ts {
        TileStyle::Wrap => "wrap",
        TileStyle::Mirror => "mirror",
        TileStyle::Clamp => "clamp",
        TileStyle::None => "none",
    }
}

/// Converts a FilterMode enum to its string representation (lowercase).
fn filter_mode_to_str(fm: FilterMode) -> &'static str {
    match fm {
        FilterMode::Linear => "linear",
        FilterMode::Nearest => "nearest",
    }
}
