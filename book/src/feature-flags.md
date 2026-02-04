# Feature Flags

lib3mf-core uses Cargo feature flags to minimize dependencies, allowing you to choose exactly what functionality you need.

## Overview

By default, lib3mf-core has **zero optional dependencies**. This gives you the smallest possible build with just core 3MF parsing and writing capabilities.

You can selectively enable features based on your requirements:

| Feature | What It Enables | Dependencies Added | When to Use |
|---------|-----------------|-------------------|-------------|
| `crypto` | Digital signatures and encryption (Secure Content Extension) | aes-gcm, rsa, sha1, sha2, x509-parser, rand, base64 (~146 crates) | Signed/encrypted 3MF files |
| `parallel` | Multi-threaded mesh processing using Rayon | rayon (~6 crates) | Large files, multi-core CPUs |
| `png-validation` | PNG texture validation | png (~15 crates) | Texture quality checks |
| `full` | All features enabled | All of the above | Complete functionality |

## Dependency Impact

**Minimal build** (no features):

```toml
[dependencies]
lib3mf-core = { version = "0.1", default-features = false }
```

**Result:** ~154 crate dependencies

**Crypto-enabled build:**

```toml
[dependencies]
lib3mf-core = { version = "0.1", features = ["crypto"] }
```

**Result:** ~300 crate dependencies (+146 from crypto)

**Full-featured build:**

```toml
[dependencies]
lib3mf-core = { version = "0.1", features = ["full"] }
```

**Result:** ~300 crate dependencies (crypto dominates)

Users who don't need cryptographic features save **48% of dependencies**.

## Feature Combinations

### Minimal — Smallest Footprint

```toml
[dependencies]
lib3mf-core = { version = "0.1", default-features = false }
```

**Provides:**
- Core 3MF parsing and writing
- All geometry types (meshes, lattices, slices, voxels)
- All material types (colors, textures, composites)
- Boolean operations and displacement (parsing only, no evaluation)
- Validation (all 4 levels)
- Single-threaded mesh processing

**Missing:**
- Digital signature verification
- Content encryption/decryption
- Multi-threaded mesh processing
- PNG texture validation

**Best for:**
- Embedded systems
- WebAssembly builds
- Size-critical applications
- Applications that don't need security features

### Crypto Only — Secure Files

```toml
[dependencies]
lib3mf-core = { version = "0.1", features = ["crypto"] }
```

**Adds:**
- XML-DSIG digital signature verification
- AES-GCM content encryption/decryption
- X.509 certificate parsing
- RSA public key operations

**Use when:**
- Working with signed 3MF files
- Working with encrypted content
- Manufacturing workflows requiring authenticity verification
- Single-threaded processing is acceptable

### Parallel Only — Fast Processing

```toml
[dependencies]
lib3mf-core = { version = "0.1", features = ["parallel"] }
```

**Adds:**
- Rayon-based parallel iteration
- Multi-threaded AABB computation
- Parallel area/volume calculation
- Parallel statistics computation

**Use when:**
- Processing large meshes (>100K triangles)
- Multi-core CPU available
- Speed is more important than dependency count
- No security features needed

### Full-Featured — Everything

```toml
[dependencies]
lib3mf-core = { version = "0.1", features = ["full"] }
```

**Enables:**
- All crypto features
- All parallel features
- PNG texture validation

**Use when:**
- Building production tools
- Need all functionality
- Dependency count not a concern
- Desktop or server applications

## Checking Dependency Count

You can verify the dependency count for different feature combinations:

```bash
# Minimal build
cargo tree -p lib3mf-core --no-default-features | wc -l

# Crypto only
cargo tree -p lib3mf-core --no-default-features --features crypto | wc -l

# Parallel only
cargo tree -p lib3mf-core --no-default-features --features parallel | wc -l

# Full-featured
cargo tree -p lib3mf-core --all-features | wc -l
```

## Build Time and Binary Size Impact

Feature flags affect both compile time and binary size:

| Configuration | Compile Time (clean) | Binary Size (release) | Notes |
|---------------|---------------------|----------------------|-------|
| Minimal | ~30s | ~2.5 MB | Fastest builds |
| Crypto only | ~90s | ~4.2 MB | Crypto adds significant compile time |
| Parallel only | ~35s | ~2.8 MB | Rayon is lightweight |
| Full | ~90s | ~4.5 MB | Crypto dominates |

*Measured on Apple M1 Max with 10 cores*

## Feature-Gated API Items

Some API items are only available when specific features are enabled. The rustdoc documentation marks these with badges:

**Crypto-gated items:**
- `model::SecureContent`
- `crypto::verify_signature()`
- `crypto::decrypt_content()`
- `model::KeyStore`

**Parallel-gated behavior:**
- `Model::compute_stats()` uses parallel iteration when `parallel` enabled
- `Mesh::compute_aabb()` parallelizes across triangles
- `validation::geometry::check_self_intersection()` uses parallel BVH construction

**PNG-validation-gated:**
- `validation::validate_png_texture()`

In your code, you can check for features with:

```rust
#[cfg(feature = "crypto")]
use lib3mf_core::crypto::verify_signature;

#[cfg(feature = "crypto")]
fn verify_model(model: &Model) -> Result<bool> {
    verify_signature(&model.signature?)
}

#[cfg(not(feature = "crypto"))]
fn verify_model(_model: &Model) -> Result<bool> {
    Err("Crypto feature not enabled".into())
}
```

## Recommendations

**For applications:**
- **CLI tools** → Use `full` (users expect all features)
- **Web services** → Use `crypto` + `parallel` (security + speed)
- **WASM builds** → Use minimal (size matters, crypto doesn't work in WASM)
- **Embedded** → Use minimal (constrained resources)

**For libraries:**
- Re-export lib3mf-core with `default-features = false`
- Let users choose features via your crate's feature flags
- Document which lib3mf features you require

**Example library pattern:**

```toml
[dependencies]
lib3mf-core = { version = "0.1", default-features = false }

[features]
default = []
crypto = ["lib3mf-core/crypto"]
parallel = ["lib3mf-core/parallel"]
full = ["crypto", "parallel"]
```

## CI/CD Testing

To ensure your code works with different feature combinations, test them all in CI:

```yaml
# GitHub Actions example
strategy:
  matrix:
    features:
      - "--no-default-features"
      - "--features crypto"
      - "--features parallel"
      - "--all-features"
steps:
  - run: cargo test -p lib3mf-core ${{ matrix.features }}
```

lib3mf-rs uses this approach to guarantee compatibility across all configurations.

## Next Steps

- **[Getting Started](getting-started.md)** — Choose features for your first program
- **[Extensions](extensions.md)** — Learn which extensions require which features
- **[API Reference](../rustdoc/lib3mf_core/index.html)** — See feature badges on API items
