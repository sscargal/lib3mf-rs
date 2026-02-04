# Getting Started

This guide will help you install lib3mf-rs and write your first 3MF program.

## Installation

### CLI Tool

The fastest way to try lib3mf-rs is to install the command-line tool:

```bash
cargo install lib3mf-cli
```

This installs the `lib3mf-cli` binary, which you can use to inspect, validate, and analyze 3MF files without writing any code.

Try it on a 3MF file:

```bash
lib3mf-cli stats path/to/model.3mf
```

### Library Dependency

To use lib3mf-rs in your Rust project, add it to your `Cargo.toml`:

```toml
[dependencies]
lib3mf-core = "0.1"
```

By default, this gives you a minimal build with no optional dependencies. If you need cryptographic features (digital signatures and encryption) or parallel processing, see the [Feature Flags](feature-flags.md) chapter.

## Quick Start with CLI

The CLI tool provides several commands for working with 3MF files:

```bash
# Get file statistics (object count, triangle count, materials)
lib3mf-cli stats model.3mf

# Get JSON output for scripting
lib3mf-cli stats model.3mf --format json

# List archive contents
lib3mf-cli list model.3mf --format tree

# Validate a file (Standard level)
lib3mf-cli validate model.3mf

# Run paranoid validation (deep geometry checks)
lib3mf-cli validate model.3mf --level paranoid

# Compare two versions
lib3mf-cli diff v1.3mf v2.3mf
```

For complete CLI documentation, see the [CLI Guide](cli-guide.md).

## Your First Program

Let's write a simple program that opens a 3MF file, prints statistics, and runs validation.

Create a new Rust project:

```bash
cargo new my_3mf_tool
cd my_3mf_tool
```

Edit `Cargo.toml` to add lib3mf-core:

```toml
[dependencies]
lib3mf-core = "0.1"
```

Edit `src/main.rs`:

```rust
use lib3mf_core::Model;
use lib3mf_core::validation::ValidationLevel;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load and parse the 3MF file
    let model = Model::from_file("model.3mf")?;

    // Compute statistics
    let stats = model.compute_stats()?;
    println!("File Statistics:");
    println!("  Objects: {}", stats.resource_counts.object_count);
    println!("  Triangles: {}", stats.geometry.triangle_count);
    println!("  Vertices: {}", stats.geometry.vertex_count);
    println!("  Build Items: {}", stats.resource_counts.build_count);

    // Run validation at Standard level
    let report = model.validate(ValidationLevel::Standard)?;

    if report.has_errors() {
        eprintln!("\nValidation Errors: {}", report.error_count());
        for issue in report.errors() {
            eprintln!("  - {}", issue.message);
        }
        std::process::exit(1);
    } else {
        println!("\nValidation: PASSED");
    }

    Ok(())
}
```

Run it:

```bash
cargo run model.3mf
```

## Reading a 3MF File (Step by Step)

The `Model::from_file()` convenience method is great for simple use cases, but sometimes you need more control. Here's how to read a 3MF file manually:

```rust
use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
use lib3mf_core::parser::parse_model;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Open the ZIP archive
    let file = File::open("model.3mf")?;
    let mut archiver = ZipArchiver::new(file)?;

    // Step 2: Find the main model XML file using OPC relationships
    let model_path = find_model_path(&mut archiver)?;
    println!("Model path: {}", model_path);

    // Step 3: Read the model XML data
    let model_data = archiver.read_entry(&model_path)?;

    // Step 4: Parse XML into Model structure
    let model = parse_model(std::io::Cursor::new(model_data))?;

    // Step 5: Access model data
    println!("Unit: {}", model.unit);
    println!("Metadata entries: {}", model.metadata.len());

    // Iterate over objects
    for (id, obj) in model.resources.iter_objects() {
        println!("Object {}: {} triangles", id.0, obj.mesh.triangles.len());
    }

    Ok(())
}
```

This lower-level approach gives you access to:
- Archive contents (thumbnails, textures, etc.)
- Raw XML data
- Individual resource inspection before full model computation

For detailed API documentation, see the [rustdoc reference for Model](../rustdoc/lib3mf_core/model/struct.Model.html).

## Writing a 3MF File

You can create a 3MF file programmatically. Here's how to create a simple cube:

```rust
use lib3mf_core::model::{Model, Object, Mesh, Vertex, Triangle, BuildItem};
use lib3mf_core::model::ResourceId;
use lib3mf_core::writer::write_package;
use glam::Vec3;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new model with millimeter units
    let mut model = Model::new("millimeter".to_string());

    // Create cube vertices (-10 to +10mm on each axis)
    let vertices = vec![
        Vertex { position: Vec3::new(-10.0, -10.0, -10.0) },
        Vertex { position: Vec3::new( 10.0, -10.0, -10.0) },
        Vertex { position: Vec3::new( 10.0,  10.0, -10.0) },
        Vertex { position: Vec3::new(-10.0,  10.0, -10.0) },
        Vertex { position: Vec3::new(-10.0, -10.0,  10.0) },
        Vertex { position: Vec3::new( 10.0, -10.0,  10.0) },
        Vertex { position: Vec3::new( 10.0,  10.0,  10.0) },
        Vertex { position: Vec3::new(-10.0,  10.0,  10.0) },
    ];

    // Create 12 triangles (2 per face)
    let triangles = vec![
        // Bottom face (z = -10)
        Triangle::new(0, 1, 2),
        Triangle::new(0, 2, 3),
        // Top face (z = +10)
        Triangle::new(4, 6, 5),
        Triangle::new(4, 7, 6),
        // Front face (y = -10)
        Triangle::new(0, 5, 1),
        Triangle::new(0, 4, 5),
        // Back face (y = +10)
        Triangle::new(3, 2, 6),
        Triangle::new(3, 6, 7),
        // Left face (x = -10)
        Triangle::new(0, 3, 7),
        Triangle::new(0, 7, 4),
        // Right face (x = +10)
        Triangle::new(1, 5, 6),
        Triangle::new(1, 6, 2),
    ];

    let mesh = Mesh::new(vertices, triangles);
    let object = Object::new_model(mesh);

    // Add object to model resources
    let object_id = ResourceId(1);
    model.resources.add_object(object_id, object)?;

    // Add object to build (what will be printed)
    let build_item = BuildItem {
        object_id,
        transform: None,  // Identity transform (no rotation/translation)
        part_number: None,
    };
    model.build.items.push(build_item);

    // Write to file
    let output = File::create("cube.3mf")?;
    write_package(&model, output)?;

    println!("Created cube.3mf");

    Ok(())
}
```

For more examples, check out the [examples directory](https://github.com/sscargal/lib3mf-rs/tree/main/crates/lib3mf-core/examples) in the repository:

- `create_cube.rs` — Building a 3MF model from scratch
- `advanced_materials.rs` — Textures and composite materials
- `geometry_validation.rs` — Using paranoid validation to find issues
- `geometry_repair.rs` — Programmatic mesh repair
- `secure_content.rs` — Digital signatures and encryption
- `beam_lattice_ext.rs` — Creating structural lattice designs
- `boolean_operations.rs` — Union, difference, intersection operations
- `displacement_mesh.rs` — Texture-driven surface modification
- `slice_data.rs` — Pre-sliced geometry for resin printers
- `streaming_stats.rs` — Processing large files with constant memory

## Next Steps

- **[CLI Guide](cli-guide.md)** — Learn all the command-line tools
- **[Architecture Overview](architecture.md)** — Understand how lib3mf-rs is structured
- **[Validation Guide](validation-guide.md)** — Deep dive into the validation system
- **[Extensions](extensions.md)** — Working with advanced 3MF features
- **[API Reference](../rustdoc/lib3mf_core/index.html)** — Complete API documentation
