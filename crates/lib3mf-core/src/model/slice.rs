use crate::model::ResourceId;
use serde::{Deserialize, Serialize};

/// A stack of 2D slices defining a geometry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SliceStack {
    pub id: ResourceId,
    #[serde(default)]
    pub z_bottom: f32,
    pub slices: Vec<Slice>,
    pub refs: Vec<SliceRef>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Slice {
    pub z_top: f32,
    pub vertices: Vec<Vertex2D>,
    pub polygons: Vec<Polygon>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Vertex2D {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Polygon {
    /// Index of the start vertex.
    pub start_segment: u32,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Segment {
    /// Index of the vertex to connect to.
    pub v2: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p1: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p2: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<ResourceId>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SliceRef {
    pub slice_stack_id: ResourceId,
    pub slice_path: String,
}
