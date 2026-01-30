use crate::archive::ArchiveReader;
use crate::error::Result;
use crate::model::stats::{GeometryStats, MaterialsStats, ModelStats, ProductionStats, VendorData};
use crate::model::{Geometry, Model};
use crate::parser::bambu_config::parse_model_settings;

impl Model {
    pub fn compute_stats(&self, archiver: &mut impl ArchiveReader) -> Result<ModelStats> {
        let mut resolver = crate::model::resolver::PartResolver::new(archiver, self.clone());
        let mut geom_stats = GeometryStats::default();

        // 1. Process Build Items (Entry points)
        for item in &self.build.items {
            geom_stats.instance_count += 1;
            self.accumulate_object_stats(
                item.object_id,
                item.path.as_deref(),
                item.transform,
                &mut resolver,
                &mut geom_stats,
            )?;
        }

        // 2. Production Stats
        let prod_stats = ProductionStats {
            uuid_count: 0, // Placeholder
        };

        // 3. Vendor Data
        let mut vendor_data = VendorData::default();
        let generator = self.metadata.get("Application").cloned();

        if let Some(app) = &generator
            && (app.contains("Bambu") || app.contains("Orca"))
        {
            if resolver.archive_mut().entry_exists("Metadata/model_settings.config")
                && let Ok(content) = resolver.archive_mut().read_entry("Metadata/model_settings.config")
                && let Ok(plates) = parse_model_settings(&content)
            {
                vendor_data.plates = plates;
            }
        }

        // 4. Material Stats
        let materials_stats = MaterialsStats {
            base_materials_count: self.resources.base_material_groups_count(),
            color_groups_count: self.resources.color_groups_count(),
            texture_2d_groups_count: self.resources.texture_2d_groups_count(),
            composite_materials_count: self.resources.composite_materials_count(),
            multi_properties_count: self.resources.multi_properties_count(),
        };

        Ok(ModelStats {
            unit: self.unit,
            generator,
            metadata: self.metadata.clone(),
            geometry: geom_stats,
            materials: materials_stats,
            production: prod_stats,
            vendor: vendor_data,
            system_info: crate::utils::hardware::detect_capabilities(),
        })
    }

    fn accumulate_object_stats(
        &self,
        id: crate::model::ResourceId,
        path: Option<&str>,
        transform: glam::Mat4,
        resolver: &mut crate::model::resolver::PartResolver<impl ArchiveReader>,
        stats: &mut GeometryStats,
    ) -> Result<()> {
        let (geom, path_to_use) = {
            let resolved = resolver.resolve_object(id, path)?;
            if let Some((_model, object)) = resolved {
                // Determine the next path to use for children. 
                // If this object was found in a specific path, children inherit it 
                // UNLESS they specify their own.
                let current_path = if path.is_none() || path == Some("ROOT") || path == Some("/3D/3dmodel.model") || path == Some("3D/3dmodel.model") {
                    None
                } else {
                    path
                };
                (Some(object.geometry.clone()), current_path.map(|s| s.to_string()))
            } else {
                (None, None)
            }
        };

        if let Some(geometry) = geom {
            match geometry {
                Geometry::Mesh(mesh) => {
                    stats.object_count += 1;
                    stats.vertex_count += mesh.vertices.len() as u64;
                    stats.triangle_count += mesh.triangles.len() as u64;

                    if let Some(mesh_aabb) = mesh.compute_aabb() {
                        let transformed_aabb = mesh_aabb.transform(transform);
                        if let Some(total_aabb) = &mut stats.bounding_box {
                            total_aabb.min[0] = total_aabb.min[0].min(transformed_aabb.min[0]);
                            total_aabb.min[1] = total_aabb.min[1].min(transformed_aabb.min[1]);
                            total_aabb.min[2] = total_aabb.min[2].min(transformed_aabb.min[2]);
                            total_aabb.max[0] = total_aabb.max[0].max(transformed_aabb.max[0]);
                            total_aabb.max[1] = total_aabb.max[1].max(transformed_aabb.max[1]);
                            total_aabb.max[2] = total_aabb.max[2].max(transformed_aabb.max[2]);
                        } else {
                            stats.bounding_box = Some(transformed_aabb);
                        }
                    }

                    let (area, volume) = mesh.compute_area_and_volume();
                    let scale_det = transform.determinant().abs() as f64;
                    let area_scale = scale_det.powf(2.0 / 3.0);
                    stats.surface_area += area * area_scale;
                    stats.volume += volume * scale_det;
                }
                Geometry::Components(comps) => {
                    for comp in comps.components {
                        // Priority: 
                        // 1. component's own path
                        // 2. path inherited from parent (path_to_use)
                        let next_path = comp.path.as_deref().or(path_to_use.as_deref());
                        
                        self.accumulate_object_stats(
                            comp.object_id,
                            next_path,
                            transform * comp.transform,
                            resolver,
                            stats,
                        )?;
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
