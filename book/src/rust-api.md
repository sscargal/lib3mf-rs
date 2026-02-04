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

Command-line interface tool providing the `3mf` binary for file inspection, validation, conversion, and manipulation from the terminal.

**Commands:** stats, list, validate, copy, diff, extract

## Format Conversion

### [lib3mf-converters](../rustdoc/lib3mf_converters/index.html)

Format converters for interoperability with other 3D file formats, including STL and OBJ conversion to and from 3MF.

## Async I/O

### [lib3mf-async](../rustdoc/lib3mf_async/index.html)

Async I/O support using tokio and async-zip for non-blocking file operations, suitable for web servers and concurrent applications.

## WebAssembly

### [lib3mf-wasm](../rustdoc/lib3mf_wasm/index.html)

WebAssembly bindings enabling lib3mf-rs to run in browsers and JavaScript environments.

---

## Viewing API Docs Locally

If you're viewing this locally (not on GitHub Pages), you can generate and open the API documentation by running:

```bash
cargo doc --workspace --all-features --no-deps --open
```

This will build the documentation for all crates and open it in your default browser.
