use crate::archive::ArchiveReader;
use crate::error::Result;
use crate::model::stats::{
    GeometryStats, MaterialsStats, ModelStats, ProductionStats, VendorData,
};
use crate::model::{Geometry, Model};
use crate::parser::bambu_config::parse_model_settings;

impl Model {
    pub fn compute_stats(&self, archiver: &mut impl ArchiveReader) -> Result<ModelStats> {
        // 1. Geometry Stats
        let mut geom_stats = GeometryStats::default();
        
        // Count resources objects
        // Note: iter_objects returns unique objects
        geom_stats.object_count = self.resources.iter_objects().count();
        
        for obj in self.resources.iter_objects() {
            match &obj.geometry {
                Geometry::Mesh(mesh) => {
                    geom_stats.vertex_count += mesh.vertices.len() as u64;
                    geom_stats.triangle_count += mesh.triangles.len() as u64;
                }
                Geometry::Components(_comps) => {
                    // Components themselves don't add vertices directly, they reference other objects.
                    // If we wanted "total instanced polygons", we'd traverse.
                    // For "unique geometry", we just count the mesh objects above.
                    // For now, standard stats usually report unique geometry count.
                }

                Geometry::SliceStack(_id) => {
                    // SliceStack stats not strictly "geometry" (mesh/production) in the same way.
                    // Could count slices or polygons if we resolved it.
                }
            }
        }
        
        // Count build items
        geom_stats.instance_count = self.build.items.len();
        // TODO: For accurate instance count, we should recursively count components too?
        // Usually instance count refers to top-level build items.

        // 2. Production Stats
        let prod_stats = ProductionStats {
            uuid_count: 0, // Placeholder, would need to walk all UUID fields
        };

        // 3. Vendor Data
        let mut vendor_data = VendorData::default();
        
        // Detect Generator
        let generator = self.metadata.get("Application").cloned();
        
        // Bambu/Prusa specific metadata
         if let Some(app) = &generator {
            if app.contains("Bambu") || app.contains("Orca") {
                // Try to read Metadata/model_settings.config
                if archiver.entry_exists("Metadata/model_settings.config") {
                    if let Ok(content) = archiver.read_entry("Metadata/model_settings.config") {
                        if let Ok(plates) = parse_model_settings(&content) {
                            vendor_data.plates = plates;
                        }
                    }
                }
            }
        }
        
        // Extract Printer Model if available (Bambu uses separate config, usually machine_settings)
        // For now, simpler metadata checks or placeholder.

        // 4. Material Stats
        let materials_stats = MaterialsStats {
            base_materials_count: self.resources.base_material_groups_count(),
            color_groups_count: self.resources.color_groups_count(),
        };

        Ok(ModelStats {
            unit: self.unit,
            generator,
            metadata: self.metadata.clone(),
            geometry: geom_stats,
            materials: materials_stats,
            production: prod_stats,
            vendor: vendor_data,
        })
    }
}
