use crate::model::{Mesh, Triangle};
use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn from_triangle(mesh: &Mesh, tri: &Triangle) -> Self {
        let v1 = mesh.vertices[tri.v1 as usize];
        let v2 = mesh.vertices[tri.v2 as usize];
        let v3 = mesh.vertices[tri.v3 as usize];

        let min = Vec3::new(
            v1.x.min(v2.x).min(v3.x),
            v1.y.min(v2.y).min(v3.y),
            v1.z.min(v2.z).min(v3.z),
        );
        let max = Vec3::new(
            v1.x.max(v2.x).max(v3.x),
            v1.y.max(v2.y).max(v3.y),
            v1.z.max(v2.z).max(v3.z),
        );
        Self { min, max }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }
}

pub struct BvhNode {
    pub aabb: AABB,
    pub content: BvhContent,
}

pub enum BvhContent {
    Leaf(Vec<usize>), // Indices of triangles
    Branch(Box<BvhNode>, Box<BvhNode>),
}

impl BvhNode {
    pub fn build(mesh: &Mesh, tri_indices: Vec<usize>) -> Self {
        let aabbs: Vec<AABB> = tri_indices
            .iter()
            .map(|&i| AABB::from_triangle(mesh, &mesh.triangles[i]))
            .collect();

        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for aabb in &aabbs {
            min = min.min(aabb.min);
            max = max.max(aabb.max);
        }

        let node_aabb = AABB { min, max };

        if tri_indices.len() <= 8 {
            return BvhNode {
                aabb: node_aabb,
                content: BvhContent::Leaf(tri_indices),
            };
        }

        // Split along largest axis
        let size = max - min;
        let axis = if size.x > size.y && size.x > size.z {
            0
        } else if size.y > size.z {
            1
        } else {
            2
        };

        let mid = (min[axis] + max[axis]) / 2.0;

        let mut left_indices = Vec::new();
        let mut right_indices = Vec::new();

        for (i, &tri_idx) in tri_indices.iter().enumerate() {
            let center = (aabbs[i].min[axis] + aabbs[i].max[axis]) / 2.0;
            if center < mid {
                left_indices.push(tri_idx);
            } else {
                right_indices.push(tri_idx);
            }
        }

        // Fallback if split failed to partition
        if left_indices.is_empty() || right_indices.is_empty() {
            return BvhNode {
                aabb: node_aabb,
                content: BvhContent::Leaf(tri_indices),
            };
        }

        BvhNode {
            aabb: node_aabb,
            content: BvhContent::Branch(
                Box::new(BvhNode::build(mesh, left_indices)),
                Box::new(BvhNode::build(mesh, right_indices)),
            ),
        }
    }

    pub fn find_intersections(
        &self,
        mesh: &Mesh,
        tri_idx: usize,
        tri_aabb: &AABB,
        results: &mut Vec<usize>,
    ) {
        if !self.aabb.intersects(tri_aabb) {
            return;
        }

        match &self.content {
            BvhContent::Leaf(indices) => {
                for &idx in indices {
                    if idx > tri_idx {
                        // Avoid double-counting and self-check
                        if tri_aabb.intersects(&AABB::from_triangle(mesh, &mesh.triangles[idx])) {
                            // Precise check:
                            if intersect_triangles(mesh, tri_idx, idx) {
                                results.push(idx);
                            }
                        }
                    }
                }
            }
            BvhContent::Branch(left, right) => {
                left.find_intersections(mesh, tri_idx, tri_aabb, results);
                right.find_intersections(mesh, tri_idx, tri_aabb, results);
            }
        }
    }
}

/// Robust triangle-triangle intersection (simplified Moller-Trumbore)
fn intersect_triangles(mesh: &Mesh, i1: usize, i2: usize) -> bool {
    let t1 = &mesh.triangles[i1];
    let t2 = &mesh.triangles[i2];

    // Shared vertices: check if they share 1, 2 or 3 vertices.
    // If they share 2 vertices, they share an edge.
    // If they share 3, they are identical.
    // In many contexts, edge/vert sharing is NOT considered intersection for "self-intersection" reporting.
    // 3MF spec: triangles MUST NOT intersect each other EXCEPT on common edges.
    let shared = count_shared_vertices(t1, t2);
    if shared >= 2 {
        return false;
    }

    let p1 = to_vec3(mesh.vertices[t1.v1 as usize]);
    let p2 = to_vec3(mesh.vertices[t1.v2 as usize]);
    let p3 = to_vec3(mesh.vertices[t1.v3 as usize]);

    let q1 = to_vec3(mesh.vertices[t2.v1 as usize]);
    let q2 = to_vec3(mesh.vertices[t2.v2 as usize]);
    let q3 = to_vec3(mesh.vertices[t2.v3 as usize]);

    tri_tri_intersect(p1, p2, p3, q1, q2, q3)
}

fn to_vec3(v: crate::model::Vertex) -> Vec3 {
    Vec3::new(v.x, v.y, v.z)
}

fn count_shared_vertices(t1: &Triangle, t2: &Triangle) -> usize {
    let mut count = 0;
    let v1 = [t1.v1, t1.v2, t1.v3];
    let v2 = [t2.v1, t2.v2, t2.v3];
    for &va in &v1 {
        for &vb in &v2 {
            if va == vb {
                count += 1;
            }
        }
    }
    count
}

// ----------------------------------------------------------------------------
// Simplified Triangle-Triangle Intersection logic (adapted)
// ----------------------------------------------------------------------------

fn tri_tri_intersect(p1: Vec3, p2: Vec3, p3: Vec3, q1: Vec3, q2: Vec3, q3: Vec3) -> bool {
    // 1. Plane of tri 2
    let n2 = (q2 - q1).cross(q3 - q1);
    if n2.length_squared() < 1e-12 {
        return false;
    } // Degenerate tri 2
    let d2 = -n2.dot(q1);

    // Distances of tri 1 vertices to plane 2
    let du0 = n2.dot(p1) + d2;
    let du1 = n2.dot(p2) + d2;
    let du2 = n2.dot(p3) + d2;

    if (du0.abs() > 1e-6 && du1.abs() > 1e-6 && du2.abs() > 1e-6)
        && ((du0 > 0.0 && du1 > 0.0 && du2 > 0.0) || (du0 < 0.0 && du1 < 0.0 && du2 < 0.0))
    {
        return false; // Tri 1 entirely on one side of plane 2
    }

    // 2. Plane of tri 1
    let n1 = (p2 - p1).cross(p3 - p1);
    if n1.length_squared() < 1e-12 {
        return false;
    } // Degenerate tri 1
    let d1 = -n1.dot(p1);

    // Distances of tri 2 vertices to plane 1
    let dv0 = n1.dot(q1) + d1;
    let dv1 = n1.dot(q2) + d1;
    let dv2 = n1.dot(q3) + d1;

    if (dv0.abs() > 1e-6 && dv1.abs() > 1e-6 && dv2.abs() > 1e-6)
        && ((dv0 > 0.0 && dv1 > 0.0 && dv2 > 0.0) || (dv0 < 0.0 && dv1 < 0.0 && dv2 < 0.0))
    {
        return false; // Tri 2 entirely on one side of plane 1
    }

    // 3. Line of intersection L
    let ld = n1.cross(n2);
    let index = if ld.x.abs() > ld.y.abs() && ld.x.abs() > ld.z.abs() {
        0
    } else if ld.y.abs() > ld.z.abs() {
        1
    } else {
        2
    };

    // Projection onto L
    let get_interval =
        |v1: Vec3, v2: Vec3, v3: Vec3, d1: f32, d2: f32, d3: f32| -> Option<(f32, f32)> {
            // Find which vertices are on which side
            if (d1 > 0.0 && d2 > 0.0 && d3 > 0.0) || (d1 < 0.0 && d2 < 0.0 && d3 < 0.0) {
                return None;
            }

            let mut pts = Vec::new();
            let tris = [(v1, v2, d1, d2), (v2, v3, d2, d3), (v3, v1, d3, d1)];
            for (a, b, da, db) in tris {
                if (da >= 0.0) != (db >= 0.0) {
                    let t = da / (da - db);
                    let p = a + (b - a) * t;
                    pts.push(p[index]);
                } else if da.abs() < 1e-7 {
                    pts.push(a[index]);
                }
            }
            if pts.len() < 2 {
                return None;
            }
            let mut min = pts[0];
            let mut max = pts[0];
            for &p in &pts {
                min = min.min(p);
                max = max.max(p);
            }
            Some((min, max))
        };

    let i1 = get_interval(p1, p2, p3, du0, du1, du2);
    let i2 = get_interval(q1, q2, q3, dv0, dv1, dv2);

    match (i1, i2) {
        (Some((t1_min, t1_max)), Some((t2_min, t2_max))) => {
            // Check overlap of intervals on line L
            // Use small epsilon to avoid reporting touching as intersection
            t1_min + 1e-6 < t2_max && t2_min + 1e-6 < t1_max
        }
        _ => false,
    }
}
