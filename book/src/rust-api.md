# Rust API Documentation

This section provides links to the auto-generated API documentation for all workspace crates. The API docs are generated from inline documentation in the source code using `rustdoc` and provide detailed information about every public function, struct, enum, and module.

## Core Library

### [lib3mf-core](../rustdoc/lib3mf_core/index.html)

Core library implementation including parser, model structures, validation, writer, and cryptographic support. This is the main crate that provides all fundamental 3MF functionality.

**Key modules:**
- `parser` - XML parsing and model construction
- `model` - Core data structures (Model, Mesh, Build, etc.)
- `validation` - Progressive validation system (Minimal/Standard/Strict/Paranoid)
- `writer` - Model serialization back to 3MF format
- `archive` - ZIP/OPC container handling
- `crypto` - Secure Content Extension (digital signatures, encryption)

## Command-Line Tools

### [lib3mf-cli](../rustdoc/lib3mf_cli/index.html)

Command-line interface tool providing the `3mf` binary for file inspection, validation, conversion, and manipulation from the terminal. Also exposes command functions as a library module for programmatic use.

**Commands:** stats, list, validate, copy, diff, extract, rels, dump, repair, benchmark, convert, sign, verify, encrypt, decrypt, thumbnails

**Key types:**
- `ExecuteStats` - Generate statistics reports (mesh counts, triangles, volumes)
- `ExecuteList` - List archive contents with tree or flat format
- `ExecuteValidate` - Validate files at Minimal/Standard/Strict/Paranoid levels
- `ExecuteCopy` - Copy and optionally repair 3MF files
- `ExecuteDiff` - Compare two 3MF files for differences

**Feature flags:**
- `crypto` - Enables sign, verify, encrypt, decrypt commands
- `parallel` - Enables multi-threaded processing in repair and stats

## Format Conversion

### [lib3mf-converters](../rustdoc/lib3mf_converters/index.html)

Format converters for interoperability with other 3D file formats. Supports importing/exporting binary STL and basic OBJ files to/from 3MF.

**Key types:**
- `StlImporter` - Read binary STL files into 3MF models
- `StlExporter` - Write 3MF models to binary STL format
- `ObjImporter` - Read OBJ files (vertices and faces) into 3MF
- `ObjExporter` - Write 3MF models to OBJ format

**Features:**
- Binary STL support (ASCII STL not supported)
- Multi-part STL export with `write_with_resolver` for filename mapping
- Basic OBJ import (no materials, textures, or normals)
- Automatic unit conversion and coordinate system handling

**Limitations:**
- STL: Binary format only, no color/material support
- OBJ: No material (.mtl) files, normals, or texture coordinates

## Async I/O

### [lib3mf-async](../rustdoc/lib3mf_async/index.html)

Async I/O support using tokio and async-zip for non-blocking file operations. Ideal for web servers, concurrent applications, and scenarios where blocking I/O would be problematic.

**Key types:**
- `AsyncArchiveReader` - Trait for async archive reading (implemented by `AsyncZipArchive`)
- `AsyncZipArchive` - Async ZIP archive reader using `async_zip`
- `load_model_async` - High-level async function to load 3MF files

**Architecture:**
- Uses `tokio` runtime for async I/O operations
- ZIP archive reading is fully async (non-blocking)
- XML parsing uses `spawn_blocking` (CPU-bound work on thread pool)
- All archive readers must be `Send + Sync` for multi-threaded async

**When to use:**
- Web servers handling concurrent requests
- Applications processing multiple files in parallel
- Scenarios where UI responsiveness matters
- When you already have a tokio runtime

**When NOT to use:**
- Single-threaded CLI tools (use synchronous `lib3mf-core` instead)
- When simplicity is more important than concurrency

## WebAssembly

### [lib3mf-wasm](../rustdoc/lib3mf_wasm/index.html)

WebAssembly bindings enabling lib3mf-rs to run in browsers and JavaScript environments. Provides a JavaScript-friendly API for parsing and analyzing 3MF files in the browser.

**Key types:**
- `WasmModel` - JavaScript-accessible model representation
- `set_panic_hook` - Improved panic messages in browser console

**Features:**
- Parse 3MF files from `Uint8Array` in JavaScript
- Access basic model information (units, objects, build items)
- Browser-compatible API using `wasm-bindgen`

**Building:**
```bash
wasm-pack build crates/lib3mf-wasm --target web
```

**Current status:**
- Early-stage bindings with limited API surface
- Basic parsing and model inspection supported
- Advanced features (validation, writing, crypto) not yet exposed
- Future expansion planned based on community needs

---

## Viewing API Docs Locally

If you're viewing this locally (not on GitHub Pages), you can generate and open the API documentation by running:

```bash
cargo doc --workspace --all-features --no-deps --open
```

This will build the documentation for all crates and open it in your default browser.
