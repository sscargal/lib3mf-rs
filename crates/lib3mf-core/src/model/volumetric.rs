use crate::model::ResourceId;
use serde::{Deserialize, Serialize};

/// Represents a stack of volumetric layers, defining a 3D volume via slices.
/// Similar to SliceStack but typically implies raster/voxel data or implicit fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VolumetricStack {
    pub id: ResourceId,
    pub version: String, // e.g. "1.0"
    pub layers: Vec<VolumetricLayer>,
    pub refs: Vec<VolumetricRef>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VolumetricLayer {
    pub z_height: f32, // The Z-height of this layer
    // Content can be an image path, or raw data reference.
    // Spec usually uses image stack approach or field.
    // We will use a flexible 'path' to resource (e.g. texture path).
    pub content_path: String, 
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VolumetricRef {
    pub stack_id: ResourceId,
    pub path: String, // Path to the other model file
}
