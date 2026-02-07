# Show HN Post Draft

**Title:** Show HN: lib3mf-rs - Pure Rust 3MF parser for 3D printing

**Body:**

I've been working on a pure Rust implementation of the 3MF file format used in 3D printing. lib3mf-rs parses, validates, and writes 3MF files with support for all 9 official extensions.

3MF is the modern replacement for STL in 3D printing—it supports colors, materials, textures, digital signatures, and advanced features like boolean operations and lattice structures. All major slicers (Bambu Studio, PrusaSlicer, Ultimaker Cura) use it.

Key differentiators from existing solutions:

• No C++ dependencies - memory-safe by design (zero unsafe code)
• Async I/O for high-throughput applications (tokio-based)
• WebAssembly support for browser/edge deployment
• Comprehensive CLI tools included (stats, validate, convert, diff, extract)
• Feature flags for minimal dependencies (154 crates minimal vs. 300 with all features)
• Progressive validation: 4 levels from minimal (14.9 ns) to paranoid (85.6 ms with BVH self-intersection detection)
• 86% pass rate on official 3MF Consortium conformance tests (44/51)

Multi-crate architecture (lib3mf-core, lib3mf-async, lib3mf-cli, lib3mf-converters, lib3mf-wasm) lets you use exactly what you need.

Try it:

  cargo install lib3mf-cli && lib3mf-cli stats your_model.3mf

GitHub: https://github.com/sscargal/lib3mf-rs
Docs: https://sscargal.github.io/lib3mf-rs/
Performance benchmarks: https://github.com/sscargal/lib3mf-rs/blob/main/docs/performance.md

---

**Note for posting:**

- Use first-person voice per HN guidelines
- Post during 9 AM - 12 PM Pacific for best visibility
- Never share link asking for upvotes
- Be prepared to answer technical questions in comments
- Keep responses professional and substantive
