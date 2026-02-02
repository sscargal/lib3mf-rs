use lib3mf_core::error::{Lib3mfError, Result};
use lib3mf_core::model::resources::ResourceId;
use lib3mf_core::model::{BuildItem, Mesh, Model, Triangle};
use std::io::{BufRead, BufReader, Read, Write};

pub struct ObjImporter;

impl ObjImporter {
    pub fn read<R: Read>(reader: R) -> Result<Model> {
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        let mut mesh = Mesh::default();

        // OBJ indices are 1-based
        // Mesh stores vertices in order added.

        while reader.read_line(&mut line).map_err(Lib3mfError::Io)? > 0 {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                line.clear();
                continue;
            }

            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.is_empty() {
                line.clear();
                continue;
            }

            match parts[0] {
                "v" => {
                    if parts.len() < 4 {
                        return Err(Lib3mfError::Validation("Invalid OBJ vertex".to_string()));
                    }
                    let x = parts[1]
                        .parse::<f32>()
                        .map_err(|_| Lib3mfError::Validation("Invalid float".to_string()))?;
                    let y = parts[2]
                        .parse::<f32>()
                        .map_err(|_| Lib3mfError::Validation("Invalid float".to_string()))?;
                    let z = parts[3]
                        .parse::<f32>()
                        .map_err(|_| Lib3mfError::Validation("Invalid float".to_string()))?;
                    mesh.add_vertex(x, y, z);
                }
                "f" => {
                    if parts.len() < 4 {
                        // Skipping point/line elements
                        line.clear();
                        continue;
                    }

                    let mut indices = Vec::new();
                    for part in &parts[1..] {
                        // Format: v, v/vt, v/vt/vn, v//vn
                        let subparts: Vec<&str> = part.split('/').collect();
                        let v_idx_str = subparts[0];
                        let v_idx = v_idx_str
                            .parse::<i32>()
                            .map_err(|_| Lib3mfError::Validation("Invalid index".to_string()))?;

                        let idx = if v_idx > 0 {
                            (v_idx - 1) as u32
                        } else {
                            // Relative index
                            return Err(Lib3mfError::Validation(
                                "Relative OBJ indices not supported yet".to_string(),
                            ));
                        };
                        indices.push(idx);
                    }

                    // Triangulate fan
                    if indices.len() >= 3 {
                        for i in 1..indices.len() - 1 {
                            mesh.triangles.push(Triangle {
                                v1: indices[0],
                                v2: indices[i],
                                v3: indices[i + 1],
                                ..Default::default()
                            });
                        }
                    }
                }
                _ => {} // Ignore vt, vn, g, usemtl etc. for now
            }

            line.clear();
        }

        let mut model = Model::default();
        let resource_id = ResourceId(1);

        let object = lib3mf_core::model::Object {
            id: resource_id,
            object_type: lib3mf_core::model::ObjectType::Model,
            name: Some("OBJ Import".to_string()),
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: lib3mf_core::model::Geometry::Mesh(mesh),
        };

        // Handle result
        let _ = model.resources.add_object(object);

        model.build.items.push(BuildItem {
            object_id: resource_id,
            transform: glam::Mat4::IDENTITY,
            part_number: None,
            uuid: None,
            path: None,
        });

        Ok(model)
    }
}

pub struct ObjExporter;

impl ObjExporter {
    pub fn write<W: Write>(model: &Model, mut writer: W) -> Result<()> {
        let mut vertex_offset = 1;

        for item in &model.build.items {
            if let Some(object) = model.resources.get_object(item.object_id)
                && let lib3mf_core::model::Geometry::Mesh(mesh) = &object.geometry
            {
                let transform = item.transform;

                writeln!(writer, "g {}", object.name.as_deref().unwrap_or("Object"))
                    .map_err(Lib3mfError::Io)?;

                // Write vertices
                for v in &mesh.vertices {
                    let p = transform.transform_point3(glam::Vec3::new(v.x, v.y, v.z));
                    writeln!(writer, "v {} {} {}", p.x, p.y, p.z).map_err(Lib3mfError::Io)?;
                }

                // Write faces
                for tri in &mesh.triangles {
                    writeln!(
                        writer,
                        "f {} {} {}",
                        tri.v1 + vertex_offset,
                        tri.v2 + vertex_offset,
                        tri.v3 + vertex_offset
                    )
                    .map_err(Lib3mfError::Io)?;
                }

                vertex_offset += mesh.vertices.len() as u32;
            }
        }
        Ok(())
    }
}
