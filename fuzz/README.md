# lib3mf-rs Fuzzing Infrastructure

This directory contains fuzz testing infrastructure for lib3mf-rs using [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) and [libFuzzer](https://llvm.org/docs/LibFuzzer.html).

## Quick Start

```bash
# Install nightly toolchain and cargo-fuzz
rustup toolchain install nightly
cargo +nightly install cargo-fuzz

# Run a 60-second smoke test
cargo +nightly fuzz run parse_model -- -max_total_time=60

# Run indefinitely (Ctrl+C to stop)
cargo +nightly fuzz run parse_model
```

## Fuzz Targets

| Target | Attack Surface | Description |
|--------|---------------|-------------|
| `parse_model` | ZIP + XML | Full 3MF file parsing with invariant checks |
| `parse_xml` | XML | Direct XML model parsing, bypasses ZIP layer |
| `parse_materials` | Materials | Color groups, textures, composites |
| `parse_crypto` | Security | Digital signatures, encryption metadata |
| `parse_extensions` | Extensions | BeamLattice, Slice, Boolean, Displacement |
| `parse_opc` | OPC | Relationship and content type XML |
| `writer_roundtrip` | Writer | Parse-write-reparse invariant testing |

## Directory Structure

```
fuzz/
├── Cargo.toml           # Fuzz crate dependencies
├── README.md            # This file
├── fuzz_targets/        # Fuzz target implementations
│   ├── parse_model.rs
│   ├── parse_xml.rs
│   ├── parse_materials.rs
│   ├── parse_crypto.rs
│   ├── parse_extensions.rs
│   ├── parse_opc.rs
│   └── writer_roundtrip.rs
├── corpus/              # Seed inputs (version controlled)
│   ├── parse_model/
│   │   └── seed.3mf
│   ├── parse_xml/
│   │   └── minimal.xml
│   └── ...
├── dictionaries/        # libFuzzer dictionaries
│   └── 3mf.dict
└── artifacts/           # Crash outputs (gitignored)
    └── {target}/
        └── crash-{sha1}
```

## Common Commands

### Running Fuzz Tests

```bash
# Basic run (indefinite)
cargo +nightly fuzz run parse_model

# Time-limited run
cargo +nightly fuzz run parse_model -- -max_total_time=60

# With dictionary for better coverage
cargo +nightly fuzz run parse_xml -- -dict=dictionaries/3mf.dict

# Parallel fuzzing (4 workers)
cargo +nightly fuzz run parse_model -- -workers=4 -jobs=4

# ASCII-only inputs (useful for XML targets)
cargo +nightly fuzz run parse_xml -- -only_ascii=1
```

### Crash Analysis

```bash
# Reproduce a crash
cargo +nightly fuzz run parse_model fuzz/artifacts/parse_model/crash-abc123

# Minimize a crash to smallest reproducing input
cargo +nightly fuzz tmin parse_model fuzz/artifacts/parse_model/crash-abc123

# Minimize with more iterations
cargo +nightly fuzz tmin parse_model crash-file -- -runs=10000
```

### Coverage Analysis

```bash
# Generate coverage data
cargo +nightly fuzz coverage parse_model

# View coverage report (requires cargo-llvm-cov)
cargo install cargo-llvm-cov
cargo cov -- show fuzz/target/*/release/parse_model \
    --format=html \
    -instr-profile=fuzz/coverage/parse_model/coverage.profdata \
    > coverage.html
```

### Corpus Management

```bash
# Merge and deduplicate corpus
cargo +nightly fuzz run parse_model -- -merge=1

# Minimize corpus (remove redundant inputs)
cargo +nightly fuzz cmin parse_model
```

## CI Integration

Fuzzing runs automatically on pull requests:

1. **Smoke Test**: Each target runs for 60 seconds
2. **Crash Handling**: If a crash is found:
   - Crash is minimized with `fuzz tmin`
   - Artifacts uploaded to GitHub Actions
   - Comment posted on PR with reproduction steps
   - Issue created with `fuzzing` and `crash` labels

Fuzzing failures are **warnings, not blockers**. PRs proceed even if fuzzing finds crashes.

## Adding a New Fuzz Target

1. Create `fuzz/fuzz_targets/new_target.rs`:
   ```rust
   #![no_main]
   use libfuzzer_sys::fuzz_target;

   fuzz_target!(|data: &[u8]| {
       // Your fuzzing code here
   });
   ```

2. Add to `fuzz/Cargo.toml`:
   ```toml
   [[bin]]
   name = "new_target"
   path = "fuzz_targets/new_target.rs"
   test = false
   doc = false
   ```

3. Create corpus directory: `mkdir fuzz/corpus/new_target`
4. Add seed files if available
5. Update `.github/workflows/fuzz.yml` matrix

## Handling Crashes

When you find a crash:

1. **Minimize**: `cargo +nightly fuzz tmin TARGET crash-file`
2. **Debug**: Run with backtrace: `RUST_BACKTRACE=1 cargo +nightly fuzz run TARGET minimized-crash`
3. **Fix**: Implement the fix in library code
4. **Regress**: Copy minimized crash to `tests/fuzz_regression/`
5. **Test**: Verify crash no longer reproduces

## Performance Tips

- Start with `-max_len=1024` for faster iteration
- Use `-only_ascii=1` for XML targets
- Increase `-workers` for multi-core fuzzing
- Monitor `exec/s` output - should be >100 for effective fuzzing
- If stuck, check coverage report for unreached code

## Troubleshooting

**"libfuzzer-sys requires nightly"**
- Ensure you're using nightly: `cargo +nightly fuzz run`

**Slow execution (<100 exec/s)**
- Add `-max_len=1024` to limit input size
- Consider `-sanitizer=none` if no unsafe code (4-10x speedup)

**"failed to reproduce"**
- Non-deterministic bug - try `-seed=N` to fix randomization
- Check for global mutable state

**Coverage not improving**
- Add more seed files to corpus
- Check if early validation rejects most inputs
- Use dictionary file for structured inputs
