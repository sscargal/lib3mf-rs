# lib3mf-cli

[![Crates.io](https://img.shields.io/crates/v/lib3mf-cli.svg)](https://crates.io/crates/lib3mf-cli)
[![docs.rs](https://docs.rs/lib3mf-cli/badge.svg)](https://docs.rs/lib3mf-cli)
[![License](https://img.shields.io/crates/l/lib3mf-cli.svg)](LICENSE)

Command-line tool for analyzing, validating, and processing 3MF files.

## When to Use This Crate

Use `lib3mf-cli` when you need:
- Quick inspection of 3MF files without writing code
- Validation and quality checks in CI/CD pipelines
- Batch processing and automation scripts
- Format conversion from command line

## Installation

```bash
cargo install lib3mf-cli
```

## Quick Start

```bash
# Analyze a 3MF file
3mf stats model.3mf

# Get JSON output
3mf stats model.3mf --format json

# Validate with paranoid checks
3mf validate model.3mf --level paranoid

# Convert STL to 3MF
3mf convert input.stl output.3mf

# List archive contents
3mf list model.3mf --format tree
```

## Commands

| Command | Description |
|---------|-------------|
| `stats` | Display model statistics (geometry, materials, metadata) |
| `validate` | Run validation checks (Minimal/Standard/Strict/Paranoid) |
| `list` | List files in 3MF archive |
| `extract` | Extract files from archive |
| `diff` | Compare two 3MF files |
| `copy` | Read and write 3MF (roundtrip test) |
| `convert` | Convert between 3MF, STL, and OBJ |

## Features

- Full validation support (4 validation levels)
- Bambu Studio project file analysis
- Digital signature verification
- JSON output for scripting
- Format conversion (STL ↔ 3MF ↔ OBJ)

## CI/CD Integration

Use in continuous integration to validate 3MF files:

```yaml
- name: Validate 3MF
  run: |
    cargo install lib3mf-cli
    3mf validate model.3mf --level strict
```

## Related

- [lib3mf-core](https://crates.io/crates/lib3mf-core) - Core parsing library
- [Full Documentation](https://sscargal.github.io/lib3mf-rs/)

## License

BSD-2-Clause
