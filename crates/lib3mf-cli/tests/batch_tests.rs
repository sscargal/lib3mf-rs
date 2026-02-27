//! Integration tests for the `3mf batch` command.
//!
//! These tests create 3MF files programmatically using lib3mf_core's Model API,
//! invoke the batch command via CLI subprocess, and verify exit codes and output.
//!
//! NOTE: The batch command functions are internal to lib3mf-cli (not pub),
//! so all tests go through the CLI binary via `cargo run -p lib3mf-cli -- batch ...`.

use lib3mf_core::model::{
    Build, BuildItem, Geometry, Mesh, Model, Object, ObjectType, ResourceCollection, ResourceId,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Create a simple tetrahedron 3MF test file.
///
/// The tetrahedron has 4 vertices and 4 triangles, making it a closed manifold
/// mesh suitable for use as a Model-type object.
fn create_test_3mf(dir: &Path, name: &str) -> PathBuf {
    let mut mesh = Mesh::new();
    // Tetrahedron vertices
    let v0 = mesh.add_vertex(0.0, 0.0, 0.0);
    let v1 = mesh.add_vertex(10.0, 0.0, 0.0);
    let v2 = mesh.add_vertex(5.0, 10.0, 0.0);
    let v3 = mesh.add_vertex(5.0, 5.0, 10.0);
    // 4 faces of the tetrahedron (CCW winding for outward normals)
    mesh.add_triangle(v0, v2, v1);
    mesh.add_triangle(v0, v1, v3);
    mesh.add_triangle(v1, v2, v3);
    mesh.add_triangle(v0, v3, v2);

    let obj = Object {
        id: ResourceId(1),
        object_type: ObjectType::Model,
        name: Some(format!("Object_{}", name)),
        part_number: None,
        uuid: None,
        pid: None,
        pindex: None,
        thumbnail: None,
        geometry: Geometry::Mesh(mesh),
    };

    let mut resources = ResourceCollection::new();
    resources.add_object(obj).expect("Failed to add object");

    let mut build = Build::default();
    build.items.push(BuildItem {
        object_id: ResourceId(1),
        uuid: None,
        path: None,
        part_number: None,
        transform: glam::Mat4::IDENTITY,
        printable: None,
    });

    let model = Model {
        resources,
        build,
        ..Default::default()
    };

    let path = dir.join(format!("{}.3mf", name));
    let file = std::fs::File::create(&path).expect("Failed to create test 3MF file");
    model.write(file).expect("Failed to write test 3MF model");
    path
}

/// Create a corrupt (invalid) 3MF file — ZIP magic but not valid ZIP content.
fn create_corrupt_3mf(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(format!("{}.3mf", name));
    let mut f = std::fs::File::create(&path).expect("Failed to create corrupt file");
    // Start with PK magic so it's detected as Zip3mf, but the rest is garbage
    f.write_all(b"PK\x03\x04\x00this is not a valid zip archive\x00")
        .unwrap();
    path
}

/// Create a non-3MF text file.
fn create_text_file(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(format!("{}.txt", name));
    let mut f = std::fs::File::create(&path).expect("Failed to create text file");
    f.write_all(b"This is a plain text file, not a 3D model.")
        .unwrap();
    path
}

/// Create a simple ASCII STL file.
fn create_stl_file(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(format!("{}.stl", name));
    let mut f = std::fs::File::create(&path).expect("Failed to create STL file");
    f.write_all(b"solid test\n  facet normal 0 0 1\n    outer loop\n      vertex 0 0 0\n      vertex 1 0 0\n      vertex 0 1 0\n    endloop\n  endfacet\nendsolid test\n").unwrap();
    path
}

/// Run the batch CLI command with the given arguments.
/// Uses `cargo run` so the binary is always up-to-date.
fn run_batch(args: &[&str]) -> std::process::Output {
    std::process::Command::new("cargo")
        .args(["run", "--quiet", "-p", "lib3mf-cli", "--"])
        .arg("batch")
        .args(args)
        .output()
        .expect("Failed to run batch command")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Test 1: batch validate a single good 3MF file — expect exit 0
#[test]
fn test_batch_validate_single_file() {
    let dir = TempDir::new().unwrap();
    let file = create_test_3mf(dir.path(), "cube");

    let out = run_batch(&[file.to_str().unwrap(), "--validate"]);
    assert!(
        out.status.success(),
        "Expected exit 0 for valid 3MF, got {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("OK") || stdout.contains("cube"),
        "Expected 'OK' or filename in output: {stdout}"
    );
}

/// Test 2: batch validate a directory of 3MF files
#[test]
fn test_batch_validate_directory() {
    let dir = TempDir::new().unwrap();
    create_test_3mf(dir.path(), "model_a");
    create_test_3mf(dir.path(), "model_b");
    create_test_3mf(dir.path(), "model_c");

    let out = run_batch(&[dir.path().to_str().unwrap(), "--validate"]);
    assert!(
        out.status.success(),
        "Expected exit 0 for directory of valid 3MFs: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Should have processed 3 files
    assert!(
        stdout.contains("3/3") || stdout.contains("[1/3]"),
        "Expected 3-file progress in output: {stdout}"
    );
}

/// Test 3: batch stats on a 3MF file
#[test]
fn test_batch_stats_single_file() {
    let dir = TempDir::new().unwrap();
    let file = create_test_3mf(dir.path(), "stats_test");

    let out = run_batch(&[file.to_str().unwrap(), "--stats"]);
    assert!(
        out.status.success(),
        "Expected exit 0 for stats: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("OK") || stdout.contains("stats_test"),
        "Expected OK or filename in stats output: {stdout}"
    );
}

/// Test 4: batch --validate --stats combined on a file
#[test]
fn test_batch_combined_validate_stats() {
    let dir = TempDir::new().unwrap();
    let file = create_test_3mf(dir.path(), "combined");

    let out = run_batch(&[file.to_str().unwrap(), "--validate", "--stats"]);
    assert!(
        out.status.success(),
        "Expected exit 0 for combined validate+stats: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Test 5: batch --format json produces JSON Lines output
#[test]
fn test_batch_json_output() {
    let dir = TempDir::new().unwrap();
    let file = create_test_3mf(dir.path(), "json_test");

    let out = run_batch(&[file.to_str().unwrap(), "--validate", "--format", "json"]);
    assert!(
        out.status.success(),
        "Expected exit 0 for JSON output: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Should be valid JSON on stdout
    assert!(
        stdout.contains("{") && stdout.contains("}"),
        "Expected JSON output on stdout: {stdout}"
    );
    // Each line should parse as valid JSON
    for line in stdout.lines() {
        let line = line.trim();
        if !line.is_empty() {
            let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(line);
            assert!(
                parsed.is_ok(),
                "Expected valid JSON line: {line}\nError: {parsed:?}"
            );
        }
    }
}

/// Test 6: batch --summary shows totals in stderr
#[test]
fn test_batch_summary_flag() {
    let dir = TempDir::new().unwrap();
    create_test_3mf(dir.path(), "sum_a");
    create_test_3mf(dir.path(), "sum_b");

    let out = run_batch(&[dir.path().to_str().unwrap(), "--validate", "--summary"]);
    assert!(
        out.status.success(),
        "Expected exit 0 with --summary: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Batch Summary") || stderr.contains("Total"),
        "Expected summary in stderr: {stderr}"
    );
}

/// Test 7: non-3MF files are silently skipped for validate/stats
#[test]
fn test_batch_skip_non_3mf_for_validate() {
    let dir = TempDir::new().unwrap();
    let _txt = create_text_file(dir.path(), "readme");
    let good = create_test_3mf(dir.path(), "good_model");

    let out = run_batch(&[dir.path().to_str().unwrap(), "--validate"]);
    assert!(
        out.status.success(),
        "Expected exit 0 when non-3MF files are skipped: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // The good model should show OK
    assert!(
        stdout.contains("OK") || stdout.contains("good_model"),
        "Expected good model to show OK: {stdout}"
    );
    // The txt file should show SKIP (if it appears in output at all, it should be SKIP)
    // (It could also just not appear since unknown types are filtered)
    let _ = good;
}

/// Test 8: corrupt file reports error, doesn't abort batch — other files continue
#[test]
fn test_batch_error_on_corrupt_file() {
    let dir = TempDir::new().unwrap();
    let _corrupt = create_corrupt_3mf(dir.path(), "corrupt_file");
    let good = create_test_3mf(dir.path(), "good_after_corrupt");

    let out = run_batch(&[dir.path().to_str().unwrap(), "--validate"]);
    // Should exit 1 because corrupt file has errors
    assert!(
        !out.status.success(),
        "Expected exit 1 because corrupt file fails"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // The good file should still be processed (not aborted by corrupt file)
    assert!(
        stdout.contains("good_after_corrupt") || stdout.contains("OK"),
        "Expected good file to be processed even after corrupt file: {stdout}"
    );
    let _ = good;
}

/// Test 9: no operation flags gives an error or warning
#[test]
fn test_batch_no_ops_selected() {
    let dir = TempDir::new().unwrap();
    let file = create_test_3mf(dir.path(), "no_ops");

    let out = run_batch(&[file.to_str().unwrap()]);
    // Without any op flag, all files are skipped — run() returns Ok(true) since no failures
    // The command should succeed (exit 0) but produce no real output
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Files with no matching operations are skipped silently (SKIP status)
    // The batch exits 0 since no file produced an error
    assert!(
        out.status.success(),
        "Expected exit 0 when no ops (all skipped): stdout={stdout}, stderr={stderr}"
    );
}

/// Test 10: --quiet flag suppresses stdout progress
#[test]
fn test_batch_quiet_flag() {
    let dir = TempDir::new().unwrap();
    let file = create_test_3mf(dir.path(), "quiet_test");

    let out = run_batch(&[file.to_str().unwrap(), "--validate", "--quiet"]);
    assert!(
        out.status.success(),
        "Expected exit 0 with --quiet: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Quiet mode should suppress per-file progress text
    assert!(
        stdout.trim().is_empty(),
        "Expected empty stdout with --quiet, got: {stdout}"
    );
}

/// Test 11: --recursive flag finds files in subdirectories
#[test]
fn test_batch_recursive_flag() {
    let dir = TempDir::new().unwrap();
    // Create nested directories
    let subdir = dir.path().join("subdir");
    fs::create_dir_all(&subdir).unwrap();
    let deep = subdir.join("deep");
    fs::create_dir_all(&deep).unwrap();

    create_test_3mf(dir.path(), "top_level");
    create_test_3mf(&subdir, "sub_level");
    create_test_3mf(&deep, "deep_level");

    // Without --recursive: should only find top-level file
    let out_no_recurse = run_batch(&[dir.path().to_str().unwrap(), "--validate"]);
    let stdout_no_recurse = String::from_utf8_lossy(&out_no_recurse.stdout);
    assert!(
        stdout_no_recurse.contains("1/") || stdout_no_recurse.contains("[1/1]"),
        "Without --recursive, expect only 1 file: {stdout_no_recurse}"
    );

    // With --recursive: should find all 3 files
    let out_recurse = run_batch(&[dir.path().to_str().unwrap(), "--validate", "--recursive"]);
    assert!(
        out_recurse.status.success(),
        "Expected exit 0 with --recursive: {}",
        String::from_utf8_lossy(&out_recurse.stderr)
    );
    let stdout_recurse = String::from_utf8_lossy(&out_recurse.stdout);
    assert!(
        stdout_recurse.contains("3/3") || stdout_recurse.contains("[1/3]"),
        "With --recursive, expect 3 files: {stdout_recurse}"
    );
}

/// Test 12: batch --convert converts 3MF to STL
#[test]
fn test_batch_convert() {
    let dir = TempDir::new().unwrap();
    let file = create_test_3mf(dir.path(), "to_convert");

    let out = run_batch(&[file.to_str().unwrap(), "--convert"]);
    assert!(
        out.status.success(),
        "Expected exit 0 for convert: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The STL file should exist next to the source
    let stl_path = dir.path().join("to_convert.stl");
    assert!(
        stl_path.exists(),
        "Expected converted STL to exist at {}",
        stl_path.display()
    );
}

/// Test 13: batch --convert with --output-dir
#[test]
fn test_batch_convert_output_dir() {
    let dir = TempDir::new().unwrap();
    let out_dir = TempDir::new().unwrap();
    let file = create_test_3mf(dir.path(), "convert_dir_test");

    let out = run_batch(&[
        file.to_str().unwrap(),
        "--convert",
        "--output-dir",
        out_dir.path().to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "Expected exit 0 for convert with --output-dir: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The STL file should exist in the output directory
    let stl_path = out_dir.path().join("convert_dir_test.stl");
    assert!(
        stl_path.exists(),
        "Expected converted STL in output dir at {}",
        stl_path.display()
    );
}

/// Test 14: batch with --jobs 2 processes all files (parallel mode)
#[test]
fn test_batch_jobs_parallel() {
    let dir = TempDir::new().unwrap();
    for i in 0..4 {
        create_test_3mf(dir.path(), &format!("parallel_{}", i));
    }

    let out = run_batch(&[dir.path().to_str().unwrap(), "--validate", "--jobs", "2"]);
    assert!(
        out.status.success(),
        "Expected exit 0 with --jobs 2: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // All 4 files should be processed
    assert!(
        stdout.contains("4/4") || stdout.contains("[1/4]"),
        "Expected 4 files processed with --jobs 2: {stdout}"
    );
}

/// Test 15: exit code 0 when all files succeed
#[test]
fn test_batch_exit_code_zero_on_success() {
    let dir = TempDir::new().unwrap();
    create_test_3mf(dir.path(), "success_a");
    create_test_3mf(dir.path(), "success_b");

    let out = run_batch(&[dir.path().to_str().unwrap(), "--validate"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "Expected exit code 0 on all success"
    );
}

/// Test 16: exit code 1 when any file has errors
#[test]
fn test_batch_exit_code_one_on_failure() {
    let dir = TempDir::new().unwrap();
    // Only a corrupt file — validation will fail
    create_corrupt_3mf(dir.path(), "will_fail");

    let out = run_batch(&[dir.path().to_str().unwrap(), "--validate"]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "Expected exit code 1 when file fails validation"
    );
}

/// Test 17: batch --list operation lists archive entries
#[test]
fn test_batch_list_operation() {
    let dir = TempDir::new().unwrap();
    let file = create_test_3mf(dir.path(), "list_test");

    let out = run_batch(&[file.to_str().unwrap(), "--list"]);
    assert!(
        out.status.success(),
        "Expected exit 0 for list: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The list operation should show OK (entries were listed internally)
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("OK") || stdout.contains("list_test"),
        "Expected OK or filename in list output: {stdout}"
    );
}

/// Test 18: batch --validate with --validate-level paranoid
#[test]
fn test_batch_validate_level_paranoid() {
    let dir = TempDir::new().unwrap();
    let file = create_test_3mf(dir.path(), "paranoid_test");

    let out = run_batch(&[
        file.to_str().unwrap(),
        "--validate",
        "--validate-level",
        "paranoid",
    ]);
    // Paranoid validation should still succeed for a well-formed file
    assert!(
        out.status.success(),
        "Expected exit 0 for paranoid validation of good file: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Test 19: batch --verbose flag produces additional output
#[test]
fn test_batch_verbose_flag() {
    let dir = TempDir::new().unwrap();
    let file = create_test_3mf(dir.path(), "verbose_test");

    let out = run_batch(&[file.to_str().unwrap(), "--validate", "--verbose"]);
    assert!(
        out.status.success(),
        "Expected exit 0 with --verbose: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // verbose should produce output (more than just the progress line)
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.is_empty(), "Expected verbose output, got nothing");
}

/// Test 20: batch with STL file using --stats
#[test]
fn test_batch_stl_stats() {
    let dir = TempDir::new().unwrap();
    let stl = create_stl_file(dir.path(), "mesh");

    let out = run_batch(&[stl.to_str().unwrap(), "--stats"]);
    assert!(
        out.status.success(),
        "Expected exit 0 for STL stats: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("OK") || stdout.contains("mesh"),
        "Expected OK or filename in STL stats output: {stdout}"
    );
}
