# Technology Stack

**Analysis Date:** 2026-02-02

## Languages

**Primary:**
- Rust 1.93.0+ - All production and library code (`crates/lib3mf-core`, `crates/lib3mf-cli`, `crates/lib3mf-converters`, `crates/lib3mf-async`, `crates/lib3mf-wasm`)

**Secondary:**
- WebAssembly - Target platform for `crates/lib3mf-wasm` bindings

## Runtime

**Environment:**
- Rust 1.93.0 stable (pinned in `rust-toolchain.toml`)

**Package Manager:**
- Cargo (built-in Rust package manager)
- Lockfile: Present (`Cargo.lock` - committed to repository)

## Frameworks

**Core:**
- Tokio 1.49.0 - Asynchronous runtime (optional feature: `async`)
- Clap 4.5 - CLI argument parsing with derive macros (`crates/lib3mf-cli`)

**Serialization:**
- Serde 1.0 - Data serialization/deserialization framework (used throughout)
- serde_json 1.0 - JSON encoding/decoding (`crates/lib3mf-cli` for output formatting)
- quick-xml 0.37.0 - Event-based XML parsing with serialization support (`crates/lib3mf-core`)

**Testing:**
- proptest 1.6.0 - Property-based testing for robustness (`crates/lib3mf-core` dev-dependencies)
- criterion 0.5 - Benchmarking framework (workspace dependency, used in `crates/lib3mf-core/benches/`)
- wasm-bindgen-test 0.3 - WebAssembly testing framework (`crates/lib3mf-wasm`)

**Build/Dev:**
- vergen-gix 9.1.0 - Build-time Git metadata injection (`crates/lib3mf-cli/build.rs`)
- wasm-pack - WebAssembly bundler for NPM publishing

## Key Dependencies

**Critical - Archive & Compression:**
- zip 2.2.0 (default-features = false, deflate feature) - ZIP container handling for 3MF files
- async_zip 0.0.18 (tokio, deflate features) - Async ZIP support for high-performance I/O

**Critical - Cryptography:**
- aes-gcm 0.10.3 - AES-GCM encryption for secure content
- rsa 0.9.10 (sha2 feature) - RSA signing for digital signatures
- sha2 0.10 - SHA-2 hashing
- sha1 0.10 - SHA-1 hashing
- x509-parser 0.16 - X.509 certificate chain validation
- base64 0.22.1 - Base64 encoding/decoding for crypto data

**Performance & Math:**
- rayon 1.10 - Data parallelization (optional feature: `parallel`)
- glam 0.31.0 (serde feature) - Fast linear algebra for geometry calculations
- lexical-core 1.0.6+ - High-performance float parsing (spec-compliant)

**Utilities:**
- anyhow 1.0 - Flexible error handling
- thiserror 2.0.18 - Structured error definitions
- uuid 1.20.0 (v4, serde, js features) - UUID generation and serialization
- byteorder 1.5 - Byte order conversions
- rand 0.8 - Random number generation

**WebAssembly-Specific:**
- wasm-bindgen 0.2 - JavaScript/Rust interop for WASM
- console_error_panic_hook 0.1 - Browser console panic logging
- getrandom 0.2 (js feature) - Random number generation in WASM

**Async Support:**
- async-trait 0.1.89 - Trait support for async functions
- futures-lite 2.6.1 - Async utilities
- tokio-util 0.7.18 (compat feature) - Tokio utilities for compatibility

## Configuration

**Environment:**
- No environment variables required for library operation
- No .env file usage detected
- Build-time configuration: Git metadata injected via `vergen-gix` (build.rs in CLI crate)

**Build:**
- `Cargo.toml` (workspace root) - Workspace configuration with shared dependencies
- Individual `Cargo.toml` in each crate (`crates/lib3mf-core/Cargo.toml`, etc.)
- `rustfmt.toml` - Code formatting: max_width = 100, edition = 2021
- `rust-toolchain.toml` - Pinned Rust version with rustfmt and clippy components

**Feature Flags:**
- `lib3mf-core` - `default = ["parallel"]`
  - `async` - Enables Tokio support
  - `parallel` - Enables Rayon for multi-threaded mesh operations
- `lib3mf-wasm` - No feature flags

## Platform Requirements

**Development:**
- Rust 1.93.0+
- Cargo (included with Rust)
- Platform-specific tools:
  - Linux/macOS: Standard build tools
  - Windows: MSVC or GNU toolchain (rustup handles this)

**Production:**
- Targets: Windows, Linux, macOS (x86_64, aarch64 via Apple Silicon)
- WebAssembly: Browser support via WASM bindings (`crates/lib3mf-wasm`)
- CLI binary: Statically linked when built with `--release`

**CI/CD Platform:**
- GitHub Actions (`.github/workflows/ci.yml`, `release.yml`, `security-audit.yml`, `wasm_ci.yml`)
- Codecov integration for code coverage

---

*Stack analysis: 2026-02-02*
