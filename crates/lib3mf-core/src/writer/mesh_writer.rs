use crate::error::Result;
use crate::model::mesh::Mesh;
use crate::writer::xml_writer::XmlWriter;
use std::io::Write;

pub fn write_mesh<W: Write>(writer: &mut XmlWriter<W>, mesh: &Mesh) -> Result<()> {
    writer.start_element("mesh").write_start()?;

    // Vertices
    writer.start_element("vertices").write_start()?;
    for v in &mesh.vertices {
        writer
            .start_element("vertex")
            .attr("x", &v.x.to_string())
            .attr("y", &v.y.to_string())
            .attr("z", &v.z.to_string())
            .write_empty()?;
    }
    writer.end_element("vertices")?;

    // Triangles
    writer.start_element("triangles").write_start()?;
    for t in &mesh.triangles {
        let mut builder = writer
            .start_element("triangle")
            .attr("v1", &t.v1.to_string())
            .attr("v2", &t.v2.to_string())
            .attr("v3", &t.v3.to_string());
            
        if let Some(p1) = t.p1 { builder = builder.attr("p1", &p1.to_string()); }
        if let Some(p2) = t.p2 { builder = builder.attr("p2", &p2.to_string()); }
        if let Some(p3) = t.p3 { builder = builder.attr("p3", &p3.to_string()); }
        if let Some(pid) = t.pid { builder = builder.attr("pid", &pid.to_string()); }
        
        builder.write_empty()?;
    }
    writer.end_element("triangles")?;

    writer.end_element("mesh")?;
    Ok(())
}
