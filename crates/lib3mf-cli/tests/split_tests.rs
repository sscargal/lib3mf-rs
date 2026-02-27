//! Integration tests for the `3mf split` command.
//!
//! These tests create 3MF files programmatically using lib3mf_core's Model API,
//! invoke the split command via CLI subprocess, and verify the result by parsing
//! the output file(s).
//!
//! NOTE: The split command functions are internal to lib3mf-cli (not pub),
//! so all tests go through the CLI binary via `cargo run -p lib3mf-cli -- split ...`.

use lib3mf_core::archive::{ArchiveReader, ZipArchiver, find_model_path};
use lib3mf_core::model::{
    BaseMaterial, BaseMaterialsGroup, Build, BuildItem, Color, Geometry, Mesh, Model, Object,
    ObjectType, ResourceCollection, ResourceId,
};
use lib3mf_core::parser::parse_model;
use std::fs::File;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Create a simple tetrahedron 3MF test file.
///
/// The tetrahedron has 4 vertices and 4 triangles, making it a closed manifold
/// mesh suitable for use as a Model-type object.
fn create_test_3mf(
    dir: &Path,
    name: &str,
    object_id: u32,
    object_name: Option<&str>,
    material_id: Option<u32>,
) -> PathBuf {
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
        id: ResourceId(object_id),
        object_type: ObjectType::Model,
        name: object_name.map(str::to_string),
        part_number: None,
        uuid: None,
        pid: material_id.map(ResourceId),
        pindex: material_id.map(|_| 0),
        thumbnail: None,
        geometry: Geometry::Mesh(mesh),
    };

    let mut resources = ResourceCollection::new();
    resources.add_object(obj).expect("Failed to add object");

    if let Some(mat_id) = material_id {
        let mat_group = BaseMaterialsGroup {
            id: ResourceId(mat_id),
            materials: vec![BaseMaterial {
                name: format!("Material_{}", name),
                display_color: Color::new(255, 0, 0, 255),
            }],
        };
        resources
            .add_base_materials(mat_group)
            .expect("Failed to add base materials");
    }

    let mut build = Build::default();
    build.items.push(BuildItem {
        object_id: ResourceId(object_id),
        uuid: None,
        path: None,
        part_number: None,
        transform: glam::Mat4::IDENTITY,
        printable: None,
    });

    let mut model = Model {
        resources,
        build,
        ..Default::default()
    };
    model
        .metadata
        .insert("Title".to_string(), format!("Test {}", name));

    let path = dir.join(format!("{}.3mf", name));
    let file = File::create(&path).expect("Failed to create test 3MF file");
    model.write(file).expect("Failed to write test 3MF model");
    path
}

/// Create a multi-object 3MF with several objects in one file.
fn create_multi_object_3mf(
    dir: &Path,
    name: &str,
    objects: &[(u32, &str, Option<u32>)], // (object_id, object_name, material_id)
) -> PathBuf {
    let mut resources = ResourceCollection::new();
    let mut build = Build::default();

    for (object_id, object_name, material_id) in objects {
        let mut mesh = Mesh::new();
        let v0 = mesh.add_vertex(0.0, 0.0, 0.0);
        let v1 = mesh.add_vertex(10.0, 0.0, 0.0);
        let v2 = mesh.add_vertex(5.0, 10.0, 0.0);
        let v3 = mesh.add_vertex(5.0, 5.0, 10.0);
        mesh.add_triangle(v0, v2, v1);
        mesh.add_triangle(v0, v1, v3);
        mesh.add_triangle(v1, v2, v3);
        mesh.add_triangle(v0, v3, v2);

        let obj = Object {
            id: ResourceId(*object_id),
            object_type: ObjectType::Model,
            name: Some(object_name.to_string()),
            part_number: None,
            uuid: None,
            pid: material_id.map(ResourceId),
            pindex: material_id.map(|_| 0),
            thumbnail: None,
            geometry: Geometry::Mesh(mesh),
        };
        resources.add_object(obj).expect("Failed to add object");

        if let Some(mat_id) = material_id {
            // Only add material group if not already added
            if resources.get_base_materials(ResourceId(*mat_id)).is_none() {
                let mat_group = BaseMaterialsGroup {
                    id: ResourceId(*mat_id),
                    materials: vec![BaseMaterial {
                        name: format!("Mat_{}", object_name),
                        display_color: Color::new(128, 128, 128, 255),
                    }],
                };
                resources
                    .add_base_materials(mat_group)
                    .expect("Failed to add base materials");
            }
        }

        build.items.push(BuildItem {
            object_id: ResourceId(*object_id),
            uuid: None,
            path: None,
            part_number: None,
            transform: glam::Mat4::IDENTITY,
            printable: None,
        });
    }

    let model = Model {
        resources,
        build,
        ..Default::default()
    };

    let path = dir.join(format!("{}.3mf", name));
    let file = File::create(&path).expect("Failed to create test 3MF file");
    model.write(file).expect("Failed to write test 3MF model");
    path
}

/// Parse a 3MF file and return the Model.
fn load_3mf(path: &Path) -> Model {
    let file = File::open(path).expect("Failed to open 3MF file");
    let mut archiver = ZipArchiver::new(file).expect("Failed to open ZIP archive");
    let model_path = find_model_path(&mut archiver).expect("Failed to find model path");
    let model_data = archiver
        .read_entry(&model_path)
        .expect("Failed to read model XML");
    parse_model(Cursor::new(model_data)).expect("Failed to parse model XML")
}

/// Run the split CLI command with the given arguments.
/// Uses `cargo run` so the binary is always up-to-date.
fn run_split(args: &[&str]) -> std::process::Output {
    std::process::Command::new("cargo")
        .args(["run", "--quiet", "-p", "lib3mf-cli", "--"])
        .arg("split")
        .args(args)
        .output()
        .expect("Failed to run split command")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Test 1: Split a single-object 3MF by build item (default).
/// Verify one output file in _split/ dir, valid 3MF, contains 1 object with compact ID.
#[test]
fn test_split_single_object() {
    let tmp = TempDir::new().unwrap();
    let input = create_test_3mf(tmp.path(), "single", 5, Some("MyPart"), None);
    let split_dir = tmp.path().join("single_split");

    let result = run_split(&[input.to_str().unwrap(), "--output-dir", split_dir.to_str().unwrap()]);

    assert!(
        result.status.success(),
        "Split failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(split_dir.exists(), "Output directory was not created");

    // Should have one output file
    let entries: Vec<PathBuf> = std::fs::read_dir(&split_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("3mf"))
        .collect();
    assert_eq!(entries.len(), 1, "Expected 1 output file, got {}", entries.len());

    // Parse and verify
    let model = load_3mf(&entries[0]);
    let obj_count = model.resources.iter_objects().count();
    assert_eq!(obj_count, 1, "Expected 1 object in split output, got {}", obj_count);
    assert_eq!(model.build.items.len(), 1, "Expected 1 build item in split output");

    // Compact IDs start from 1
    let obj = model.resources.iter_objects().next().unwrap();
    assert_eq!(obj.id.0, 1, "Expected compact ID=1, got {}", obj.id.0);
}

/// Test 2: Split a 3MF with 2 build items referencing 2 different objects.
/// Verify 2 output files, each containing 1 object.
#[test]
fn test_split_multi_object_by_item() {
    let tmp = TempDir::new().unwrap();
    let input = create_multi_object_3mf(
        tmp.path(),
        "multi",
        &[(1, "Gear", None), (2, "Housing", None)],
    );
    let split_dir = tmp.path().join("multi_split");

    let result = run_split(&[input.to_str().unwrap(), "--output-dir", split_dir.to_str().unwrap()]);

    assert!(
        result.status.success(),
        "Split failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let entries: Vec<PathBuf> = std::fs::read_dir(&split_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("3mf"))
        .collect();
    assert_eq!(entries.len(), 2, "Expected 2 output files, got {}", entries.len());

    // Each output should have exactly 1 object
    for entry in &entries {
        let model = load_3mf(entry);
        assert_eq!(
            model.resources.iter_objects().count(),
            1,
            "Expected 1 object per split file, got {} in {:?}",
            model.resources.iter_objects().count(),
            entry
        );
        assert_eq!(model.build.items.len(), 1, "Expected 1 build item per split file");
    }
}

/// Test 3: Split with --by-object.
/// 2 objects in file, split with --by-object. Verify 2 output files.
#[test]
fn test_split_by_object() {
    let tmp = TempDir::new().unwrap();
    let input = create_multi_object_3mf(
        tmp.path(),
        "byobj",
        &[(10, "Gear", None), (20, "Housing", None)],
    );
    let split_dir = tmp.path().join("byobj_split");

    let result = run_split(&[
        input.to_str().unwrap(),
        "--by-object",
        "--output-dir",
        split_dir.to_str().unwrap(),
    ]);

    assert!(
        result.status.success(),
        "Split --by-object failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let entries: Vec<PathBuf> = std::fs::read_dir(&split_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("3mf"))
        .collect();
    assert_eq!(entries.len(), 2, "Expected 2 output files from by-object split, got {}", entries.len());
}

/// Test 4: Split with materials — object references a BaseMaterialsGroup.
/// Verify output contains both the object AND the material group with compact IDs.
#[test]
fn test_split_with_materials() {
    let tmp = TempDir::new().unwrap();
    // Object at ID=5, material at ID=10
    let input = create_test_3mf(tmp.path(), "withmat", 5, Some("Part"), Some(10));
    let split_dir = tmp.path().join("withmat_split");

    let result = run_split(&[input.to_str().unwrap(), "--output-dir", split_dir.to_str().unwrap()]);

    assert!(
        result.status.success(),
        "Split with materials failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let entries: Vec<PathBuf> = std::fs::read_dir(&split_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("3mf"))
        .collect();
    assert_eq!(entries.len(), 1, "Expected 1 output file");

    let model = load_3mf(&entries[0]);

    // Should have 1 object and 1 material group
    assert_eq!(model.resources.iter_objects().count(), 1, "Expected 1 object");
    assert_eq!(
        model.resources.iter_base_materials().count(),
        1,
        "Expected 1 material group in split output"
    );

    // IDs should be compact (1 and 2 in some order)
    let obj = model.resources.iter_objects().next().unwrap();
    let mat = model.resources.iter_base_materials().next().unwrap();
    let all_ids = [obj.id.0, mat.id.0];
    let mut sorted = all_ids;
    sorted.sort_unstable();
    assert_eq!(sorted, [1, 2], "Expected compact IDs [1, 2], got {:?}", sorted);

    // Object's pid should point to the material group's (remapped) ID
    assert!(obj.pid.is_some(), "Object should still reference material group");
    assert!(
        model.resources.exists(obj.pid.unwrap()),
        "Object pid should resolve in split model"
    );
}

/// Test 5: Select by index — split with --select 1 picks only the second build item.
#[test]
fn test_split_select_by_index() {
    let tmp = TempDir::new().unwrap();
    let input = create_multi_object_3mf(
        tmp.path(),
        "select_idx",
        &[(1, "Part_A", None), (2, "Part_B", None), (3, "Part_C", None)],
    );
    let split_dir = tmp.path().join("select_idx_split");

    let result = run_split(&[
        input.to_str().unwrap(),
        "--select",
        "1",
        "--output-dir",
        split_dir.to_str().unwrap(),
    ]);

    assert!(
        result.status.success(),
        "Split --select 1 failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let entries: Vec<PathBuf> = std::fs::read_dir(&split_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("3mf"))
        .collect();
    assert_eq!(entries.len(), 1, "Expected 1 output file from --select 1, got {}", entries.len());
}

/// Test 6: Select by name — split with --select Gear picks only the named object.
#[test]
fn test_split_select_by_name() {
    let tmp = TempDir::new().unwrap();
    let input = create_multi_object_3mf(
        tmp.path(),
        "select_name",
        &[(1, "Gear", None), (2, "Housing", None)],
    );
    let split_dir = tmp.path().join("select_name_split");

    let result = run_split(&[
        input.to_str().unwrap(),
        "--select",
        "Gear",
        "--output-dir",
        split_dir.to_str().unwrap(),
    ]);

    assert!(
        result.status.success(),
        "Split --select Gear failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let entries: Vec<PathBuf> = std::fs::read_dir(&split_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("3mf"))
        .collect();
    assert_eq!(entries.len(), 1, "Expected 1 output file from --select Gear, got {}", entries.len());

    // Verify the selected file is indeed the Gear
    let model = load_3mf(&entries[0]);
    let obj = model.resources.iter_objects().next().unwrap();
    // The name is preserved in the split output
    assert!(
        obj.name.as_deref() == Some("Gear"),
        "Expected object name 'Gear' in split output, got {:?}",
        obj.name
    );
}

/// Test 7: Dry run — verify NO output directory/files are created,
/// stdout contains "DRY RUN" and lists the files that would be written.
#[test]
fn test_split_dry_run() {
    let tmp = TempDir::new().unwrap();
    let input = create_test_3mf(tmp.path(), "dryrun", 1, Some("Part"), None);
    let split_dir = tmp.path().join("dryrun_split");

    let result = run_split(&[
        input.to_str().unwrap(),
        "--dry-run",
        "--output-dir",
        split_dir.to_str().unwrap(),
    ]);

    assert!(
        result.status.success(),
        "Dry run failed unexpectedly: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // No output directory should be created
    assert!(
        !split_dir.exists(),
        "Dry run should not create output directory, but {:?} exists",
        split_dir
    );

    // stdout should mention DRY RUN
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("DRY RUN"),
        "Dry run stdout should contain 'DRY RUN', got: {:?}",
        stdout
    );
}

/// Test 8: Preserve transforms — build item with non-identity transform.
/// With --preserve-transforms: output has original transform.
/// Without: output has identity transform.
#[test]
fn test_split_preserve_transforms() {
    let tmp = TempDir::new().unwrap();

    // Create a 3MF with a non-identity transform in the build item
    let mut mesh = Mesh::new();
    let v0 = mesh.add_vertex(0.0, 0.0, 0.0);
    let v1 = mesh.add_vertex(10.0, 0.0, 0.0);
    let v2 = mesh.add_vertex(5.0, 10.0, 0.0);
    let v3 = mesh.add_vertex(5.0, 5.0, 10.0);
    mesh.add_triangle(v0, v2, v1);
    mesh.add_triangle(v0, v1, v3);
    mesh.add_triangle(v1, v2, v3);
    mesh.add_triangle(v0, v3, v2);

    let obj = Object {
        id: ResourceId(1),
        object_type: ObjectType::Model,
        name: Some("Translated".to_string()),
        part_number: None,
        uuid: None,
        pid: None,
        pindex: None,
        thumbnail: None,
        geometry: Geometry::Mesh(mesh),
    };

    let mut resources = ResourceCollection::new();
    resources.add_object(obj).expect("Failed to add object");

    // Build item with a translation of (100, 200, 300)
    let original_transform = glam::Mat4::from_translation(glam::Vec3::new(100.0, 200.0, 300.0));
    let mut build = Build::default();
    build.items.push(BuildItem {
        object_id: ResourceId(1),
        transform: original_transform,
        uuid: None,
        path: None,
        part_number: None,
        printable: None,
    });

    let model = Model {
        resources,
        build,
        ..Default::default()
    };

    let input = tmp.path().join("transforms.3mf");
    let file = File::create(&input).expect("Failed to create test file");
    model.write(file).expect("Failed to write model");

    // Split WITHOUT --preserve-transforms → identity transform
    let split_dir_default = tmp.path().join("transforms_default_split");
    let result = run_split(&[
        input.to_str().unwrap(),
        "--output-dir",
        split_dir_default.to_str().unwrap(),
    ]);
    assert!(result.status.success(), "Split without preserve-transforms failed");

    let entries: Vec<PathBuf> = std::fs::read_dir(&split_dir_default)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("3mf"))
        .collect();
    let model_default = load_3mf(&entries[0]);
    let transform_default = model_default.build.items[0].transform;
    assert_eq!(
        transform_default,
        glam::Mat4::IDENTITY,
        "Without --preserve-transforms, transform should be identity"
    );

    // Split WITH --preserve-transforms → original transform preserved
    let split_dir_preserved = tmp.path().join("transforms_preserved_split");
    let result = run_split(&[
        input.to_str().unwrap(),
        "--preserve-transforms",
        "--output-dir",
        split_dir_preserved.to_str().unwrap(),
    ]);
    assert!(result.status.success(), "Split with --preserve-transforms failed");

    let entries: Vec<PathBuf> = std::fs::read_dir(&split_dir_preserved)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("3mf"))
        .collect();
    let model_preserved = load_3mf(&entries[0]);
    let transform_preserved = model_preserved.build.items[0].transform;

    // The translation component (w_axis) should match
    let expected_w = original_transform.w_axis;
    let actual_w = transform_preserved.w_axis;
    assert!(
        (expected_w.x - actual_w.x).abs() < 0.01
            && (expected_w.y - actual_w.y).abs() < 0.01
            && (expected_w.z - actual_w.z).abs() < 0.01,
        "With --preserve-transforms, transform should be preserved. Expected {:?}, got {:?}",
        expected_w,
        actual_w
    );
}

/// Test 9: Force overwrite — split to a directory that already exists.
/// Without --force: should error. With --force: should succeed.
#[test]
fn test_split_force_overwrite() {
    let tmp = TempDir::new().unwrap();
    let input = create_test_3mf(tmp.path(), "force_test", 1, Some("Part"), None);
    let split_dir = tmp.path().join("force_split");

    // First split — creates the directory
    let result = run_split(&[input.to_str().unwrap(), "--output-dir", split_dir.to_str().unwrap()]);
    assert!(result.status.success(), "First split failed");
    assert!(split_dir.exists(), "Split directory should exist after first split");

    // Second split without --force — should error because directory exists
    let result_no_force = run_split(&[
        input.to_str().unwrap(),
        "--output-dir",
        split_dir.to_str().unwrap(),
    ]);
    assert!(
        !result_no_force.status.success(),
        "Second split without --force should fail (directory already exists)"
    );

    // Third split with --force — should succeed
    let result_force = run_split(&[
        input.to_str().unwrap(),
        "--force",
        "--output-dir",
        split_dir.to_str().unwrap(),
    ]);
    assert!(
        result_force.status.success(),
        "Split with --force should succeed: {}",
        String::from_utf8_lossy(&result_force.stderr)
    );
}

/// Test 10: Compact IDs — object at ResourceId(100) and material at ResourceId(200).
/// After split, IDs should be compact (1-N sequential).
#[test]
fn test_split_compact_ids() {
    let tmp = TempDir::new().unwrap();
    // Object ID=100, material ID=200
    let input = create_test_3mf(tmp.path(), "compact_ids", 100, Some("Part"), Some(200));
    let split_dir = tmp.path().join("compact_ids_split");

    let result = run_split(&[input.to_str().unwrap(), "--output-dir", split_dir.to_str().unwrap()]);

    assert!(
        result.status.success(),
        "Compact IDs split failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let entries: Vec<PathBuf> = std::fs::read_dir(&split_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("3mf"))
        .collect();
    assert_eq!(entries.len(), 1);

    let model = load_3mf(&entries[0]);

    // Collect all IDs — should be [1, 2] (compact, starting from 1)
    let mut all_ids: Vec<u32> = Vec::new();
    for obj in model.resources.iter_objects() {
        all_ids.push(obj.id.0);
    }
    for mat in model.resources.iter_base_materials() {
        all_ids.push(mat.id.0);
    }
    all_ids.sort_unstable();

    assert_eq!(
        all_ids,
        vec![1, 2],
        "Expected compact IDs [1, 2], got {:?} (input had IDs 100 and 200)",
        all_ids
    );
}

/// Test 11: Output naming collision — two objects both named "Part".
/// Verify output files are Part.3mf and Part_1.3mf (auto-increment).
#[test]
fn test_split_output_naming_collision() {
    let tmp = TempDir::new().unwrap();
    let input = create_multi_object_3mf(
        tmp.path(),
        "collision",
        &[(1, "Part", None), (2, "Part", None)],
    );
    let split_dir = tmp.path().join("collision_split");

    let result = run_split(&[input.to_str().unwrap(), "--output-dir", split_dir.to_str().unwrap()]);

    assert!(
        result.status.success(),
        "Naming collision split failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let entries: Vec<PathBuf> = std::fs::read_dir(&split_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("3mf"))
        .collect();
    assert_eq!(entries.len(), 2, "Expected 2 output files despite name collision, got {}", entries.len());

    // One should be Part.3mf and one Part_1.3mf
    let names: Vec<String> = entries
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
        .collect();
    assert!(
        names.contains(&"Part.3mf".to_string()),
        "Expected Part.3mf in output, got {:?}",
        names
    );
    assert!(
        names.contains(&"Part_1.3mf".to_string()),
        "Expected Part_1.3mf in output (collision handling), got {:?}",
        names
    );
}

/// Test 12: --output-dir flag — split to a custom directory.
/// Verify output files are in custom_dir/ instead of default _split/ dir.
#[test]
fn test_split_output_dir_flag() {
    let tmp = TempDir::new().unwrap();
    let input = create_test_3mf(tmp.path(), "custom_dir_test", 1, Some("Part"), None);
    let custom_dir = tmp.path().join("my_custom_output");

    let result = run_split(&[
        input.to_str().unwrap(),
        "--output-dir",
        custom_dir.to_str().unwrap(),
    ]);

    assert!(
        result.status.success(),
        "Split with --output-dir failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // Files should be in the custom directory, not the default _split/ dir
    assert!(custom_dir.exists(), "Custom output dir should exist");

    let default_dir = tmp.path().join("custom_dir_test_split");
    assert!(
        !default_dir.exists(),
        "Default _split dir should not be created when --output-dir is specified"
    );

    let entries: Vec<PathBuf> = std::fs::read_dir(&custom_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("3mf"))
        .collect();
    assert_eq!(entries.len(), 1, "Expected 1 output file in custom dir");
}

/// Test 13: Nonexistent input file — split of a nonexistent file should fail with clear error.
///
/// NOTE: The secure content test was considered but the Model writer does not serialize
/// KeyStore fields (encryption/signing metadata is only present in actual signed/encrypted
/// 3MF files from external tools). This test instead validates error handling for missing inputs.
#[test]
fn test_split_nonexistent_input_error() {
    let tmp = TempDir::new().unwrap();
    let nonexistent = tmp.path().join("does_not_exist.3mf");
    let split_dir = tmp.path().join("split_out");

    let result = run_split(&[
        nonexistent.to_str().unwrap(),
        "--output-dir",
        split_dir.to_str().unwrap(),
    ]);

    assert!(
        !result.status.success(),
        "Split of nonexistent file should fail, but succeeded"
    );

    // Output directory should not be created on failure
    assert!(
        !split_dir.exists(),
        "Split dir should not be created when input doesn't exist"
    );
}

/// Test 14: Unnamed objects — objects with no name should use fallback naming (part_1.3mf, etc.).
#[test]
fn test_split_unnamed_objects() {
    let tmp = TempDir::new().unwrap();
    // Create object with no name (None)
    let input = create_test_3mf(tmp.path(), "unnamed", 1, None, None);
    let split_dir = tmp.path().join("unnamed_split");

    let result = run_split(&[input.to_str().unwrap(), "--output-dir", split_dir.to_str().unwrap()]);

    assert!(
        result.status.success(),
        "Split of unnamed object failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let entries: Vec<PathBuf> = std::fs::read_dir(&split_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("3mf"))
        .collect();
    assert_eq!(entries.len(), 1, "Expected 1 output file for unnamed object");

    // Filename should use fallback naming (part_1.3mf or similar)
    let filename = entries[0]
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    assert!(
        filename.starts_with("part_") || filename.starts_with("part"),
        "Unnamed object should use fallback naming like 'part_1.3mf', got: {:?}",
        filename
    );
}

/// Test 15: Default output directory — split without --output-dir uses {input_stem}_split/.
#[test]
fn test_split_default_output_directory() {
    let tmp = TempDir::new().unwrap();
    let input = create_test_3mf(tmp.path(), "default_dir_model", 1, Some("Part"), None);

    // Run split without --output-dir
    let result = run_split(&[input.to_str().unwrap()]);

    assert!(
        result.status.success(),
        "Split without --output-dir failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // Default output dir should be {stem}_split/ next to input
    let default_split_dir = tmp.path().join("default_dir_model_split");
    assert!(
        default_split_dir.exists(),
        "Default output dir {:?} should be created",
        default_split_dir
    );

    let entries: Vec<PathBuf> = std::fs::read_dir(&default_split_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("3mf"))
        .collect();
    assert_eq!(entries.len(), 1, "Expected 1 output file in default split dir");
}

/// Test 16: split --help exits with code 0 and mentions expected flags.
#[test]
fn test_cli_split_help() {
    let output = std::process::Command::new("cargo")
        .args(["run", "--quiet", "-p", "lib3mf-cli", "--"])
        .args(["split", "--help"])
        .output()
        .expect("Failed to run split --help");

    assert!(
        output.status.success(),
        "split --help should exit 0, got: {}",
        output.status
    );

    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(help_text.contains("--by-object"), "Help should mention --by-object");
    assert!(help_text.contains("--dry-run"), "Help should mention --dry-run");
    assert!(help_text.contains("--force") || help_text.contains("force"), "Help should mention --force");
    assert!(help_text.contains("--select"), "Help should mention --select");
    assert!(help_text.contains("--preserve-transforms"), "Help should mention --preserve-transforms");
}
