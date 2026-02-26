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
//! 9. Write merged model to output
//!
//! Note: The `run()` function is a stub implemented in Plan 02. The helper
//! functions in this module are the complete merge engine implementation.
// Allow dead_code for helper functions until run() is wired up in Plan 02.
#![allow(dead_code)]

use glob::glob;
use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path, opc};
use lib3mf_core::model::{
    BaseMaterialsGroup, ColorGroup, CompositeMaterials, Displacement2D, Geometry, Model,
    MultiProperties, Object, ResourceId, SliceStack, Texture2D, Texture2DGroup, VolumetricStack,
};
use lib3mf_core::parser::parse_model;
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
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

/// Entry point for the merge command. Implemented fully in Plan 02.
#[allow(unused_variables)]
pub fn run(
    inputs: Vec<PathBuf>,
    output: PathBuf,
    force: bool,
    single_plate: bool,
    pack: PackAlgorithm,
    verbosity: Verbosity,
) -> anyhow::Result<()> {
    todo!("Implemented in Plan 02")
}

// ---------------------------------------------------------------------------
// Internal helper: load a 3MF file with full attachments
// ---------------------------------------------------------------------------

pub(crate) fn load_full(path: &Path) -> anyhow::Result<Model> {
    let file = File::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to open {:?}: {}", path, e))?;
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
        new_resources.add_base_materials(mat).expect("no ID collision");
    }
    for col in color_groups {
        new_resources.add_color_group(col).expect("no ID collision");
    }
    for tex in texture_2d {
        new_resources.add_texture_2d(tex).expect("no ID collision");
    }
    for grp in texture_2d_groups {
        new_resources.add_texture_2d_group(grp).expect("no ID collision");
    }
    for comp in composite_materials {
        new_resources.add_composite_materials(comp).expect("no ID collision");
    }
    for mp in multi_properties {
        new_resources.add_multi_properties(mp).expect("no ID collision");
    }
    for ss in slice_stacks {
        new_resources.add_slice_stack(ss).expect("no ID collision");
    }
    for vs in volumetric_stacks {
        new_resources.add_volumetric_stack(vs).expect("no ID collision");
    }
    for d in displacement_2d {
        new_resources.add_displacement_2d(d).expect("no ID collision");
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
                    p.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase())
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
