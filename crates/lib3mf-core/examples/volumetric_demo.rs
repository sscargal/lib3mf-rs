//! Demonstrates the Volumetric Extension for layer-based volume data.
//!
//! The Volumetric extension (v0.8.0) allows defining 3D volumes using
//! a stack of 2D layers, typically images representing cross-sections.
//! This is useful for representing voxel data, implicit fields, or
//! layer-based manufacturing processes that use image stacks.
//!
//! Run with: cargo run -p lib3mf-core --example volumetric_demo

use lib3mf_core::model::{Geometry, ResourceId, VolumetricLayer, VolumetricRef, VolumetricStack};
use lib3mf_core::parser::parse_model;
use std::io::Cursor;

fn main() -> anyhow::Result<()> {
    println!("=== Volumetric Extension Demo ===\n");

    // 1. Parse a model with volumetric data
    let model = parse_model(Cursor::new(VOLUMETRIC_MODEL))?;
    let object_count = model.resources.iter_objects().count();
    println!("Parsed model with {} objects", object_count);

    // 2. Access the volumetric stack using get_volumetric_stack
    let stack = model
        .resources
        .get_volumetric_stack(ResourceId(10))
        .expect("Volumetric stack not found");

    println!("\nVolumetric Stack (id={}):", stack.id.0);
    println!("  Layers: {}", stack.layers.len());
    println!("  External refs: {}", stack.refs.len());

    // 3. Inspect layers
    println!("\nLayers:");
    for (i, layer) in stack.layers.iter().enumerate() {
        println!(
            "  [{}] z={:.2} -> {}",
            i, layer.z_height, layer.content_path
        );
    }

    // 4. Inspect external references
    if !stack.refs.is_empty() {
        println!("\nExternal References:");
        for r in &stack.refs {
            println!("  Stack {} -> {}", r.stack_id.0, r.path);
        }
    }

    // 5. Check object binding
    let obj = model
        .resources
        .get_object(ResourceId(1))
        .expect("Object 1 not found");
    if let Geometry::VolumetricStack(sid) = obj.geometry {
        println!("\nObject 1 uses VolumetricStack {}", sid.0);
    }

    // 6. Demonstrate creating volumetric data programmatically
    println!("\n--- Creating volumetric stack programmatically ---");
    let custom_stack = create_custom_stack();
    println!("Created stack with {} layers", custom_stack.layers.len());
    println!("Layer z-heights:");
    for (i, layer) in custom_stack.layers.iter().take(5).enumerate() {
        println!(
            "  Layer {}: z={:.1}, path={}",
            i, layer.z_height, layer.content_path
        );
    }
    if custom_stack.layers.len() > 5 {
        println!("  ... ({} more layers)", custom_stack.layers.len() - 5);
    }
    println!("External references: {}", custom_stack.refs.len());

    println!("\n=== Demo Complete ===");
    Ok(())
}

fn create_custom_stack() -> VolumetricStack {
    VolumetricStack {
        id: ResourceId(100),
        version: "1.0".to_string(),
        layers: (0..10)
            .map(|i| VolumetricLayer {
                z_height: i as f32 * 0.1,
                content_path: format!("/3D/layer{:03}.png", i),
            })
            .collect(),
        refs: vec![VolumetricRef {
            stack_id: ResourceId(200),
            path: "/3D/external.model".to_string(),
        }],
    }
}

const VOLUMETRIC_MODEL: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel">
    <resources>
        <volumetricstack id="10">
            <layer z="0.0" path="/3D/layer0.png" />
            <layer z="0.5" path="/3D/layer1.png" />
            <layer z="1.0" path="/3D/layer2.png" />
            <volumetricref volumetricstackid="20" path="/3D/other.model" />
        </volumetricstack>
        <object id="1" type="model" volumetricstackid="10" />
    </resources>
    <build>
        <item objectid="1" />
    </build>
</model>"##;
