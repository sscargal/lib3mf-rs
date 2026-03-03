use crate::model::ResourceId;
use serde::{Deserialize, Serialize};

/// A stack of 2D slices defining a geometry (Slice Extension).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SliceStack {
    /// Unique resource ID for this slice stack.
    pub id: ResourceId,
    /// Z-coordinate of the bottom of the first slice.
    #[serde(default)]
    pub z_bottom: f32,
    /// Inline slices contained in this stack.
    pub slices: Vec<Slice>,
    /// References to external slice stacks in other model parts.
    pub refs: Vec<SliceRef>,
}

/// A single 2D slice at a specific Z height.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Slice {
    /// Z-coordinate of the top of this slice.
    pub z_top: f32,
    /// 2D vertex positions used by polygons in this slice.
    pub vertices: Vec<Vertex2D>,
    /// Contour polygons defining the cross-section at this Z height.
    pub polygons: Vec<Polygon>,
}

/// A 2D vertex with X and Y coordinates.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Vertex2D {
    /// X coordinate in the slice plane.
    pub x: f32,
    /// Y coordinate in the slice plane.
    pub y: f32,
}

/// A closed polygon contour within a slice, defined by a sequence of segments.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Polygon {
    /// Index of the start vertex.
    pub start_segment: u32,
    /// Segments connecting vertices to form the closed contour.
    pub segments: Vec<Segment>,
}

/// A single segment within a polygon contour.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Segment {
    /// Index of the vertex to connect to.
    pub v2: u32,
    /// Property index for the start vertex (for material interpolation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p1: Option<u32>,
    /// Property index for the end vertex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p2: Option<u32>,
    /// Property resource ID for this segment (overrides object-level pid).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<ResourceId>,
}

/// A reference to a slice stack in an external model part.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SliceRef {
    /// ID of the referenced slice stack.
    pub slice_stack_id: ResourceId,
    /// Package path to the model part containing the referenced slice stack.
    pub slice_path: String,
}
