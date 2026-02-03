//! Example: Creating and reading 3MF files with Displacement Meshes
//!
//! This example demonstrates:
//! - Creating a displacement mesh with normal vectors
//! - Creating a displacement texture resource
//! - Writing and reading displacement data
//! - Validating displacement meshes
//!
//! Run with: cargo run -p lib3mf-core --example displacement_mesh

use lib3mf_core::model::{
    Build, BuildItem, Channel, Displacement2D, DisplacementMesh, DisplacementTriangle,
    FilterMode, Geometry, Model, NormalVector, Object, ObjectType, ResourceId, TileStyle, Vertex,
};
use lib3mf_core::validation::{ValidationLevel, ValidationSeverity};
use std::io::Cursor;

fn main() -> anyhow::Result<()> {
    println!("=== Displacement Extension Example ===\n");

    // Create a simple displacement mesh model
    let model = create_displacement_model();

    // Validate
    println!("Validating model...");
    let report = model.validate(ValidationLevel::Standard);
    if report.has_errors() {
        for item in report.items.iter() {
            if item.severity == ValidationSeverity::Error {
                println!("  [{}] {}", item.code, item.message);
            }
        }
    } else {
        println!("Model is valid!\n");
    }

    // Write to buffer
    let mut buffer = Vec::new();
    model.write_xml(&mut buffer, None)?;

    println!("Generated XML ({} bytes):", buffer.len());
    println!("{}\n", String::from_utf8_lossy(&buffer));

    // Parse back
    println!("Parsing generated XML...");
    let parsed = lib3mf_core::parser::parse_model(Cursor::new(&buffer))?;

    // Inspect displacement mesh
    if let Some(obj) = parsed.resources.get_object(ResourceId(1)) {
        if let Geometry::DisplacementMesh(dmesh) = &obj.geometry {
            println!("Parsed DisplacementMesh:");
            println!("  Vertices: {}", dmesh.vertices.len());
            println!("  Triangles: {}", dmesh.triangles.len());
            println!("  Normals: {}", dmesh.normals.len());
            println!(
                "  Gradients: {}",
                dmesh.gradients.as_ref().map_or(0, |g| g.len())
            );
        }
    }

    println!("\nExample complete!");
    Ok(())
}

fn create_displacement_model() -> Model {
    // Create simple triangle with displacement
    let mut model = Model::default();

    // Add displacement texture resource (references would be in attachments)
    let texture = Displacement2D {
        id: ResourceId(100),
        path: "/3D/Textures/height.png".to_string(),
        channel: Channel::G,
        tile_style: TileStyle::Wrap,
        filter: FilterMode::Linear,
        height: 2.0,
        offset: 0.0,
    };
    model.resources.add_displacement_2d(texture).unwrap();

    // Create displacement mesh
    let dmesh = DisplacementMesh {
        vertices: vec![
            Vertex {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Vertex {
                x: 10.0,
                y: 0.0,
                z: 0.0,
            },
            Vertex {
                x: 5.0,
                y: 10.0,
                z: 0.0,
            },
        ],
        triangles: vec![DisplacementTriangle {
            v1: 0,
            v2: 1,
            v3: 2,
            d1: Some(0),
            d2: Some(1),
            d3: Some(2),
            ..Default::default()
        }],
        normals: vec![
            NormalVector {
                nx: 0.0,
                ny: 0.0,
                nz: 1.0,
            },
            NormalVector {
                nx: 0.0,
                ny: 0.0,
                nz: 1.0,
            },
            NormalVector {
                nx: 0.0,
                ny: 0.0,
                nz: 1.0,
            },
        ],
        gradients: None,
    };

    model
        .resources
        .add_object(Object {
            id: ResourceId(1),
            object_type: ObjectType::Model,
            name: Some("Displacement Triangle".to_string()),
            part_number: None,
            uuid: None,
            pid: None,
            pindex: None,
            thumbnail: None,
            geometry: Geometry::DisplacementMesh(dmesh),
        })
        .unwrap();

    model.build = Build {
        items: vec![BuildItem {
            object_id: ResourceId(1),
            transform: glam::Mat4::IDENTITY,
            uuid: None,
            part_number: None,
            path: None,
        }],
    };

    model
}
