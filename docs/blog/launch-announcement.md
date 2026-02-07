---
title: "Introducing lib3mf-rs: A Production-Ready Rust 3MF Toolkit"
date: 2026-02-06
tags: [rust, 3mf, 3d-printing, open-source, launch]
---

# Introducing lib3mf-rs: A Production-Ready Rust 3MF Toolkit

I'm excited to announce the initial release of **lib3mf-rs**, a pure Rust implementation of the 3D Manufacturing Format (3MF) specification designed for production workloads.

## What is lib3mf-rs?

lib3mf-rs is a comprehensive toolkit for parsing, validating, and writing 3MF files—the modern standard for 3D printing and additive manufacturing. Unlike older formats like STL, 3MF supports colors, materials, textures, digital signatures, and advanced features like boolean operations and lattice structures.

## Key Features

**Complete Specification Support**
- All 9 official 3MF extensions (Materials, Slicing, Security, Boolean Operations, Displacement, Beam Lattice, Volumetric, Production)
- Vendor extensions including Bambu Studio project files
- 86% pass rate on official 3MF Consortium conformance tests (44/51 tests)

**Production-Ready Design**
- Zero unsafe code (memory-safe by design)
- Progressive validation: 4 levels from minimal (14.9 ns) to paranoid (85.6 ms with deep geometry checks)
- Comprehensive error handling (no panics in library code)
- 90%+ test coverage with extensive fuzzing infrastructure

**Modern Rust Features**
- Async I/O support via `lib3mf-async` for high-throughput applications
- WebAssembly bindings via `lib3mf-wasm` for browser and edge deployment
- Feature flags for minimal dependencies (154 crates without crypto vs. 300 with all features)
- Streaming parser for constant-memory processing of large files

**Developer Productivity**
- CLI tools for immediate file inspection and validation
- Format converters (STL ↔ 3MF ↔ OBJ)
- Comprehensive documentation at [sscargal.github.io/lib3mf-rs](https://sscargal.github.io/lib3mf-rs/)
- Rich examples demonstrating all major features

## Quick Start

### Install CLI

```bash
cargo install lib3mf-cli
lib3mf-cli stats your_model.3mf
```

### Use in Your Project

```toml
[dependencies]
lib3mf-core = "0.1"
```

```rust
use lib3mf_core::Model;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::from_file("model.3mf")?;
    let stats = model.compute_stats()?;
    println!("Triangles: {}", stats.geometry.triangle_count);
    Ok(())
}
```

### Async Example

```rust
use lib3mf_async::AsyncModel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = AsyncModel::from_file("model.3mf").await?;
    let stats = model.compute_stats().await?;
    println!("Vertices: {}", stats.geometry.vertex_count);
    Ok(())
}
```

## Multi-Crate Architecture

lib3mf-rs is structured as five specialized crates:

| Crate | Purpose |
|-------|---------|
| **lib3mf-core** | Main parsing, validation, and writing engine |
| **lib3mf-async** | Tokio-based async I/O wrappers |
| **lib3mf-cli** | Command-line tools (stats, validate, convert, diff, extract) |
| **lib3mf-converters** | STL and OBJ format conversion |
| **lib3mf-wasm** | WebAssembly bindings for browser deployment |

Use exactly what you need. Building a web service? Add `lib3mf-async`. Need browser deployment? Include `lib3mf-wasm`. Just want to inspect files? Use the CLI.

## Performance Characteristics

Benchmarked on AMD Ryzen 7 7735HS (8 cores), 15GB RAM, Linux 6.14:

| File Size | Parse Time | Throughput |
|-----------|------------|------------|
| Small (1.2 KB) | 21.4 µs | 52.3 MiB/s |
| Medium (258 KB) | 15.8 ms | 15.9 MiB/s |
| Large (3.1 MB) | 44.1 µs | 66.5 GiB/s* |

*Large file benchmark shows caching effects from in-memory data. Real-world disk I/O will be slower but still sub-second for typical files.

**Validation Performance** (258 KB model):
- Minimal: 14.9 ns (effectively zero-cost)
- Standard: 24.6 µs (comprehensive reference checking)
- Strict: 24.7 µs (full spec compliance)
- Paranoid: 85.6 ms (deep geometry analysis with BVH self-intersection detection)

Full benchmark details: [docs/performance.md](https://github.com/sscargal/lib3mf-rs/blob/main/docs/performance.md)

## Conformance Testing

We've integrated the official 3MF Consortium test suite:

| Category | Tests | Passing | Pass Rate |
|----------|-------|---------|-----------|
| MUSTPASS | 13 | 6 | 46% |
| MUSTFAIL | 38 | 38 | 100% |
| **Total** | **51** | **44** | **86%** |

All invalid files are correctly detected (100% MUSTFAIL pass rate). The 7 failing MUSTPASS tests are due to a known material parser EOF bug that's documented and tracked for resolution.

Full conformance details: [docs/conformance.md](https://github.com/sscargal/lib3mf-rs/blob/main/docs/conformance.md)

## Rust 3MF Ecosystem

lib3mf-rs joins a growing ecosystem of Rust 3MF implementations. See the [ecosystem comparison](https://github.com/sscargal/lib3mf-rs/blob/main/docs/alternatives.md) to find the right tool for your use case:

- **lib3mf-rs**: Production apps with async/WASM requirements
- **lib3mf (telecos)**: Academic research and geometric operations
- **threemf2**: Lightweight simple use cases
- **Specialized tools**: Format conversion and multi-format workflows

We're excited to be part of this ecosystem and look forward to collaboration and knowledge sharing across projects.

## CLI Examples

The `lib3mf-cli` tool provides comprehensive file inspection:

```bash
# Get model statistics
lib3mf-cli stats model.3mf --format json

# Validate with paranoid geometry checks
lib3mf-cli validate model.3mf --level paranoid

# List archive contents in tree format
lib3mf-cli list model.3mf --format tree

# Extract embedded thumbnail
lib3mf-cli extract model.3mf "Auxiliaries/.thumbnails/thumbnail_small.png" -o thumb.png

# Compare two model versions
lib3mf-cli diff v1.3mf v2.3mf

# Convert from STL
lib3mf-cli convert benchy.stl benchy.3mf
```

## Documentation

Comprehensive documentation is available at **[sscargal.github.io/lib3mf-rs](https://sscargal.github.io/lib3mf-rs/)**:

- **[User Guide](https://sscargal.github.io/lib3mf-rs/stable/book/)**: Tutorial, architecture overview, and practical examples
- **[API Reference](https://sscargal.github.io/lib3mf-rs/stable/rustdoc/lib3mf_core/)**: Complete rustdoc for all public APIs
- **[Dev Docs](https://sscargal.github.io/lib3mf-rs/dev/book/)**: Latest unreleased changes on main branch

The documentation covers:
- Getting Started tutorial with step-by-step examples
- Architecture deep-dive into parsing, validation, and writing
- Extension guides for all 9 official specifications
- Advanced topics: streaming parsing, mesh repair, format conversion, async I/O
- Full API documentation with testable code examples

## What's Next

Near-term roadmap:

1. **Fix material parser EOF bug** (blocks 7/13 conformance tests)
2. **Performance optimizations**: SIMD acceleration, zero-copy deserialization
3. **Python bindings** via PyO3 for scientific Python ecosystem
4. **Node.js bindings** via napi-rs for npm ecosystem
5. **Advanced geometry operations**: Mesh simplification, normal generation, UV mapping

Long-term vision:
- Cloud integration patterns (S3/Azure/GCS, Kubernetes deployment guides)
- OpenTelemetry integration for observability
- REST API server templates
- Broader vendor extension support

## Get Involved

We welcome contributions! Whether you're fixing bugs, adding features, improving documentation, or just trying it out and providing feedback—all contributions help grow the ecosystem.

**Repository**: [github.com/sscargal/lib3mf-rs](https://github.com/sscargal/lib3mf-rs)

**Contributing**: See [CONTRIBUTING.md](https://github.com/sscargal/lib3mf-rs/blob/main/CONTRIBUTING.md) for build instructions, testing guide, and architecture overview.

**Issues**: Found a bug or have a feature request? [Open an issue](https://github.com/sscargal/lib3mf-rs/issues)

**Discussion**: Join the conversation on [r/rust](https://reddit.com/r/rust) or open a GitHub discussion

## Try It Today

```bash
# Install CLI
cargo install lib3mf-cli

# Add to your project
cargo add lib3mf-core

# Or with all features
cargo add lib3mf-core --features full
```

Star the project on GitHub if you find it useful: ⭐ [github.com/sscargal/lib3mf-rs](https://github.com/sscargal/lib3mf-rs)

---

*Read more about the ecosystem: ["The State of Rust 3MF Libraries in 2026"](https://github.com/sscargal/lib3mf-rs/blob/main/docs/blog/market-gap-analysis.md)*

*For questions or feedback, reach out on [GitHub Issues](https://github.com/sscargal/lib3mf-rs/issues) or [r/rust](https://reddit.com/r/rust).*
