use crate::model::ResourceId;
use glam::Mat4;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The build section defining what objects to print and where.
///
/// The build section is the "print job" in a 3MF model. It specifies which
/// objects should be manufactured and their positions/orientations on the
/// build platform. Only objects referenced in the build will be printed;
/// other objects in the model are available for reference but won't be
/// manufactured unless included in a build item.
///
/// # Examples
///
/// ```
/// use lib3mf_core::model::{Build, BuildItem, ResourceId};
///
/// let mut build = Build::default();
/// build.items.push(BuildItem {
///     object_id: ResourceId(1),
///     uuid: None,
///     path: None,
///     part_number: None,
///     transform: glam::Mat4::IDENTITY,
/// });
/// assert_eq!(build.items.len(), 1);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Build {
    /// The list of objects to manufacture.
    pub items: Vec<BuildItem>,
}

/// A reference to an object that should be manufactured.
///
/// Build items specify which objects from the resource collection should be
/// printed and where they should be positioned on the build platform. Each
/// item can apply a transformation to position/rotate the object.
///
/// Only objects with types that can appear in the build (not `ObjectType::Other`)
/// are valid build item references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildItem {
    /// The ID of the object to manufacture.
    pub object_id: ResourceId,

    /// Production Extension UUID for tracking this build item instance (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,

    /// External reference path for production tracking (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Part number for this build item instance (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_number: Option<String>,

    /// Transformation matrix positioning the object on the build platform.
    /// Defaults to identity (origin position, no rotation).
    /// The transformation is applied using the glam Mat4 type.
    #[serde(default = "default_transform", skip_serializing_if = "is_identity")]
    pub transform: Mat4,
}

fn default_transform() -> Mat4 {
    Mat4::IDENTITY
}

fn is_identity(transform: &Mat4) -> bool {
    *transform == Mat4::IDENTITY
}
