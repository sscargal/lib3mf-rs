# Rust 3MF Ecosystem

This document provides an objective comparison of Rust libraries for working with 3MF files.

## Libraries Overview

| Library | Focus | Maintained | Repository |
|---------|-------|------------|------------|
| [lib3mf-rs](https://github.com/sscargal/lib3mf-rs) | Enterprise/production toolkit | Active | github.com/sscargal/lib3mf-rs |
| [lib3mf](https://github.com/telecos/lib3mf_rust) | Research/conformance | Active | github.com/telecos/lib3mf_rust |
| [threemf2](https://crates.io/crates/threemf2) | Simple 3MF reading | Limited | - |
| [thdmaker](https://crates.io/crates/thdmaker) | STL/AMF focus | Active | github.com/khvorov45/thdmaker |
| [mesh_rs](https://crates.io/crates/mesh_rs) | General mesh formats | Active | github.com/happyrust/mesh_rs |
| [stlto3mf](https://crates.io/crates/stlto3mf) | STL to 3MF conversion | Stable | github.com/jguhlin/stlto3mf |

## Detailed Comparison

### lib3mf-rs (this project)

**Crates:** lib3mf-core, lib3mf-async, lib3mf-cli, lib3mf-converters, lib3mf-wasm

**Version:** 0.1.0 (published 2026-02-04)

**Core capabilities:**
- Parse 3MF files (all 9 official extensions)
- Write 3MF files
- 4-level validation system (Minimal/Standard/Strict/Paranoid)
- Extension support: Materials, Slicing, Security, Boolean Operations, Displacement, Beam Lattice, Volumetric, Production
- Bambu Studio project files

**Modern features:**
- Async I/O support (tokio-based)
- WebAssembly bindings for browser deployment
- CLI tools included
- Format converters (STL ↔ 3MF ↔ OBJ)
- Streaming parser for large files

**Quality metrics:**
- 90%+ test coverage
- Comprehensive fuzzing infrastructure
- Zero unsafe code
- 86% conformance on 3MF Consortium test suite (44/51 tests)

**License:** MIT OR Apache-2.0

**Best for:** Production applications requiring async I/O, browser deployment, comprehensive validation, or CLI tools.

### lib3mf (telecos)

**Crate:** lib3mf

**Version:** 0.1.0 (published 2026-02-04)

**Core capabilities:**
- Parse 3MF files
- Write 3MF files
- Validation support
- Extension support: Production, Materials, Slice, Beam Lattice

**Modern features:**
- Geometric operations via parry3d integration
- 2,200+ test cases

**Quality metrics:**
- Extensive test suite (2,200+ tests)
- 3MF Consortium test suite integration
- Academic/research focus

**License:** MIT

**Best for:** Academic research, geometric operations, conformance testing.

### threemf2

**Crate:** threemf2

**Version:** 0.1.2 (last updated 2025-11-30)

**Core capabilities:**
- Parse 3MF files (basic support)
- Write 3MF files (basic support)
- Limited extension support

**Modern features:** None

**Quality metrics:** Basic test coverage

**License:** MIT

**Best for:** Simple 3MF reading use cases with minimal dependencies.

### thdmaker

**Crate:** thdmaker

**Version:** 0.0.4 (last updated 2026-01-11)

**Core capabilities:**
- STL file parsing
- AMF file parsing
- Some 3MF support

**Modern features:** None

**Quality metrics:** Basic

**License:** MIT

**Best for:** Projects primarily focused on STL/AMF formats that need some 3MF support.

### mesh_rs

**Crate:** mesh_rs

**Version:** 1.0.4 (last updated 2025-12-17)

**Core capabilities:**
- General mesh format support (STL, OBJ, PLY, OFF)
- Limited 3MF support

**Modern features:** None

**Quality metrics:** Basic

**License:** MIT

**Best for:** Projects needing a general-purpose mesh library with multi-format support.

### stlto3mf

**Crate:** stlto3mf

**Version:** 0.1.0 (last updated 2024-07-16)

**Core capabilities:**
- STL to 3MF conversion only
- Basic 3MF writing

**Modern features:** None

**Quality metrics:** Basic

**License:** MIT/Apache-2.0

**Best for:** Single-purpose STL to 3MF conversion tool.

## Choosing the Right Library

### Choose lib3mf-rs when:
- Building production applications
- Need async I/O or WASM support
- Require comprehensive validation (4 levels)
- Want CLI tools included
- Need format conversion (STL/OBJ)
- Require zero unsafe code guarantees
- Need extensive documentation

### Choose lib3mf when:
- Focused on conformance testing
- Need geometric operations (parry3d integration)
- Academic/research use cases
- Want extensive test suite (2,200+ tests)

### Choose threemf2 when:
- Simple 3MF reading is sufficient
- Minimal dependencies preferred
- Basic use case

### Choose thdmaker when:
- Primary focus is STL/AMF formats
- Need some 3MF support

### Choose mesh_rs when:
- Need general mesh library
- Working with multiple formats (STL, OBJ, PLY)
- 3MF is secondary concern

### Choose stlto3mf when:
- Only need STL to 3MF conversion
- Single-purpose tool sufficient

## Feature Comparison Matrix

| Feature | lib3mf-rs | lib3mf | threemf2 | thdmaker | mesh_rs | stlto3mf |
|---------|-----------|--------|----------|----------|---------|----------|
| Parse 3MF | ✅ | ✅ | ✅ | Partial | Partial | ❌ |
| Write 3MF | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ |
| Validation Levels | 4 | Yes | Basic | ❌ | ❌ | ❌ |
| Materials Extension | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Slicing Extension | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Security Extension | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Boolean Operations | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Displacement | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Beam Lattice | ✅ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Async I/O | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| WASM Support | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| CLI Tools | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Format Conversion | ✅ | ❌ | ❌ | Partial | ✅ | STL only |
| Streaming Parser | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Geometric Ops | ❌ | ✅ (parry3d) | ❌ | ❌ | ❌ | ❌ |
| Zero Unsafe | ✅ | ? | ? | ? | ? | ? |
| Test Coverage | 90%+ | 2200+ tests | Basic | Basic | Basic | Basic |
| Fuzzing | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |

## Community

Both lib3mf-rs and lib3mf are valuable contributions to the Rust 3D printing ecosystem. We encourage users to evaluate both based on their specific needs. The existence of multiple implementations demonstrates the maturity and interest in Rust for 3D manufacturing workflows.

If you're building a production system with async requirements, browser deployment, or need comprehensive CLI tools, lib3mf-rs is designed for that use case. If you're focused on academic research, conformance testing, or need geometric operations, lib3mf provides those capabilities.

For other use cases like simple format conversion or general mesh handling, the specialized libraries (threemf2, thdmaker, mesh_rs, stlto3mf) may be sufficient.

## Contributing

Contributions to any of these projects help grow the Rust 3D printing ecosystem. Check each project's repository for contribution guidelines.

## Additional Resources

- [3MF Consortium](https://3mf.io/) - Official 3MF specification
- [3MF Samples](https://github.com/3MFConsortium/3mf-samples) - Official test files
- [lib3mf (C++)](https://github.com/3MFConsortium/lib3mf) - Official C++ implementation
