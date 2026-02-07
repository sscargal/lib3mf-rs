# Benchmark Quick Start

Quick guide to validate the performance claims in `docs/performance.md` and compare against the competitor (lib3mf).

## Option 1: Automated Script (Recommended)

### Benchmark lib3mf-rs Only

```bash
# Run lib3mf-rs benchmarks
./scripts/compare_benchmarks.sh --lib3mf-rs-only

# View HTML report
xdg-open target/criterion/report/index.html  # Linux
open target/criterion/report/index.html      # macOS
```

**Time:** ~10-15 minutes
**Output:** `benchmark_results/lib3mf_rs_TIMESTAMP.txt`

### Full Comparison (lib3mf-rs vs lib3mf)

```bash
# First time: Set up competitor
./scripts/compare_benchmarks.sh --setup-competitor

# Run both benchmarks and compare
./scripts/compare_benchmarks.sh

# View results
ls benchmark_results/
```

**Time:** ~30-40 minutes (includes cloning and building competitor)
**Output:**
- `benchmark_results/lib3mf_rs_TIMESTAMP.txt` - Our results
- `benchmark_results/lib3mf_TIMESTAMP.txt` - Competitor results
- `benchmark_results/comparison_TIMESTAMP.md` - Side-by-side comparison

## Option 2: Manual Step-by-Step

### Step 1: Verify Prerequisites

```bash
# Ensure submodules are initialized
git submodule update --init --recursive

# Verify test files
ls tests/conformance/3mf-samples/examples/core/box.3mf
ls models/Benchy.3mf
```

### Step 2: Run lib3mf-rs Benchmarks

```bash
# Run full benchmark suite
cargo bench --bench comparison_bench

# Results saved to:
# - Terminal output (timing data)
# - target/criterion/report/index.html (interactive charts)
```

### Step 3: Extract Key Metrics

```bash
# View parse speed results
grep "parse_speed/full_parse" target/criterion/*/estimates.json

# Or view in browser
xdg-open target/criterion/report/index.html
```

### Step 4: (Optional) Compare with lib3mf

See detailed instructions in `docs/benchmark-comparison-guide.md`.

## Expected Results (lib3mf-rs)

Based on AMD Ryzen 7 7735HS (8 cores), your results should be similar:

| Benchmark | Expected Time |
|-----------|---------------|
| Parse small file (1.2 KB) | ~20 µs |
| Parse medium file (258 KB) | ~15 ms |
| Parse large file (3.1 MB) | ~40 µs (cached) |
| Validation (Standard) | ~25 µs |
| Validation (Paranoid) | ~85 ms |

**Note:** Absolute times vary by CPU, but relative performance should match.

## Validating Published Claims

To verify claims in `docs/performance.md`:

| Claim | How to Verify | Expected Result |
|-------|---------------|-----------------|
| "Standard validation is nearly free" | Compare `validation_levels/validate/standard` vs `parse_speed` | <1% overhead |
| "Archive operations <15 µs" | Check `archive_ops/zip_opc` benchmark | All results <15 µs |
| "~140 MiB/s XML throughput" | Check `xml_parsing` throughput column | ~130-150 MiB/s |
| "Paranoid is 3,500× slower" | Divide `paranoid` time by `standard` time | ~3000-4000× |

## Troubleshooting

### "Failed to read test file"

```bash
# Reinitialize submodules
git submodule update --init --recursive --force
```

### "Benchmarks take too long"

```bash
# Run quick sampling (less accurate but faster)
cargo bench --bench comparison_bench -- --sample-size 10
```

### "Results are inconsistent"

```bash
# Disable CPU frequency scaling (Linux)
sudo cpupower frequency-set -g performance

# Close background applications
# Run benchmark 3 times and take median
```

### "Competitor (lib3mf) won't compile"

See their README: https://github.com/telecos/lib3mf_rust#readme

May require system dependencies or specific Rust version.

## Next Steps

After benchmarking:

1. **View detailed results:**
   - HTML report: `target/criterion/report/index.html`
   - Raw data: `target/criterion/*/new/estimates.json`

2. **Compare against published results:**
   - See `docs/performance.md` for our published baseline
   - Your CPU will affect absolute times, but relative comparisons should match

3. **Run competitor benchmarks:**
   - Follow `docs/benchmark-comparison-guide.md` for detailed instructions
   - Use `./scripts/compare_benchmarks.sh --setup-competitor`

4. **Share results:**
   - Open issue if results differ significantly from published claims
   - Include: CPU model, RAM, OS version, Rust version

## Documentation

- **This file**: Quick start guide
- **docs/benchmark-comparison-guide.md**: Comprehensive manual instructions
- **docs/performance.md**: Published performance characteristics
- **scripts/compare_benchmarks.sh**: Automated benchmark runner

## Questions?

Open an issue: https://github.com/sscargal/lib3mf-rs/issues
