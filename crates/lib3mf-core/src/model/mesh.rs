use crate::model::ResourceId;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of 3MF object determining validation requirements and build behavior.
///
/// Per 3MF Core Specification:
/// - Model/SolidSupport: Must be manifold, closed volumes
/// - Support/Surface/Other: Can be non-manifold, open meshes
///
/// # Examples
///
/// ```
/// use lib3mf_core::model::ObjectType;
///
/// let obj_type = ObjectType::default();
/// assert_eq!(obj_type, ObjectType::Model);
///
/// // Check validation requirements
/// assert!(ObjectType::Model.requires_manifold());
/// assert!(!ObjectType::Support.requires_manifold());
///
/// // Check build constraints
/// assert!(ObjectType::Model.can_be_in_build());
/// assert!(!ObjectType::Other.can_be_in_build());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ObjectType {
    /// Printable part - requires manifold mesh (default)
    #[default]
    Model,
    /// Support structure - non-manifold allowed, can be ignored by consumer
    Support,
    /// Solid support structure - manifold required, filled like model
    #[serde(rename = "solidsupport")]
    SolidSupport,
    /// Surface geometry - non-manifold allowed
    Surface,
    /// Other geometry - non-manifold allowed, cannot be referenced in build
    Other,
}

impl std::fmt::Display for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectType::Model => write!(f, "model"),
            ObjectType::Support => write!(f, "support"),
            ObjectType::SolidSupport => write!(f, "solidsupport"),
            ObjectType::Surface => write!(f, "surface"),
            ObjectType::Other => write!(f, "other"),
        }
    }
}

impl ObjectType {
    /// Returns true if this type requires manifold mesh validation
    pub fn requires_manifold(&self) -> bool {
        matches!(self, ObjectType::Model | ObjectType::SolidSupport)
    }

    /// Returns true if this type can be referenced in build items
    pub fn can_be_in_build(&self) -> bool {
        !matches!(self, ObjectType::Other)
    }
}

/// A resource representing a 3D object.
///
/// An object is a reusable resource that defines geometry (Mesh or Components).
/// It can be referenced by Build items or other Components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Object {
    /// Unique identifier for this resource within the model.
    pub id: ResourceId,
    /// Object type determining validation rules and build behavior.
    #[serde(default)]
    pub object_type: ObjectType,
    /// Human-readable name (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Part number for inventory tracking (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_number: Option<String>,
    /// Production Extension UUID (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,
    /// Default Property ID (material/color) for this object (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<ResourceId>,
    /// Default Property Index for this object (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pindex: Option<u32>,
    /// Path to the thumbnail image in the package (optional).
    /// Used for object-level thumbnails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    /// The actual geometric content of the object.
    pub geometry: Geometry,
}

/// The geometric data of an object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Geometry {
    /// A triangle mesh.
    Mesh(Mesh),
    /// A hierarchical assembly of other objects.
    Components(Components),
    /// A stack of 2D slices (Slice Extension).
    SliceStack(ResourceId),
    /// Voxel data (Volumetric Extension).
    VolumetricStack(ResourceId),
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Components {
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    pub object_id: ResourceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,
    #[serde(default = "default_transform", skip_serializing_if = "is_identity")]
    pub transform: glam::Mat4,
}

fn default_transform() -> glam::Mat4 {
    glam::Mat4::IDENTITY
}

fn is_identity(transform: &glam::Mat4) -> bool {
    *transform == glam::Mat4::IDENTITY
}

/// A mesh defined by vertices and triangles.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Mesh {
    /// List of vertices (points in 3D space).
    pub vertices: Vec<Vertex>,
    /// List of triangles connecting vertices.
    pub triangles: Vec<Triangle>,
    /// Beam Lattice extension data (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beam_lattice: Option<BeamLattice>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BeamLattice {
    #[serde(default)]
    pub min_length: f32,
    #[serde(default)]
    pub precision: f32,
    #[serde(default)]
    pub clipping_mode: ClippingMode,
    pub beams: Vec<Beam>,
    pub beam_sets: Vec<BeamSet>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClippingMode {
    #[default]
    None,
    Inside,
    Outside,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Beam {
    pub v1: u32,
    pub v2: u32,
    pub r1: f32,
    pub r2: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p1: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p2: Option<u32>,
    #[serde(default)]
    pub cap_mode: CapMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CapMode {
    #[default]
    Sphere,
    Hemisphere,
    Butt,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BeamSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    pub refs: Vec<u32>,
}

impl Mesh {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_vertex(&mut self, x: f32, y: f32, z: f32) -> u32 {
        self.vertices.push(Vertex { x, y, z });
        (self.vertices.len() - 1) as u32
    }

    pub fn add_triangle(&mut self, v1: u32, v2: u32, v3: u32) {
        self.triangles.push(Triangle {
            v1,
            v2,
            v3,
            ..Default::default()
        });
    }

    pub fn compute_aabb(&self) -> Option<crate::model::stats::BoundingBox> {
        if self.vertices.is_empty() {
            return None;
        }

        let initial = (
            f32::INFINITY,
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        );

        #[cfg(feature = "parallel")]
        let (min_x, min_y, min_z, max_x, max_y, max_z) = {
            use rayon::prelude::*;
            self.vertices
                .par_iter()
                .fold(
                    || initial,
                    |acc, v| {
                        (
                            acc.0.min(v.x),
                            acc.1.min(v.y),
                            acc.2.min(v.z),
                            acc.3.max(v.x),
                            acc.4.max(v.y),
                            acc.5.max(v.z),
                        )
                    },
                )
                .reduce(
                    || initial,
                    |a, b| {
                        (
                            a.0.min(b.0),
                            a.1.min(b.1),
                            a.2.min(b.2),
                            a.3.max(b.3),
                            a.4.max(b.4),
                            a.5.max(b.5),
                        )
                    },
                )
        };

        #[cfg(not(feature = "parallel"))]
        let (min_x, min_y, min_z, max_x, max_y, max_z) =
            self.vertices.iter().fold(initial, |acc, v| {
                (
                    acc.0.min(v.x),
                    acc.1.min(v.y),
                    acc.2.min(v.z),
                    acc.3.max(v.x),
                    acc.4.max(v.y),
                    acc.5.max(v.z),
                )
            });

        Some(crate::model::stats::BoundingBox {
            min: [min_x, min_y, min_z],
            max: [max_x, max_y, max_z],
        })
    }

    pub fn compute_area_and_volume(&self) -> (f64, f64) {
        if self.triangles.is_empty() {
            return (0.0, 0.0);
        }

        #[cfg(feature = "parallel")]
        let (area, volume) = {
            use rayon::prelude::*;
            self.triangles
                .par_iter()
                .fold(
                    || (0.0f64, 0.0f64),
                    |acc, t| {
                        let (area, volume) = self.compute_triangle_stats(t);
                        (acc.0 + area, acc.1 + volume)
                    },
                )
                .reduce(|| (0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1))
        };

        #[cfg(not(feature = "parallel"))]
        let (area, volume) = self.triangles.iter().fold((0.0f64, 0.0f64), |acc, t| {
            let (area, volume) = self.compute_triangle_stats(t);
            (acc.0 + area, acc.1 + volume)
        });

        (area, volume)
    }

    fn compute_triangle_stats(&self, t: &Triangle) -> (f64, f64) {
        let v1 = glam::Vec3::new(
            self.vertices[t.v1 as usize].x,
            self.vertices[t.v1 as usize].y,
            self.vertices[t.v1 as usize].z,
        );
        let v2 = glam::Vec3::new(
            self.vertices[t.v2 as usize].x,
            self.vertices[t.v2 as usize].y,
            self.vertices[t.v2 as usize].z,
        );
        let v3 = glam::Vec3::new(
            self.vertices[t.v3 as usize].x,
            self.vertices[t.v3 as usize].y,
            self.vertices[t.v3 as usize].z,
        );

        // Area using cross product
        let edge1 = v2 - v1;
        let edge2 = v3 - v1;
        let cross = edge1.cross(edge2);
        let triangle_area = 0.5 * cross.length() as f64;

        // Signed volume of tetrahedron from origin
        let triangle_volume = (v1.dot(v2.cross(v3)) / 6.0) as f64;

        (triangle_area, triangle_volume)
    }

    pub fn compute_triangle_area(&self, triangle: &Triangle) -> f64 {
        let v1 = glam::Vec3::new(
            self.vertices[triangle.v1 as usize].x,
            self.vertices[triangle.v1 as usize].y,
            self.vertices[triangle.v1 as usize].z,
        );
        let v2 = glam::Vec3::new(
            self.vertices[triangle.v2 as usize].x,
            self.vertices[triangle.v2 as usize].y,
            self.vertices[triangle.v2 as usize].z,
        );
        let v3 = glam::Vec3::new(
            self.vertices[triangle.v3 as usize].x,
            self.vertices[triangle.v3 as usize].y,
            self.vertices[triangle.v3 as usize].z,
        );

        let edge1 = v2 - v1;
        let edge2 = v3 - v1;
        let cross = edge1.cross(edge2);
        0.5 * cross.length() as f64
    }
}

/// A single point in 3D space.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for Vertex {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

/// A triangle defined by three vertex indices.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Triangle {
    /// Index of the first vertex.
    pub v1: u32,
    /// Index of the second vertex.
    pub v2: u32,
    /// Index of the third vertex.
    pub v3: u32,

    /// Property index for v1 (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p1: Option<u32>,
    /// Property index for v2 (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p2: Option<u32>,
    /// Property index for v3 (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p3: Option<u32>,

    /// Property ID for the entire triangle (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}
