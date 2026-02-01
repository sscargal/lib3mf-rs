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
    /// Whether to harmonize triangle winding (orientations).
    pub harmonize_orientations: bool,
    /// Whether to remove disconnected components (islands). If true, only the largest component is kept.
    pub remove_islands: bool,
    /// Whether to attempt to fill holes (boundary loops).
    pub fill_holes: bool,
}

impl Default for RepairOptions {
    fn default() -> Self {
        Self {
            stitch_epsilon: 1e-4, // 0.1 microns effectively
            remove_degenerate: true,
            remove_duplicate_faces: true,
            harmonize_orientations: true,
            remove_islands: false,
            fill_holes: false,
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
    pub triangles_flipped: usize,
    pub triangles_added: usize,
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

        if options.remove_islands {
            let removed = remove_islands(self);
            stats.triangles_removed += removed;
        }

        if options.fill_holes {
            let added = fill_holes(self);
            stats.triangles_added += added;
        }

        if options.harmonize_orientations {
            let flipped = harmonize_orientations(self);
            stats.triangles_flipped += flipped;
        }

        stats
    }
}

fn remove_islands(mesh: &mut Mesh) -> usize {
    if mesh.triangles.is_empty() {
        return 0;
    }

    // 1. Group triangles by component
    let mut edge_to_tris: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
    for (i, tri) in mesh.triangles.iter().enumerate() {
        let edges = [
            sort_unord_edge(tri.v1, tri.v2),
            sort_unord_edge(tri.v2, tri.v3),
            sort_unord_edge(tri.v3, tri.v1),
        ];
        for e in edges {
            edge_to_tris.entry(e).or_default().push(i);
        }
    }

    let mut visited = vec![false; mesh.triangles.len()];
    let mut component_tris = Vec::new();

    for start_idx in 0..mesh.triangles.len() {
        if visited[start_idx] {
            continue;
        }

        let mut current_comp = Vec::new();
        let mut stack = vec![start_idx];
        visited[start_idx] = true;

        while let Some(curr_idx) = stack.pop() {
            current_comp.push(curr_idx);
            let tri = &mesh.triangles[curr_idx];
            let edges = [
                sort_unord_edge(tri.v1, tri.v2),
                sort_unord_edge(tri.v2, tri.v3),
                sort_unord_edge(tri.v3, tri.v1),
            ];

            for e in edges {
                if let Some(neighbors) = edge_to_tris.get(&e) {
                    for &neigh_idx in neighbors {
                        if !visited[neigh_idx] {
                            visited[neigh_idx] = true;
                            stack.push(neigh_idx);
                        }
                    }
                }
            }
        }
        component_tris.push(current_comp);
    }

    if component_tris.len() <= 1 {
        return 0;
    }

    // 2. Keep only the largest component
    component_tris.sort_by_key(|b| std::cmp::Reverse(b.len()));

    let initial_count = mesh.triangles.len();
    let largest_comp = &component_tris[0];
    let mut new_triangles = Vec::with_capacity(largest_comp.len());
    for &idx in largest_comp {
        new_triangles.push(mesh.triangles[idx]);
    }

    mesh.triangles = new_triangles;
    initial_count - mesh.triangles.len()
}

fn sort_unord_edge(v1: u32, v2: u32) -> (u32, u32) {
    if v1 < v2 { (v1, v2) } else { (v2, v1) }
}

fn fill_holes(mesh: &mut Mesh) -> usize {
    if mesh.triangles.is_empty() {
        return 0;
    }

    // 1. Identify boundary edges (count == 1)
    let mut edge_counts = HashMap::new();
    for tri in &mesh.triangles {
        let edges = [
            sort_unord_edge(tri.v1, tri.v2),
            sort_unord_edge(tri.v2, tri.v3),
            sort_unord_edge(tri.v3, tri.v1),
        ];
        for e in edges {
            *edge_counts.entry(e).or_insert(0) += 1;
        }
    }

    let mut boundary_edges = Vec::new();
    for (edge, count) in edge_counts {
        if count == 1 {
            boundary_edges.push(edge);
        }
    }

    if boundary_edges.is_empty() {
        return 0;
    }

    // 2. Build adjacency for boundary vertices
    let mut adj = HashMap::new();
    for &(v1, v2) in &boundary_edges {
        adj.entry(v1).or_insert_with(Vec::new).push(v2);
        adj.entry(v2).or_insert_with(Vec::new).push(v1);
    }

    // 3. Find loops
    let mut added_count = 0;
    let mut visited_verts = std::collections::HashSet::new();

    let verts: Vec<u32> = adj.keys().cloned().collect();
    for &start_v in &verts {
        if visited_verts.contains(&start_v) {
            continue;
        }

        // Trace a loop
        let mut loop_verts = Vec::new();
        let mut curr = start_v;

        loop {
            loop_verts.push(curr);
            visited_verts.insert(curr);

            let possible_next = adj.get(&curr).unwrap();
            let mut next_v = None;
            for &n in possible_next {
                if n == start_v && loop_verts.len() >= 3 {
                    // Loop closed
                    break;
                }
                if !visited_verts.contains(&n) {
                    next_v = Some(n);
                    break;
                }
            }

            if let Some(nv) = next_v {
                curr = nv;
            } else {
                break;
            }
        }

        // Simple capping if loop has >= 3 vertices
        if loop_verts.len() >= 3 {
            let v0 = loop_verts[0];
            for i in 1..(loop_verts.len() - 1) {
                let v1 = loop_verts[i];
                let v2 = loop_verts[i + 1];
                mesh.add_triangle(v0, v1, v2);
                added_count += 1;
            }
        }
    }

    added_count
}

fn harmonize_orientations(mesh: &mut Mesh) -> usize {
    if mesh.triangles.is_empty() {
        return 0;
    }

    // 1. Build edge-to-triangle map
    // Edge is (min, max) -> Vec<(tri_index, directed_edge)>
    let mut edge_to_tris = HashMap::new();
    for (i, tri) in mesh.triangles.iter().enumerate() {
        let edges = [(tri.v1, tri.v2), (tri.v2, tri.v3), (tri.v3, tri.v1)];
        for e in edges {
            let key = if e.0 < e.1 { (e.0, e.1) } else { (e.1, e.0) };
            edge_to_tris
                .entry(key)
                .or_insert_with(Vec::new)
                .push((i, e));
        }
    }

    let mut flipped_count = 0;
    let mut visited = vec![false; mesh.triangles.len()];

    for start_idx in 0..mesh.triangles.len() {
        if visited[start_idx] {
            continue;
        }

        let mut queue = std::collections::VecDeque::new();
        queue.push_back(start_idx);
        visited[start_idx] = true;

        while let Some(curr_idx) = queue.pop_front() {
            let curr_tri = mesh.triangles[curr_idx];
            let curr_edges = [
                (curr_tri.v1, curr_tri.v2),
                (curr_tri.v2, curr_tri.v3),
                (curr_tri.v3, curr_tri.v1),
            ];

            for e in curr_edges {
                let key = if e.0 < e.1 { (e.0, e.1) } else { (e.1, e.0) };
                if let Some(neighbors) = edge_to_tris.get(&key) {
                    for &(neigh_idx, neigh_edge) in neighbors {
                        if visited[neigh_idx] {
                            continue;
                        }

                        // Check consistency:
                        // If current triangle has edge (v1, v2), neighbor should have (v2, v1)
                        // If they both have (v1, v2), they are inconsistent.
                        if e == neigh_edge {
                            // Inconsistent, flip neighbor
                            let nt = &mut mesh.triangles[neigh_idx];
                            std::mem::swap(&mut nt.v2, &mut nt.v3);
                            // Also flip its property indices if they exist
                            std::mem::swap(&mut nt.p2, &mut nt.p3);
                            flipped_count += 1;
                        }

                        visited[neigh_idx] = true;
                        queue.push_back(neigh_idx);
                    }
                }
            }
        }
    }

    flipped_count
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
