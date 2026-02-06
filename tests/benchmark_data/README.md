# Benchmark Test Data

This directory tracks the test data used for performance benchmarking.

## Data Sources

The benchmark suite uses 3MF files from multiple sources to ensure realistic performance measurements:

1. **3MF Consortium Example Files** - Located in `tests/conformance/3mf-samples/examples/`
   - These are official reference implementations from the 3MF specification
   - Submodule: https://github.com/3MFConsortium/3mf-samples

2. **Real-World Models** - Located in `models/`
   - `Benchy.3mf` (3.1 MB) - Popular 3D printing test model

## File Categories Used in Benchmarks

### Small Files (~1-2 KB)
- `tests/conformance/3mf-samples/examples/core/box.3mf` (1.2 KB)
- Simple geometry, minimal complexity
- Tests parser overhead and basic throughput

### Medium Files (~100-300 KB)
- `tests/conformance/3mf-samples/examples/core/cube_gears.3mf` (258 KB)
- Moderate complexity with multiple objects
- Tests realistic file handling

### Large Files (3+ MB)
- `models/Benchy.3mf` (3.1 MB)
- Complex geometry with high triangle count
- Tests performance on production-scale models

## Benchmark Reproducibility

To ensure reproducible benchmarks:

1. Use the same test files across runs
2. Files are version-controlled (committed or in submodule)
3. File sizes and checksums documented in this README
4. Run benchmarks on consistent hardware/configuration

## Adding New Test Files

When adding test data:
1. Document the source and purpose
2. Add file size and triangle count
3. Ensure file is committed or referenced via submodule
4. Update benchmark suite to include the new file
