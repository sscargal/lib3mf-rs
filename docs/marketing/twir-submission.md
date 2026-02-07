# This Week in Rust Submission

**Where to submit:** https://users.rust-lang.org/t/crate-of-the-week/2704

**Post content:**

I'd like to suggest [lib3mf-rs](https://crates.io/crates/lib3mf-core) for Crate of the Week.

lib3mf-rs is a pure Rust implementation of the 3MF (3D Manufacturing Format) specification, providing production-ready parsing, validation, and writing of 3MF files used in 3D printing and additive manufacturing.

Key features:

- All 9 official 3MF extensions (Materials, Slicing, Secure Content, Boolean Operations, Displacement, Beam Lattice, Volumetric, Production)
- Async I/O support via tokio for high-throughput applications
- WebAssembly bindings for browser and edge deployment
- CLI tools for immediate productivity (stats, validate, convert, diff, extract)
- Format converters (STL ↔ 3MF ↔ OBJ)
- Zero unsafe code with 90%+ test coverage and comprehensive fuzzing
- Progressive validation system (4 levels: minimal, standard, strict, paranoid)
- Multi-crate architecture (core, async, cli, converters, wasm) for modular use
- 100% pass rate on official 3MF Consortium MUSTPASS tests (13/13 valid files)
- Feature flags for minimal dependencies (154 crates without crypto vs. 300 with all features)

3MF is the modern standard for 3D printing, used by Bambu Studio, PrusaSlicer, and Ultimaker Cura. It's the successor to STL with support for colors, materials, textures, and advanced manufacturing features.

GitHub: https://github.com/sscargal/lib3mf-rs
Docs: https://sscargal.github.io/lib3mf-rs/
Performance: https://github.com/sscargal/lib3mf-rs/blob/main/docs/performance.md
Conformance: https://github.com/sscargal/lib3mf-rs/blob/main/docs/conformance.md

---

**Note:**

- Submit to the forum thread, not via email or DM
- Wait until blog posts and README updates are live (foundation first)
- Be patient - TWIR editors review many suggestions
- No need to bump or resubmit unless explicitly requested
