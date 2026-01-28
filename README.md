# lib3mf-rs

![CI](https://github.com/stevescargall/lib3mf-rs/workflows/CI/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

A pure Rust implementation of the [3D Manufacturing Format (3MF)](https://3mf.io/) standard. `lib3mf-rs` provides a memory-safe, high-performance library and CLI tools for reading, analyzing, and processing 3MF files.

## Features

- **Pure Rust**: No C++ bindings, memory-safe.
- **Robust Parsing**: Validation of XML structure and relationships.
- **Model Statistics**: Compute geometry counts (vertices, triangles) and instance counts.
- **Vendor Extensions**: Native support for **Bambu Studio** project files (recognizing plates and metadata).
- **CLI Tool**: Inspect 3MF files directly from the command line.

## usage

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable)

### Building from Source

Clone the repository and build the project:

```bash
git clone https://github.com/stevescargall/lib3mf-rs.git
cd lib3mf-rs
```

**Development Build (Debug)**:
Faster compilation, but slower runtime execution. Best for testing and development.
```bash
cargo build
```

**Release Build**:
Optimized for performance. Use this for production or benchmarking.
```bash
cargo build --release
```

### Running the CLI

The `3mf` CLI tool allows you to inspect and analyze 3MF files.

#### 1. Quick Stats
Get a summary of the model, including geometry counts and vendor metadata (like Bambu Studio plates).

```bash
cargo run -p lib3mf-cli -- stats path/to/model.3mf

# Output as JSON
cargo run -p lib3mf-cli -- stats path/to/model.3mf --format json
```

#### 2. List Archive Contents
See what's inside the 3MF container. Use `--format tree` for a file tree view.

```bash
cargo run -p lib3mf-cli -- list path/to/model.3mf --format tree
```

#### 3. Inspect Relationships
Debug OPC relationships and content types.

```bash
cargo run -p lib3mf-cli -- rels path/to/model.3mf
```

#### 4. Extract Files
Extract specific files (like thumbnails or internal config) from the archive.

```bash
cargo run -p lib3mf-cli -- extract path/to/model.3mf "Auxiliaries/.thumbnails/thumbnail_small.png" --output thumb.png
```

#### 5. Copy (Roundtrip)
Read a 3MF file and write it back to a new file. Validates the read/write cycle.

```bash
cargo run -p lib3mf-cli -- copy path/to/model.3mf output.3mf
```

### Library Usage

Add `lib3mf-core` to your `Cargo.toml`:

```toml
[dependencies]
lib3mf-core = { path = "crates/lib3mf-core" } # Or git dependency
```

Example: parsing a model and getting stats.

```rust
use lib3mf_core::archive::ZipArchiver;
use lib3mf_core::parser::parse_model;
use lib3mf_core::archive::{ArchiveReader, find_model_path};
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("models/Benchy.3mf")?;
    let mut archiver = ZipArchiver::new(file)?;
    
    // 1. Find and parse the 3D model
    let model_path = find_model_path(&mut archiver)?;
    let model_data = archiver.read_entry(&model_path)?;
    let model = parse_model(std::io::Cursor::new(model_data))?;
    
    // 2. Compute statistics (needs archiver to look up extensions like Bambu configs)
    let stats = model.compute_stats(&mut archiver)?;
    
    println!("Triangles: {}", stats.geometry.triangle_count);
    Ok(())
}
```

## Running Tests

We have a comprehensive test suite covering spec compliance and vendor integrations (Bambu Studio).

```bash
cargo test
```

## Project Structure
- `crates/lib3mf-core`: The main library implementation.
- `crates/lib3mf-cli`: The `3mf` command-line interface.

## Contributing
Please read [AGENTS.md](AGENTS.md) for AI agent protocols.
