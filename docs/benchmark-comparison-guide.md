# Benchmark Comparison Guide: lib3mf-rs vs lib3mf

This guide provides step-by-step instructions for running performance benchmarks comparing lib3mf-rs against the competitor library (telecos/lib3mf_rust).

## Prerequisites

- Rust toolchain (stable and nightly)
- Git with submodule support
- At least 4GB free disk space
- ~30 minutes for full benchmark suite

## Part 1: Benchmarking lib3mf-rs

### Step 1: Set Up lib3mf-rs

```bash
# Clone repository (if you haven't already)
git clone https://github.com/sscargal/lib3mf-rs.git
cd lib3mf-rs

# Initialize test file submodule
git submodule update --init --recursive

# Verify test files are available
ls tests/conformance/3mf-samples/examples/core/box.3mf
ls tests/conformance/3mf-samples/examples/core/cube_gears.3mf
ls models/Benchy.3mf
```

### Step 2: Run lib3mf-rs Benchmarks

```bash
# Run full benchmark suite (takes ~10-15 minutes)
cargo bench --bench comparison_bench

# Results are saved to: target/criterion/
# HTML reports at: target/criterion/report/index.html
```

**Benchmark groups created:**
- `parse_speed` - Full parsing (ZIP + XML) for small/medium/large files
- `validation_levels` - All 4 validation levels (Minimal/Standard/Strict/Paranoid)
- `statistics` - Stats computation performance
- `archive_ops` - ZIP + OPC overhead
- `xml_parsing` - Pure XML parsing performance
- `memory_access` - Access pattern performance

### Step 3: Extract lib3mf-rs Results

```bash
# View summary results
cat target/criterion/*/new/estimates.json | jq '.mean.point_estimate'

# Or view full HTML report
xdg-open target/criterion/report/index.html  # Linux
open target/criterion/report/index.html      # macOS
```

**Key metrics to record:**
- Parse time for small file (box.3mf, ~1.2 KB)
- Parse time for medium file (cube_gears.3mf, ~258 KB)
- Parse time for large file (Benchy.3mf, ~3.1 MB)
- Validation time at Standard level
- Memory usage (use `/usr/bin/time -v` for detailed memory stats)

## Part 2: Benchmarking lib3mf (Competitor)

### Step 1: Clone Competitor Repository

```bash
# Clone in a separate directory
cd ~/benchmarks  # or any working directory
git clone https://github.com/telecos/lib3mf_rust.git
cd lib3mf_rust
```

### Step 2: Examine Competitor's Test/Benchmark Setup

```bash
# Check for existing benchmarks
ls benches/ 2>/dev/null || echo "No benches/ directory"

# Check their Cargo.toml for benchmark configuration
grep -A 10 "\\[\\[bench\\]\\]" Cargo.toml

# Check for test files
find . -name "*.3mf" -type f

# Note: As of 2026-02-06, lib3mf may not have Criterion benchmarks set up
```

### Step 3: Create Comparable Benchmark for lib3mf

Since the competitor may not have benchmarks, create a comparable test:

```bash
# Create benches directory if it doesn't exist
mkdir -p benches

# Copy test files from lib3mf-rs
cp ~/lib3mf-rs/tests/conformance/3mf-samples/examples/core/box.3mf ./test_files/
cp ~/lib3mf-rs/tests/conformance/3mf-samples/examples/core/cube_gears.3mf ./test_files/
cp ~/lib3mf-rs/models/Benchy.3mf ./test_files/
```

Create `benches/lib3mf_comparison.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lib3mf::model::Model;
use std::io::Cursor;

fn get_test_file(size_category: &str) -> Vec<u8> {
    match size_category {
        "small" => std::fs::read("test_files/box.3mf")
            .expect("Failed to read small test file"),
        "medium" => std::fs::read("test_files/cube_gears.3mf")
            .expect("Failed to read medium test file"),
        "large" => std::fs::read("test_files/Benchy.3mf")
            .expect("Failed to read large test file"),
        _ => panic!("Unknown size category: {}", size_category),
    }
}

fn bench_parse_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_speed");

    for size in ["small", "medium", "large"].iter() {
        let data = get_test_file(size);
        let bytes = data.len() as u64;

        group.throughput(Throughput::Bytes(bytes));
        group.bench_with_input(BenchmarkId::new("full_parse", size), &data, |b, data| {
            b.iter(|| {
                // Adjust API calls based on lib3mf's actual API
                let model = Model::from_reader(black_box(Cursor::new(data)))
                    .expect("Failed to parse model");
                black_box(model);
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_parse_speed);
criterion_main!(benches);
```

Add to `Cargo.toml`:

```toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "lib3mf_comparison"
harness = false
```

### Step 4: Run lib3mf Benchmarks

```bash
# Run benchmarks
cargo bench --bench lib3mf_comparison

# View results
xdg-open target/criterion/report/index.html
```

**Important Notes:**
- Adjust the benchmark code based on lib3mf's actual API (may differ from example above)
- Check their README for parsing API: https://github.com/telecos/lib3mf_rust#usage
- If they use different file formats or APIs, adapt accordingly

## Part 3: Fair Comparison Methodology

### Step 1: Ensure Comparable Test Conditions

Both benchmarks should use:
- **Same test files** (copy from lib3mf-rs repository)
- **Same machine** (run sequentially, not concurrently)
- **Same build profile** (release mode with optimizations)
- **Same Rust version** (check with `rustc --version`)
- **No other CPU-intensive tasks** running

```bash
# Verify Rust version is the same for both
cd ~/lib3mf-rs && rustc --version
cd ~/benchmarks/lib3mf_rust && rustc --version
```

### Step 2: Measure Memory Usage

For lib3mf-rs:
```bash
cd ~/lib3mf-rs

# Create a simple parse test
cat > /tmp/parse_test_lib3mf_rs.sh << 'EOF'
#!/bin/bash
cargo run --release --bin 3mf -- stats models/Benchy.3mf > /dev/null
EOF
chmod +x /tmp/parse_test_lib3mf_rs.sh

# Measure with time
/usr/bin/time -v /tmp/parse_test_lib3mf_rs.sh 2>&1 | grep -E "(Maximum resident|User time|System time)"
```

For lib3mf:
```bash
cd ~/benchmarks/lib3mf_rust

# Create comparable test (adjust command based on their CLI)
cat > /tmp/parse_test_lib3mf.sh << 'EOF'
#!/bin/bash
# Adjust this command based on lib3mf's actual CLI/API
cargo run --release --example parse -- ~/lib3mf-rs/models/Benchy.3mf > /dev/null
EOF
chmod +x /tmp/parse_test_lib3mf.sh

# Measure with time
/usr/bin/time -v /tmp/parse_test_lib3mf.sh 2>&1 | grep -E "(Maximum resident|User time|System time)"
```

### Step 3: Document Comparison Results

Create a comparison table:

| Metric | lib3mf-rs | lib3mf | Notes |
|--------|-----------|--------|-------|
| Parse time (small, 1.2 KB) | [your result] | [your result] | box.3mf |
| Parse time (medium, 258 KB) | [your result] | [your result] | cube_gears.3mf |
| Parse time (large, 3.1 MB) | [your result] | [your result] | Benchy.3mf |
| Peak memory (Benchy.3mf) | [your result] | [your result] | From `/usr/bin/time -v` |
| Binary size (release) | [your result] | [your result] | `ls -lh target/release/3mf` |

## Part 4: Understanding the Results

### Performance Factors to Consider

**lib3mf-rs optimizations:**
- `quick-xml` event-based parser (fast, zero-copy where possible)
- `lexical-core` for fast float parsing
- Optional `rayon` parallelization (disabled by default)
- Minimal allocations via arena patterns

**lib3mf optimizations:**
- Check their dependencies (`cargo tree | grep xml`)
- Check for parry3d integration (geometric operations)
- Check for parallel processing

### Fair Comparison Checklist

✅ **Same test files** - Use identical 3MF files for both
✅ **Same machine** - Run on same hardware
✅ **Same conditions** - No background tasks, same CPU governor
✅ **Multiple runs** - Run benchmarks 3+ times, report median
✅ **Warm cache** - Run once to warm filesystem cache before measuring
✅ **Document differences** - Note different feature sets (async, WASM, etc.)

### What NOT to Compare

❌ **Different feature sets** - Don't penalize lib3mf for lacking async/WASM
❌ **Different use cases** - lib3mf focuses on geometry, lib3mf-rs on I/O
❌ **Synthetic loads** - Use real 3MF files, not generated stress tests
❌ **Unequal optimization** - Both should use `--release` builds

## Part 5: Quick Validation Script

Here's a script to run both benchmarks and compare:

```bash
#!/bin/bash
# save as: benchmark_comparison.sh

set -e

echo "=== lib3mf-rs vs lib3mf Benchmark Comparison ==="
echo ""

# lib3mf-rs
echo ">>> Running lib3mf-rs benchmarks..."
cd ~/lib3mf-rs
cargo bench --bench comparison_bench -- --noplot 2>&1 | tee /tmp/lib3mf_rs_results.txt

# lib3mf (adjust path as needed)
echo ""
echo ">>> Running lib3mf benchmarks..."
if [ -d ~/benchmarks/lib3mf_rust ]; then
    cd ~/benchmarks/lib3mf_rust
    cargo bench --bench lib3mf_comparison -- --noplot 2>&1 | tee /tmp/lib3mf_results.txt
else
    echo "lib3mf not set up. Skipping competitor benchmark."
fi

# Extract and compare key results
echo ""
echo "=== Comparison Summary ==="
echo ""
echo "lib3mf-rs parse/small:"
grep "parse_speed/full_parse/small" /tmp/lib3mf_rs_results.txt | grep "time:" || echo "Not found"

echo ""
echo "lib3mf parse/small:"
grep "parse_speed/full_parse/small" /tmp/lib3mf_results.txt | grep "time:" 2>/dev/null || echo "Not available"

echo ""
echo "Full results:"
echo "  lib3mf-rs: /tmp/lib3mf_rs_results.txt"
echo "  lib3mf:    /tmp/lib3mf_results.txt"
```

Make executable and run:
```bash
chmod +x benchmark_comparison.sh
./benchmark_comparison.sh
```

## Part 6: Reproducing Our Published Results

To verify the results in `docs/performance.md`:

```bash
cd ~/lib3mf-rs

# Run benchmarks
cargo bench --bench comparison_bench

# Compare against published results in docs/performance.md
# Published results were from:
# - CPU: AMD Ryzen 7 7735HS (8 cores)
# - RAM: 15 GB
# - OS: Linux 6.14.0-34-generic
# - Rust: 1.93.0

# Your results may differ based on hardware, but relative performance
# characteristics should be similar
```

## Troubleshooting

**Problem**: `git submodule update` fails
**Solution**:
```bash
git submodule update --init --recursive --force
```

**Problem**: lib3mf doesn't compile
**Solution**: Check their README for build requirements. May need:
- Specific Rust version
- System dependencies (libxml2, etc.)
- Feature flag adjustments

**Problem**: Can't find comparable API in lib3mf
**Solution**:
```bash
# Explore their API
cd ~/benchmarks/lib3mf_rust
cargo doc --open
# Look for parsing/loading functions
```

**Problem**: Benchmark results are inconsistent
**Solution**:
- Run benchmarks 3-5 times and take median
- Disable CPU frequency scaling: `sudo cpupower frequency-set -g performance`
- Close background applications
- Use `nice -n -20` for higher priority

## Expected Results

Based on our testing (2026-02-06), expected performance characteristics:

**lib3mf-rs:**
- Small files (1-10 KB): 15-25 µs
- Medium files (100-500 KB): 10-20 ms
- Large files (1-5 MB): 40-60 µs (cached) or 50-100 ms (cold)
- Validation (Standard): 20-30 µs
- Peak memory: ~2-5× compressed file size

**lib3mf (estimated, verify yourself):**
- Parse performance: Comparable to lib3mf-rs
- May have additional geometry operations (parry3d)
- Different feature set (focus on geometric operations vs I/O)

## Conclusion

This guide allows you to:
1. ✅ Run lib3mf-rs benchmarks independently
2. ✅ Set up and run comparable lib3mf benchmarks
3. ✅ Compare results fairly with controlled conditions
4. ✅ Validate published performance claims

For questions or issues, please open an issue at:
https://github.com/sscargal/lib3mf-rs/issues
