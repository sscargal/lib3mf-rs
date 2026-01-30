use crate::model::Mesh;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepairOptions {
    /// Epsilon for merging vertices. Vertices closer than this will be merged.
    pub stitch_epsilon: f32,
    /// Whether to remove triangles with zero/near-zero area.
    pub remove_degenerate: bool,
    /// Whether to remove duplicate triangles (sharing same sorted vertices).
    pub remove_duplicate_faces: bool,
}

impl Default for RepairOptions {
    fn default() -> Self {
        Self {
            stitch_epsilon: 1e-4, // 0.1 microns effectively
            remove_degenerate: true,
            remove_duplicate_faces: true,
        }
    }
}

pub trait MeshRepair {
    /// Attempt to repair the mesh in-place based on the provided options.
    /// Returns a report of what was done (e.g., number of vertices merged).
    fn repair(&mut self, options: RepairOptions) -> RepairStats;
}

#[derive(Debug, Clone, Default)]
pub struct RepairStats {
    pub vertices_removed: usize,
    pub triangles_removed: usize,
}

impl MeshRepair for Mesh {
    fn repair(&mut self, options: RepairOptions) -> RepairStats {
        let mut stats = RepairStats::default();

        if options.stitch_epsilon > 0.0 {
            let removed = stitch_vertices(self, options.stitch_epsilon);
            stats.vertices_removed += removed;
        }

        if options.remove_degenerate || options.remove_duplicate_faces {
            let removed_tris = clean_triangles(
                self,
                options.remove_degenerate,
                options.remove_duplicate_faces,
            );
            stats.triangles_removed += removed_tris;
        }

        // Always clean up unused vertices after operations if any changes occurred or if explicitly requested
        // For now, we do it if we stitched or removed triangles, or just always for a "repair"
        let removed_verts = remove_unused_vertices(self);
        stats.vertices_removed += removed_verts;

        stats
    }
}

fn remove_unused_vertices(mesh: &mut Mesh) -> usize {
    let initial_count = mesh.vertices.len();
    if initial_count == 0 {
        return 0;
    }

    // 1. Mark used vertices
    let mut used = vec![false; initial_count];
    for tri in &mesh.triangles {
        used[tri.v1 as usize] = true;
        used[tri.v2 as usize] = true;
        used[tri.v3 as usize] = true;
    }

    // 2. Create remapping
    let mut new_vertices = Vec::with_capacity(initial_count);
    let mut old_to_new = vec![0u32; initial_count];

    for (old_idx, &is_used) in used.iter().enumerate() {
        if is_used {
            old_to_new[old_idx] = new_vertices.len() as u32;
            new_vertices.push(mesh.vertices[old_idx]);
        }
    }

    let removed = initial_count - new_vertices.len();

    // 3. Update mesh
    mesh.vertices = new_vertices;
    for tri in &mut mesh.triangles {
        tri.v1 = old_to_new[tri.v1 as usize];
        tri.v2 = old_to_new[tri.v2 as usize];
        tri.v3 = old_to_new[tri.v3 as usize];
    }

    removed
}

fn stitch_vertices(mesh: &mut Mesh, epsilon: f32) -> usize {
    if mesh.vertices.is_empty() {
        return 0;
    }
    // ...

    let initial_count = mesh.vertices.len();
    let mut new_vertices = Vec::with_capacity(initial_count);
    // Map from (x, y, z) logical integer coordinates to new vertex index
    let mut point_map: HashMap<(i64, i64, i64), u32> = HashMap::new();
    // Map from old index to new index
    let mut index_remap = vec![0u32; initial_count];

    // Inverse epsilon for coordinate quantization
    let inv_eps = 1.0 / epsilon;

    for (old_idx, v) in mesh.vertices.iter().enumerate() {
        let key = (
            (v.x * inv_eps).round() as i64,
            (v.y * inv_eps).round() as i64,
            (v.z * inv_eps).round() as i64,
        );

        if let Some(&existing_idx) = point_map.get(&key) {
            // Found a match, remap this old vertex to the existing new one
            index_remap[old_idx] = existing_idx;
        } else {
            // New unique vertex
            let new_idx = new_vertices.len() as u32;
            new_vertices.push(*v);
            point_map.insert(key, new_idx);
            index_remap[old_idx] = new_idx;
        }
    }

    let merged_count = initial_count - new_vertices.len();

    // Update mesh
    mesh.vertices = new_vertices;

    // Remap triangles
    for tri in &mut mesh.triangles {
        tri.v1 = index_remap[tri.v1 as usize];
        tri.v2 = index_remap[tri.v2 as usize];
        tri.v3 = index_remap[tri.v3 as usize];
    }

    merged_count
}

fn clean_triangles(mesh: &mut Mesh, remove_degenerate: bool, remove_duplicates: bool) -> usize {
    let initial_count = mesh.triangles.len();
    let mut valid_triangles = Vec::with_capacity(initial_count);

    // For duplicate check
    let mut seen_faces = std::collections::HashSet::new();

    for tri in &mesh.triangles {
        // 1. Check Index degeneracy
        if (tri.v1 == tri.v2 || tri.v2 == tri.v3 || tri.v3 == tri.v1) && remove_degenerate {
            continue;
        }

        // 2. Check Area degeneracy
        if remove_degenerate {
            let area = mesh.compute_triangle_area(tri);
            if area <= 1e-9 {
                continue;
            }
        }

        // 3. Check Duplicate Faces
        if remove_duplicates {
            let mut indices = [tri.v1, tri.v2, tri.v3];
            indices.sort_unstable(); // Sort to handle (0,1,2) same as (1,2,0) same as (2,1,0) if ordering ignored
            // Strictly speaking, (0,1,2) and (0,2,1) are opposite faces.
            // Often "duplicate" means literally redundant geometry, regardless of winding if we are cleaning mess.
            // But usually we want to keep one.
            if !seen_faces.insert(indices) {
                continue;
            }
        }

        valid_triangles.push(*tri);
    }

    let removed = initial_count - valid_triangles.len();
    mesh.triangles = valid_triangles;
    removed
}
