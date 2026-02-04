# Introduction

Welcome to the **lib3mf-rs Guide**, comprehensive documentation for the pure Rust implementation of the 3D Manufacturing Format (3MF) specification.

## What is lib3mf-rs?

`lib3mf-rs` is a production-ready, memory-safe library for reading, writing, validating, and processing 3MF files used in 3D printing, additive manufacturing, and CAD workflows. It provides complete support for digital signatures, encryption, advanced materials, slicing, boolean operations, and all official 3MF specifications.

## What is 3MF?

The **3D Manufacturing Format (3MF)** is an open, XML-based file format designed specifically for additive manufacturing and 3D printing. Unlike older formats like STL or OBJ, 3MF can store:

- Complete 3D geometry (meshes, lattices, voxels)
- Full-color textures and advanced materials
- Manufacturing metadata (part numbers, UUIDs, production paths)
- Digital signatures and encrypted content
- Pre-sliced data for resin printers
- Boolean operations on geometry
- Surface displacement via textures

3MF is maintained by the [3MF Consortium](https://3mf.io/) and supported by major CAD and slicing software including PrusaSlicer, Bambu Studio, Cura, Fusion 360, and many others.

## Why lib3mf-rs?

**Pure Rust** — No C++ dependencies, guaranteed memory safety, first-class WASM support.

**Complete Specification Support** — Implements all 9 official 3MF extensions with 100% coverage across 345 features:
- Core Specification v1.4.0
- Materials and Properties Extension v1.2.1
- Production Extension v1.1.2
- Beam Lattice Extension v1.2.0
- Slice Extension v1.0.2
- Volumetric Extension v0.8.0
- Secure Content Extension v1.0.2
- Boolean Operations Extension v1.1.1
- Displacement Extension v1.0.0

**Production Ready** — Progressive validation system (Minimal/Standard/Strict/Paranoid), geometry repair utilities, mesh topology analysis, digital signature verification.

**High Performance** — Optional multi-threading for large files, streaming parser for low memory usage, efficient XML processing, BVH-accelerated geometry checks.

**Flexible Feature Flags** — Minimal build with ~154 dependencies, optional crypto support adds ~146 more. Choose what you need.

**Vendor Extensions** — Native support for Bambu Studio project files including multi-plate layouts and filament metadata.

## What This Book Covers

This guide is organized into two main sections:

**User Guide** — Practical tutorials for common workflows:
- [Getting Started](getting-started.md) — Installation, quick start, your first 3MF program
- [CLI Guide](cli-guide.md) — Command-line tools for inspecting and analyzing files
- [Feature Flags](feature-flags.md) — Minimizing dependencies for your use case
- [Validation Guide](validation-guide.md) — Using the 4-level validation system

**Reference** — Deep technical documentation:
- [Architecture Overview](architecture.md) — Crate structure, parsing pipeline, design patterns
- [Extensions](extensions.md) — Details on all 9 3MF extensions
- [Contributing](contributing.md) — Development setup, testing, adding features

**API Reference** — For complete API documentation:
- **[Rustdoc API Reference](https://sscargal.github.io/lib3mf-rs/dev/rustdoc/lib3mf_core/)** — Module, struct, and function documentation with examples
- **[docs.rs](https://docs.rs/lib3mf-core)** — Versioned API docs for released crates

## Quick Example

```rust
use lib3mf_core::Model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load and parse a 3MF file
    let model = Model::from_file("model.3mf")?;

    // Get statistics
    let stats = model.compute_stats()?;
    println!("Triangles: {}", stats.geometry.triangle_count);
    println!("Vertices: {}", stats.geometry.vertex_count);

    // Run validation
    let report = model.validate(ValidationLevel::Standard)?;
    if report.has_errors() {
        eprintln!("Validation errors found: {}", report.error_count());
    }

    Ok(())
}
```

## Project Links

- **GitHub Repository:** [https://github.com/sscargal/lib3mf-rs](https://github.com/sscargal/lib3mf-rs)
- **API Documentation:** [https://docs.rs/lib3mf-core](https://docs.rs/lib3mf-core)
- **Crates.io:** [https://crates.io/crates/lib3mf-core](https://crates.io/crates/lib3mf-core)
- **3MF Consortium:** [https://3mf.io/](https://3mf.io/)

## License

lib3mf-rs is distributed under the MIT License. See the [LICENSE](https://github.com/sscargal/lib3mf-rs/blob/main/LICENSE) file for details.
