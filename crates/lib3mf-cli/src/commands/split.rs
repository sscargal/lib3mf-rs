//! Split command — extracts individual objects from a 3MF file into separate output files.
//!
//! This module implements the core split engine for the `3mf split` command.
//! The split pipeline:
//! 1. Load the source 3MF file with full attachments
//! 2. Check for secure content (signing/encryption) — error if found
//! 3. Collect all split targets (by build item or by object resource)
//! 4. Apply --select filter to cherry-pick specific items
//! 5. Phase 1 — Trace dependencies for ALL items before writing any files
//!    (dry-run output or validate completeness)
//! 6. Phase 2 — Write each split model to a separate output file
//! 7. Print summary of written files

use crate::commands::merge::{Verbosity, check_secure_content, load_full};
use lib3mf_core::model::{
    BaseMaterialsGroup, Build, BuildItem, ColorGroup, CompositeMaterials, Displacement2D, Geometry,
    Model, MultiProperties, Object, ResourceCollection, ResourceId, SliceStack, Texture2D,
    Texture2DGroup, VolumetricStack,
};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Determines how the source 3MF is partitioned into output files.
#[derive(Debug, Clone, Copy, clap::ValueEnum, Default)]
pub enum SplitMode {
    /// One output file per build item (default).
    #[default]
    ByItem,
    /// One output file per unique object resource.
    ByObject,
}

// ---------------------------------------------------------------------------
// Internal types
// ---------------------------------------------------------------------------

/// A candidate item to extract into its own output file.
struct SplitTarget {
    /// Root object ID to trace dependencies from.
    root_object_id: ResourceId,
    /// Build item (if available) for transform info.
    build_item: Option<BuildItem>,
    /// Display name for output naming.
    name: String,
    /// Index in the original list (for --select).
    index: usize,
}

/// Pre-computed data for one output file, ready for writing.
struct PreparedSplit {
    target: SplitTarget,
    needed_ids: HashSet<ResourceId>,
    needed_attachments: HashSet<String>,
    id_remap: HashMap<ResourceId, ResourceId>,
    output_path: PathBuf,
    object_count: usize,
    material_count: usize,
    texture_count: usize,
}

// ---------------------------------------------------------------------------
// Dependency Collector
// ---------------------------------------------------------------------------

/// Walks the transitive dependency graph starting from a root object to collect
/// the minimal set of resource IDs and attachment paths needed in the output file.
struct DependencyCollector<'a> {
    resources: &'a ResourceCollection,
    needed_ids: HashSet<ResourceId>,
    needed_attachment_paths: HashSet<String>,
}

impl<'a> DependencyCollector<'a> {
    fn new(resources: &'a ResourceCollection) -> Self {
        Self {
            resources,
            needed_ids: HashSet::new(),
            needed_attachment_paths: HashSet::new(),
        }
    }

    /// Walk all references reachable from an object, collecting resource IDs and attachment paths.
    /// Uses `HashSet::insert` return value for cycle detection (returns false if already visited).
    fn collect_object(&mut self, id: ResourceId) {
        if !self.needed_ids.insert(id) {
            return; // Already visited — prevents infinite loops in component graphs
        }

        let Some(obj) = self.resources.get_object(id) else {
            return;
        };

        // Object-level material reference
        if let Some(pid) = obj.pid {
            self.collect_property(pid);
        }

        // Object thumbnail attachment
        if let Some(ref thumb) = obj.thumbnail {
            self.needed_attachment_paths.insert(thumb.clone());
        }

        // Geometry-specific references
        match &obj.geometry {
            Geometry::Mesh(mesh) => {
                for tri in &mesh.triangles {
                    if let Some(pid) = tri.pid {
                        self.collect_property(ResourceId(pid));
                    }
                }
            }
            Geometry::Components(comps) => {
                for comp in &comps.components {
                    self.collect_object(comp.object_id); // recurse into child objects
                }
            }
            Geometry::BooleanShape(shape) => {
                self.collect_object(shape.base_object_id);
                for op in &shape.operations {
                    self.collect_object(op.object_id);
                }
            }
            Geometry::SliceStack(stack_id) => {
                self.needed_ids.insert(*stack_id);
            }
            Geometry::VolumetricStack(stack_id) => {
                self.needed_ids.insert(*stack_id);
            }
            Geometry::DisplacementMesh(dm) => {
                for tri in &dm.triangles {
                    if let Some(pid) = tri.pid {
                        self.collect_property(ResourceId(pid));
                    }
                }
            }
        }
    }

    /// Collect a property group and its transitive dependencies.
    fn collect_property(&mut self, pid: ResourceId) {
        if !self.needed_ids.insert(pid) {
            return; // Already visited
        }

        // Texture2DGroup -> Texture2D -> attachment
        if let Some(grp) = self.resources.get_texture_2d_group(pid) {
            let tex_id = grp.texture_id;
            self.needed_ids.insert(tex_id);
            // Look up Texture2D to get the attachment path
            if let Some(tex) = self.resources.iter_texture_2d().find(|t| t.id == tex_id) {
                self.needed_attachment_paths.insert(tex.path.clone());
            }
        }

        // CompositeMaterials -> BaseMaterialsGroup
        if let Some(comp) = self.resources.get_composite_materials(pid) {
            let base_id = comp.base_material_id;
            self.collect_property(base_id);
        }

        // MultiProperties -> multiple property groups (Vec<ResourceId>)
        if let Some(mp) = self.resources.get_multi_properties(pid) {
            let sub_pids: Vec<ResourceId> = mp.pids.clone();
            for sub_pid in sub_pids {
                self.collect_property(sub_pid);
            }
        }

        // Displacement2D -> attachment path
        if let Some(disp) = self.resources.get_displacement_2d(pid) {
            self.needed_attachment_paths.insert(disp.path.clone());
        }

        // BaseMaterialsGroup and ColorGroup: no further dependencies (leaves)
    }
}

// ---------------------------------------------------------------------------
// Compact ID remap builder
// ---------------------------------------------------------------------------

/// Assigns sequential IDs starting from 1 to all needed resource IDs.
/// Returns a map from old ID -> new ID.
fn build_compact_remap(needed_ids: &HashSet<ResourceId>) -> HashMap<ResourceId, ResourceId> {
    let mut sorted: Vec<u32> = needed_ids.iter().map(|id| id.0).collect();
    sorted.sort_unstable();
    sorted
        .iter()
        .enumerate()
        .map(|(new_idx, &old_id)| (ResourceId(old_id), ResourceId((new_idx + 1) as u32)))
        .collect()
}

// ---------------------------------------------------------------------------
// Remap helpers (lookup-based, unlike merge.rs which uses offset arithmetic)
// ---------------------------------------------------------------------------

#[inline]
fn remap_id(id: &mut ResourceId, remap: &HashMap<ResourceId, ResourceId>) {
    if let Some(&new_id) = remap.get(id) {
        *id = new_id;
    }
}

#[inline]
fn remap_opt_id(id: &mut Option<ResourceId>, remap: &HashMap<ResourceId, ResourceId>) {
    if let Some(inner) = id
        && let Some(&new_id) = remap.get(inner)
    {
        *inner = new_id;
    }
}

#[inline]
fn remap_opt_u32_pid(pid: &mut Option<u32>, remap: &HashMap<ResourceId, ResourceId>) {
    if let Some(p) = pid
        && let Some(&new_id) = remap.get(&ResourceId(*p))
    {
        *p = new_id.0;
    }
}

// ---------------------------------------------------------------------------
// Build split model
// ---------------------------------------------------------------------------

/// Construct a new Model containing only the resources for one extracted item.
fn build_split_model(
    source: &Model,
    target: &SplitTarget,
    preserve_transforms: bool,
    id_remap: &HashMap<ResourceId, ResourceId>,
    needed_ids: &HashSet<ResourceId>,
    needed_attachments: &HashSet<String>,
) -> anyhow::Result<Model> {
    // Copy model-level fields; metadata is annotated with source provenance (Pattern 8)
    let mut metadata = source.metadata.clone();
    metadata.insert(
        "Source".to_string(),
        source
            .metadata
            .get("Source")
            .cloned()
            .unwrap_or_default(),
    );
    metadata.insert("SourceObject".to_string(), target.name.clone());

    let mut out = Model {
        unit: source.unit,
        language: source.language.clone(),
        metadata,
        resources: ResourceCollection::new(),
        build: Build::default(),
        attachments: std::collections::HashMap::new(),
        existing_relationships: std::collections::HashMap::new(),
        extra_namespaces: source.extra_namespaces.clone(),
    };

    // --- Add only needed objects, with remapped IDs ---
    let objects: Vec<Object> = source
        .resources
        .iter_objects()
        .filter(|obj| needed_ids.contains(&obj.id))
        .cloned()
        .collect();
    for mut obj in objects {
        remap_id(&mut obj.id, id_remap);
        remap_opt_id(&mut obj.pid, id_remap);
        match &mut obj.geometry {
            Geometry::Mesh(mesh) => {
                for tri in &mut mesh.triangles {
                    remap_opt_u32_pid(&mut tri.pid, id_remap);
                }
            }
            Geometry::Components(comps) => {
                for comp in &mut comps.components {
                    remap_id(&mut comp.object_id, id_remap);
                }
            }
            Geometry::BooleanShape(shape) => {
                remap_id(&mut shape.base_object_id, id_remap);
                for op in &mut shape.operations {
                    remap_id(&mut op.object_id, id_remap);
                }
            }
            Geometry::SliceStack(id) => {
                remap_id(&mut *id, id_remap);
            }
            Geometry::VolumetricStack(id) => {
                remap_id(&mut *id, id_remap);
            }
            Geometry::DisplacementMesh(dm) => {
                for tri in &mut dm.triangles {
                    remap_opt_u32_pid(&mut tri.pid, id_remap);
                }
            }
        }
        out.resources
            .add_object(obj)
            .map_err(|e| anyhow::anyhow!("Failed to add object to split model: {}", e))?;
    }

    // --- Base materials ---
    let base_materials: Vec<BaseMaterialsGroup> = source
        .resources
        .iter_base_materials()
        .filter(|m| needed_ids.contains(&m.id))
        .cloned()
        .collect();
    for mut mat in base_materials {
        remap_id(&mut mat.id, id_remap);
        out.resources
            .add_base_materials(mat)
            .map_err(|e| anyhow::anyhow!("Failed to add base materials to split model: {}", e))?;
    }

    // --- Color groups ---
    let color_groups: Vec<ColorGroup> = source
        .resources
        .iter_color_groups()
        .filter(|c| needed_ids.contains(&c.id))
        .cloned()
        .collect();
    for mut col in color_groups {
        remap_id(&mut col.id, id_remap);
        out.resources
            .add_color_group(col)
            .map_err(|e| anyhow::anyhow!("Failed to add color group to split model: {}", e))?;
    }

    // --- Texture2D ---
    let texture_2d: Vec<Texture2D> = source
        .resources
        .iter_texture_2d()
        .filter(|t| needed_ids.contains(&t.id))
        .cloned()
        .collect();
    for mut tex in texture_2d {
        remap_id(&mut tex.id, id_remap);
        out.resources
            .add_texture_2d(tex)
            .map_err(|e| anyhow::anyhow!("Failed to add texture 2D to split model: {}", e))?;
    }

    // --- Texture2DGroup ---
    let texture_2d_groups: Vec<Texture2DGroup> = source
        .resources
        .iter_textures()
        .filter(|g| needed_ids.contains(&g.id))
        .cloned()
        .collect();
    for mut grp in texture_2d_groups {
        remap_id(&mut grp.id, id_remap);
        remap_id(&mut grp.texture_id, id_remap);
        out.resources
            .add_texture_2d_group(grp)
            .map_err(|e| anyhow::anyhow!("Failed to add texture group to split model: {}", e))?;
    }

    // --- CompositeMaterials ---
    let composite_materials: Vec<CompositeMaterials> = source
        .resources
        .iter_composite_materials()
        .filter(|c| needed_ids.contains(&c.id))
        .cloned()
        .collect();
    for mut comp in composite_materials {
        remap_id(&mut comp.id, id_remap);
        remap_id(&mut comp.base_material_id, id_remap);
        out.resources
            .add_composite_materials(comp)
            .map_err(|e| {
                anyhow::anyhow!("Failed to add composite materials to split model: {}", e)
            })?;
    }

    // --- MultiProperties ---
    let multi_properties: Vec<MultiProperties> = source
        .resources
        .iter_multi_properties()
        .filter(|m| needed_ids.contains(&m.id))
        .cloned()
        .collect();
    for mut mp in multi_properties {
        remap_id(&mut mp.id, id_remap);
        for pid in &mut mp.pids {
            remap_id(pid, id_remap);
        }
        out.resources
            .add_multi_properties(mp)
            .map_err(|e| anyhow::anyhow!("Failed to add multi-properties to split model: {}", e))?;
    }

    // --- SliceStack ---
    let slice_stacks: Vec<SliceStack> = source
        .resources
        .iter_slice_stacks()
        .filter(|s| needed_ids.contains(&s.id))
        .cloned()
        .collect();
    for mut ss in slice_stacks {
        remap_id(&mut ss.id, id_remap);
        out.resources
            .add_slice_stack(ss)
            .map_err(|e| anyhow::anyhow!("Failed to add slice stack to split model: {}", e))?;
    }

    // --- VolumetricStack ---
    let volumetric_stacks: Vec<VolumetricStack> = source
        .resources
        .iter_volumetric_stacks()
        .filter(|v| needed_ids.contains(&v.id))
        .cloned()
        .collect();
    for mut vs in volumetric_stacks {
        remap_id(&mut vs.id, id_remap);
        out.resources
            .add_volumetric_stack(vs)
            .map_err(|e| {
                anyhow::anyhow!("Failed to add volumetric stack to split model: {}", e)
            })?;
    }

    // --- Displacement2D ---
    let displacement_2d: Vec<Displacement2D> = source
        .resources
        .iter_displacement_2d()
        .filter(|d| needed_ids.contains(&d.id))
        .cloned()
        .collect();
    for mut d in displacement_2d {
        remap_id(&mut d.id, id_remap);
        out.resources
            .add_displacement_2d(d)
            .map_err(|e| anyhow::anyhow!("Failed to add displacement 2D to split model: {}", e))?;
    }

    // --- Build section: one item for the root object ---
    let new_root_id = id_remap
        .get(&target.root_object_id)
        .copied()
        .unwrap_or(target.root_object_id);
    let transform = if preserve_transforms {
        target
            .build_item
            .as_ref()
            .map(|bi| bi.transform)
            .unwrap_or(glam::Mat4::IDENTITY)
    } else {
        glam::Mat4::IDENTITY
    };
    out.build.items.push(BuildItem {
        object_id: new_root_id,
        transform,
        uuid: None,
        path: None,
        part_number: None,
        printable: None,
    });

    // --- Attachments: only needed ones ---
    for (path, data) in &source.attachments {
        if needed_attachments.contains(path) {
            out.attachments.insert(path.clone(), data.clone());
        }
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// Output naming helpers
// ---------------------------------------------------------------------------

/// Replace characters invalid in filenames with underscores.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Derive a base output filename from the object name (or fall back to index).
fn derive_output_name(obj: &Object, index: usize) -> String {
    obj.name
        .as_deref()
        .filter(|n| !n.is_empty())
        .map(sanitize_filename)
        .unwrap_or_else(|| format!("part_{}", index + 1))
}

/// Resolve a unique output path, auto-incrementing to avoid collisions.
/// Tracks used names within this run AND checks filesystem (unless force is set).
fn resolve_split_output_path(
    dir: &Path,
    base_name: &str,
    used_names: &mut HashSet<String>,
    force: bool,
) -> PathBuf {
    let candidate_name = format!("{base_name}.3mf");
    if !used_names.contains(&candidate_name) && (force || !dir.join(&candidate_name).exists()) {
        used_names.insert(candidate_name.clone());
        return dir.join(candidate_name);
    }
    // Auto-increment: Part.3mf -> Part_1.3mf -> Part_2.3mf
    for n in 1u32..=9999 {
        let candidate_name = format!("{base_name}_{n}.3mf");
        if !used_names.contains(&candidate_name) && (force || !dir.join(&candidate_name).exists()) {
            used_names.insert(candidate_name.clone());
            return dir.join(candidate_name);
        }
    }
    // Should be unreachable in practice
    dir.join(format!("{base_name}_overflow.3mf"))
}

/// Compute the output directory.
/// If output_dir is provided, use it; else derive `{input_stem}_split/` next to the input file.
fn compute_output_dir(input: &Path, output_dir: Option<&Path>) -> PathBuf {
    if let Some(dir) = output_dir {
        return dir.to_path_buf();
    }
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("model");
    let parent = input.parent().unwrap_or(Path::new("."));
    parent.join(format!("{stem}_split"))
}

// ---------------------------------------------------------------------------
// Count helpers for summary output
// ---------------------------------------------------------------------------

fn count_objects_in(needed_ids: &HashSet<ResourceId>, resources: &ResourceCollection) -> usize {
    resources
        .iter_objects()
        .filter(|obj| needed_ids.contains(&obj.id))
        .count()
}

fn count_materials_in(needed_ids: &HashSet<ResourceId>, resources: &ResourceCollection) -> usize {
    resources
        .iter_base_materials()
        .filter(|m| needed_ids.contains(&m.id))
        .count()
        + resources
            .iter_color_groups()
            .filter(|c| needed_ids.contains(&c.id))
            .count()
        + resources
            .iter_texture_2d()
            .filter(|t| needed_ids.contains(&t.id))
            .count()
}

fn count_textures_in(needed_attachment_paths: &HashSet<String>) -> usize {
    needed_attachment_paths.len()
}

// ---------------------------------------------------------------------------
// Split mode: collect all targets
// ---------------------------------------------------------------------------

/// Collect all items to extract, based on the split mode.
fn collect_all_targets(model: &Model, mode: SplitMode) -> Vec<SplitTarget> {
    match mode {
        SplitMode::ByItem => {
            // One output per build item
            model
                .build
                .items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    // Derive name from the referenced object's name
                    let name = model
                        .resources
                        .get_object(item.object_id)
                        .and_then(|obj| obj.name.clone())
                        .filter(|n| !n.is_empty())
                        .unwrap_or_else(|| format!("part_{}", index + 1));
                    SplitTarget {
                        root_object_id: item.object_id,
                        build_item: Some(item.clone()),
                        name,
                        index,
                    }
                })
                .collect()
        }
        SplitMode::ByObject => {
            // One output per unique printable object resource
            let mut objects: Vec<&Object> = model
                .resources
                .iter_objects()
                .filter(|obj| obj.object_type.can_be_in_build())
                .collect();
            // Sort by ID for deterministic ordering
            objects.sort_by_key(|obj| obj.id.0);

            objects
                .iter()
                .enumerate()
                .map(|(index, obj)| {
                    // Find a matching build item if one exists
                    let build_item = model
                        .build
                        .items
                        .iter()
                        .find(|item| item.object_id == obj.id)
                        .cloned();
                    let name = obj
                        .name
                        .clone()
                        .filter(|n| !n.is_empty())
                        .unwrap_or_else(|| format!("part_{}", index + 1));
                    SplitTarget {
                        root_object_id: obj.id,
                        build_item,
                        name,
                        index,
                    }
                })
                .collect()
        }
    }
}

// ---------------------------------------------------------------------------
// --select filter
// ---------------------------------------------------------------------------

/// Filter split targets to only the selected items.
/// Selectors can be numeric indices or case-insensitive name contains-matches.
fn select_items(all_targets: Vec<SplitTarget>, selectors: &[String]) -> Vec<SplitTarget> {
    if selectors.is_empty() {
        return all_targets;
    }

    all_targets
        .into_iter()
        .filter(|target| {
            selectors.iter().any(|sel| {
                // Try parsing as index
                if let Ok(n) = sel.parse::<usize>() {
                    return target.index == n;
                }
                // Try case-insensitive name contains-match
                target
                    .name
                    .to_lowercase()
                    .contains(&sel.to_lowercase())
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// pub fn run() — main entry point
// ---------------------------------------------------------------------------

/// Entry point for the split command.
#[allow(clippy::too_many_arguments)]
pub fn run(
    input: PathBuf,
    output_dir: Option<PathBuf>,
    mode: SplitMode,
    select: Vec<String>,
    preserve_transforms: bool,
    dry_run: bool,
    force: bool,
    verbosity: Verbosity,
) -> anyhow::Result<()> {
    // Step 1: Load source file with full attachments
    let source = load_full(&input)?;

    // Step 2: Check for secure content — error if present (consistent with merge)
    check_secure_content(&source).map_err(|_| {
        anyhow::anyhow!(
            "Cannot split signed/encrypted 3MF files. Strip signatures first using the decrypt command."
        )
    })?;

    // Step 3: Collect all split targets
    let all_targets = collect_all_targets(&source, mode);

    if all_targets.is_empty() {
        anyhow::bail!("No objects or build items found to split in {:?}", input);
    }

    // Step 4: Apply --select filter
    let targets = select_items(all_targets, &select);

    if targets.is_empty() {
        anyhow::bail!(
            "--select matched no items. Use a valid index or object name substring."
        );
    }

    // Step 5: Compute output directory
    let out_dir = compute_output_dir(&input, output_dir.as_deref());

    // Step 6: Phase 1 — Trace all dependencies for ALL items BEFORE creating any files.
    // This ensures we don't create a partial output directory if tracing fails.
    let mut used_names: HashSet<String> = HashSet::new();
    let mut prepared: Vec<PreparedSplit> = Vec::with_capacity(targets.len());

    for target in targets {
        if matches!(verbosity, Verbosity::Verbose) {
            eprintln!(
                "  Tracing dependencies for '{}' (root ID={})",
                target.name, target.root_object_id.0
            );
        }

        // Trace transitive dependencies
        let mut collector = DependencyCollector::new(&source.resources);
        collector.collect_object(target.root_object_id);
        let needed_ids = collector.needed_ids;
        let needed_attachments = collector.needed_attachment_paths;

        // Build compact ID remap
        let id_remap = build_compact_remap(&needed_ids);

        // Derive output filename
        let base_name = derive_output_name(
            source
                .resources
                .get_object(target.root_object_id)
                .unwrap_or_else(|| {
                    // Fallback: use target.name directly if object lookup fails
                    // This should not happen, but handle gracefully
                    panic!("Object {} not found", target.root_object_id.0)
                }),
            target.index,
        );
        let output_path = resolve_split_output_path(&out_dir, &base_name, &mut used_names, force);

        // Count summaries for output/dry-run
        let object_count = count_objects_in(&needed_ids, &source.resources);
        let material_count = count_materials_in(&needed_ids, &source.resources);
        let texture_count = count_textures_in(&needed_attachments);

        if matches!(verbosity, Verbosity::Verbose) {
            eprintln!(
                "    Found {} objects, {} materials, {} textures",
                object_count, material_count, texture_count
            );
            eprintln!("    Output: {}", output_path.display());
        }

        prepared.push(PreparedSplit {
            target,
            needed_ids,
            needed_attachments,
            id_remap,
            output_path,
            object_count,
            material_count,
            texture_count,
        });
    }

    // Step 7: Dry-run — print summary and return without writing
    if dry_run {
        println!("DRY RUN: Would write {} files to {}/", prepared.len(), out_dir.display());
        for p in &prepared {
            let filename = p
                .output_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?");
            println!(
                "  {:<30} ({} object{}, {} material{}, {} texture{})",
                filename,
                p.object_count,
                if p.object_count == 1 { "" } else { "s" },
                p.material_count,
                if p.material_count == 1 { "" } else { "s" },
                p.texture_count,
                if p.texture_count == 1 { "" } else { "s" },
            );
        }
        return Ok(());
    }

    // Step 8: Create output directory.
    // If it exists and --force is not set, bail. Otherwise proceed.
    if out_dir.exists() && !force {
        anyhow::bail!(
            "Output directory {:?} already exists. Use --force to overwrite files inside it.",
            out_dir
        );
    }
    fs::create_dir_all(&out_dir).map_err(|e| {
        anyhow::anyhow!("Failed to create output directory {:?}: {}", out_dir, e)
    })?;

    // Step 9: Phase 2 — Write each split model to its output file
    for p in &prepared {
        if matches!(verbosity, Verbosity::Verbose) {
            eprintln!("  Writing {}", p.output_path.display());
        }

        let split_model = build_split_model(
            &source,
            &p.target,
            preserve_transforms,
            &p.id_remap,
            &p.needed_ids,
            &p.needed_attachments,
        )?;

        let file = File::create(&p.output_path).map_err(|e| {
            anyhow::anyhow!("Failed to create output file {:?}: {}", p.output_path, e)
        })?;
        split_model
            .write(BufWriter::new(file))
            .map_err(|e| anyhow::anyhow!("Failed to write split model {:?}: {}", p.output_path, e))?;
    }

    // Step 10: Print summary
    if !matches!(verbosity, Verbosity::Quiet) {
        println!(
            "Split {} item{} from {} -> {}/",
            prepared.len(),
            if prepared.len() == 1 { "" } else { "s" },
            input.display(),
            out_dir.display()
        );
        for p in &prepared {
            let filename = p
                .output_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?");
            println!(
                "  {:<30} ({} object{}, {} material{}, {} texture{})",
                filename,
                p.object_count,
                if p.object_count == 1 { "" } else { "s" },
                p.material_count,
                if p.material_count == 1 { "" } else { "s" },
                p.texture_count,
                if p.texture_count == 1 { "" } else { "s" },
            );
        }
    }

    Ok(())
}
