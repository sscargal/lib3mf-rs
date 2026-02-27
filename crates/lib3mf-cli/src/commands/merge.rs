//! Merge command — combines multiple 3MF files into a single output file.
//!
//! This module implements the core merge engine for the `3mf merge` command.
//! The merge pipeline:
//! 1. Expand glob patterns in inputs
//! 2. Load all input files with full attachments
//! 3. Check for secure content (signing/encryption) — error if found
//! 4. Compute ID offset for each file (max ID in merged so far + 1)
//! 5. Remap all ResourceId cross-references in each model
//! 6. Merge attachments with path deduplication
//! 7. Update texture/displacement paths after attachment remap
//! 8. Combine all resources into one merged Model
//! 9. Apply placement (plate-per-file or single-plate)
//! 10. Write merged model to output

use glob::glob;
use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path, opc};
use lib3mf_core::model::{
    BaseMaterialsGroup, ColorGroup, CompositeMaterials, Displacement2D, Geometry, Model,
    MultiProperties, Object, ResourceId, SliceStack, Texture2D, Texture2DGroup, VolumetricStack,
    stats::BoundingBox,
};
use lib3mf_core::parser::parse_model;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Cursor};
use std::path::{Path, PathBuf};

/// Packing algorithm for single-plate mode.
#[derive(Debug, Clone, Copy, clap::ValueEnum, Default)]
pub enum PackAlgorithm {
    /// Arrange objects in a grid (default)
    #[default]
    Grid,
}

/// Verbosity level for merge output.
#[derive(Debug, Clone, Copy)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

/// Entry point for the merge command.
pub fn run(
    inputs: Vec<PathBuf>,
    output: PathBuf,
    force: bool,
    single_plate: bool,
    pack: PackAlgorithm,
    verbosity: Verbosity,
) -> anyhow::Result<()> {
    // Step 1: Expand glob patterns
    let expanded = expand_inputs(inputs)?;
    let file_count = expanded.len();

    if matches!(verbosity, Verbosity::Verbose) {
        eprintln!("Merging {} files:", file_count);
        for p in &expanded {
            eprintln!("  {}", p.display());
        }
    }

    // Step 2: Resolve output path (auto-increment if exists and force=false)
    let out_path = resolve_output_path(&output, force)?;

    // Step 3: Load all models up front — fail fast on any error (no partial output)
    let mut loaded: Vec<Model> = Vec::with_capacity(file_count);
    for (i, path) in expanded.iter().enumerate() {
        if matches!(verbosity, Verbosity::Verbose) {
            eprintln!("  Loading [{}/{}] {}", i + 1, file_count, path.display());
        }
        let model = load_full(path)?;
        check_secure_content(&model)?;
        loaded.push(model);
    }

    // Step 4: Merge all models
    // First model becomes the base; subsequent models are remapped and transferred into it.
    let mut merged = loaded.remove(0);
    let mut total_objects = count_objects(&merged);
    let mut total_materials = count_materials(&merged);

    for (file_index, mut source) in loaded.into_iter().enumerate() {
        // file_index 0 = second input file (index 1 overall)
        let actual_file_index = file_index + 1;

        // Step 4a: Compute ID offset (max resource ID in merged so far + 1)
        let offset = max_resource_id(&merged) + 1;

        if matches!(verbosity, Verbosity::Verbose) {
            eprintln!(
                "  Merging file {} with ID offset {}",
                actual_file_index + 1,
                offset
            );
        }

        // Step 4b: Remap all IDs in source model
        remap_model(&mut source, offset);

        // Step 4c: Merge attachments with path deduplication
        // Take attachments out of source first so we can still borrow source mutably after.
        let src_attachments = std::mem::take(&mut source.attachments);
        let path_remap =
            merge_attachments(&mut merged.attachments, src_attachments, actual_file_index);

        // Step 4d: Update texture/displacement paths after attachment remap
        update_texture_paths(&mut source, &path_remap);

        // Count before transfer (resources will be moved)
        let src_objects = count_objects_resources(&source.resources);
        let src_materials = count_materials_resources(&source.resources);

        // Step 4e: Transfer all resources from source into merged
        transfer_resources(&mut merged, source.resources)?;

        // Step 4f: Transfer build items
        merged.build.items.extend(source.build.items);

        // Step 4g: Merge metadata, relationships, namespaces
        merge_metadata(&mut merged.metadata, &source.metadata);
        merge_relationships(
            &mut merged.existing_relationships,
            source.existing_relationships,
        );
        merge_extra_namespaces(&mut merged.extra_namespaces, &source.extra_namespaces);

        total_objects += src_objects;
        total_materials += src_materials;
    }

    // Step 5: Apply placement
    if single_plate {
        apply_single_plate_placement(&mut merged, pack, verbosity)?;
    } else {
        check_build_item_overlaps(&merged, verbosity);
    }

    // Step 6: Write output atomically (write to .tmp, rename to final)
    let tmp_path = out_path.with_extension(format!(
        "{}.tmp",
        out_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("3mf")
    ));
    {
        let tmp_file = File::create(&tmp_path)
            .map_err(|e| anyhow::anyhow!("Failed to create temp output {:?}: {}", tmp_path, e))?;
        let buf_writer = BufWriter::new(tmp_file);
        merged
            .write(buf_writer)
            .map_err(|e| anyhow::anyhow!("Failed to write merged 3MF: {}", e))?;
    }
    std::fs::rename(&tmp_path, &out_path).map_err(|e| {
        // Attempt cleanup
        let _ = std::fs::remove_file(&tmp_path);
        anyhow::anyhow!("Failed to finalize output file {:?}: {}", out_path, e)
    })?;

    // Step 7: Print summary
    if !matches!(verbosity, Verbosity::Quiet) {
        println!("Merged {} files -> {}", file_count, out_path.display());
        println!(
            "  Objects: {}  |  Materials: {}",
            total_objects, total_materials
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Resolve output path — auto-increment if exists and force=false
// ---------------------------------------------------------------------------

fn resolve_output_path(output: &Path, force: bool) -> anyhow::Result<PathBuf> {
    if force || !output.exists() {
        return Ok(output.to_path_buf());
    }

    // Auto-increment: try output.3mf.1, output.3mf.2, ... up to .999
    for n in 1u32..=999 {
        let candidate = PathBuf::from(format!("{}.{}", output.display(), n));
        if !candidate.exists() {
            if !matches!(std::env::var("RUST_TEST_QUIET").as_deref(), Ok("1")) {
                eprintln!(
                    "Warning: output {:?} exists, writing to {:?}",
                    output, candidate
                );
            }
            return Ok(candidate);
        }
    }

    anyhow::bail!(
        "Output file {:?} exists and no free auto-increment slot found (tried .1 through .999). Use --force to overwrite.",
        output
    )
}

// ---------------------------------------------------------------------------
// Transfer all resources from a source ResourceCollection into merged model
// ---------------------------------------------------------------------------

fn transfer_resources(
    merged: &mut Model,
    source: lib3mf_core::model::ResourceCollection,
) -> anyhow::Result<()> {
    for obj in source.iter_objects().cloned().collect::<Vec<_>>() {
        merged
            .resources
            .add_object(obj)
            .map_err(|e| anyhow::anyhow!("Failed to add object during merge: {}", e))?;
    }
    for mat in source.iter_base_materials().cloned().collect::<Vec<_>>() {
        merged
            .resources
            .add_base_materials(mat)
            .map_err(|e| anyhow::anyhow!("Failed to add base materials during merge: {}", e))?;
    }
    for col in source.iter_color_groups().cloned().collect::<Vec<_>>() {
        merged
            .resources
            .add_color_group(col)
            .map_err(|e| anyhow::anyhow!("Failed to add color group during merge: {}", e))?;
    }
    for tex in source.iter_texture_2d().cloned().collect::<Vec<_>>() {
        merged
            .resources
            .add_texture_2d(tex)
            .map_err(|e| anyhow::anyhow!("Failed to add texture 2D during merge: {}", e))?;
    }
    for grp in source.iter_textures().cloned().collect::<Vec<_>>() {
        merged
            .resources
            .add_texture_2d_group(grp)
            .map_err(|e| anyhow::anyhow!("Failed to add texture 2D group during merge: {}", e))?;
    }
    for comp in source
        .iter_composite_materials()
        .cloned()
        .collect::<Vec<_>>()
    {
        merged
            .resources
            .add_composite_materials(comp)
            .map_err(|e| {
                anyhow::anyhow!("Failed to add composite materials during merge: {}", e)
            })?;
    }
    for mp in source.iter_multi_properties().cloned().collect::<Vec<_>>() {
        merged
            .resources
            .add_multi_properties(mp)
            .map_err(|e| anyhow::anyhow!("Failed to add multi-properties during merge: {}", e))?;
    }
    for ss in source.iter_slice_stacks().cloned().collect::<Vec<_>>() {
        merged
            .resources
            .add_slice_stack(ss)
            .map_err(|e| anyhow::anyhow!("Failed to add slice stack during merge: {}", e))?;
    }
    for vs in source.iter_volumetric_stacks().cloned().collect::<Vec<_>>() {
        merged
            .resources
            .add_volumetric_stack(vs)
            .map_err(|e| anyhow::anyhow!("Failed to add volumetric stack during merge: {}", e))?;
    }
    for d in source.iter_displacement_2d().cloned().collect::<Vec<_>>() {
        merged
            .resources
            .add_displacement_2d(d)
            .map_err(|e| anyhow::anyhow!("Failed to add displacement 2D during merge: {}", e))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Count helpers for summary output
// ---------------------------------------------------------------------------

fn count_objects(model: &Model) -> usize {
    model.resources.iter_objects().count()
}

fn count_materials(model: &Model) -> usize {
    model.resources.iter_base_materials().count()
        + model.resources.iter_color_groups().count()
        + model.resources.iter_texture_2d().count()
}

fn count_objects_resources(resources: &lib3mf_core::model::ResourceCollection) -> usize {
    resources.iter_objects().count()
}

fn count_materials_resources(resources: &lib3mf_core::model::ResourceCollection) -> usize {
    resources.iter_base_materials().count()
        + resources.iter_color_groups().count()
        + resources.iter_texture_2d().count()
}

// ---------------------------------------------------------------------------
// Placement: plate-per-file mode (check bounding box overlaps)
// ---------------------------------------------------------------------------

/// In plate-per-file mode, preserve all transforms as-is and warn about overlaps.
pub(crate) fn check_build_item_overlaps(model: &Model, verbosity: Verbosity) {
    if matches!(verbosity, Verbosity::Quiet) {
        return;
    }

    // Collect (build_item_index, world_space AABB) for each item with a mesh
    let world_aabbs: Vec<(usize, BoundingBox)> = model
        .build
        .items
        .iter()
        .enumerate()
        .filter_map(|(i, item)| {
            let obj = model
                .resources
                .iter_objects()
                .find(|o| o.id == item.object_id)?;
            let aabb = mesh_aabb_for_object(obj, model)?;
            let world_aabb = aabb.transform(item.transform);
            Some((i, world_aabb))
        })
        .collect();

    // O(n^2) pairwise overlap check — fine for typical merge counts
    let n = world_aabbs.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let (idx_i, ref aabb_i) = world_aabbs[i];
            let (idx_j, ref aabb_j) = world_aabbs[j];
            if aabbs_overlap(aabb_i, aabb_j) {
                eprintln!(
                    "Warning: build items {} and {} have overlapping bounding boxes",
                    idx_i, idx_j
                );
            }
        }
    }
}

/// Get the mesh AABB for an object (recursively resolves component objects).
/// Returns None for non-mesh geometry or empty meshes.
fn mesh_aabb_for_object(obj: &Object, model: &Model) -> Option<BoundingBox> {
    match &obj.geometry {
        Geometry::Mesh(mesh) => mesh.compute_aabb(),
        Geometry::DisplacementMesh(dm) => {
            // DisplacementMesh uses the same vertex layout as Mesh — compute AABB from vertices
            if dm.vertices.is_empty() {
                return None;
            }
            let mut min = [f32::INFINITY; 3];
            let mut max = [f32::NEG_INFINITY; 3];
            for v in &dm.vertices {
                min[0] = min[0].min(v.x);
                min[1] = min[1].min(v.y);
                min[2] = min[2].min(v.z);
                max[0] = max[0].max(v.x);
                max[1] = max[1].max(v.y);
                max[2] = max[2].max(v.z);
            }
            Some(BoundingBox { min, max })
        }
        Geometry::Components(comps) => {
            // For component objects, compute union of child AABBs
            let mut combined: Option<BoundingBox> = None;
            for comp in &comps.components {
                if let Some(child_obj) = model
                    .resources
                    .iter_objects()
                    .find(|o| o.id == comp.object_id)
                    && let Some(child_aabb) = mesh_aabb_for_object(child_obj, model)
                {
                    let transformed = child_aabb.transform(comp.transform);
                    combined = Some(match combined {
                        None => transformed,
                        Some(existing) => union_aabb(existing, transformed),
                    });
                }
            }
            combined
        }
        _ => None,
    }
}

fn union_aabb(a: BoundingBox, b: BoundingBox) -> BoundingBox {
    BoundingBox {
        min: [
            a.min[0].min(b.min[0]),
            a.min[1].min(b.min[1]),
            a.min[2].min(b.min[2]),
        ],
        max: [
            a.max[0].max(b.max[0]),
            a.max[1].max(b.max[1]),
            a.max[2].max(b.max[2]),
        ],
    }
}

fn aabbs_overlap(a: &BoundingBox, b: &BoundingBox) -> bool {
    a.min[0] < b.max[0]
        && a.max[0] > b.min[0]
        && a.min[1] < b.max[1]
        && a.max[1] > b.min[1]
        && a.min[2] < b.max[2]
        && a.max[2] > b.min[2]
}

// ---------------------------------------------------------------------------
// Placement: single-plate mode (grid layout)
// ---------------------------------------------------------------------------

/// In single-plate mode, replace ALL build item transforms with grid-computed ones.
pub(crate) fn apply_single_plate_placement(
    model: &mut Model,
    _pack: PackAlgorithm,
    verbosity: Verbosity,
) -> anyhow::Result<()> {
    const SPACING_MM: f32 = 10.0;

    // Collect (item_index, size_x, size_y) for grid layout
    // Items without computable AABBs get placed at origin
    struct ItemInfo {
        size_x: f32,
        size_y: f32,
    }

    let item_infos: Vec<ItemInfo> = model
        .build
        .items
        .iter()
        .map(|item| {
            let aabb = model
                .resources
                .iter_objects()
                .find(|o| o.id == item.object_id)
                .and_then(|obj| mesh_aabb_for_object(obj, model));
            match aabb {
                Some(bb) => ItemInfo {
                    size_x: (bb.max[0] - bb.min[0]).max(0.0),
                    size_y: (bb.max[1] - bb.min[1]).max(0.0),
                },
                None => ItemInfo {
                    size_x: 0.0,
                    size_y: 0.0,
                },
            }
        })
        .collect();

    let n = item_infos.len();
    if n == 0 {
        return Ok(());
    }

    // Grid: square layout — number of columns = ceil(sqrt(n))
    let cols = (n as f64).sqrt().ceil() as usize;

    // Track current x/y cursor and row heights
    let mut row_heights: Vec<f32> = Vec::new();
    let mut col_widths: Vec<f32> = Vec::new();

    // Pre-compute per-cell widths and heights for the grid
    for (idx, info) in item_infos.iter().enumerate() {
        let col = idx % cols;
        let row = idx / cols;
        if col_widths.len() <= col {
            col_widths.resize(col + 1, 0.0_f32);
        }
        if row_heights.len() <= row {
            row_heights.resize(row + 1, 0.0_f32);
        }
        col_widths[col] = col_widths[col].max(info.size_x);
        row_heights[row] = row_heights[row].max(info.size_y);
    }

    // Compute cumulative x/y positions
    let mut x_offsets = vec![0.0_f32; cols + 1];
    let mut y_offsets = vec![0.0_f32; row_heights.len() + 1];
    for c in 0..cols {
        x_offsets[c + 1] = x_offsets[c] + col_widths[c] + SPACING_MM;
    }
    for r in 0..row_heights.len() {
        y_offsets[r + 1] = y_offsets[r] + row_heights[r] + SPACING_MM;
    }

    // Apply transforms: place each item at its grid cell's (x_offset, y_offset, 0)
    for (idx, item) in model.build.items.iter_mut().enumerate() {
        let col = idx % cols;
        let row = idx / cols;
        let tx = x_offsets[col];
        let ty = y_offsets[row];
        item.transform = glam::Mat4::from_translation(glam::Vec3::new(tx, ty, 0.0));
    }

    if matches!(verbosity, Verbosity::Verbose) {
        eprintln!(
            "Single-plate grid layout: {} items in {}x{} grid",
            n,
            cols,
            row_heights.len()
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helper: load a 3MF file with full attachments
// ---------------------------------------------------------------------------

pub(crate) fn load_full(path: &Path) -> anyhow::Result<Model> {
    let file = File::open(path).map_err(|e| anyhow::anyhow!("Failed to open {:?}: {}", path, e))?;
    let mut archiver = ZipArchiver::new(file)
        .map_err(|e| anyhow::anyhow!("Failed to open zip archive {:?}: {}", path, e))?;
    let model_path = find_model_path(&mut archiver)
        .map_err(|e| anyhow::anyhow!("Failed to find model path in {:?}: {}", path, e))?;
    let model_data = archiver
        .read_entry(&model_path)
        .map_err(|e| anyhow::anyhow!("Failed to read model XML from {:?}: {}", path, e))?;
    let mut model = parse_model(Cursor::new(model_data))
        .map_err(|e| anyhow::anyhow!("Failed to parse model XML from {:?}: {}", path, e))?;

    // Load all entries — preserve attachments and relationships, like the copy command.
    let all_files = archiver
        .list_entries()
        .map_err(|e| anyhow::anyhow!("Failed to list archive entries in {:?}: {}", path, e))?;

    for entry_path in all_files {
        // Skip files that PackageWriter regenerates
        if entry_path == model_path
            || entry_path == "_rels/.rels"
            || entry_path == "[Content_Types].xml"
        {
            continue;
        }

        // Load .rels files to preserve relationships
        if entry_path.ends_with(".rels") {
            if let Ok(data) = archiver.read_entry(&entry_path)
                && let Ok(rels) = opc::parse_relationships(&data)
            {
                model.existing_relationships.insert(entry_path, rels);
            }
            continue;
        }

        // Load other data as attachments
        if let Ok(data) = archiver.read_entry(&entry_path) {
            model.attachments.insert(entry_path, data);
        }
    }

    Ok(model)
}

// ---------------------------------------------------------------------------
// Internal helper: check for secure content
// ---------------------------------------------------------------------------

pub(crate) fn check_secure_content(model: &Model) -> anyhow::Result<()> {
    if model.resources.key_store.is_some() {
        anyhow::bail!(
            "Cannot merge signed/encrypted 3MF files. Strip signatures first using the decrypt command."
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helper: compute maximum ResourceId across all resource types
// ---------------------------------------------------------------------------

pub(crate) fn max_resource_id(model: &Model) -> u32 {
    let r = &model.resources;
    let mut max = 0u32;
    for obj in r.iter_objects() {
        max = max.max(obj.id.0);
    }
    for mat in r.iter_base_materials() {
        max = max.max(mat.id.0);
    }
    for col in r.iter_color_groups() {
        max = max.max(col.id.0);
    }
    for tex in r.iter_texture_2d() {
        max = max.max(tex.id.0);
    }
    for grp in r.iter_textures() {
        max = max.max(grp.id.0);
    }
    for comp in r.iter_composite_materials() {
        max = max.max(comp.id.0);
    }
    for mp in r.iter_multi_properties() {
        max = max.max(mp.id.0);
    }
    for ss in r.iter_slice_stacks() {
        max = max.max(ss.id.0);
    }
    for vs in r.iter_volumetric_stacks() {
        max = max.max(vs.id.0);
    }
    for d in r.iter_displacement_2d() {
        max = max.max(d.id.0);
    }
    max
}

// ---------------------------------------------------------------------------
// ID remap helpers
// ---------------------------------------------------------------------------

#[inline]
fn remap_id(id: &mut ResourceId, offset: u32) {
    id.0 += offset;
}

#[inline]
fn remap_opt_id(id: &mut Option<ResourceId>, offset: u32) {
    if let Some(inner) = id {
        inner.0 += offset;
    }
}

#[inline]
fn remap_opt_u32_pid(pid: &mut Option<u32>, offset: u32) {
    if let Some(p) = pid {
        *p += offset;
    }
}

// ---------------------------------------------------------------------------
// Internal helper: remap all ResourceId cross-references in a model
// ---------------------------------------------------------------------------
//
// After calling this function, every resource in the model has IDs shifted
// by `offset`, and every cross-reference is updated to match. This allows
// models with overlapping ID namespaces to be merged without collisions.

pub(crate) fn remap_model(model: &mut Model, offset: u32) {
    if offset == 0 {
        return;
    }

    // --- Collect all resources out of the collection so we can mutate them ---
    // Since ResourceCollection uses private HashMap fields, we collect resources
    // via iterators (cloning), remap their IDs, and then rebuild the collection
    // using add_* methods.

    let old_resources = std::mem::take(&mut model.resources);

    // Collect and remap objects
    let mut objects: Vec<Object> = old_resources.iter_objects().cloned().collect();
    for obj in &mut objects {
        remap_id(&mut obj.id, offset);
        remap_opt_id(&mut obj.pid, offset);
        match &mut obj.geometry {
            Geometry::Mesh(mesh) => {
                for tri in &mut mesh.triangles {
                    remap_opt_u32_pid(&mut tri.pid, offset);
                }
            }
            Geometry::Components(comps) => {
                for comp in &mut comps.components {
                    remap_id(&mut comp.object_id, offset);
                }
            }
            Geometry::BooleanShape(shape) => {
                remap_id(&mut shape.base_object_id, offset);
                for op in &mut shape.operations {
                    remap_id(&mut op.object_id, offset);
                }
            }
            Geometry::SliceStack(id) => {
                remap_id(&mut *id, offset);
            }
            Geometry::VolumetricStack(id) => {
                remap_id(&mut *id, offset);
            }
            Geometry::DisplacementMesh(dm) => {
                for tri in &mut dm.triangles {
                    remap_opt_u32_pid(&mut tri.pid, offset);
                }
            }
        }
    }

    // Collect and remap base material groups
    let mut base_materials: Vec<BaseMaterialsGroup> =
        old_resources.iter_base_materials().cloned().collect();
    for mat in &mut base_materials {
        remap_id(&mut mat.id, offset);
    }

    // Collect and remap color groups
    let mut color_groups: Vec<ColorGroup> = old_resources.iter_color_groups().cloned().collect();
    for col in &mut color_groups {
        remap_id(&mut col.id, offset);
    }

    // Collect and remap Texture2D resources
    let mut texture_2d: Vec<Texture2D> = old_resources.iter_texture_2d().cloned().collect();
    for tex in &mut texture_2d {
        remap_id(&mut tex.id, offset);
    }

    // Collect and remap Texture2DGroup resources
    let mut texture_2d_groups: Vec<Texture2DGroup> =
        old_resources.iter_textures().cloned().collect();
    for grp in &mut texture_2d_groups {
        remap_id(&mut grp.id, offset);
        remap_id(&mut grp.texture_id, offset);
    }

    // Collect and remap composite materials
    let mut composite_materials: Vec<CompositeMaterials> =
        old_resources.iter_composite_materials().cloned().collect();
    for comp in &mut composite_materials {
        remap_id(&mut comp.id, offset);
        remap_id(&mut comp.base_material_id, offset);
    }

    // Collect and remap multi-properties
    let mut multi_properties: Vec<MultiProperties> =
        old_resources.iter_multi_properties().cloned().collect();
    for mp in &mut multi_properties {
        remap_id(&mut mp.id, offset);
        for pid in &mut mp.pids {
            remap_id(pid, offset);
        }
    }

    // Collect and remap slice stacks
    let mut slice_stacks: Vec<SliceStack> = old_resources.iter_slice_stacks().cloned().collect();
    for ss in &mut slice_stacks {
        remap_id(&mut ss.id, offset);
    }

    // Collect and remap volumetric stacks
    let mut volumetric_stacks: Vec<VolumetricStack> =
        old_resources.iter_volumetric_stacks().cloned().collect();
    for vs in &mut volumetric_stacks {
        remap_id(&mut vs.id, offset);
    }

    // Collect and remap Displacement2D resources
    let mut displacement_2d: Vec<Displacement2D> =
        old_resources.iter_displacement_2d().cloned().collect();
    for d in &mut displacement_2d {
        remap_id(&mut d.id, offset);
    }

    // Preserve key_store as-is (secure content check already done before remap)
    let key_store = old_resources.key_store;

    // Rebuild ResourceCollection with remapped resources
    let mut new_resources = lib3mf_core::model::ResourceCollection::new();
    for obj in objects {
        new_resources
            .add_object(obj)
            .expect("Remapped IDs should not collide within the same model");
    }
    for mat in base_materials {
        new_resources
            .add_base_materials(mat)
            .expect("Remapped IDs should not collide within the same model");
    }
    for col in color_groups {
        new_resources
            .add_color_group(col)
            .expect("Remapped IDs should not collide within the same model");
    }
    for tex in texture_2d {
        new_resources
            .add_texture_2d(tex)
            .expect("Remapped IDs should not collide within the same model");
    }
    for grp in texture_2d_groups {
        new_resources
            .add_texture_2d_group(grp)
            .expect("Remapped IDs should not collide within the same model");
    }
    for comp in composite_materials {
        new_resources
            .add_composite_materials(comp)
            .expect("Remapped IDs should not collide within the same model");
    }
    for mp in multi_properties {
        new_resources
            .add_multi_properties(mp)
            .expect("Remapped IDs should not collide within the same model");
    }
    for ss in slice_stacks {
        new_resources
            .add_slice_stack(ss)
            .expect("Remapped IDs should not collide within the same model");
    }
    for vs in volumetric_stacks {
        new_resources
            .add_volumetric_stack(vs)
            .expect("Remapped IDs should not collide within the same model");
    }
    for d in displacement_2d {
        new_resources
            .add_displacement_2d(d)
            .expect("Remapped IDs should not collide within the same model");
    }
    if let Some(ks) = key_store {
        new_resources.set_key_store(ks);
    }

    model.resources = new_resources;

    // Remap build items
    for item in &mut model.build.items {
        remap_id(&mut item.object_id, offset);
    }
}

// ---------------------------------------------------------------------------
// Internal helper: merge attachments with path deduplication
// ---------------------------------------------------------------------------
//
// For each attachment from the source model:
// - If path doesn't exist in merged: insert directly
// - If path exists and content is byte-identical: dedup (reuse existing path)
// - If path exists and content differs: rename to "{path}.{file_index}"
//
// Returns a path remap map: old_path -> new_path. Caller must update
// Texture2D.path and Displacement2D.path in the remapped model.

pub(crate) fn merge_attachments(
    merged: &mut HashMap<String, Vec<u8>>,
    source: HashMap<String, Vec<u8>>,
    file_index: usize,
) -> HashMap<String, String> {
    let mut path_remap: HashMap<String, String> = HashMap::new();

    for (path, data) in source {
        if let Some(existing) = merged.get(&path) {
            if *existing == data {
                // Same content — deduplicate, keep existing path
                path_remap.insert(path.clone(), path);
            } else {
                // Different content — rename with file index suffix
                let new_path = format!("{}.{}", path, file_index);
                merged.insert(new_path.clone(), data);
                path_remap.insert(path, new_path);
            }
        } else {
            merged.insert(path.clone(), data);
            path_remap.insert(path.clone(), path);
        }
    }

    path_remap
}

// ---------------------------------------------------------------------------
// Internal helper: merge metadata with semicolon concatenation
// ---------------------------------------------------------------------------

pub(crate) fn merge_metadata(
    merged: &mut HashMap<String, String>,
    source: &HashMap<String, String>,
) {
    for (key, value) in source {
        merged
            .entry(key.clone())
            .and_modify(|existing| {
                existing.push_str("; ");
                existing.push_str(value);
            })
            .or_insert_with(|| value.clone());
    }
}

// ---------------------------------------------------------------------------
// Internal helper: merge OPC relationships
// ---------------------------------------------------------------------------

pub(crate) fn merge_relationships(
    merged: &mut HashMap<String, Vec<lib3mf_core::archive::opc::Relationship>>,
    source: HashMap<String, Vec<lib3mf_core::archive::opc::Relationship>>,
) {
    for (path, rels) in source {
        merged.entry(path).or_insert(rels);
    }
}

// ---------------------------------------------------------------------------
// Internal helper: merge extra XML namespaces
// ---------------------------------------------------------------------------

pub(crate) fn merge_extra_namespaces(
    merged: &mut HashMap<String, String>,
    source: &HashMap<String, String>,
) {
    for (prefix, uri) in source {
        if let Some(existing_uri) = merged.get(prefix) {
            if existing_uri != uri {
                // Prefix collision with different URI — log a warning and skip
                eprintln!(
                    "Warning: XML namespace prefix '{prefix}' has conflicting URIs ('{existing_uri}' vs '{uri}'). Keeping first."
                );
            }
            // Same prefix+URI: silently skip duplicate
        } else {
            merged.insert(prefix.clone(), uri.clone());
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helper: update texture and displacement paths after attachment remap
// ---------------------------------------------------------------------------

pub(crate) fn update_texture_paths(model: &mut Model, path_remap: &HashMap<String, String>) {
    if path_remap.is_empty() {
        return;
    }

    // Collect updated Texture2D resources
    let old_resources = std::mem::take(&mut model.resources);

    let mut texture_2d: Vec<Texture2D> = old_resources.iter_texture_2d().cloned().collect();
    for tex in &mut texture_2d {
        if let Some(new_path) = path_remap.get(&tex.path) {
            tex.path = new_path.clone();
        }
    }

    let mut displacement_2d: Vec<Displacement2D> =
        old_resources.iter_displacement_2d().cloned().collect();
    for d in &mut displacement_2d {
        if let Some(new_path) = path_remap.get(&d.path) {
            d.path = new_path.clone();
        }
    }

    // Only rebuild if something changed — otherwise just put back
    let textures_changed = texture_2d
        .iter()
        .zip(old_resources.iter_texture_2d())
        .any(|(new, old)| new.path != old.path);
    let displacement_changed = displacement_2d
        .iter()
        .zip(old_resources.iter_displacement_2d())
        .any(|(new, old)| new.path != old.path);

    if !textures_changed && !displacement_changed {
        model.resources = old_resources;
        return;
    }

    // Rebuild ResourceCollection with updated paths
    let key_store = old_resources.key_store.clone();
    let objects: Vec<Object> = old_resources.iter_objects().cloned().collect();
    let base_materials: Vec<_> = old_resources.iter_base_materials().cloned().collect();
    let color_groups: Vec<_> = old_resources.iter_color_groups().cloned().collect();
    let texture_2d_groups: Vec<Texture2DGroup> = old_resources.iter_textures().cloned().collect();
    let composite_materials: Vec<_> = old_resources.iter_composite_materials().cloned().collect();
    let multi_properties: Vec<_> = old_resources.iter_multi_properties().cloned().collect();
    let slice_stacks: Vec<_> = old_resources.iter_slice_stacks().cloned().collect();
    let volumetric_stacks: Vec<_> = old_resources.iter_volumetric_stacks().cloned().collect();

    let mut new_resources = lib3mf_core::model::ResourceCollection::new();
    for obj in objects {
        new_resources.add_object(obj).expect("no ID collision");
    }
    for mat in base_materials {
        new_resources
            .add_base_materials(mat)
            .expect("no ID collision");
    }
    for col in color_groups {
        new_resources.add_color_group(col).expect("no ID collision");
    }
    for tex in texture_2d {
        new_resources.add_texture_2d(tex).expect("no ID collision");
    }
    for grp in texture_2d_groups {
        new_resources
            .add_texture_2d_group(grp)
            .expect("no ID collision");
    }
    for comp in composite_materials {
        new_resources
            .add_composite_materials(comp)
            .expect("no ID collision");
    }
    for mp in multi_properties {
        new_resources
            .add_multi_properties(mp)
            .expect("no ID collision");
    }
    for ss in slice_stacks {
        new_resources.add_slice_stack(ss).expect("no ID collision");
    }
    for vs in volumetric_stacks {
        new_resources
            .add_volumetric_stack(vs)
            .expect("no ID collision");
    }
    for d in displacement_2d {
        new_resources
            .add_displacement_2d(d)
            .expect("no ID collision");
    }
    if let Some(ks) = key_store {
        new_resources.set_key_store(ks);
    }

    model.resources = new_resources;
}

// ---------------------------------------------------------------------------
// Internal helper: expand glob patterns in input list
// ---------------------------------------------------------------------------

pub(crate) fn expand_inputs(raw_inputs: Vec<PathBuf>) -> anyhow::Result<Vec<PathBuf>> {
    let mut expanded = Vec::new();
    for input in raw_inputs {
        let pattern = input.to_string_lossy();
        if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
            let matches: Vec<PathBuf> = glob(&pattern)
                .map_err(|e| anyhow::anyhow!("Invalid glob pattern {:?}: {}", input, e))?
                .filter_map(|r| r.ok())
                .filter(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.to_lowercase())
                        == Some("3mf".to_string())
                })
                .collect();
            if matches.is_empty() {
                anyhow::bail!("Glob pattern {:?} matched no .3mf files", input);
            }
            expanded.extend(matches);
        } else {
            expanded.push(input);
        }
    }
    if expanded.len() < 2 {
        anyhow::bail!(
            "Merge requires at least 2 input files (got {})",
            expanded.len()
        );
    }
    Ok(expanded)
}
