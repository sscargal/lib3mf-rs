use lib3mf_core::model::{BuildItem, Geometry, Mesh, Model, Object, ObjectType, ResourceId, Unit};
use std::fs::File;

fn main() -> anyhow::Result<()> {
    println!("Creating simple 3MF cube...");

    let mut model = Model {
        unit: Unit::Millimeter,
        ..Default::default()
    };

    // 1. Create a Mesh (Cube)
    let mut mesh = Mesh::new();

    // Add 8 vertices
    // Bottom
    let v0 = mesh.add_vertex(0.0, 0.0, 0.0);
    let v1 = mesh.add_vertex(10.0, 0.0, 0.0);
    let v2 = mesh.add_vertex(10.0, 10.0, 0.0);
    let v3 = mesh.add_vertex(0.0, 10.0, 0.0);
    // Top
    let v4 = mesh.add_vertex(0.0, 0.0, 10.0);
    let v5 = mesh.add_vertex(10.0, 0.0, 10.0);
    let v6 = mesh.add_vertex(10.0, 10.0, 10.0);
    let v7 = mesh.add_vertex(0.0, 10.0, 10.0);

    // Add 12 triangles (2 per face)
    // Bottom
    mesh.add_triangle(v0, v2, v1);
    mesh.add_triangle(v0, v3, v2);
    // Top
    mesh.add_triangle(v4, v5, v6);
    mesh.add_triangle(v4, v6, v7);
    // Front
    mesh.add_triangle(v0, v1, v5);
    mesh.add_triangle(v0, v5, v4);
    // Right
    mesh.add_triangle(v1, v2, v6);
    mesh.add_triangle(v1, v6, v5);
    // Back
    mesh.add_triangle(v2, v3, v7);
    mesh.add_triangle(v2, v7, v6);
    // Left
    mesh.add_triangle(v3, v0, v4);
    mesh.add_triangle(v3, v4, v7);

    // 2. Create Model Object (printable part)
    let object_id = ResourceId(1);
    let object = Object {
        id: object_id,
        object_type: ObjectType::Model, // Printable part - requires manifold mesh
        name: Some("Simple Cube".to_string()),
        part_number: None,
        uuid: None,
        pid: None,
        thumbnail: None,
        pindex: None,
        geometry: Geometry::Mesh(mesh),
    };

    println!(
        "Object type: {} - requires manifold: {}",
        object.object_type,
        object.object_type.requires_manifold()
    );

    // 3. Create Support Object (optional - demonstrates different type)
    let mut support_mesh = Mesh::new();
    // Simple support geometry (can be non-manifold)
    support_mesh.add_vertex(5.0, 5.0, -5.0);
    support_mesh.add_vertex(6.0, 5.0, -5.0);
    support_mesh.add_vertex(5.0, 6.0, -5.0);
    support_mesh.add_triangle(0, 1, 2);

    let support_id = ResourceId(2);
    let support_object = Object {
        id: support_id,
        object_type: ObjectType::Support, // Support structure - non-manifold allowed
        name: Some("Support".to_string()),
        part_number: None,
        uuid: None,
        pid: None,
        thumbnail: None,
        pindex: None,
        geometry: Geometry::Mesh(support_mesh),
    };

    println!(
        "Support type: {} - requires manifold: {}",
        support_object.object_type,
        support_object.object_type.requires_manifold()
    );

    // 4. Add to Resources
    model.resources.add_object(object)?;
    model.resources.add_object(support_object)?;

    // 5. Create Build Items (only model object - supports can be ignored by consumer)
    let item = BuildItem {
        object_id,
        transform: glam::Mat4::IDENTITY,
        part_number: None,
        uuid: None,
        path: None,
    };
    model.build.items.push(item);

    // 6. Write to file
    let file = File::create("cube.3mf")?;
    model.write(file)?;

    println!("\nWritten to cube.3mf");
    println!("Model contains:");
    println!("  - 1 model object (printable cube)");
    println!("  - 1 support object (removable support)");

    Ok(())
}
