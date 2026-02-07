# r/rust Post Draft

**Subreddit:** r/rust

**Title:** Expanding Rust 3D printing ecosystem with lib3mf-rs - modern 3MF toolkit

**Body:**

Hi r/rust!

I've been working on a pure Rust implementation of the 3MF file format used in 3D printing. Today I'm excited to share lib3mf-rs with the community.

**What is 3MF?**

3MF (3D Manufacturing Format) is the modern successor to STL for 3D printing, supporting colors, textures, materials, and much more. It's backed by the 3MF Consortium (Microsoft, HP, Autodesk, Ultimaker, Prusa) and used by all modern slicers.

**Key features:**

- Pure Rust, zero unsafe code
- All 9 official 3MF extensions (Materials, Slicing, Security, Boolean Operations, Displacement, Beam Lattice, Volumetric, Production)
- Async I/O support (lib3mf-async with tokio)
- WebAssembly bindings (lib3mf-wasm for browser deployment)
- CLI tools included (lib3mf-cli)
- Format converters (STL ↔ 3MF ↔ OBJ)
- Progressive validation (4 levels from minimal to paranoid geometry checks)
- Feature flags for minimal dependencies (154 crates without crypto vs. 300 with all features)
- 90%+ test coverage with comprehensive fuzzing
- 100% pass rate on official 3MF Consortium MUSTPASS tests (13/13 valid files)

**Quick start:**

```bash
# Install CLI
cargo install lib3mf-cli
lib3mf-cli stats your_model.3mf

# Or add to your project
cargo add lib3mf-core
```

**Multi-crate architecture:**

The project is structured as five specialized crates:
- `lib3mf-core` - Main parsing/validation/writing engine
- `lib3mf-async` - Tokio-based async I/O
- `lib3mf-cli` - Command-line tools
- `lib3mf-converters` - STL/OBJ format conversion
- `lib3mf-wasm` - WebAssembly bindings

Use exactly what you need. Building a web service? Add async. Need browser deployment? Include WASM.

**Performance:**

Benchmarked on AMD Ryzen 7 (8 cores):
- Small files (1.2 KB): 21.4 µs parse time
- Medium files (258 KB): 15.8 ms parse time
- Validation overhead: Minimal (14.9 ns) to Paranoid (85.6 ms with deep geometry checks)

Full details: https://github.com/sscargal/lib3mf-rs/blob/main/docs/performance.md

**Ecosystem:**

lib3mf-rs joins other Rust 3MF implementations like telecos/lib3mf_rust (great for academic research and geometric operations), threemf2 (lightweight basic support), and specialized conversion tools. Multiple implementations help validate the spec and serve different use cases.

**Links:**

- GitHub: https://github.com/sscargal/lib3mf-rs
- Documentation: https://sscargal.github.io/lib3mf-rs/
- Crates.io: https://crates.io/crates/lib3mf-core
- Conformance testing: https://github.com/sscargal/lib3mf-rs/blob/main/docs/conformance.md

I'd love to hear your feedback, especially from anyone working with 3D printing or CAD in Rust!

---

**Note for posting:**

- Post during US working hours (9-11 AM Pacific) for best visibility
- Do not ask for upvotes
- Be responsive to comments and questions
- Cross-link to related discussions if relevant
