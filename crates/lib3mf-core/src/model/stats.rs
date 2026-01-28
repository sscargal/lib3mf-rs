use crate::model::Unit;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStats {
    pub unit: Unit,
    pub generator: Option<String>,
    pub metadata: HashMap<String, String>,
    pub geometry: GeometryStats,
    pub materials: MaterialsStats,
    pub production: ProductionStats,
    pub vendor: VendorData,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialsStats {
    pub base_materials_count: usize,
    pub color_groups_count: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeometryStats {
    pub object_count: usize,
    pub instance_count: usize,
    pub triangle_count: u64,
    pub vertex_count: u64,
    // pub bounds: BoundingBox, // To be implemented
    pub is_manifold: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilamentInfo {
    pub type_: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlateInfo {
    pub id: u32,
    pub name: Option<String>,
    // pub items: Vec<ResourceId>,
}
