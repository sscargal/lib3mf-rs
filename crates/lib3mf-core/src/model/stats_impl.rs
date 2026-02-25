use crate::archive::ArchiveReader;
use crate::error::Result;
use crate::model::stats::{
    DisplacementStats, FilamentInfo, GeometryStats, MaterialsStats, ModelStats, ProductionStats,
    VendorData,
};
use crate::model::{Geometry, Model};

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

        // 3. Vendor Data (Bambu Studio / OrcaSlicer)
        let mut vendor_data = VendorData::default();
        let generator = self.metadata.get("Application").cloned();

        let is_bambu = generator
            .as_ref()
            .map_or(false, |app| app.contains("Bambu") || app.contains("Orca"));

        if is_bambu {
            let archive = resolver.archive_mut();

            // 3a. Parse slice_info.config (slicer version, printer model, filaments, print time, warnings)
            if archive.entry_exists("Metadata/slice_info.config") {
                if let Ok(content) = archive.read_entry("Metadata/slice_info.config") {
                    if let Ok(slice_info) =
                        crate::parser::bambu_config::parse_slice_info(&content)
                    {
                        vendor_data.slicer_version = slice_info.client_version.as_ref().map(|v| {
                            let client =
                                slice_info.client_type.as_deref().unwrap_or("BambuStudio");
                            format!("{}-{}", client.replace(' ', ""), v)
                        });

                        // Aggregate print time and filaments across all plates
                        let mut total_time_secs: u32 = 0;
                        for plate in &slice_info.plates {
                            if let Some(pred) = plate.prediction {
                                total_time_secs += pred;
                            }
                            // Collect slicer warnings
                            for w in &plate.warnings {
                                vendor_data.slicer_warnings.push(w.clone());
                            }
                        }

                        if total_time_secs > 0 {
                            vendor_data.print_time_estimate =
                                Some(format_duration(total_time_secs));
                        }

                        // Filaments from first plate (they are per-plate but typically same)
                        if let Some(first_plate) = slice_info.plates.first() {
                            for f in &first_plate.filaments {
                                vendor_data.filaments.push(FilamentInfo {
                                    id: f.id,
                                    tray_info_idx: f.tray_info_idx.clone(),
                                    type_: f.type_.clone().unwrap_or_default(),
                                    color: f.color.clone(),
                                    used_m: f.used_m,
                                    used_g: f.used_g,
                                });
                            }
                        }
                    }
                }
            }

            // 3b. Parse model_settings.config (plates, objects, assembly)
            if archive.entry_exists("Metadata/model_settings.config") {
                if let Ok(content) = archive.read_entry("Metadata/model_settings.config") {
                    if let Ok(data) = crate::parser::bambu_config::parse_model_settings(&content) {
                        vendor_data.plates = data.plates;
                        vendor_data.object_metadata = data.objects;
                        vendor_data.assembly_info = data.assembly;
                    }
                }
            }

            // 3c. Parse project_settings.config (printer model, bed type, layer height, etc.)
            if archive.entry_exists("Metadata/project_settings.config") {
                if let Ok(content) = archive.read_entry("Metadata/project_settings.config") {
                    if let Ok(settings) =
                        crate::parser::bambu_config::parse_project_settings(&content)
                    {
                        // Use project settings for printer model if not already set from slice_info
                        if vendor_data.printer_model.is_none() {
                            vendor_data.printer_model = settings
                                .printer_inherits
                                .clone()
                                .or_else(|| settings.printer_model.clone());
                        }
                        if vendor_data.nozzle_diameter.is_none() {
                            vendor_data.nozzle_diameter = settings.nozzle_diameter.first().copied();
                        }
                        vendor_data.project_settings = Some(settings);
                    }
                }
            }

            // 3d. Parse per-profile configs (filament_settings_N.config, machine_settings_N.config, process_settings_N.config)
            for config_type in &["filament", "machine", "process"] {
                for n in 0u32..16 {
                    let path = format!("Metadata/{}_settings_{}.config", config_type, n);
                    if archive.entry_exists(&path) {
                        if let Ok(content) = archive.read_entry(&path) {
                            if let Ok(config) =
                                crate::parser::bambu_config::parse_profile_config(&content, config_type, n)
                            {
                                vendor_data.profile_configs.push(config);
                            }
                        }
                    }
                }
            }

            // Try to get printer model from machine profile if still not set
            if vendor_data.printer_model.is_none() {
                if let Some(machine_config) = vendor_data
                    .profile_configs
                    .iter()
                    .find(|c| c.config_type == "machine")
                {
                    vendor_data.printer_model = machine_config
                        .inherits
                        .clone()
                        .or_else(|| machine_config.name.clone());
                }
            }

            // 3e. Read OPC relationships and identify Bambu-specific entries
            if archive.entry_exists("_rels/.rels") {
                if let Ok(rels_data) = archive.read_entry("_rels/.rels") {
                    if let Ok(rels) = crate::archive::opc::parse_relationships(&rels_data) {
                        use crate::archive::opc::bambu_rel_types;
                        for rel in &rels {
                            match rel.rel_type.as_str() {
                                bambu_rel_types::COVER_THUMBNAIL_MIDDLE
                                | bambu_rel_types::COVER_THUMBNAIL_SMALL => {
                                    if vendor_data.bambu_cover_thumbnail.is_none() {
                                        vendor_data.bambu_cover_thumbnail =
                                            Some(rel.target.clone());
                                    }
                                }
                                bambu_rel_types::GCODE => {
                                    vendor_data.bambu_gcode = Some(rel.target.clone());
                                }
                                _ => {}
                            }
                        }
                    }
                }
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

        // 5. Displacement Stats
        let displacement_stats = self.compute_displacement_stats();

        // 6. Thumbnails
        // Check archiver for package thumbnail (attachments may not be loaded)
        let pkg_thumb = archiver.entry_exists("Metadata/thumbnail.png")
            || archiver.entry_exists("/Metadata/thumbnail.png");
        let obj_thumb_count = self
            .resources
            .iter_objects()
            .filter(|o| o.thumbnail.is_some())
            .count();

        Ok(ModelStats {
            unit: self.unit,
            generator,
            metadata: self.metadata.clone(),
            geometry: geom_stats,
            materials: materials_stats,
            production: prod_stats,
            displacement: displacement_stats,
            vendor: vendor_data,
            system_info: crate::utils::hardware::detect_capabilities(),
            thumbnails: crate::model::stats::ThumbnailStats {
                package_thumbnail_present: pkg_thumb,
                object_thumbnail_count: obj_thumb_count,
            },
        })
    }

    fn compute_displacement_stats(&self) -> DisplacementStats {
        let mut stats = DisplacementStats {
            texture_count: self.resources.displacement_2d_count(),
            ..Default::default()
        };

        // Count DisplacementMesh objects and accumulate metrics
        for obj in self.resources.iter_objects() {
            if let Geometry::DisplacementMesh(dmesh) = &obj.geometry {
                stats.mesh_count += 1;
                stats.normal_count += dmesh.normals.len() as u64;
                stats.gradient_count += dmesh.gradients.as_ref().map_or(0, |g| g.len() as u64);
                stats.total_triangle_count += dmesh.triangles.len() as u64;

                // Count triangles with displacement indices
                for tri in &dmesh.triangles {
                    if tri.d1.is_some() || tri.d2.is_some() || tri.d3.is_some() {
                        stats.displaced_triangle_count += 1;
                    }
                }
            }
        }

        stats
    }

    fn accumulate_object_stats(
        &self,
        id: crate::model::ResourceId,
        path: Option<&str>,
        transform: glam::Mat4,
        resolver: &mut crate::model::resolver::PartResolver<impl ArchiveReader>,
        stats: &mut GeometryStats,
    ) -> Result<()> {
        let (geom, path_to_use, obj_type) = {
            let resolved = resolver.resolve_object(id, path)?;
            if let Some((_model, object)) = resolved {
                // Determine the next path to use for children.
                // If this object was found in a specific path, children inherit it
                // UNLESS they specify their own.
                let current_path = if path.is_none()
                    || path == Some("ROOT")
                    || path == Some("/3D/3dmodel.model")
                    || path == Some("3D/3dmodel.model")
                {
                    None
                } else {
                    path
                };
                (
                    Some(object.geometry.clone()),
                    current_path.map(|s| s.to_string()),
                    Some(object.object_type),
                )
            } else {
                (None, None, None)
            }
        };

        if let Some(geometry) = geom {
            // Count object by type
            if let Some(ot) = obj_type {
                *stats.type_counts.entry(ot.to_string()).or_insert(0) += 1;
            }

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

/// Format seconds as human-readable duration (e.g., "31m 35s", "2h 15m 3s").
pub fn format_duration(total_secs: u32) -> String {
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        if seconds > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}h", hours)
        }
    } else if minutes > 0 {
        if seconds > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}m", minutes)
        }
    } else {
        format!("{}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::format_duration;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(0), "0s");
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(60), "1m");
        assert_eq!(format_duration(61), "1m 1s");
        assert_eq!(format_duration(1895), "31m 35s");
        assert_eq!(format_duration(3600), "1h");
        assert_eq!(format_duration(3660), "1h 1m");
        assert_eq!(format_duration(3661), "1h 1m 1s");
        assert_eq!(format_duration(7200), "2h");
    }
}
