use crate::model::Unit;
use crate::utils::hardware::HardwareCapabilities;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive statistics and metadata for a 3MF model.
///
/// Aggregates various statistics about the model's geometry, materials,
/// production metadata, and vendor-specific information. Used by the CLI
/// `stats` command and for model analysis.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelStats {
    /// Unit of measurement for the model
    pub unit: Unit,
    /// Software that generated the model (from metadata)
    pub generator: Option<String>,
    /// Custom metadata key-value pairs from the model
    pub metadata: HashMap<String, String>,
    /// Geometric statistics (vertices, triangles, volume, etc.)
    pub geometry: GeometryStats,
    /// Material and property statistics
    pub materials: MaterialsStats,
    /// Production extension metadata statistics
    pub production: ProductionStats,
    /// Displacement extension statistics
    pub displacement: DisplacementStats,
    /// Vendor-specific data (e.g., Bambu Studio project info)
    pub vendor: VendorData,
    /// System hardware capabilities info
    pub system_info: HardwareCapabilities,
    /// Thumbnail statistics
    pub thumbnails: ThumbnailStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThumbnailStats {
    pub package_thumbnail_present: bool,
    pub object_thumbnail_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialsStats {
    pub base_materials_count: usize,
    pub color_groups_count: usize,
    pub texture_2d_groups_count: usize,
    pub composite_materials_count: usize,
    pub multi_properties_count: usize,
}

/// Geometric statistics for the model.
///
/// Aggregates counts and measurements of the model's geometry including
/// vertices, triangles, bounding box, surface area, and volume.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeometryStats {
    /// Total number of object resources
    pub object_count: usize,
    /// Number of build items (instances to print)
    pub instance_count: usize,
    /// Total number of triangles across all meshes
    pub triangle_count: u64,
    /// Total number of vertices across all meshes
    pub vertex_count: u64,
    /// Axis-aligned bounding box of the entire model
    pub bounding_box: Option<BoundingBox>,
    /// Total surface area in square model units
    pub surface_area: f64,
    /// Total volume in cubic model units
    pub volume: f64,
    /// Whether all meshes are manifold (watertight)
    pub is_manifold: bool,
    /// Count of objects by type (e.g., {"model": 5, "support": 2})
    #[serde(default)]
    pub type_counts: HashMap<String, usize>,
}

/// An axis-aligned bounding box in 3D space.
///
/// Represents the smallest box (aligned with coordinate axes) that
/// contains all geometry. Useful for understanding model size and
/// for spatial queries.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct BoundingBox {
    /// Minimum corner coordinates [x, y, z]
    pub min: [f32; 3],
    /// Maximum corner coordinates [x, y, z]
    pub max: [f32; 3],
}

impl BoundingBox {
    pub fn transform(&self, matrix: glam::Mat4) -> Self {
        let corners = [
            glam::Vec3::new(self.min[0], self.min[1], self.min[2]),
            glam::Vec3::new(self.min[0], self.min[1], self.max[2]),
            glam::Vec3::new(self.min[0], self.max[1], self.min[2]),
            glam::Vec3::new(self.min[0], self.max[1], self.max[2]),
            glam::Vec3::new(self.max[0], self.min[1], self.min[2]),
            glam::Vec3::new(self.max[0], self.min[1], self.max[2]),
            glam::Vec3::new(self.max[0], self.max[1], self.min[2]),
            glam::Vec3::new(self.max[0], self.max[1], self.max[2]),
        ];

        let mut transformed_min = glam::Vec3::splat(f32::INFINITY);
        let mut transformed_max = glam::Vec3::splat(f32::NEG_INFINITY);

        for corner in corners {
            let transformed = matrix.transform_point3(corner);
            transformed_min = transformed_min.min(transformed);
            transformed_max = transformed_max.max(transformed);
        }

        Self {
            min: [transformed_min.x, transformed_min.y, transformed_min.z],
            max: [transformed_max.x, transformed_max.y, transformed_max.z],
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProductionStats {
    pub uuid_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisplacementStats {
    pub mesh_count: usize,
    pub texture_count: usize,
    pub normal_count: u64,
    pub gradient_count: u64,
    pub displaced_triangle_count: u64,
    pub total_triangle_count: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VendorData {
    pub printer_model: Option<String>,
    pub filaments: Vec<FilamentInfo>,
    pub plates: Vec<PlateInfo>,
    pub print_time_estimate: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilamentInfo {
    pub type_: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlateInfo {
    pub id: u32,
    pub name: Option<String>,
    // pub items: Vec<ResourceId>,
}
