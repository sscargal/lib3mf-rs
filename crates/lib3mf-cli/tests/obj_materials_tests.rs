//! Integration tests for OBJ material import through the CLI.
//!
//! These tests create OBJ + MTL files in temp directories, invoke CLI commands
//! via subprocess, and verify that materials flow correctly through convert,
//! stats, validate, and batch commands.
//!
//! NOTE: CLI commands are invoked via `cargo run -p lib3mf-cli` subprocess
//! (same pattern as merge_tests.rs, split_tests.rs, batch_tests.rs).

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Write an OBJ file to the given directory.
fn write_test_obj(dir: &Path, filename: &str, content: &str) -> PathBuf {
    let path = dir.join(filename);
    fs::write(&path, content).expect("Failed to write test OBJ file");
    path
}

/// Write an MTL file to the given directory.
fn write_test_mtl(dir: &Path, filename: &str, content: &str) -> PathBuf {
    let path = dir.join(filename);
    fs::write(&path, content).expect("Failed to write test MTL file");
    path
}

/// Run an arbitrary CLI command via `cargo run -p lib3mf-cli`.
fn run_cli(args: &[&str]) -> std::process::Output {
    std::process::Command::new("cargo")
        .args(["run", "--quiet", "-p", "lib3mf-cli", "--"])
        .args(args)
        .output()
        .expect("Failed to run CLI command")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Test 1: OBJ with MTL -> convert to 3MF -> verify materials in output stats.
#[test]
fn test_obj_convert_with_materials() {
    let dir = TempDir::new().unwrap();

    write_test_obj(
        dir.path(),
        "input.obj",
        "mtllib test.mtl\n\
         usemtl Red\n\
         v 0 0 0\n\
         v 10 0 0\n\
         v 0 10 0\n\
         v 5 5 10\n\
         f 1 2 3\n\
         f 1 2 4\n\
         f 2 3 4\n\
         f 1 4 3\n",
    );
    write_test_mtl(dir.path(), "test.mtl", "newmtl Red\nKd 1.0 0.0 0.0\n");

    let output_3mf = dir.path().join("output.3mf");
    let out = run_cli(&[
        "convert",
        dir.path().join("input.obj").to_str().unwrap(),
        output_3mf.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "convert should succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(output_3mf.exists(), "Output 3MF file should exist");

    // Verify materials in the converted file
    let stats_out = run_cli(&["stats", output_3mf.to_str().unwrap(), "--format", "json"]);
    assert!(stats_out.status.success(), "stats should succeed");
    let json: serde_json::Value =
        serde_json::from_slice(&stats_out.stdout).expect("Failed to parse stats JSON");
    let base_count = json["materials"]["base_materials_count"]
        .as_u64()
        .unwrap_or(0);
    assert_eq!(
        base_count, 1,
        "Converted 3MF should have 1 base materials group, got {base_count}"
    );
}

/// Test 2: Bare OBJ (no mtllib, no usemtl) -> convert -> no materials (backward compat).
#[test]
fn test_obj_convert_no_materials() {
    let dir = TempDir::new().unwrap();

    write_test_obj(
        dir.path(),
        "bare.obj",
        "v 0 0 0\nv 10 0 0\nv 0 10 0\nf 1 2 3\n",
    );

    let output_3mf = dir.path().join("bare_output.3mf");
    let out = run_cli(&[
        "convert",
        dir.path().join("bare.obj").to_str().unwrap(),
        output_3mf.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "convert bare OBJ should succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stats_out = run_cli(&["stats", output_3mf.to_str().unwrap(), "--format", "json"]);
    assert!(stats_out.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&stats_out.stdout).expect("Failed to parse stats JSON");
    let base_count = json["materials"]["base_materials_count"]
        .as_u64()
        .unwrap_or(0);
    assert_eq!(
        base_count, 0,
        "Bare OBJ with no materials should have 0 base materials groups, got {base_count}"
    );
}

/// Test 3: OBJ with group directives -> convert -> multiple objects.
#[test]
fn test_obj_convert_with_groups() {
    let dir = TempDir::new().unwrap();

    write_test_obj(
        dir.path(),
        "groups.obj",
        "g GroupA\n\
         v 0 0 0\n\
         v 10 0 0\n\
         v 0 10 0\n\
         f 1 2 3\n\
         g GroupB\n\
         v 20 0 0\n\
         v 30 0 0\n\
         v 20 10 0\n\
         f 4 5 6\n",
    );

    let output_3mf = dir.path().join("groups_output.3mf");
    let out = run_cli(&[
        "convert",
        dir.path().join("groups.obj").to_str().unwrap(),
        output_3mf.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "convert with groups should succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stats_out = run_cli(&["stats", output_3mf.to_str().unwrap(), "--format", "json"]);
    assert!(stats_out.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&stats_out.stdout).expect("Failed to parse stats JSON");
    let obj_count = json["geometry"]["object_count"].as_u64().unwrap_or(0);
    assert!(
        obj_count >= 2,
        "Two OBJ groups should produce at least 2 objects, got {obj_count}"
    );
}

/// Test 4: OBJ referencing nonexistent MTL -> warns but succeeds.
#[test]
fn test_obj_missing_mtl_warns() {
    let dir = TempDir::new().unwrap();

    write_test_obj(
        dir.path(),
        "missing.obj",
        "mtllib nonexistent.mtl\n\
         v 0 0 0\n\
         v 10 0 0\n\
         v 0 10 0\n\
         f 1 2 3\n",
    );

    let output_3mf = dir.path().join("missing_output.3mf");
    let out = run_cli(&[
        "convert",
        dir.path().join("missing.obj").to_str().unwrap(),
        output_3mf.to_str().unwrap(),
    ]);

    // Should succeed despite missing MTL
    assert!(
        out.status.success(),
        "convert should succeed even with missing MTL: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(output_3mf.exists(), "Output file should still be created");

    // Should warn about missing MTL on stderr
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Warning") && stderr.contains("MTL file not found"),
        "Expected 'Warning' and 'MTL file not found' in stderr, got: {stderr}"
    );
}

/// Test 5: OBJ uses undefined material name -> warns and uses default gray.
#[test]
fn test_obj_undefined_material_warns() {
    let dir = TempDir::new().unwrap();

    write_test_obj(
        dir.path(),
        "undef.obj",
        "mtllib test.mtl\n\
         usemtl Unknown\n\
         v 0 0 0\n\
         v 10 0 0\n\
         v 0 10 0\n\
         f 1 2 3\n",
    );
    write_test_mtl(dir.path(), "test.mtl", "newmtl KnownMat\nKd 0.5 0.5 0.5\n");

    let output_3mf = dir.path().join("undef_output.3mf");
    let out = run_cli(&[
        "convert",
        dir.path().join("undef.obj").to_str().unwrap(),
        output_3mf.to_str().unwrap(),
    ]);

    assert!(
        out.status.success(),
        "convert should succeed with undefined material: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Warning") && stderr.contains("undefined material"),
        "Expected 'Warning' and 'undefined material' in stderr, got: {stderr}"
    );
}

/// Test 6: Stats command directly on OBJ with materials.
#[test]
fn test_obj_stats_with_materials() {
    let dir = TempDir::new().unwrap();

    write_test_obj(
        dir.path(),
        "mat.obj",
        "mtllib mat.mtl\n\
         usemtl Blue\n\
         v 0 0 0\n\
         v 10 0 0\n\
         v 0 10 0\n\
         f 1 2 3\n",
    );
    write_test_mtl(dir.path(), "mat.mtl", "newmtl Blue\nKd 0.0 0.0 1.0\n");

    let out = run_cli(&[
        "stats",
        dir.path().join("mat.obj").to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(
        out.status.success(),
        "stats on OBJ should succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("Failed to parse stats JSON");
    let base_count = json["materials"]["base_materials_count"]
        .as_u64()
        .unwrap_or(0);
    assert_eq!(
        base_count, 1,
        "Stats on OBJ with MTL should show 1 base materials group, got {base_count}"
    );
}

/// Test 7: Validate command on OBJ with materials.
#[test]
fn test_obj_validate_with_materials() {
    let dir = TempDir::new().unwrap();

    write_test_obj(
        dir.path(),
        "valid.obj",
        "mtllib valid.mtl\n\
         usemtl Green\n\
         v 0 0 0\n\
         v 10 0 0\n\
         v 0 10 0\n\
         v 5 5 10\n\
         f 1 3 2\n\
         f 1 2 4\n\
         f 2 3 4\n\
         f 1 4 3\n",
    );
    write_test_mtl(dir.path(), "valid.mtl", "newmtl Green\nKd 0.0 1.0 0.0\n");

    let out = run_cli(&["validate", dir.path().join("valid.obj").to_str().unwrap()]);
    assert!(
        out.status.success(),
        "validate should succeed for OBJ with materials: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Passed") || stdout.contains("passed") || stdout.contains("OK"),
        "Validate output should indicate success: {stdout}"
    );
}

/// Test 8: Multiple materials in a single OBJ/MTL pair.
#[test]
fn test_obj_multiple_materials() {
    let dir = TempDir::new().unwrap();

    write_test_obj(
        dir.path(),
        "multi.obj",
        "mtllib multi.mtl\n\
         usemtl Red\n\
         v 0 0 0\n\
         v 10 0 0\n\
         v 0 10 0\n\
         f 1 2 3\n\
         usemtl Green\n\
         v 20 0 0\n\
         v 30 0 0\n\
         v 20 10 0\n\
         f 4 5 6\n\
         usemtl Blue\n\
         v 40 0 0\n\
         v 50 0 0\n\
         v 40 10 0\n\
         f 7 8 9\n",
    );
    write_test_mtl(
        dir.path(),
        "multi.mtl",
        "newmtl Red\nKd 1.0 0.0 0.0\n\n\
         newmtl Green\nKd 0.0 1.0 0.0\n\n\
         newmtl Blue\nKd 0.0 0.0 1.0\n",
    );

    let output_3mf = dir.path().join("multi_output.3mf");
    let out = run_cli(&[
        "convert",
        dir.path().join("multi.obj").to_str().unwrap(),
        output_3mf.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "convert with multiple materials should succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stats_out = run_cli(&["stats", output_3mf.to_str().unwrap(), "--format", "json"]);
    assert!(stats_out.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&stats_out.stdout).expect("Failed to parse stats JSON");
    let base_count = json["materials"]["base_materials_count"]
        .as_u64()
        .unwrap_or(0);
    assert!(
        base_count >= 1,
        "Multiple materials should produce at least 1 base materials group, got {base_count}"
    );
}

/// Test 9: Groups combined with materials.
#[test]
fn test_obj_groups_with_materials() {
    let dir = TempDir::new().unwrap();

    write_test_obj(
        dir.path(),
        "group_mat.obj",
        "mtllib group_mat.mtl\n\
         g Part1\n\
         usemtl Red\n\
         v 0 0 0\n\
         v 10 0 0\n\
         v 0 10 0\n\
         f 1 2 3\n\
         g Part2\n\
         usemtl Blue\n\
         v 20 0 0\n\
         v 30 0 0\n\
         v 20 10 0\n\
         f 4 5 6\n",
    );
    write_test_mtl(
        dir.path(),
        "group_mat.mtl",
        "newmtl Red\nKd 1.0 0.0 0.0\n\nnewmtl Blue\nKd 0.0 0.0 1.0\n",
    );

    let output_3mf = dir.path().join("group_mat_output.3mf");
    let out = run_cli(&[
        "convert",
        dir.path().join("group_mat.obj").to_str().unwrap(),
        output_3mf.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "convert with groups+materials should succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stats_out = run_cli(&["stats", output_3mf.to_str().unwrap(), "--format", "json"]);
    assert!(stats_out.status.success());
    let json: serde_json::Value =
        serde_json::from_slice(&stats_out.stdout).expect("Failed to parse stats JSON");

    let obj_count = json["geometry"]["object_count"].as_u64().unwrap_or(0);
    assert!(
        obj_count >= 2,
        "Two groups should produce at least 2 objects, got {obj_count}"
    );

    let base_count = json["materials"]["base_materials_count"]
        .as_u64()
        .unwrap_or(0);
    assert!(
        base_count >= 1,
        "Groups with materials should have base materials, got {base_count}"
    );
}

/// Test 10: Batch command processes OBJ with materials.
#[test]
fn test_batch_obj_with_materials() {
    let dir = TempDir::new().unwrap();

    write_test_obj(
        dir.path(),
        "batch_test.obj",
        "mtllib batch_test.mtl\n\
         usemtl Red\n\
         v 0 0 0\n\
         v 10 0 0\n\
         v 0 10 0\n\
         f 1 2 3\n",
    );
    write_test_mtl(dir.path(), "batch_test.mtl", "newmtl Red\nKd 1.0 0.0 0.0\n");

    let out = run_cli(&[
        "batch",
        "--stats",
        "--format",
        "json",
        dir.path().join("batch_test.obj").to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "batch stats on OBJ should succeed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Failed to parse batch JSON output");
    let base_count = json["operations"]["stats"]["materials"]["base_materials_count"]
        .as_u64()
        .unwrap_or(0);
    assert_eq!(
        base_count, 1,
        "Batch stats should show 1 base materials group for OBJ with MTL, got {base_count}"
    );
}
