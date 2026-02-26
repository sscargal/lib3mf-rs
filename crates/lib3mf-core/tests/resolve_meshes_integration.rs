//! Integration tests for `resolve_meshes()` against real BambuStudio 3MF files.
//!
//! Tests against real BambuStudio files in `tmp/models/`.
//! Each test skips gracefully if the test file is not available,
//! since these files live in `tmp/models/` and may not be present in CI.
//!
//! Purpose: The unit tests in Plan 01 verify logic with synthetic models. These
//! integration tests confirm the API works end-to-end with real-world files that
//! have the actual BambuStudio multi-file component structure (root model with
//! `Geometry::Components`, sub-model files with actual meshes, modifier volumes,
//! printable flags).

use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path};
use lib3mf_core::model::resolver::{PartResolver, ResolveOptions};
use lib3mf_core::model::{ObjectType, Unit};
use lib3mf_core::parser::parse_model;
use std::fs::File;
use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn model_path(filename: &str) -> std::path::PathBuf {
    repo_root().join("tmp/models").join(filename)
}

/// Open a 3MF file and return (archiver, root_model).
///
/// Returns `None` if the file does not exist (graceful skip for CI).
fn open_3mf(
    filename: &str,
) -> Option<(
    lib3mf_core::archive::ZipArchiver<File>,
    lib3mf_core::model::Model,
)> {
    let path = model_path(filename);
    if !path.exists() {
        return None;
    }
    let file = File::open(&path).expect("Failed to open test file");
    let mut archiver = ZipArchiver::new(file).expect("Failed to open ZIP");
    let model_path_str = find_model_path(&mut archiver).expect("Failed to find model path");
    let model_data = archiver
        .read_entry(&model_path_str)
        .expect("Failed to read model");
    let model = parse_model(std::io::Cursor::new(model_data)).expect("Failed to parse model");
    Some((archiver, model))
}

// ---------------------------------------------------------------------------
// Test 1: 3DBenchy with default filtering — only the type="model" mesh returned
// ---------------------------------------------------------------------------

#[test]
fn test_benchy_resolve_default_filtering() {
    let Some((mut archiver, model)) = open_3mf("3DBenchy_PLA.3mf") else {
        return;
    };

    let mut resolver = PartResolver::new(&mut archiver, model);
    let resolved = resolver
        .resolve_meshes(&ResolveOptions::default())
        .expect("resolve_meshes failed");

    // Default filtering: filter_other_objects=true → only ObjectType::Model meshes
    // 3DBenchy has 1 printable model object + 6 modifier volumes (type="other")
    assert_eq!(
        resolved.len(),
        1,
        "Expected exactly 1 resolved mesh (type=model) with default filtering, got {}",
        resolved.len()
    );

    let mesh = &resolved[0];

    // Vertex count: ~113K (allow ±5K tolerance for format differences)
    assert!(
        (110_000..=120_000).contains(&mesh.mesh.vertices.len()),
        "Expected ~113K vertices, got {}",
        mesh.mesh.vertices.len()
    );

    // Triangle count: ~225K (allow ±10K tolerance)
    assert!(
        (220_000..=235_000).contains(&mesh.mesh.triangles.len()),
        "Expected ~225K triangles, got {}",
        mesh.mesh.triangles.len()
    );

    // Type assertion
    assert_eq!(
        mesh.object_type,
        ObjectType::Model,
        "Expected ObjectType::Model, got {:?}",
        mesh.object_type
    );

    // Unit: BambuStudio defaults to millimeter
    assert_eq!(
        mesh.unit,
        Unit::Millimeter,
        "Expected Unit::Millimeter, got {:?}",
        mesh.unit
    );

    // Transform should be non-identity (Benchy is positioned at a build location)
    assert_ne!(
        mesh.transform,
        glam::Mat4::IDENTITY,
        "Expected non-identity transform for positioned Benchy build item"
    );
}

// ---------------------------------------------------------------------------
// Test 2: 3DBenchy with all objects — 1 model + 6 modifier volumes
// ---------------------------------------------------------------------------

#[test]
fn test_benchy_resolve_all_objects() {
    let Some((mut archiver, model)) = open_3mf("3DBenchy_PLA.3mf") else {
        return;
    };

    let mut resolver = PartResolver::new(&mut archiver, model);
    let opts = ResolveOptions {
        filter_other_objects: false,
        ..Default::default()
    };
    let resolved = resolver
        .resolve_meshes(&opts)
        .expect("resolve_meshes failed");

    // Unfiltered: 1 type=model + 6 type=other modifier volumes = 7 total
    assert_eq!(
        resolved.len(),
        7,
        "Expected 7 resolved meshes (1 model + 6 other) with filter_other_objects=false, got {}",
        resolved.len()
    );

    let model_count = resolved
        .iter()
        .filter(|m| m.object_type == ObjectType::Model)
        .count();
    let other_count = resolved
        .iter()
        .filter(|m| m.object_type == ObjectType::Other)
        .count();

    assert_eq!(
        model_count, 1,
        "Expected exactly 1 ObjectType::Model mesh, got {}",
        model_count
    );
    assert_eq!(
        other_count, 6,
        "Expected exactly 6 ObjectType::Other meshes, got {}",
        other_count
    );
}

// ---------------------------------------------------------------------------
// Test 3: SimplePyramid — minimal 3MF with 4 vertices and 4 triangles
// ---------------------------------------------------------------------------

#[test]
fn test_simple_pyramid_resolve() {
    let Some((mut archiver, model)) = open_3mf("SimplePyramid.3mf") else {
        return;
    };

    let mut resolver = PartResolver::new(&mut archiver, model);
    let resolved = resolver
        .resolve_meshes(&ResolveOptions::default())
        .expect("resolve_meshes failed");

    assert!(
        !resolved.is_empty(),
        "Expected at least 1 resolved mesh for SimplePyramid"
    );

    // Total vertex count across all resolved meshes: pyramid has 4 vertices
    let total_vertices: usize = resolved.iter().map(|m| m.mesh.vertices.len()).sum();
    assert_eq!(
        total_vertices, 4,
        "Expected 4 total vertices for SimplePyramid, got {}",
        total_vertices
    );

    // Total triangle count: pyramid has 4 faces
    let total_triangles: usize = resolved.iter().map(|m| m.mesh.triangles.len()).sum();
    assert_eq!(
        total_triangles, 4,
        "Expected 4 total triangles for SimplePyramid, got {}",
        total_triangles
    );

    // Unit: BambuStudio defaults to millimeter
    assert_eq!(
        resolved[0].unit,
        Unit::Millimeter,
        "Expected Unit::Millimeter, got {:?}",
        resolved[0].unit
    );
}

// ---------------------------------------------------------------------------
// Test 4: Cube02ProfileMini — non-zero geometry sanity check
// ---------------------------------------------------------------------------

#[test]
fn test_cube_profile_mini_resolve() {
    let Some((mut archiver, model)) = open_3mf("Cube02ProfileMini.3mf") else {
        return;
    };

    let mut resolver = PartResolver::new(&mut archiver, model);
    let resolved = resolver
        .resolve_meshes(&ResolveOptions::default())
        .expect("resolve_meshes failed");

    assert!(
        !resolved.is_empty(),
        "Expected at least 1 resolved mesh for Cube02ProfileMini"
    );

    let total_vertices: usize = resolved.iter().map(|m| m.mesh.vertices.len()).sum();
    let total_triangles: usize = resolved.iter().map(|m| m.mesh.triangles.len()).sum();

    assert!(
        total_vertices > 0,
        "Expected non-zero vertex count for Cube02ProfileMini"
    );
    assert!(
        total_triangles > 0,
        "Expected non-zero triangle count for Cube02ProfileMini"
    );

    // Unit: BambuStudio defaults to millimeter
    assert_eq!(
        resolved[0].unit,
        Unit::Millimeter,
        "Expected Unit::Millimeter, got {:?}",
        resolved[0].unit
    );
}

// ---------------------------------------------------------------------------
// Test 5: Benchy transform is non-identity
// ---------------------------------------------------------------------------

#[test]
fn test_benchy_transforms_not_identity() {
    let Some((mut archiver, model)) = open_3mf("3DBenchy_PLA.3mf") else {
        return;
    };

    let mut resolver = PartResolver::new(&mut archiver, model);
    let resolved = resolver
        .resolve_meshes(&ResolveOptions::default())
        .expect("resolve_meshes failed");

    assert!(
        !resolved.is_empty(),
        "Expected at least 1 resolved mesh for Benchy"
    );

    println!("Benchy transform: {:?}", resolved[0].transform);

    assert_ne!(
        resolved[0].transform,
        glam::Mat4::IDENTITY,
        "Expected non-identity transform for positioned Benchy build item"
    );
}

// ---------------------------------------------------------------------------
// Test 6: Object count from resolve_meshes() matches compute_stats() object_count
// ---------------------------------------------------------------------------

#[test]
fn test_benchy_resolve_object_count_matches_stats() {
    let path = model_path("3DBenchy_PLA.3mf");
    if !path.exists() {
        return;
    }

    // First archiver: for PartResolver
    let file1 = File::open(&path).expect("Failed to open test file (archiver 1)");
    let mut archiver1 = ZipArchiver::new(file1).expect("Failed to open ZIP (archiver 1)");
    let model_path_str1 = find_model_path(&mut archiver1).expect("Failed to find model path");
    let model_data1 = archiver1
        .read_entry(&model_path_str1)
        .expect("Failed to read model (archiver 1)");
    let model1 = parse_model(std::io::Cursor::new(model_data1)).expect("Failed to parse model 1");

    // Second archiver: for compute_stats() (PartResolver consumes archiver1)
    let file2 = File::open(&path).expect("Failed to open test file (archiver 2)");
    let mut archiver2 = ZipArchiver::new(file2).expect("Failed to open ZIP (archiver 2)");
    let model_path_str2 = find_model_path(&mut archiver2).expect("Failed to find model path 2");
    let model_data2 = archiver2
        .read_entry(&model_path_str2)
        .expect("Failed to read model (archiver 2)");
    let model2 = parse_model(std::io::Cursor::new(model_data2)).expect("Failed to parse model 2");

    // Compute stats using archiver2 (still has all entries available)
    let stats = model2
        .compute_stats(&mut archiver2)
        .expect("Failed to compute stats");

    // Resolve all objects (no type filtering) using archiver1
    let opts = ResolveOptions {
        filter_other_objects: false,
        ..Default::default()
    };
    let mut resolver = PartResolver::new(&mut archiver1, model1);
    let resolved = resolver
        .resolve_meshes(&opts)
        .expect("resolve_meshes failed");

    // Both systems should agree on the number of leaf mesh objects.
    // stats.geometry.object_count counts Geometry::Mesh nodes (same as resolve_meshes() does).
    // NOTE: triangle_count comparison is intentionally skipped — stats counts ALL geometry
    // types (SliceStack, VolumetricStack, etc.) while resolve_meshes only yields Mesh leaves.
    assert_eq!(
        resolved.len(),
        stats.geometry.object_count,
        "resolve_meshes() returned {} meshes but compute_stats() counted {} objects",
        resolved.len(),
        stats.geometry.object_count
    );
}
