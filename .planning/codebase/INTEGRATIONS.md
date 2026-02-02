# External Integrations

**Analysis Date:** 2026-02-02

## APIs & External Services

**3MF Specification Compliance:**
- 3MF.io standard implementation (no API integration, offline spec-compliant parsing/writing)
- Implements standard extension specifications:
  - 3MF Core Specification v1.4.0
  - Beam Lattice Extension v1.2.0
  - Boolean Operations Extension v1.1.1
  - Displacement Extension v1.0.0
  - Materials and Properties Extension v1.2.1
  - Production Extension v1.1.2
  - Secure Content Extension v1.0.2
  - Slice Extension v1.0.2
  - Volumetric Extension v0.8.0

**Vendor Extensions:**
- Bambu Studio 3MF project files parsing (vendor-specific metadata extraction)
  - Located in: `crates/lib3mf-core/src/parser/bambu_config.rs`
  - Extracts: Plate configurations, filament data, print times, project metadata
  - No network communication - local file parsing only

## Data Storage

**Databases:**
- None - Stateless library-only architecture

**File Storage:**
- Local filesystem only
- ZIP archive (OPC container) reading/writing:
  - Archive handling: `crates/lib3mf-core/src/archive/zip_archive.rs`
  - OPC parsing: `crates/lib3mf-core/src/archive/opc.rs`
  - async ZIP support: `crates/lib3mf-async/src/lib.rs` via `async_zip` crate

**Caching:**
- No external caching service
- In-memory model representation after parsing (immutable cache of parsed structures)

## Authentication & Identity

**Auth Provider:**
- None required - Library does not authenticate with external services

**Digital Signatures:**
- XML-DSIG certificate validation (X.509)
  - Implementation: `crates/lib3mf-core/src/crypto/mod.rs`
  - Parser: `crates/lib3mf-core/src/parser/crypto_parser.rs`
  - Uses: `x509-parser 0.16` for certificate chain validation
  - No external PKI lookup - validates embedded certificates only

## Monitoring & Observability

**Error Tracking:**
- None - No error tracking service integration

**Logs:**
- Console output only via standard Rust logging (println!, eprintln!)
- CLI tool output: Human-readable text or JSON format
  - JSON output: `serde_json` serialization in `crates/lib3mf-cli/src/main.rs`
  - No remote log aggregation

## CI/CD & Deployment

**Hosting:**
- GitHub as source repository
- Crates.io for Rust library publication
- NPM Registry for WebAssembly packages

**CI Pipeline:**
- GitHub Actions (`.github/workflows/`)
  - `ci.yml` - Build, test, lint on push to main and pull requests
    - Multi-platform testing: Ubuntu (x86_64), macOS, Windows
    - Code coverage: Codecov integration (optional token: `secrets.CODECOV_TOKEN`)
    - Cargo llvm-cov for coverage generation
  - `security-audit.yml` - cargo-audit and cargo-deny checks
  - `release.yml` - Triggered on version tag (v*)
    - Publishes to crates.io (5 crates via `CRATES_IO_TOKEN`)
    - Publishes WASM to NPM (via `NPM_TOKEN`)
    - Builds and bundles WASM with wasm-pack
  - `wasm_ci.yml` - WebAssembly-specific CI pipeline

## Environment Configuration

**Required env vars:**
- None for core library operation
- CI/CD only (GitHub Secrets):
  - `CRATES_IO_TOKEN` - Cargo publish authentication
  - `NPM_TOKEN` - NPM registry authentication
  - `CODECOV_TOKEN` - Codecov upload token (optional, non-blocking)

**Secrets location:**
- GitHub Secrets vault (`.github/workflows/` references)
- No local .env files (not applicable for library)

## Webhooks & Callbacks

**Incoming:**
- None - Library does not expose webhook endpoints

**Outgoing:**
- None - Library does not make outbound webhook calls
- GitHub Actions webhook: Standard GitHub-triggered on push/PR (built-in)

## Network & Remote Access

**Network Requirements:**
- None for core library - purely offline file processing
- Cargo downloads (during build only, to crates.io registry)
- Optional: Codecov.io for coverage reports (non-critical)

**Third-Party Service Dependencies:**
- crates.io registry (standard Rust dependency resolution)
- GitHub Actions runtime (CI/CD only)
- Codecov API endpoint (optional, development-only)

## Format Converters & Integrations

**STL (Stereolithography):**
- Bidirectional conversion: STL ↔ 3MF
- Implementation: `crates/lib3mf-converters/src/stl.rs`
- No network communication - pure in-memory conversion

**OBJ (Wavefront):**
- Bidirectional conversion: OBJ ↔ 3MF
- Implementation: `crates/lib3mf-converters/src/obj.rs`
- No network communication - pure in-memory conversion

## Security & Crypto Integrations

**Encryption:**
- AES-GCM (Advanced Encryption Standard in Galois/Counter Mode)
  - Crate: `aes-gcm 0.10.3`
  - For: Secure content extension data encryption/decryption
  - Implementation: `crates/lib3mf-core/src/crypto/encryption.rs`

**Digital Signatures:**
- RSA-SHA2 signature validation
  - Crate: `rsa 0.9.10` with sha2 feature
  - X.509 certificate chain: `x509-parser 0.16`
  - For: Secure content extension signature verification
  - Implementation: `crates/lib3mf-core/src/crypto/signature.rs`

**Random Generation:**
- `rand 0.8` for cryptographic random data
- WASM-compatible via `getrandom 0.2` with js feature

---

*Integration audit: 2026-02-02*
