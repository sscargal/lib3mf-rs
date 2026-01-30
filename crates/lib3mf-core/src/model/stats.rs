use crate::model::Unit;
use crate::utils::hardware::HardwareCapabilities;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelStats {
    pub unit: Unit,
    pub generator: Option<String>,
    pub metadata: HashMap<String, String>,
    pub geometry: GeometryStats,
    pub materials: MaterialsStats,
    pub production: ProductionStats,
    pub vendor: VendorData,
    pub system_info: HardwareCapabilities,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialsStats {
    pub base_materials_count: usize,
    pub color_groups_count: usize,
    pub texture_2d_groups_count: usize,
    pub composite_materials_count: usize,
    pub multi_properties_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeometryStats {
    pub object_count: usize,
    pub instance_count: usize,
    pub triangle_count: u64,
    pub vertex_count: u64,
    pub bounding_box: Option<BoundingBox>,
    pub surface_area: f64,
    pub volume: f64,
    pub is_manifold: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: [f32; 3],
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
