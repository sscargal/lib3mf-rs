# CLI Guide

The `lib3mf-cli` command-line tool provides comprehensive commands for inspecting, validating, and analyzing 3MF files without writing code.

## Installation

```bash
cargo install lib3mf-cli
```

This installs the `lib3mf-cli` binary. You can verify installation:

```bash
lib3mf-cli --version
```

## Quick Reference

| Command | Purpose |
|---------|---------|
| `stats` | Display file statistics (objects, triangles, materials) |
| `list` | List archive contents |
| `validate` | Run validation checks |
| `copy` | Copy/rewrite a 3MF file (roundtrip test) |
| `diff` | Compare two 3MF files |
| `extract` | Extract files from the archive |

## Commands in Detail

### `stats` — File Statistics

Get a summary of the model including geometry counts, materials, and vendor metadata.

**Basic usage:**

```bash
lib3mf-cli stats model.3mf
```

**Example output:**

```
File: model.3mf
Unit: millimeter

Resources:
  Objects: 5
    - Model: 3
    - Support: 1
    - SolidSupport: 1
  Materials: 2
  Textures: 1
  Build Items: 3

Geometry:
  Triangles: 48,562
  Vertices: 24,281
  Surface Area: 15,234.5 mm²
  Volume: 42,891.2 mm³

Extensions:
  - Materials and Properties v1.2.1
  - Production Extension v1.1.2
```

**JSON output for scripting:**

```bash
lib3mf-cli stats model.3mf --format json
```

```json
{
  "file_path": "model.3mf",
  "unit": "millimeter",
  "resource_counts": {
    "object_count": 5,
    "material_count": 2,
    "texture_count": 1,
    "build_count": 3,
    "type_counts": {
      "Model": 3,
      "Support": 1,
      "SolidSupport": 1
    }
  },
  "geometry": {
    "triangle_count": 48562,
    "vertex_count": 24281,
    "surface_area": 15234.5,
    "volume": 42891.2
  },
  "extensions": [
    "Materials and Properties v1.2.1",
    "Production Extension v1.1.2"
  ]
}
```

**When to use:**
- Quick inspection of 3MF files
- Automated testing (check triangle counts)
- CI/CD validation pipelines
- Comparing file complexity

### `list` — Archive Contents

List all files inside the 3MF archive (3MF is a ZIP container).

**Basic usage:**

```bash
lib3mf-cli list model.3mf
```

**Example output:**

```
[Content_Types].xml (358 bytes)
_rels/.rels (237 bytes)
3D/3dmodel.model (125,482 bytes)
3D/_rels/3dmodel.model.rels (412 bytes)
Metadata/thumbnail.png (18,592 bytes)
Metadata/model_thumbnail.png (22,145 bytes)
```

**Tree view:**

```bash
lib3mf-cli list model.3mf --format tree
```

**Example output:**

```
model.3mf/
├── [Content_Types].xml (358 bytes)
├── _rels/
│   └── .rels (237 bytes)
├── 3D/
│   ├── 3dmodel.model (125,482 bytes)
│   └── _rels/
│       └── 3dmodel.model.rels (412 bytes)
└── Metadata/
    ├── thumbnail.png (18,592 bytes)
    └── model_thumbnail.png (22,145 bytes)
```

**When to use:**
- Debugging OPC structure issues
- Finding thumbnails or textures
- Understanding vendor-specific file layouts
- Investigating Bambu Studio project files

### `validate` — Validation Checks

Run validation at different levels to check file integrity and spec compliance.

**Basic usage (Standard level):**

```bash
lib3mf-cli validate model.3mf
```

**Example output (no errors):**

```
Validation Level: Standard
Status: PASSED
Issues: 0 errors, 0 warnings, 1 info

Info:
  - Model uses millimeter units
```

**Example output (with errors):**

```
Validation Level: Standard
Status: FAILED
Issues: 2 errors, 1 warning, 0 info

Errors:
  - Object 5 referenced in build but not defined in resources
  - Triangle 142 in object 3 references vertex index 500 (out of bounds)

Warnings:
  - Object 7 defined but never used in build
```

**Validation levels:**

```bash
# Minimal — Basic structural checks only
lib3mf-cli validate model.3mf --level minimal

# Standard — Reference integrity (default)
lib3mf-cli validate model.3mf --level standard

# Strict — Full spec compliance
lib3mf-cli validate model.3mf --level strict

# Paranoid — Deep geometry analysis (manifoldness, self-intersection)
lib3mf-cli validate model.3mf --level paranoid
```

**JSON output:**

```bash
lib3mf-cli validate model.3mf --format json
```

```json
{
  "level": "Standard",
  "passed": false,
  "error_count": 2,
  "warning_count": 1,
  "info_count": 0,
  "issues": [
    {
      "severity": "Error",
      "code": 3001,
      "message": "Object 5 referenced in build but not defined in resources"
    },
    {
      "severity": "Error",
      "code": 2042,
      "message": "Triangle 142 in object 3 references vertex index 500 (out of bounds)"
    },
    {
      "severity": "Warning",
      "code": 4010,
      "message": "Object 7 defined but never used in build"
    }
  ]
}
```

**When to use:**
- Pre-flight checks before printing
- CI/CD quality gates
- Debugging invalid files
- Compliance certification

See the [Validation Guide](validation-guide.md) for details on validation levels and error codes.

### `copy` — Roundtrip Test

Read a 3MF file and write it to a new file. This tests the parser → model → writer pipeline.

**Usage:**

```bash
lib3mf-cli copy input.3mf output.3mf
```

**Example output:**

```
Reading: input.3mf
Parsing model...
Writing: output.3mf
Done. Objects: 3, Triangles: 1,542
```

**When to use:**
- Testing parser and writer compatibility
- Normalizing 3MF files (remove vendor extensions)
- Verifying roundtrip fidelity
- Converting between 3MF versions

### `diff` — Compare Models

Compare two 3MF files to find structural or metadata differences.

**Usage:**

```bash
lib3mf-cli diff v1.3mf v2.3mf
```

**Example output:**

```
Comparing:
  v1.3mf
  v2.3mf

Differences found:

Geometry:
  ✓ Triangle count: 1,542 (same)
  ✗ Vertex count: 771 → 823 (+52)

Resources:
  ✗ Objects: 3 → 4 (+1)
  ✓ Materials: 2 (same)

Build:
  ✓ Build items: 3 (same)

Metadata:
  ✗ Title: "Model v1" → "Model v2"
  + Author: "John Doe" (added)
```

**When to use:**
- Reviewing CAD export changes
- Debugging slicer modifications
- Version control for 3MF files
- Regression testing

### `extract` — Extract Archive Files

Extract specific files from the 3MF archive (thumbnails, textures, etc.).

**Usage:**

```bash
lib3mf-cli extract model.3mf "Metadata/thumbnail.png" --output thumb.png
```

**Example output:**

```
Extracting: Metadata/thumbnail.png
Written to: thumb.png (18,592 bytes)
```

**Extract multiple files:**

```bash
# Extract all thumbnails
lib3mf-cli extract model.3mf "Metadata/thumbnail.png"
lib3mf-cli extract model.3mf "Metadata/model_thumbnail.png"
```

**When to use:**
- Extracting preview images
- Debugging texture issues
- Analyzing vendor-specific files
- Inspecting encrypted content metadata

## Common Workflows

### Pre-Print Validation

```bash
# Run paranoid validation before printing
lib3mf-cli validate model.3mf --level paranoid

# If validation passes, check geometry stats
lib3mf-cli stats model.3mf
```

### CI/CD Pipeline

```bash
#!/bin/bash
# QA script for 3MF exports

# Validate file
lib3mf-cli validate $1 --level strict --format json > validation.json

# Check for errors
if grep -q '"passed": false' validation.json; then
    echo "Validation failed"
    cat validation.json
    exit 1
fi

# Verify expected geometry
lib3mf-cli stats $1 --format json > stats.json
triangle_count=$(grep -o '"triangle_count": [0-9]*' stats.json | grep -o '[0-9]*')

if [ "$triangle_count" -lt 1000 ]; then
    echo "Error: Too few triangles ($triangle_count)"
    exit 1
fi

echo "QA passed: $triangle_count triangles"
```

### Debugging Invalid Files

```bash
# Step 1: Check archive structure
lib3mf-cli list broken.3mf --format tree

# Step 2: Run minimal validation (skip geometry checks)
lib3mf-cli validate broken.3mf --level minimal

# Step 3: If minimal passes, try standard
lib3mf-cli validate broken.3mf --level standard

# Step 4: Get detailed stats
lib3mf-cli stats broken.3mf
```

### Comparing Slicer Exports

```bash
# Compare before and after slicer processing
lib3mf-cli diff original.3mf sliced.3mf

# Extract thumbnails to visually compare
lib3mf-cli extract sliced.3mf "Metadata/thumbnail.png" --output sliced_thumb.png
```

## Exit Codes

The CLI uses standard exit codes:

- `0` — Success
- `1` — Validation failed or file error
- `2` — Invalid arguments or usage

This allows easy integration with shell scripts:

```bash
if lib3mf-cli validate model.3mf --level strict; then
    echo "File is valid"
else
    echo "File has errors"
fi
```

## Next Steps

- **[Validation Guide](validation-guide.md)** — Understanding validation levels and error codes
- **[Feature Flags](feature-flags.md)** — Building custom CLI tools
- **[API Reference](../rustdoc/lib3mf_cli/index.html)** — CLI implementation details
