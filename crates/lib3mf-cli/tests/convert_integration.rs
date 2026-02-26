use lib3mf_cli::commands::convert;
use lib3mf_core::model::{
    Build, BuildItem, Geometry, Mesh, Model, Object, ObjectType, ResourceCollection, ResourceId,
};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

/// Create a minimal single-triangle 3MF file at the given path.
///
/// The mesh contains 3 vertices and 1 triangle:
///   v0 = (0,0,0), v1 = (1,0,0), v2 = (0,1,0)
///   triangle = (0, 1, 2)
fn write_minimal_3mf(path: &Path) {
    let mut mesh = Mesh::new();
    let v0 = mesh.add_vertex(0.0, 0.0, 0.0);
    let v1 = mesh.add_vertex(1.0, 0.0, 0.0);
    let v2 = mesh.add_vertex(0.0, 1.0, 0.0);
    mesh.add_triangle(v0, v1, v2);

    let obj = Object {
        id: ResourceId(1),
        object_type: ObjectType::Model,
        name: None,
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

    let file = File::create(path).expect("Failed to create 3MF file");
    model.write(file).expect("Failed to write 3MF model");
}

#[test]
fn test_convert_3mf_to_ascii_stl() {
    let tmp = std::env::temp_dir();
    let input = tmp.join("lib3mf_test_ascii_in.3mf");
    let output = tmp.join("lib3mf_test_ascii_out.stl");

    write_minimal_3mf(&input);

    convert(input.clone(), output.clone(), true).expect("ASCII STL conversion failed");

    let content = fs::read_to_string(&output).expect("Failed to read ASCII STL output");

    assert!(
        content.contains("solid"),
        "ASCII STL must contain 'solid' keyword"
    );
    assert!(
        content.contains("endsolid"),
        "ASCII STL must contain 'endsolid' keyword"
    );
    assert!(
        content.contains("facet normal"),
        "ASCII STL must contain 'facet normal' declarations"
    );
    assert!(
        content.contains("vertex"),
        "ASCII STL must contain 'vertex' declarations"
    );

    let _ = fs::remove_file(&input);
    let _ = fs::remove_file(&output);
}

#[test]
fn test_convert_3mf_to_binary_stl_default() {
    let tmp = std::env::temp_dir();
    let input = tmp.join("lib3mf_test_binary_in.3mf");
    let output = tmp.join("lib3mf_test_binary_out.stl");

    write_minimal_3mf(&input);

    convert(input.clone(), output.clone(), false).expect("Binary STL conversion failed");

    let bytes = fs::read(&output).expect("Failed to read binary STL output");

    // Binary STL format: 80-byte header + 4-byte triangle count + N * 50 bytes per triangle
    // For 1 triangle: 80 + 4 + 1*50 = 134 bytes
    let expected_size = 80 + 4 + 1 * 50;
    assert_eq!(
        bytes.len(),
        expected_size,
        "Binary STL with 1 triangle should be {} bytes, got {}",
        expected_size,
        bytes.len()
    );

    let _ = fs::remove_file(&input);
    let _ = fs::remove_file(&output);
}

#[test]
fn test_convert_benchy_to_ascii_stl() {
    let input = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../models/Benchy.3mf");

    if !input.exists() {
        eprintln!("Skipping: Benchy.3mf not found at {:?}", input);
        return;
    }

    let tmp = std::env::temp_dir();
    let output = tmp.join("lib3mf_test_benchy_ascii.stl");

    convert(input, output.clone(), true).expect("Benchy ASCII STL conversion failed");

    let content = fs::read_to_string(&output).expect("Failed to read Benchy ASCII STL output");

    assert!(
        content.trim_start().starts_with("solid"),
        "Benchy ASCII STL must start with 'solid' (got: {:?})",
        &content[..content.len().min(50)]
    );
    assert!(
        content.contains("facet normal"),
        "Benchy ASCII STL must contain 'facet normal'"
    );
    assert!(
        content.len() > 1000,
        "Benchy ASCII STL should be large (> 1000 bytes), got {} bytes",
        content.len()
    );

    let _ = fs::remove_file(&output);
}
