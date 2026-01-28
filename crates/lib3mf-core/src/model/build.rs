use crate::model::ResourceId;
use glam::Mat4;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The build section of the model, containing items to be printed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Build {
    pub items: Vec<BuildItem>,
}

/// An item in the build, referencing an object resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildItem {
    pub object_id: ResourceId,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_number: Option<String>,
    
    #[serde(default = "default_transform", skip_serializing_if = "is_identity")]
    pub transform: Mat4,
}

fn default_transform() -> Mat4 {
    Mat4::IDENTITY
}

fn is_identity(transform: &Mat4) -> bool {
    *transform == Mat4::IDENTITY
}
