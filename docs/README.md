# Lib3mf-rs User Guide

Welcome to the `lib3mf-rs` User Guide. This document provides a comprehensive overview of how to use the library for reading, writing, and analyzing 3MF model files.

## 1. Introduction

`lib3mf-rs` is a pure Rust implementation of the 3D Manufacturing Format (3MF) standard. It is designed to be:
- **Safe**: Leveraging Rust's memory safety guarantees.
- **Fast**: Zero-copy parsing where possible.
- **Complete**: Supporting core spec + major extensions (Slice, BeamLattice, Materials).

## 2. Installation

Add the core library to your `Cargo.toml`:

```toml
[dependencies]
lib3mf-core = "0.1.0" # Replace with actual version or git path
```

## 3. Reading a 3MF File

Reading involves opening the ZIP archive and identifying the model stream.

```rust
use lib3mf_core::archive::{ZipArchiver, ArchiveReader, find_model_path};
use lib3mf_core::parser::parse_model;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Open the file
    let file = File::open("test.3mf")?;
    
    // 2. Initialize Archiver (ZIP handler)
    let mut archiver = ZipArchiver::new(file)?;
    
    // 3. Find the entry point (usually 3D/3dmodel.model)
    let path = find_model_path(&mut archiver)?;
    
    // 4. Read the XML data
    let xml_data = archiver.read_entry(&path)?;
    
    // 5. Parse the Model
    let model = parse_model(std::io::Cursor::new(xml_data))?;
    
    println!("Unit: {:?}", model.unit);
    Ok(())
}
```

## 4. The Model Structure

The `Model` struct is the root of your data.

- **`resources`**: Contains all reuseable definitions (Meshes, Materials, Textures).
- **`build`**: Defines what is actually printed (Instances of Objects).
- **`metadata`**: Key-value pairs (Title, Designer, etc.).

### Accessing Meshes

Meshes are stored in the `resources` collection inside `Object`s.

```rust
use lib3mf_core::model::Geometry;

for object in model.resources.objects() {
    if let Geometry::Mesh(mesh) = &object.geometry {
        println!("Object {} has {} triangles", object.id, mesh.triangles.len());
        
        for v in &mesh.vertices {
             // Access x, y, z
        }
    }
}
```

## 5. Writing a 3MF File

To create a new 3MF file:

1. Create a `Model`.
2. Add `Object`s to `resources`.
3. Add `Component`s to `build`.
4. Use `ZipArchiveWriter`.

*(Note: Writer API is currently low-level and requires constructing the Archive struct manually. High-level helpers are in progress.)*

## 6. Extensions

We support several official extensions. These are usually handled automatically by the parser.

- **Production**: UUIDs for parts.
- **Beam Lattice**: Lattice structures inside meshes.
- **Slice**: Stack of 2D polygons for layer-based printing.

## 7. CLI Usage

See `crates/lib3mf-cli/README.md` for details on the `3mf` command line tool.
