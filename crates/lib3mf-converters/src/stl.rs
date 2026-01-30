use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use lib3mf_core::error::{Lib3mfError, Result};
use lib3mf_core::model::resources::ResourceId;
use lib3mf_core::model::{BuildItem, Mesh, Model, Triangle, Vertex};
use std::io::{Read, Write};

pub struct StlImporter;

impl Default for StlImporter {
    fn default() -> Self {
        Self::new()
    }
}

impl StlImporter {
    pub fn new() -> Self {
        Self
    }

    pub fn read<R: Read>(mut reader: R) -> Result<Model> {
        // STL Format:
        // 80 bytes header
        // 4 bytes triangle info (u32)
        // Triangles...

        let mut header = [0u8; 80];
        reader.read_exact(&mut header).map_err(Lib3mfError::Io)?;

        let triangle_count = reader.read_u32::<LittleEndian>().map_err(|_| {
            Lib3mfError::Validation("Failed to read STL triangle count".to_string())
        })?;

        let mut mesh = Mesh::default();
        use std::collections::HashMap;
        let mut vert_map: HashMap<[u32; 3], u32> = HashMap::new();

        for _ in 0..triangle_count {
            // Normal (3 floats) - Ignored
            let _nx = reader.read_f32::<LittleEndian>().map_err(Lib3mfError::Io)?;
            let _ny = reader.read_f32::<LittleEndian>().map_err(Lib3mfError::Io)?;
            let _nz = reader.read_f32::<LittleEndian>().map_err(Lib3mfError::Io)?;

            let mut indices = [0u32; 3];

            for index in &mut indices {
                let x = reader.read_f32::<LittleEndian>().map_err(Lib3mfError::Io)?;
                let y = reader.read_f32::<LittleEndian>().map_err(Lib3mfError::Io)?;
                let z = reader.read_f32::<LittleEndian>().map_err(Lib3mfError::Io)?;

                let key = [x.to_bits(), y.to_bits(), z.to_bits()];

                let idx = *vert_map.entry(key).or_insert_with(|| {
                    let new_idx = mesh.vertices.len() as u32;
                    mesh.vertices.push(Vertex { x, y, z });
                    new_idx
                });
                *index = idx;
            }

            let _attr_byte_count = reader.read_u16::<LittleEndian>().map_err(Lib3mfError::Io)?;

            mesh.triangles.push(Triangle {
                v1: indices[0],
                v2: indices[1],
                v3: indices[2],
                ..Default::default()
            });
        }

        let mut model = Model::default();
        let resource_id = ResourceId(1); // Default ID

        let object = lib3mf_core::model::Object {
            id: resource_id,
            name: Some("STL Import".to_string()),
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            geometry: lib3mf_core::model::Geometry::Mesh(mesh),
        };

        let _ = model.resources.add_object(object);

        model.build.items.push(BuildItem {
            object_id: resource_id,
            transform: glam::Mat4::IDENTITY,
            part_number: None,
            uuid: None, // Generate one?
            path: None,
        });

        Ok(model)
    }
}

pub struct StlExporter;

impl StlExporter {
    pub fn write<W: Write>(model: &Model, mut writer: W) -> Result<()> {
        // 1. Collect all triangles from all build items
        let mut triangles: Vec<(glam::Vec3, glam::Vec3, glam::Vec3)> = Vec::new(); // v1, v2, v3

        for item in &model.build.items {
            if let Some(object) = model.resources.get_object(item.object_id)
                && let lib3mf_core::model::Geometry::Mesh(mesh) = &object.geometry
            {
                let transform = item.transform;

                for tri in &mesh.triangles {
                    let v1_local = mesh.vertices[tri.v1 as usize];
                    let v2_local = mesh.vertices[tri.v2 as usize];
                    let v3_local = mesh.vertices[tri.v3 as usize];

                    let v1 = transform
                        .transform_point3(glam::Vec3::new(v1_local.x, v1_local.y, v1_local.z));
                    let v2 = transform
                        .transform_point3(glam::Vec3::new(v2_local.x, v2_local.y, v2_local.z));
                    let v3 = transform
                        .transform_point3(glam::Vec3::new(v3_local.x, v3_local.y, v3_local.z));

                    triangles.push((v1, v2, v3));
                }
            }
        }

        // 2. Write Header (80 bytes)
        let header = [0u8; 80];
        writer.write_all(&header).map_err(Lib3mfError::Io)?;

        // 3. Write Count
        writer
            .write_u32::<LittleEndian>(triangles.len() as u32)
            .map_err(Lib3mfError::Io)?;

        // 4. Write Triangles
        for (v1, v2, v3) in triangles {
            // Normal (0,0,0) - let viewer calculate
            writer
                .write_f32::<LittleEndian>(0.0)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(0.0)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(0.0)
                .map_err(Lib3mfError::Io)?;

            // v1
            writer
                .write_f32::<LittleEndian>(v1.x)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v1.y)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v1.z)
                .map_err(Lib3mfError::Io)?;

            // v2
            writer
                .write_f32::<LittleEndian>(v2.x)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v2.y)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v2.z)
                .map_err(Lib3mfError::Io)?;

            // v3
            writer
                .write_f32::<LittleEndian>(v3.x)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v3.y)
                .map_err(Lib3mfError::Io)?;
            writer
                .write_f32::<LittleEndian>(v3.z)
                .map_err(Lib3mfError::Io)?;

            // Attribute byte count (0)
            writer
                .write_u16::<LittleEndian>(0)
                .map_err(Lib3mfError::Io)?;
        }

        Ok(())
    }
}
