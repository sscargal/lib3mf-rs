//! Example: Creating and reading 3MF files with Boolean Operations
//!
//! This example demonstrates:
//! - Creating mesh objects for boolean operations
//! - Defining boolean shape operations (union, difference, intersection)
//! - Writing 3MF with boolean operations
//! - Reading and validating boolean operations
//!
//! Run with: cargo run -p lib3mf-core --example boolean_operations

use lib3mf_core::model::{
    BooleanOperation, BooleanOperationType, BooleanShape, Build, BuildItem, Geometry, Mesh, Model,
    Object, ObjectType, ResourceId,
};
use lib3mf_core::validation::ValidationLevel;
use std::io::Cursor;

fn main() -> anyhow::Result<()> {
    println!("=== Boolean Operations Extension Example ===\n");

    // Create a model with boolean operations
    let model = create_boolean_model();

    // Validate the model
    println!("Validating model...");
    let report = model.validate(ValidationLevel::Standard);
    if report.has_errors() {
        println!("Validation errors:");
        for err in report.items.iter().filter(|i| {
            matches!(
                i.severity,
                lib3mf_core::validation::ValidationSeverity::Error
            )
        }) {
            println!("  [{}] {}", err.code, err.message);
        }
    } else {
        println!("Model is valid!\n");
    }

    // Write to buffer
    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None)?;

    println!("Generated XML ({} bytes):", buffer.len());
    println!("{}\n", String::from_utf8_lossy(&buffer));

    // Parse it back
    println!("Parsing generated XML...");
    let parsed = lib3mf_core::parser::parse_model(Cursor::new(&buffer))?;

    // Inspect the boolean shape
    if let Some(obj) = parsed.resources.get_object(ResourceId(100)) {
        if let Geometry::BooleanShape(bs) = &obj.geometry {
            println!("Parsed BooleanShape:");
            println!("  Base object ID: {}", bs.base_object_id.0);
            println!("  Operations: {}", bs.operations.len());
            for (i, op) in bs.operations.iter().enumerate() {
                println!(
                    "    [{}] {:?} with object {}",
                    i, op.operation_type, op.object_id.0
                );
            }
        }
    }

    println!("\nExample complete!");
    Ok(())
}

/// Creates a model with two cubes and a boolean difference operation
fn create_boolean_model() -> Model {
    let mut model = Model::default();

    // Create first cube (base object for boolean)
    let cube1 = create_cube_mesh(0.0, 0.0, 0.0, 20.0);
    model
        .resources
        .add_object(Object {
            id: ResourceId(1),
            object_type: ObjectType::Model,
            name: Some("Base Cube".to_string()),
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::Mesh(cube1),
        })
        .unwrap();

    // Create second cube (will be subtracted)
    let cube2 = create_cube_mesh(5.0, 5.0, 5.0, 15.0);
    model
        .resources
        .add_object(Object {
            id: ResourceId(2),
            object_type: ObjectType::Model,
            name: Some("Subtraction Cube".to_string()),
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::Mesh(cube2),
        })
        .unwrap();

    // Create third cube (will be intersected)
    let cube3 = create_cube_mesh(10.0, 10.0, 0.0, 15.0);
    model
        .resources
        .add_object(Object {
            id: ResourceId(3),
            object_type: ObjectType::Model,
            name: Some("Intersection Cube".to_string()),
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::Mesh(cube3),
        })
        .unwrap();

    // Create boolean shape: cube1 - cube2 intersect cube3
    let boolean_shape = BooleanShape {
        base_object_id: ResourceId(1),
        base_transform: glam::Mat4::IDENTITY,
        base_path: None,
        operations: vec![
            BooleanOperation {
                operation_type: BooleanOperationType::Difference,
                object_id: ResourceId(2),
                transform: glam::Mat4::IDENTITY,
                path: None,
            },
            BooleanOperation {
                operation_type: BooleanOperationType::Intersection,
                object_id: ResourceId(3),
                transform: glam::Mat4::IDENTITY,
                path: None,
            },
        ],
    };

    model
        .resources
        .add_object(Object {
            id: ResourceId(100),
            object_type: ObjectType::Model,
            name: Some("Boolean Result".to_string()),
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::BooleanShape(boolean_shape),
        })
        .unwrap();

    // Build references the boolean result
    model.build = Build {
        items: vec![BuildItem {
            object_id: ResourceId(100),
            transform: glam::Mat4::IDENTITY,
            uuid: None,
            part_number: None,
            path: None,
        }],
    };

    model
}

/// Creates a simple cube mesh at the given position with the given size
fn create_cube_mesh(x: f32, y: f32, z: f32, size: f32) -> Mesh {
    let mut mesh = Mesh::new();

    // 8 vertices of the cube
    let v0 = mesh.add_vertex(x, y, z);
    let v1 = mesh.add_vertex(x + size, y, z);
    let v2 = mesh.add_vertex(x + size, y + size, z);
    let v3 = mesh.add_vertex(x, y + size, z);
    let v4 = mesh.add_vertex(x, y, z + size);
    let v5 = mesh.add_vertex(x + size, y, z + size);
    let v6 = mesh.add_vertex(x + size, y + size, z + size);
    let v7 = mesh.add_vertex(x, y + size, z + size);

    // 12 triangles (2 per face)
    // Bottom face
    mesh.add_triangle(v0, v2, v1);
    mesh.add_triangle(v0, v3, v2);
    // Top face
    mesh.add_triangle(v4, v5, v6);
    mesh.add_triangle(v4, v6, v7);
    // Front face
    mesh.add_triangle(v0, v1, v5);
    mesh.add_triangle(v0, v5, v4);
    // Back face
    mesh.add_triangle(v2, v3, v7);
    mesh.add_triangle(v2, v7, v6);
    // Left face
    mesh.add_triangle(v0, v4, v7);
    mesh.add_triangle(v0, v7, v3);
    // Right face
    mesh.add_triangle(v1, v2, v6);
    mesh.add_triangle(v1, v6, v5);

    mesh
}
