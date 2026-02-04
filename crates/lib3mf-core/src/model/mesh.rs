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

/// A 3D object resource containing geometry and metadata.
///
/// Objects are the primary reusable resources in a 3MF model. They define geometry
/// (meshes, components, boolean shapes, etc.) and can be referenced by build items
/// or composed into other objects via components.
///
/// Each object has an [`ObjectType`] that determines its validation requirements and
/// whether it can appear in the build. For example, `Model` and `SolidSupport` types
/// must be manifold closed volumes, while `Support` and `Surface` types can be
/// non-manifold.
///
/// # Examples
///
/// See [`Mesh`] for examples of creating geometry to place in an object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Object {
    /// Unique identifier for this resource within the model.
    /// See [`ResourceId`] for details on the global namespace.
    pub id: ResourceId,
    /// Object type determining validation rules and build behavior.
    #[serde(default)]
    pub object_type: ObjectType,
    /// Human-readable name for the object (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Part number for inventory/manufacturing tracking (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_number: Option<String>,
    /// Production Extension UUID for unique identification (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,
    /// Default property resource ID (material/color) for this object (optional).
    /// Used when triangles don't specify their own properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<ResourceId>,
    /// Default property index within the property resource (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pindex: Option<u32>,
    /// Path to the thumbnail image in the 3MF package (optional).
    /// Used for object-level thumbnails (distinct from package thumbnail).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    /// The actual geometric content of the object.
    pub geometry: Geometry,
}

/// The geometric data contained in an object.
///
/// Represents the different types of geometry that can be stored in a 3MF object.
/// The basic types are meshes and component assemblies, with various extensions
/// adding support for slices, voxels, boolean operations, and displacement mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Geometry {
    /// A triangle mesh (the most common geometry type).
    Mesh(Mesh),
    /// A hierarchical assembly of other objects via components.
    Components(Components),
    /// A stack of 2D slices for layer-based printing (Slice Extension).
    /// References a slice stack resource by ID.
    SliceStack(ResourceId),
    /// Voxel-based volumetric data (Volumetric Extension).
    /// References a volumetric stack resource by ID.
    VolumetricStack(ResourceId),
    /// Constructive solid geometry from boolean operations (Boolean Operations Extension).
    BooleanShape(BooleanShape),
    /// A mesh with displacement mapping for fine surface detail (Displacement Extension).
    DisplacementMesh(DisplacementMesh),
}

impl Geometry {
    /// Returns true if this geometry contains actual content (non-empty mesh,
    /// components, boolean shapes, or displacement meshes).
    ///
    /// A default-constructed `Geometry::Mesh(Mesh::default())` has no content
    /// (no vertices, no triangles). This is the default return from
    /// `parse_object_geometry` when no `<mesh>` or `<components>` child element
    /// is present.
    pub fn has_content(&self) -> bool {
        match self {
            Geometry::Mesh(mesh) => !mesh.vertices.is_empty() || !mesh.triangles.is_empty(),
            Geometry::Components(c) => !c.components.is_empty(),
            Geometry::BooleanShape(_) => true,
            Geometry::DisplacementMesh(_) => true,
            // SliceStack and VolumetricStack are references, not inline content
            Geometry::SliceStack(_) | Geometry::VolumetricStack(_) => false,
        }
    }
}

/// A collection of components forming a hierarchical assembly.
///
/// Components allow building complex objects by composing and transforming
/// other objects. This enables reuse and efficient representation of
/// assemblies with repeated parts.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Components {
    /// The list of component instances in this assembly.
    pub components: Vec<Component>,
}

/// A reference to another object with optional transformation.
///
/// Components enable hierarchical object composition by referencing other
/// objects (which can themselves contain meshes or more components). Each
/// component can apply a transformation matrix to position/rotate/scale
/// the referenced object.
///
/// # Examples
///
/// Components are commonly used to:
/// - Create assemblies from multiple parts
/// - Reuse the same object in different positions (instances)
/// - Apply transformations (rotation, scaling, translation) to objects
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    /// ID of the object being referenced.
    pub object_id: ResourceId,
    /// External reference path for production tracking (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Production Extension UUID for this component instance (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<Uuid>,
    /// Transformation matrix applied to the referenced object.
    /// Defaults to identity (no transformation).
    #[serde(default = "default_transform", skip_serializing_if = "is_identity")]
    pub transform: glam::Mat4,
}

fn default_transform() -> glam::Mat4 {
    glam::Mat4::IDENTITY
}

fn is_identity(transform: &glam::Mat4) -> bool {
    *transform == glam::Mat4::IDENTITY
}

/// Type of boolean operation to apply between shapes.
///
/// Per 3MF Boolean Operations Extension v1.1.1:
/// - Union: Combines both shapes (default)
/// - Difference: Subtracts operation shape from base
/// - Intersection: Only keeps overlapping volume
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BooleanOperationType {
    /// Combine both shapes (default)
    #[default]
    Union,
    /// Subtract operation shape from base
    Difference,
    /// Keep only overlapping volume
    Intersection,
}

/// A single boolean operation applied to a shape.
///
/// Represents one `<boolean>` element within a `<booleanshape>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BooleanOperation {
    /// Type of boolean operation (union, difference, intersection)
    #[serde(default)]
    pub operation_type: BooleanOperationType,
    /// Reference to the object to apply
    pub object_id: ResourceId,
    /// Transformation matrix applied to the operation object
    #[serde(default = "default_transform", skip_serializing_if = "is_identity")]
    pub transform: glam::Mat4,
    /// Optional external reference path (p:path)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// A boolean shape combining multiple objects with CSG operations.
///
/// Represents a `<booleanshape>` resource that defines geometry through
/// constructive solid geometry (CSG) operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BooleanShape {
    /// Base object to start with
    pub base_object_id: ResourceId,
    /// Transformation applied to base object
    #[serde(default = "default_transform", skip_serializing_if = "is_identity")]
    pub base_transform: glam::Mat4,
    /// Optional external reference path for base (p:path)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_path: Option<String>,
    /// Ordered list of boolean operations to apply
    pub operations: Vec<BooleanOperation>,
}

/// A triangle mesh representing 3D geometry.
///
/// A mesh is the fundamental geometry container in 3MF, consisting of vertices
/// (3D points) and triangles that connect those vertices. Meshes can optionally
/// include beam lattice structures for lightweight, high-strength geometry.
///
/// # Examples
///
/// ```
/// use lib3mf_core::model::Mesh;
///
/// let mut mesh = Mesh::new();
/// let v1 = mesh.add_vertex(0.0, 0.0, 0.0);
/// let v2 = mesh.add_vertex(1.0, 0.0, 0.0);
/// let v3 = mesh.add_vertex(0.0, 1.0, 0.0);
/// mesh.add_triangle(v1, v2, v3);
/// assert_eq!(mesh.triangles.len(), 1);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Mesh {
    /// List of vertices (points in 3D space).
    pub vertices: Vec<Vertex>,
    /// List of triangles connecting vertices by their indices.
    pub triangles: Vec<Triangle>,
    /// Beam Lattice extension data for structural lattice geometry (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beam_lattice: Option<BeamLattice>,
}

/// Beam lattice structure for lightweight, high-strength geometry.
///
/// The Beam Lattice extension allows representing cylindrical beams between
/// vertices as an alternative to solid triangle meshes. This is particularly
/// useful for lightweight structures, scaffolding, and lattice infill patterns.
///
/// Each beam is a cylinder connecting two vertices with potentially different
/// radii at each end (creating tapered beams).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BeamLattice {
    /// Minimum beam length threshold (beams shorter than this may be ignored).
    #[serde(default)]
    pub min_length: f32,
    /// Precision for beam representation (implementation-defined).
    #[serde(default)]
    pub precision: f32,
    /// How beams should be clipped by the mesh boundary.
    #[serde(default)]
    pub clipping_mode: ClippingMode,
    /// The list of beams in this lattice.
    pub beams: Vec<Beam>,
    /// Named groups of beams for organization and material assignment.
    pub beam_sets: Vec<BeamSet>,
}

/// Clipping mode for beam lattice geometry.
///
/// Controls how beams interact with the mesh boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClippingMode {
    /// No clipping applied (default).
    #[default]
    None,
    /// Clip beams to inside the mesh boundary.
    Inside,
    /// Clip beams to outside the mesh boundary.
    Outside,
}

/// A cylindrical beam connecting two vertices.
///
/// Beams are defined by two vertex indices and a radius at each end,
/// allowing for tapered beams. They can have different materials at
/// each endpoint via property indices.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Beam {
    /// Index of the first vertex.
    pub v1: u32,
    /// Index of the second vertex.
    pub v2: u32,
    /// Radius at the first vertex.
    pub r1: f32,
    /// Radius at the second vertex.
    pub r2: f32,
    /// Property index at the first vertex (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p1: Option<u32>,
    /// Property index at the second vertex (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p2: Option<u32>,
    /// Cap style for the beam ends.
    #[serde(default)]
    pub cap_mode: CapMode,
}

/// End cap style for beams.
///
/// Determines how the ends of beams are terminated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CapMode {
    /// Spherical cap (default) - fully rounded.
    #[default]
    Sphere,
    /// Hemispherical cap - half sphere.
    Hemisphere,
    /// Flat cap - no rounding.
    Butt,
}

/// A named group of beams.
///
/// Beam sets allow organizing beams and applying properties to groups.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BeamSet {
    /// Human-readable name for this beam set (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Machine-readable identifier (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    /// Indices of beams in this set (references into the beams array).
    pub refs: Vec<u32>,
}

impl Mesh {
    /// Creates a new empty mesh.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a vertex to the mesh and returns its index.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate in model units
    /// * `y` - Y coordinate in model units
    /// * `z` - Z coordinate in model units
    ///
    /// # Returns
    ///
    /// The index of the newly added vertex, which can be used to reference this vertex in triangles.
    pub fn add_vertex(&mut self, x: f32, y: f32, z: f32) -> u32 {
        self.vertices.push(Vertex { x, y, z });
        (self.vertices.len() - 1) as u32
    }

    /// Adds a triangle to the mesh connecting three vertices.
    ///
    /// # Arguments
    ///
    /// * `v1` - Index of the first vertex
    /// * `v2` - Index of the second vertex
    /// * `v3` - Index of the third vertex
    ///
    /// The vertex indices should be in counter-clockwise order when viewed from outside
    /// the mesh for correct normal orientation.
    pub fn add_triangle(&mut self, v1: u32, v2: u32, v3: u32) {
        self.triangles.push(Triangle {
            v1,
            v2,
            v3,
            ..Default::default()
        });
    }

    /// Computes the axis-aligned bounding box (AABB) of the mesh.
    ///
    /// Returns `None` if the mesh has no vertices.
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

    /// Computes the total surface area and volume of the mesh.
    ///
    /// Uses triangle area calculation and signed tetrahedron volumes.
    /// Returns (0.0, 0.0) if the mesh has no triangles.
    ///
    /// # Returns
    ///
    /// A tuple of (surface_area, volume) in square and cubic model units respectively.
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

    /// Computes the area of a single triangle.
    ///
    /// # Arguments
    ///
    /// * `triangle` - Reference to the triangle whose area to compute
    ///
    /// # Returns
    ///
    /// The area of the triangle in square model units.
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
///
/// Represents a vertex position in the mesh coordinate system.
/// Coordinates are in model units (see [`Unit`](crate::model::Unit)).
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Vertex {
    /// X coordinate in model units
    pub x: f32,
    /// Y coordinate in model units
    pub y: f32,
    /// Z coordinate in model units
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

/// A triangle face defined by three vertex indices.
///
/// Triangles are the fundamental building blocks of 3MF meshes. They reference
/// vertices by index and can optionally specify material properties per-vertex
/// or per-triangle.
///
/// # Material Properties
///
/// The property system allows materials/colors to be assigned at different levels:
/// - **Triangle-level**: Use `pid` to assign a property resource to the entire triangle
/// - **Vertex-level**: Use `p1`, `p2`, `p3` to assign different properties to each vertex
/// - **Object-level**: If no triangle properties are set, the object's default `pid`/`pindex` apply
///
/// The property resolution hierarchy is: Vertex → Triangle → Object → None
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Triangle {
    /// Index of the first vertex (counter-clockwise winding).
    pub v1: u32,
    /// Index of the second vertex (counter-clockwise winding).
    pub v2: u32,
    /// Index of the third vertex (counter-clockwise winding).
    pub v3: u32,

    /// Property index for v1 (optional, for per-vertex material assignment).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p1: Option<u32>,
    /// Property index for v2 (optional, for per-vertex material assignment).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p2: Option<u32>,
    /// Property index for v3 (optional, for per-vertex material assignment).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p3: Option<u32>,

    /// Property ID resource for the entire triangle (optional, for per-triangle material assignment).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

/// A normal vector for displacement mesh vertices.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct NormalVector {
    pub nx: f32,
    pub ny: f32,
    pub nz: f32,
}

/// A gradient vector for displacement texture mapping.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct GradientVector {
    pub gu: f32,
    pub gv: f32,
}

/// A triangle in a displacement mesh with displacement coordinate indices.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct DisplacementTriangle {
    /// Index of the first vertex.
    pub v1: u32,
    /// Index of the second vertex.
    pub v2: u32,
    /// Index of the third vertex.
    pub v3: u32,

    /// Displacement coordinate index for v1 (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d1: Option<u32>,
    /// Displacement coordinate index for v2 (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d2: Option<u32>,
    /// Displacement coordinate index for v3 (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub d3: Option<u32>,

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

/// A mesh with displacement mapping support.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DisplacementMesh {
    /// List of vertices (points in 3D space).
    pub vertices: Vec<Vertex>,
    /// List of triangles connecting vertices.
    pub triangles: Vec<DisplacementTriangle>,
    /// Per-vertex normal vectors (must match vertex count).
    pub normals: Vec<NormalVector>,
    /// Per-vertex gradient vectors (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gradients: Option<Vec<GradientVector>>,
}
