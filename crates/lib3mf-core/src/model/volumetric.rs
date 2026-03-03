use crate::model::ResourceId;
use serde::{Deserialize, Serialize};

/// Represents a stack of volumetric layers, defining a 3D volume via slices.
/// Similar to SliceStack but typically implies raster/voxel data or implicit fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VolumetricStack {
    /// Unique resource ID for this volumetric stack.
    pub id: ResourceId,
    /// Extension version string (e.g., `"1.0"`).
    pub version: String, // e.g. "1.0"
    /// Inline volumetric layers.
    pub layers: Vec<VolumetricLayer>,
    /// References to volumetric stacks in external model parts.
    pub refs: Vec<VolumetricRef>,
}

/// A single volumetric layer at a specific Z height.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VolumetricLayer {
    /// Z-height of this layer in model units.
    pub z_height: f32, // The Z-height of this layer
    // Content can be an image path, or raw data reference.
    // Spec usually uses image stack approach or field.
    // We will use a flexible 'path' to resource (e.g. texture path).
    /// Path to the content resource (e.g., a texture image path in the package).
    pub content_path: String,
}

/// A reference to a volumetric stack in an external model part.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VolumetricRef {
    /// ID of the referenced volumetric stack.
    pub stack_id: ResourceId,
    /// Package path to the model part containing the referenced volumetric stack.
    pub path: String, // Path to the other model file
}
