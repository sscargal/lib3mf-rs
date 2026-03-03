//! Batch command — process multiple 3MF/STL/OBJ files with a single invocation.
//!
//! This module implements the core batch engine for the `3mf batch` command.
//! The batch pipeline:
//! 1. Expand glob patterns, walk directories, deduplicate inputs
//! 2. Detect file type from magic bytes (Zip3mf, Stl, Obj, Unknown)
//! 3. Filter files by requested operations (e.g. validate only applies to 3MF)
//! 4. Process files sequentially (jobs=1) or in parallel via rayon (jobs>1)
//! 5. Accumulate per-file results — never abort on single failure
//! 6. Emit text progress [N/M] or JSON Lines output per file
//! 7. Optionally print summary (totals + failed list)
//!
//! SAFETY RULE: This module NEVER calls crate::commands::validate / stats / list.
//! Those functions print directly to stdout and call std::process::exit().
//! Instead, we call lib3mf-core APIs directly:
//!   - model.validate(level)         → ValidationReport
//!   - model.compute_stats(archiver) → ModelStats
//!   - archiver.list_entries()       → `Vec<String>`

// ---------------------------------------------------------------------------
// (A) Imports
// ---------------------------------------------------------------------------

use crate::commands::OutputFormat;
use crate::commands::merge::Verbosity;
use glob::glob;
use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path};
use lib3mf_core::model::{Geometry, Model};
use lib3mf_core::parser::parse_model;
use lib3mf_core::validation::ValidationLevel;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ---------------------------------------------------------------------------
// (A) Core types
// ---------------------------------------------------------------------------

/// Detected type of a file based on magic bytes / extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectedFileType {
    /// ZIP-based 3MF container (PK\x03\x04 magic)
    Zip3mf,
    /// STL model (ASCII "solid" or binary header)
    Stl,
    /// Wavefront OBJ model (by .obj extension)
    Obj,
    /// Unrecognized or unsupported file type
    Unknown,
}

/// Error category for per-operation failures.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    /// I/O or archive error (cannot open, read, or parse file)
    FileError,
    /// Operation failed after the file was successfully opened
    OperationError,
}

/// Error record for a single failed operation on a file.
#[derive(Debug, Clone, Serialize)]
pub struct FileError {
    /// The broad category of the error (IO, Parse, Validation, etc.).
    pub category: ErrorCategory,
    /// The name of the operation that failed (e.g. "validate", "stats").
    pub operation: String,
    /// Human-readable description of the error.
    pub message: String,
}

/// Result of processing one file.
#[derive(Debug, Clone, Serialize)]
pub struct FileResult {
    /// 1-based index in the discovered file list
    pub index: usize,
    /// Absolute path to the file
    pub path: PathBuf,
    /// Detected file type
    pub file_type: DetectedFileType,
    /// True if this file was skipped (type not accepted by any requested op)
    pub skipped: bool,
    /// Accumulated errors from all operations
    pub errors: Vec<FileError>,
    /// Number of operations that completed without error
    pub ops_completed: usize,
    /// Per-operation JSON-serialisable results (keyed by operation name)
    pub operations: HashMap<String, serde_json::Value>,
}

/// Set of operations to perform in a batch run.
#[derive(Debug, Clone, Default)]
pub struct BatchOps {
    /// Run validation at this level (None = skip validate)
    pub validate: bool,
    /// Validation level string (minimal/standard/strict/paranoid)
    pub validate_level: Option<String>,
    /// Compute and emit model statistics
    pub stats: bool,
    /// List archive entries
    pub list: bool,
    /// Convert to another format
    pub convert: bool,
    /// Use ASCII format for STL conversion output (false = binary)
    pub convert_ascii: bool,
    /// Output directory for converted files
    pub output_dir: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// (B) File type detection and discovery
// ---------------------------------------------------------------------------

/// Reads magic bytes to classify a file's type.
///
/// - `PK\x03\x04` → Zip3mf
/// - `solid` (case-insensitive, ASCII STL) → Stl
/// - `.stl` extension fallback → Stl (binary STL has no reliable short magic)
/// - `.obj` extension → Obj
/// - `.3mf` extension → Zip3mf
/// - Everything else → Unknown
pub fn detect_file_type(path: &Path) -> DetectedFileType {
    let mut buf = [0u8; 16];
    let Ok(mut f) = File::open(path) else {
        return DetectedFileType::Unknown;
    };
    let n = f.read(&mut buf).unwrap_or(0);

    // ZIP magic: PK\x03\x04
    if n >= 4 && buf[0] == b'P' && buf[1] == b'K' && buf[2] == 0x03 && buf[3] == 0x04 {
        return DetectedFileType::Zip3mf;
    }

    // ASCII STL starts with "solid" (trimmed, case-insensitive)
    let as_lower: Vec<u8> = buf[..n].iter().map(|b| b.to_ascii_lowercase()).collect();
    if as_lower.starts_with(b"solid") {
        return DetectedFileType::Stl;
    }

    // Extension fallback (binary STL has no reliable short magic)
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        match ext_lower.as_str() {
            "stl" => return DetectedFileType::Stl,
            "obj" => return DetectedFileType::Obj,
            "3mf" => return DetectedFileType::Zip3mf,
            _ => {}
        }
    }

    DetectedFileType::Unknown
}

/// Returns true if the operation set accepts the given file type.
fn file_type_accepted(ft: DetectedFileType, ops: &BatchOps) -> bool {
    match ft {
        DetectedFileType::Zip3mf => {
            // 3MF files support: validate, stats, list, convert
            ops.validate || ops.stats || ops.list || ops.convert
        }
        DetectedFileType::Stl => {
            // STL files support: validate (basic), stats, convert (to 3MF)
            ops.validate || ops.stats || ops.convert
        }
        DetectedFileType::Obj => {
            // OBJ files support: validate (basic), stats, convert (to 3MF)
            ops.validate || ops.stats || ops.convert
        }
        DetectedFileType::Unknown => false,
    }
}

/// Expands glob patterns, walks directories recursively (if recursive=true),
/// and collects individual file paths. Deduplicates by canonical path.
pub fn discover_files(raw_inputs: &[PathBuf], recursive: bool) -> anyhow::Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for input in raw_inputs {
        let input_str = input.to_string_lossy();

        // Try glob expansion first
        let glob_matches: Vec<PathBuf> = glob(&input_str)
            .map(|g| g.filter_map(|r| r.ok()).collect())
            .unwrap_or_default();

        if !glob_matches.is_empty() {
            for p in glob_matches {
                if p.is_dir() {
                    collect_from_dir(&p, recursive, &mut paths, &mut seen);
                } else {
                    insert_unique(p, &mut paths, &mut seen);
                }
            }
        } else {
            // Not a glob — treat as literal path
            let p = input.clone();
            if p.is_dir() {
                collect_from_dir(&p, recursive, &mut paths, &mut seen);
            } else if p.exists() {
                insert_unique(p, &mut paths, &mut seen);
            }
        }
    }

    Ok(paths)
}

fn collect_from_dir(
    dir: &Path,
    recursive: bool,
    paths: &mut Vec<PathBuf>,
    seen: &mut std::collections::HashSet<PathBuf>,
) {
    let walker = if recursive {
        WalkDir::new(dir)
    } else {
        WalkDir::new(dir).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let p = entry.path().to_path_buf();
        if p.is_file() {
            insert_unique(p, paths, seen);
        }
    }
}

fn insert_unique(
    path: PathBuf,
    paths: &mut Vec<PathBuf>,
    seen: &mut std::collections::HashSet<PathBuf>,
) {
    // Use canonical path for dedup when possible
    let key = std::fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
    if seen.insert(key) {
        paths.push(path);
    }
}

// ---------------------------------------------------------------------------
// (C) process_file — per-file operation dispatch using lib3mf-core APIs directly
// ---------------------------------------------------------------------------

/// Processes a single file, running all requested operations.
///
/// NEVER calls crate::commands::validate / stats / list — those have side
/// effects (process::exit). We call lib3mf-core APIs directly:
///   - model.validate(level)           → ValidationReport
///   - model.compute_stats(&archiver)  → ModelStats
///   - archiver.list_entries()         → `Vec<String>`
pub fn process_file(index: usize, path: &Path, ops: &BatchOps) -> FileResult {
    let file_type = detect_file_type(path);

    let mut result = FileResult {
        index,
        path: path.to_path_buf(),
        file_type,
        skipped: false,
        errors: Vec::new(),
        ops_completed: 0,
        operations: HashMap::new(),
    };

    if !file_type_accepted(file_type, ops) {
        result.skipped = true;
        return result;
    }

    match file_type {
        DetectedFileType::Zip3mf => process_3mf_file(&mut result, path, ops),
        DetectedFileType::Stl => process_stl_file(&mut result, path, ops),
        DetectedFileType::Obj => process_obj_file(&mut result, path, ops),
        DetectedFileType::Unknown => {
            result.skipped = true;
        }
    }

    result
}

/// Process a 3MF (ZIP) file — supports validate, stats, list, convert.
fn process_3mf_file(result: &mut FileResult, path: &Path, ops: &BatchOps) {
    // Open archive — failure is a FileError that applies to all ops
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            result.errors.push(FileError {
                category: ErrorCategory::FileError,
                operation: "open".to_string(),
                message: e.to_string(),
            });
            return;
        }
    };

    let mut archiver = match ZipArchiver::new(file) {
        Ok(a) => a,
        Err(e) => {
            result.errors.push(FileError {
                category: ErrorCategory::FileError,
                operation: "open_zip".to_string(),
                message: e.to_string(),
            });
            return;
        }
    };

    // Parse model (needed for validate, stats, convert)
    let model_needed = ops.validate || ops.stats || ops.convert;
    let model: Option<Model> = if model_needed {
        let model_path = match find_model_path(&mut archiver) {
            Ok(p) => p,
            Err(e) => {
                result.errors.push(FileError {
                    category: ErrorCategory::FileError,
                    operation: "find_model_path".to_string(),
                    message: e.to_string(),
                });
                // Continue to list even if model parse fails
                if ops.list {
                    run_list_op(result, &mut archiver);
                }
                return;
            }
        };

        let model_data = match archiver.read_entry(&model_path) {
            Ok(d) => d,
            Err(e) => {
                result.errors.push(FileError {
                    category: ErrorCategory::FileError,
                    operation: "read_model".to_string(),
                    message: e.to_string(),
                });
                if ops.list {
                    run_list_op(result, &mut archiver);
                }
                return;
            }
        };

        match parse_model(std::io::Cursor::new(model_data)) {
            Ok(m) => Some(m),
            Err(e) => {
                result.errors.push(FileError {
                    category: ErrorCategory::FileError,
                    operation: "parse_model".to_string(),
                    message: e.to_string(),
                });
                if ops.list {
                    run_list_op(result, &mut archiver);
                }
                return;
            }
        }
    } else {
        None
    };

    // validate operation — calls model.validate(level) directly
    if ops.validate
        && let Some(ref m) = model
    {
        run_validate_op(result, m, ops);
    }

    // stats operation — calls model.compute_stats(&mut archiver) directly
    if ops.stats
        && let Some(ref m) = model
    {
        run_stats_op(result, m, &mut archiver);
    }

    // list operation — calls archiver.list_entries() directly
    if ops.list {
        run_list_op(result, &mut archiver);
    }

    // convert operation: 3MF → STL
    if ops.convert {
        run_convert_3mf_op(result, model.as_ref(), path, ops);
    }
}

/// Validate using model.validate(level) — returns ValidationReport directly.
fn run_validate_op(result: &mut FileResult, model: &Model, ops: &BatchOps) {
    let level = match ops
        .validate_level
        .as_deref()
        .unwrap_or("standard")
        .to_ascii_lowercase()
        .as_str()
    {
        "minimal" => ValidationLevel::Minimal,
        "strict" => ValidationLevel::Strict,
        "paranoid" => ValidationLevel::Paranoid,
        _ => ValidationLevel::Standard,
    };

    // DIRECT API CALL: model.validate(level) — never calls commands::validate()
    let report = model.validate(level);

    let errors: Vec<serde_json::Value> = report
        .items
        .iter()
        .filter(|i| i.severity == lib3mf_core::validation::ValidationSeverity::Error)
        .map(|i| serde_json::json!({ "code": i.code, "message": i.message }))
        .collect();
    let warnings: Vec<serde_json::Value> = report
        .items
        .iter()
        .filter(|i| i.severity == lib3mf_core::validation::ValidationSeverity::Warning)
        .map(|i| serde_json::json!({ "code": i.code, "message": i.message }))
        .collect();
    let info: Vec<serde_json::Value> = report
        .items
        .iter()
        .filter(|i| i.severity == lib3mf_core::validation::ValidationSeverity::Info)
        .map(|i| serde_json::json!({ "code": i.code, "message": i.message }))
        .collect();

    let passed = !report.has_errors();
    let error_count = errors.len();

    result.operations.insert(
        "validate".to_string(),
        serde_json::json!({
            "passed": passed,
            "level": format!("{level:?}").to_lowercase(),
            "errors": errors,
            "warnings": warnings,
            "info": info,
        }),
    );

    if !passed {
        result.errors.push(FileError {
            category: ErrorCategory::OperationError,
            operation: "validate".to_string(),
            message: format!("Validation failed: {error_count} error(s)"),
        });
    } else {
        result.ops_completed += 1;
    }
}

/// Stats using model.compute_stats(&mut archiver) — returns ModelStats directly.
fn run_stats_op(result: &mut FileResult, model: &Model, archiver: &mut ZipArchiver<File>) {
    // DIRECT API CALL: model.compute_stats(archiver) — never calls commands::stats()
    match model.compute_stats(archiver) {
        Ok(stats) => {
            let v = serde_json::json!({
                "geometry": {
                    "object_count": stats.geometry.object_count,
                    "triangle_count": stats.geometry.triangle_count,
                    "vertex_count": stats.geometry.vertex_count,
                    "instance_count": stats.geometry.instance_count,
                    "surface_area": stats.geometry.surface_area,
                    "volume": stats.geometry.volume,
                    "is_manifold": stats.geometry.is_manifold,
                },
                "materials": {
                    "base_materials_count": stats.materials.base_materials_count,
                    "color_groups_count": stats.materials.color_groups_count,
                    "texture_2d_groups_count": stats.materials.texture_2d_groups_count,
                    "composite_materials_count": stats.materials.composite_materials_count,
                    "multi_properties_count": stats.materials.multi_properties_count,
                },
            });
            result.operations.insert("stats".to_string(), v);
            result.ops_completed += 1;
        }
        Err(e) => {
            result.errors.push(FileError {
                category: ErrorCategory::OperationError,
                operation: "stats".to_string(),
                message: e.to_string(),
            });
        }
    }
}

/// List using archiver.list_entries() — returns Vec<String> directly.
fn run_list_op(result: &mut FileResult, archiver: &mut ZipArchiver<File>) {
    // DIRECT API CALL: archiver.list_entries() — never calls commands::list()
    match archiver.list_entries() {
        Ok(entries) => {
            let count = entries.len();
            result.operations.insert(
                "list".to_string(),
                serde_json::json!({ "entries": entries, "count": count }),
            );
            result.ops_completed += 1;
        }
        Err(e) => {
            result.errors.push(FileError {
                category: ErrorCategory::OperationError,
                operation: "list".to_string(),
                message: e.to_string(),
            });
        }
    }
}

/// Convert 3MF → STL (writes to output_dir if set, else same directory as source).
fn run_convert_3mf_op(
    result: &mut FileResult,
    model: Option<&Model>,
    source_path: &Path,
    ops: &BatchOps,
) {
    let Some(model) = model else {
        result.errors.push(FileError {
            category: ErrorCategory::OperationError,
            operation: "convert".to_string(),
            message: "Model not loaded; cannot convert".to_string(),
        });
        return;
    };

    let stem = source_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());

    let out_name = format!("{}.stl", stem);
    let out_dir = ops
        .output_dir
        .as_deref()
        .unwrap_or_else(|| source_path.parent().unwrap_or(Path::new(".")));
    let out_path = out_dir.join(&out_name);

    let out_file = match File::create(&out_path) {
        Ok(f) => f,
        Err(e) => {
            result.errors.push(FileError {
                category: ErrorCategory::OperationError,
                operation: "convert".to_string(),
                message: format!("Cannot create {}: {}", out_path.display(), e),
            });
            return;
        }
    };

    let write_result = if ops.convert_ascii {
        lib3mf_converters::stl::AsciiStlExporter::write(model, out_file)
    } else {
        lib3mf_converters::stl::BinaryStlExporter::write(model, out_file)
    };

    match write_result {
        Ok(()) => {
            result.operations.insert(
                "convert".to_string(),
                serde_json::json!({ "output": out_path.display().to_string() }),
            );
            result.ops_completed += 1;
        }
        Err(e) => {
            result.errors.push(FileError {
                category: ErrorCategory::OperationError,
                operation: "convert".to_string(),
                message: e.to_string(),
            });
        }
    }
}

/// Helper: count mesh triangles in a model from Geometry enum variants.
fn count_triangles_vertices(model: &Model) -> (usize, usize) {
    model
        .resources
        .iter_objects()
        .map(|o| match &o.geometry {
            Geometry::Mesh(m) => (m.triangles.len(), m.vertices.len()),
            _ => (0, 0),
        })
        .fold((0, 0), |(ta, va), (t, v)| (ta + t, va + v))
}

/// Process an STL file — supports validate (basic), stats (via converter), convert (to 3MF).
fn process_stl_file(result: &mut FileResult, path: &Path, ops: &BatchOps) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            result.errors.push(FileError {
                category: ErrorCategory::FileError,
                operation: "open".to_string(),
                message: e.to_string(),
            });
            return;
        }
    };

    // StlImporter::read takes a Read + Seek
    let model = match lib3mf_converters::stl::StlImporter::read(file) {
        Ok(m) => m,
        Err(e) => {
            result.errors.push(FileError {
                category: ErrorCategory::FileError,
                operation: "parse_stl".to_string(),
                message: e.to_string(),
            });
            return;
        }
    };

    if ops.validate {
        run_validate_op(result, &model, ops);
    }

    if ops.stats {
        let obj_count = model.resources.iter_objects().count();
        let (tri_count, vert_count) = count_triangles_vertices(&model);
        result.operations.insert(
            "stats".to_string(),
            serde_json::json!({
                "geometry": {
                    "object_count": obj_count,
                    "triangle_count": tri_count,
                    "vertex_count": vert_count,
                    "instance_count": model.build.items.len(),
                },
                "materials": {
                    "base_materials_count": 0,
                    "color_groups_count": 0,
                    "texture_2d_groups_count": 0,
                },
            }),
        );
        result.ops_completed += 1;
    }

    if ops.convert {
        // STL → 3MF
        convert_to_3mf(result, &model, path, ops);
    }
}

/// Process an OBJ file — supports validate (basic), stats (via converter), convert (to 3MF).
fn process_obj_file(result: &mut FileResult, path: &Path, ops: &BatchOps) {
    let model = match lib3mf_converters::obj::ObjImporter::read_from_path(path) {
        Ok(m) => m,
        Err(e) => {
            result.errors.push(FileError {
                category: ErrorCategory::FileError,
                operation: "parse_obj".to_string(),
                message: e.to_string(),
            });
            return;
        }
    };

    if ops.validate {
        run_validate_op(result, &model, ops);
    }

    if ops.stats {
        let obj_count = model.resources.iter_objects().count();
        let (tri_count, vert_count) = count_triangles_vertices(&model);
        let base_mat_count = model.resources.iter_base_materials().count();
        result.operations.insert(
            "stats".to_string(),
            serde_json::json!({
                "geometry": {
                    "object_count": obj_count,
                    "triangle_count": tri_count,
                    "vertex_count": vert_count,
                    "instance_count": model.build.items.len(),
                },
                "materials": {
                    "base_materials_count": base_mat_count,
                    "color_groups_count": 0,
                    "texture_2d_groups_count": 0,
                },
            }),
        );
        result.ops_completed += 1;
    }

    if ops.convert {
        // OBJ → 3MF
        convert_to_3mf(result, &model, path, ops);
    }
}

/// Convert an in-memory model (parsed from STL/OBJ) to a 3MF archive.
fn convert_to_3mf(result: &mut FileResult, model: &Model, source_path: &Path, ops: &BatchOps) {
    let stem = source_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "output".to_string());

    let out_dir = ops
        .output_dir
        .as_deref()
        .unwrap_or_else(|| source_path.parent().unwrap_or(Path::new(".")));
    let out_path = out_dir.join(format!("{}.3mf", stem));

    let out_file = match File::create(&out_path) {
        Ok(f) => f,
        Err(e) => {
            result.errors.push(FileError {
                category: ErrorCategory::OperationError,
                operation: "convert".to_string(),
                message: format!("Cannot create {}: {}", out_path.display(), e),
            });
            return;
        }
    };

    // Use model.write() — the public API on Model (via model_write_zip.rs)
    match model.write(out_file) {
        Ok(()) => {
            result.operations.insert(
                "convert".to_string(),
                serde_json::json!({ "output": out_path.display().to_string() }),
            );
            result.ops_completed += 1;
        }
        Err(e) => {
            result.errors.push(FileError {
                category: ErrorCategory::OperationError,
                operation: "convert".to_string(),
                message: e.to_string(),
            });
        }
    }
}

// ---------------------------------------------------------------------------
// (D) run() pipeline — main entry point
// ---------------------------------------------------------------------------

/// Top-level configuration for a batch run (passed to `run()`).
pub struct BatchConfig {
    /// Parallelism level (1 = sequential, >1 = rayon parallel)
    pub jobs: usize,
    /// Walk directories recursively
    pub recursive: bool,
    /// Print summary totals + failed list at end
    pub summary: bool,
    /// Verbosity level
    pub verbosity: Verbosity,
    /// Output format (text or JSON)
    pub format: OutputFormat,
    /// Suppress 100+ file warning
    pub yes: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        BatchConfig {
            jobs: 1,
            recursive: false,
            summary: false,
            verbosity: Verbosity::Normal,
            format: OutputFormat::Text,
            yes: false,
        }
    }
}

/// Entry point for the batch command.
///
/// # Arguments
/// - `inputs`  — Raw glob/path/directory inputs
/// - `ops`     — Which operations to run
/// - `config`  — Execution configuration (jobs, verbosity, format, etc.)
///
/// # Returns
/// `Ok(true)` if all files processed without errors, `Ok(false)` if any failed.
pub fn run(inputs: Vec<PathBuf>, ops: BatchOps, config: BatchConfig) -> anyhow::Result<bool> {
    let BatchConfig {
        jobs,
        recursive,
        summary,
        verbosity,
        format,
        yes,
    } = config;
    // 1. Discover files
    let files = discover_files(&inputs, recursive)?;
    let total = files.len();

    if total == 0 {
        eprintln!("No files found matching the given inputs.");
        return Ok(true);
    }

    // 2. Warn if 100+ files
    if total >= 100 && !yes {
        eprintln!(
            "Warning: {} files discovered. Processing this many files may take a while.",
            total
        );
        eprintln!("Pass --yes to suppress this warning and proceed.");
    }

    if matches!(verbosity, Verbosity::Verbose) {
        eprintln!("Batch processing {} file(s) with jobs={}", total, jobs);
    }

    // 3. Process files (parallel or sequential)
    let results: Vec<FileResult> = if jobs > 1 {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .unwrap_or_else(|_| rayon::ThreadPoolBuilder::new().build().unwrap());

        let mut collected: Vec<FileResult> = pool.install(|| {
            files
                .par_iter()
                .enumerate()
                .map(|(i, path)| process_file(i + 1, path, &ops))
                .collect()
        });
        // Sort by original index for deterministic output
        collected.sort_by_key(|r| r.index);

        // Emit progress after parallel collection
        if !matches!(format, OutputFormat::Json) && !matches!(verbosity, Verbosity::Quiet) {
            for res in &collected {
                print_file_progress(res, res.index, total, &verbosity);
            }
        }

        collected
    } else {
        // Sequential: emit progress as we go
        files
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let res = process_file(i + 1, path, &ops);
                if !matches!(format, OutputFormat::Json) && !matches!(verbosity, Verbosity::Quiet) {
                    print_file_progress(&res, i + 1, total, &verbosity);
                }
                res
            })
            .collect()
    };

    // 4. JSON Lines output (one JSON object per file per line)
    if matches!(format, OutputFormat::Json) {
        for res in &results {
            println!("{}", serde_json::to_string(res)?);
        }
    }

    // 5. Summary
    let failed: Vec<&FileResult> = results.iter().filter(|r| !r.errors.is_empty()).collect();
    let skipped: Vec<&FileResult> = results.iter().filter(|r| r.skipped).collect();
    let succeeded = results
        .iter()
        .filter(|r| r.errors.is_empty() && !r.skipped)
        .count();

    if summary {
        print_summary(total, succeeded, skipped.len(), &failed, &format);
    }

    Ok(failed.is_empty())
}

// ---------------------------------------------------------------------------
// (E) Output formatting
// ---------------------------------------------------------------------------

/// Print progress for a single file in text mode.
fn print_file_progress(res: &FileResult, index: usize, total: usize, verbosity: &Verbosity) {
    let status = if res.skipped {
        "SKIP".to_string()
    } else if res.errors.is_empty() {
        "OK".to_string()
    } else {
        format!("FAIL({})", res.errors.len())
    };

    println!(
        "[{}/{}] {} -- {:?} -- {}",
        index,
        total,
        res.path.display(),
        res.file_type,
        status
    );

    if matches!(verbosity, Verbosity::Verbose) {
        for (op, val) in &res.operations {
            println!("  {}: {}", op, val);
        }
        for err in &res.errors {
            eprintln!(
                "  ERROR ({}/{}): {}",
                err.operation,
                format_category(&err.category),
                err.message
            );
        }
    }
}

/// Print summary totals and failed file list to stderr.
fn print_summary(
    total: usize,
    succeeded: usize,
    skipped: usize,
    failed: &[&FileResult],
    format: &OutputFormat,
) {
    if matches!(format, OutputFormat::Json) {
        let failed_paths: Vec<String> = failed
            .iter()
            .map(|r| r.path.display().to_string())
            .collect();
        let summary = serde_json::json!({
            "summary": {
                "total": total,
                "succeeded": succeeded,
                "skipped": skipped,
                "failed": failed.len(),
                "failed_files": failed_paths,
            }
        });
        eprintln!("{summary}");
    } else {
        eprintln!("--- Batch Summary ---");
        eprintln!("Total:     {total}");
        eprintln!("Succeeded: {succeeded}");
        eprintln!("Skipped:   {skipped}");
        eprintln!("Failed:    {}", failed.len());
        if !failed.is_empty() {
            eprintln!("Failed files:");
            for r in failed {
                eprintln!("  {}", r.path.display());
                for e in &r.errors {
                    eprintln!(
                        "    [{}] {}: {}",
                        format_category(&e.category),
                        e.operation,
                        e.message
                    );
                }
            }
        }
    }
}

fn format_category(cat: &ErrorCategory) -> &'static str {
    match cat {
        ErrorCategory::FileError => "file-error",
        ErrorCategory::OperationError => "op-error",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn make_zip_file(dir: &Path, name: &str) -> PathBuf {
        // PK\x03\x04 ZIP magic
        let path = dir.join(name);
        let mut f = File::create(&path).unwrap();
        f.write_all(b"PK\x03\x04\x00\x00\x00\x00").unwrap();
        path
    }

    fn make_ascii_stl_file(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        let mut f = File::create(&path).unwrap();
        f.write_all(b"solid test\nendsolid test\n").unwrap();
        path
    }

    fn make_txt_file(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        let mut f = File::create(&path).unwrap();
        f.write_all(b"not a 3D file").unwrap();
        path
    }

    #[test]
    fn test_detect_file_type_zip() {
        let dir = TempDir::new().unwrap();
        let p = make_zip_file(dir.path(), "model.3mf");
        assert_eq!(detect_file_type(&p), DetectedFileType::Zip3mf);
    }

    #[test]
    fn test_detect_file_type_ascii_stl() {
        let dir = TempDir::new().unwrap();
        let p = make_ascii_stl_file(dir.path(), "model.stl");
        assert_eq!(detect_file_type(&p), DetectedFileType::Stl);
    }

    #[test]
    fn test_detect_file_type_extension_fallback_obj() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("model.obj");
        let mut f = File::create(&path).unwrap();
        f.write_all(b"v 0 0 0\n").unwrap();
        assert_eq!(detect_file_type(&path), DetectedFileType::Obj);
    }

    #[test]
    fn test_detect_file_type_unknown() {
        let dir = TempDir::new().unwrap();
        let p = make_txt_file(dir.path(), "readme.txt");
        assert_eq!(detect_file_type(&p), DetectedFileType::Unknown);
    }

    #[test]
    fn test_discover_files_single() {
        let dir = TempDir::new().unwrap();
        let p = make_zip_file(dir.path(), "a.3mf");
        let discovered = discover_files(&[p.clone()], false).unwrap();
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0], p);
    }

    #[test]
    fn test_discover_files_dedup() {
        let dir = TempDir::new().unwrap();
        let p = make_zip_file(dir.path(), "a.3mf");
        // Provide the same file twice — should deduplicate
        let discovered = discover_files(&[p.clone(), p.clone()], false).unwrap();
        assert_eq!(discovered.len(), 1);
    }

    #[test]
    fn test_discover_files_directory() {
        let dir = TempDir::new().unwrap();
        make_zip_file(dir.path(), "a.3mf");
        make_zip_file(dir.path(), "b.3mf");
        let discovered = discover_files(&[dir.path().to_path_buf()], false).unwrap();
        assert_eq!(discovered.len(), 2);
    }

    #[test]
    fn test_file_type_accepted_zip3mf() {
        let ops = BatchOps {
            validate: true,
            ..Default::default()
        };
        assert!(file_type_accepted(DetectedFileType::Zip3mf, &ops));
        assert!(!file_type_accepted(DetectedFileType::Unknown, &ops));
    }

    #[test]
    fn test_file_type_not_accepted_when_no_ops() {
        let ops = BatchOps::default();
        assert!(!file_type_accepted(DetectedFileType::Zip3mf, &ops));
        assert!(!file_type_accepted(DetectedFileType::Stl, &ops));
    }

    #[test]
    fn test_process_file_skipped_for_unknown_type() {
        let dir = TempDir::new().unwrap();
        let p = make_txt_file(dir.path(), "readme.txt");
        let ops = BatchOps {
            validate: true,
            ..Default::default()
        };
        let result = process_file(1, &p, &ops);
        assert!(result.skipped);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_process_file_skipped_when_no_ops() {
        let dir = TempDir::new().unwrap();
        let p = make_zip_file(dir.path(), "model.3mf");
        let ops = BatchOps::default(); // no ops enabled
        let result = process_file(1, &p, &ops);
        assert!(result.skipped);
    }
}
