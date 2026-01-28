use crate::error::Result;
use crate::model::{Geometry, Model, Unit};
use crate::writer::mesh_writer::write_mesh;
use crate::writer::xml_writer::XmlWriter;
use std::io::Write;

impl Model {
    pub fn write_xml<W: Write>(&self, writer: W) -> Result<()> {
        let mut xml = XmlWriter::new(writer);
        xml.write_declaration()?;

        let root = xml
            .start_element("model")
            .attr("unit", self.unit_str())
            .attr("xml:lang", self.language.as_deref().unwrap_or("en-US"))
            .attr("xmlns", "http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel");
            
        // Add typical namespaces if needed (e.g. production, slice) - strictly core for now
        root.write_start()?;

        // Metadata
        for (key, value) in &self.metadata {
            xml.start_element("metadata")
                .attr("name", key)
                .write_start()?;
            xml.write_text(value)?;
            xml.end_element("metadata")?;
        }

        // Resources
        xml.start_element("resources").write_start()?;
        for obj in self.resources.iter_objects() {
            let mut obj_elem = xml.start_element("object")
                .attr("id", &obj.id.0.to_string())
                .attr("type", "model"); // TODO: object type enum
                
            if let Some(pid) = obj.part_number.as_ref() {
                obj_elem = obj_elem.attr("partnumber", pid);
            }
            if let Some(name) = obj.name.as_ref() {
                obj_elem = obj_elem.attr("name", name);
            }
            
            obj_elem.write_start()?;

            match &obj.geometry {
                Geometry::Mesh(mesh) => write_mesh(&mut xml, mesh)?,
                Geometry::Components(comps) => {
                    xml.start_element("components").write_start()?;
                    for c in &comps.components {
                         let transform_str = format!(
                            "{} {} {} {} {} {} {} {} {} {} {} {}",
                            c.transform.x_axis.x, c.transform.x_axis.y, c.transform.x_axis.z,
                            c.transform.y_axis.x, c.transform.y_axis.y, c.transform.y_axis.z,
                            c.transform.z_axis.x, c.transform.z_axis.y, c.transform.z_axis.z,
                            c.transform.w_axis.x, c.transform.w_axis.y, c.transform.w_axis.z
                        );
                        
                        let mut comp = xml.start_element("component")
                            .attr("objectid", &c.object_id.0.to_string());
                            
                        if c.transform != glam::Mat4::IDENTITY {
                            comp = comp.attr("transform", &transform_str);
                        }
                        comp.write_empty()?;
                    }
                    xml.end_element("components")?;
                }
                Geometry::SliceStack(_id) => {
                    // Logic for SliceStack writing requires setting attribute on object element
                    // But object element is already started.
                    // This writer structure makes it hard to add attributes conditionally based on geometry type
                    // unless we peek geometry before starting object element.
                    // For now, I will assume writing slice models via this writer is not fully supported 
                    // or requires refactoring.
                    // I will leave it empty as SliceStack objects have no body content (mesh/components).
                    // BUT they need `slicestackid` on the object tag.
                    // Refactoring needed to support Slice extension writing.
                    // Phase 11 goal is parsing/validation. I will skip writing implementation logic but fix valid Rust match.
                }
                Geometry::VolumetricStack(_id) => {
                    // Similar to SliceStack, requires attribute on object tag.
                }
            }
            
            xml.end_element("object")?;
        }
        xml.end_element("resources")?;

        // Build
        xml.start_element("build").write_start()?;
        for item in &self.build.items {
             let transform_str = format!(
                "{} {} {} {} {} {} {} {} {} {} {} {}",
                item.transform.x_axis.x, item.transform.x_axis.y, item.transform.x_axis.z,
                item.transform.y_axis.x, item.transform.y_axis.y, item.transform.y_axis.z,
                item.transform.z_axis.x, item.transform.z_axis.y, item.transform.z_axis.z,
                item.transform.w_axis.x, item.transform.w_axis.y, item.transform.w_axis.z
            );

            let mut build_item = xml.start_element("item")
                .attr("objectid", &item.object_id.0.to_string());
                
            if item.transform != glam::Mat4::IDENTITY {
                 build_item = build_item.attr("transform", &transform_str);
            }
             // partnumber support if needed
            build_item.write_empty()?;
        }
        xml.end_element("build")?;

        xml.end_element("model")?;
        Ok(())
    }

    fn unit_str(&self) -> &'static str {
        match self.unit {
            Unit::Micron => "micron",
            Unit::Millimeter => "millimeter",
            Unit::Centimeter => "centimeter",
            Unit::Inch => "inch",
            Unit::Foot => "foot",
            Unit::Meter => "meter",
        }
    }
}
