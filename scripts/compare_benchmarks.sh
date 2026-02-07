#!/bin/bash
# compare_benchmarks.sh - Automated benchmark comparison for lib3mf-rs vs lib3mf
# Usage: ./scripts/compare_benchmarks.sh [--setup-competitor]

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
COMPETITOR_DIR="$HOME/benchmarks/lib3mf_rust"
RESULTS_DIR="$PROJECT_ROOT/benchmark_results"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE} $1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

setup_competitor() {
    print_header "Setting Up Competitor Library (lib3mf)"

    if [ -d "$COMPETITOR_DIR" ]; then
        print_warning "Competitor directory already exists at $COMPETITOR_DIR"
        read -p "Remove and re-clone? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$COMPETITOR_DIR"
        else
            return 0
        fi
    fi

    # Clone competitor
    mkdir -p "$(dirname "$COMPETITOR_DIR")"
    print_success "Cloning lib3mf to $COMPETITOR_DIR..."
    git clone https://github.com/telecos/lib3mf_rust.git "$COMPETITOR_DIR"

    cd "$COMPETITOR_DIR"

    # Try to build it
    print_success "Building lib3mf..."
    if cargo build --release; then
        print_success "lib3mf built successfully"
    else
        print_error "lib3mf failed to build. Check their README for dependencies."
        exit 1
    fi

    # Copy test files
    print_success "Copying test files..."
    mkdir -p test_files
    cp "$PROJECT_ROOT/tests/conformance/3mf-samples/examples/core/box.3mf" test_files/ 2>/dev/null || true
    cp "$PROJECT_ROOT/tests/conformance/3mf-samples/examples/core/cube_gears.3mf" test_files/ 2>/dev/null || true
    cp "$PROJECT_ROOT/models/Benchy.3mf" test_files/ 2>/dev/null || true

    print_success "Competitor setup complete"
    echo
}

verify_prerequisites() {
    print_header "Verifying Prerequisites"

    # Check Rust
    if command -v cargo &> /dev/null; then
        RUST_VERSION=$(rustc --version | cut -d' ' -f2)
        print_success "Rust $RUST_VERSION installed"
    else
        print_error "Rust not found. Install from https://rustup.rs"
        exit 1
    fi

    # Check git submodules
    cd "$PROJECT_ROOT"
    if [ -f "tests/conformance/3mf-samples/examples/core/box.3mf" ]; then
        print_success "Test files available"
    else
        print_warning "Test files missing. Initializing submodules..."
        git submodule update --init --recursive
        print_success "Submodules initialized"
    fi

    # Check for Benchy.3mf
    if [ ! -f "models/Benchy.3mf" ]; then
        print_warning "Benchy.3mf not found in models/ - large file benchmark will be skipped"
    fi

    echo
}

benchmark_lib3mf_rs() {
    print_header "Benchmarking lib3mf-rs"

    cd "$PROJECT_ROOT"

    # Create results directory
    mkdir -p "$RESULTS_DIR"
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    RESULT_FILE="$RESULTS_DIR/lib3mf_rs_${TIMESTAMP}.txt"

    print_success "Running benchmarks (this may take 10-15 minutes)..."
    echo "Results will be saved to: $RESULT_FILE"
    echo

    # Run benchmarks
    cargo bench --bench comparison_bench 2>&1 | tee "$RESULT_FILE"

    # Extract key metrics
    echo
    print_header "lib3mf-rs Summary Results"

    echo "Parse Speed:"
    grep "parse_speed/full_parse" "$RESULT_FILE" | grep "time:" || echo "  (results not found)"

    echo
    echo "Validation Levels:"
    grep "validation_levels/validate" "$RESULT_FILE" | grep "time:" || echo "  (results not found)"

    echo
    print_success "Full results: $RESULT_FILE"
    print_success "HTML report: $PROJECT_ROOT/target/criterion/report/index.html"
    echo

    echo "$RESULT_FILE"
}

benchmark_lib3mf() {
    print_header "Benchmarking lib3mf (Competitor)"

    if [ ! -d "$COMPETITOR_DIR" ]; then
        print_error "Competitor not set up. Run: $0 --setup-competitor"
        return 1
    fi

    cd "$COMPETITOR_DIR"

    # Create results directory
    mkdir -p "$RESULTS_DIR"
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    RESULT_FILE="$RESULTS_DIR/lib3mf_${TIMESTAMP}.txt"

    # Check if they have benchmarks
    if [ ! -d "benches" ] && [ ! -f "benches/lib3mf_comparison.rs" ]; then
        print_warning "lib3mf does not have Criterion benchmarks set up"
        print_warning "Will attempt simple parse timing instead..."

        # Simple timing test
        echo "Simple parse timing test:" > "$RESULT_FILE"
        echo "" >> "$RESULT_FILE"

        for size in small medium large; do
            if [ "$size" = "small" ]; then
                FILE="test_files/box.3mf"
            elif [ "$size" = "medium" ]; then
                FILE="test_files/cube_gears.3mf"
            else
                FILE="test_files/Benchy.3mf"
            fi

            if [ -f "$FILE" ]; then
                echo "Testing $size file ($FILE)..." | tee -a "$RESULT_FILE"
                # This is a placeholder - adjust based on lib3mf's actual CLI/API
                # You may need to create a simple Rust program to parse and measure
                echo "  NOTE: Manual timing needed - lib3mf API not auto-detected" | tee -a "$RESULT_FILE"
            fi
        done

        print_warning "Competitor benchmarks incomplete. See $RESULT_FILE for details."
        print_warning "Refer to docs/benchmark-comparison-guide.md for manual setup."
    else
        print_success "Running competitor benchmarks..."
        cargo bench --bench lib3mf_comparison 2>&1 | tee "$RESULT_FILE"
    fi

    echo
    print_success "Competitor results: $RESULT_FILE"
    echo

    echo "$RESULT_FILE"
}

compare_results() {
    print_header "Comparison Summary"

    # Find most recent results
    LIB3MF_RS_RESULT=$(ls -t "$RESULTS_DIR"/lib3mf_rs_*.txt 2>/dev/null | head -1)
    LIB3MF_RESULT=$(ls -t "$RESULTS_DIR"/lib3mf_*.txt 2>/dev/null | head -1)

    if [ -z "$LIB3MF_RS_RESULT" ]; then
        print_error "No lib3mf-rs results found. Run benchmarks first."
        return 1
    fi

    echo "lib3mf-rs results: $LIB3MF_RS_RESULT"

    if [ -z "$LIB3MF_RESULT" ]; then
        print_warning "No lib3mf results found. Skipping comparison."
        echo
        echo "To set up competitor benchmarks, run:"
        echo "  $0 --setup-competitor"
        return 0
    fi

    echo "lib3mf results: $LIB3MF_RESULT"
    echo

    # Create comparison table
    COMPARISON_FILE="$RESULTS_DIR/comparison_$(date +%Y%m%d_%H%M%S).md"

    cat > "$COMPARISON_FILE" << EOF
# Benchmark Comparison Results

Generated: $(date)

## Environment

- CPU: $(lscpu | grep "Model name" | cut -d: -f2 | xargs)
- Rust: $(rustc --version)
- OS: $(uname -sr)

## Parse Speed Comparison

| Test | lib3mf-rs | lib3mf | Winner |
|------|-----------|--------|--------|
| Small (1.2 KB) | [extract from results] | [extract from results] | TBD |
| Medium (258 KB) | [extract from results] | [extract from results] | TBD |
| Large (3.1 MB) | [extract from results] | [extract from results] | TBD |

## Notes

- Both libraries tested on same hardware
- Same test files used (3MF Consortium samples + Benchy.3mf)
- Release builds with optimizations
- Results are median of multiple runs

## Source Files

- lib3mf-rs: $LIB3MF_RS_RESULT
- lib3mf: $LIB3MF_RESULT

## Interpretation

[Add analysis here based on results]

EOF

    print_success "Comparison template created: $COMPARISON_FILE"
    print_warning "Please manually extract values from result files and fill in the table"
    echo
}

show_help() {
    cat << EOF
Benchmark Comparison Tool for lib3mf-rs vs lib3mf

Usage: $0 [OPTIONS]

Options:
    --setup-competitor    Clone and set up competitor library for comparison
    --lib3mf-rs-only      Run only lib3mf-rs benchmarks
    --lib3mf-only         Run only lib3mf (competitor) benchmarks
    --compare             Compare most recent benchmark results
    --help                Show this help message

Examples:
    # First-time setup and full comparison
    $0 --setup-competitor
    $0

    # Run only lib3mf-rs benchmarks
    $0 --lib3mf-rs-only

    # Compare existing results
    $0 --compare

For detailed manual instructions, see:
    docs/benchmark-comparison-guide.md

EOF
}

# Main execution
main() {
    case "${1:-}" in
        --setup-competitor)
            setup_competitor
            ;;
        --lib3mf-rs-only)
            verify_prerequisites
            LIB3MF_RS_RESULT=$(benchmark_lib3mf_rs)
            ;;
        --lib3mf-only)
            LIB3MF_RESULT=$(benchmark_lib3mf)
            ;;
        --compare)
            compare_results
            ;;
        --help)
            show_help
            ;;
        "")
            # Full workflow
            verify_prerequisites
            LIB3MF_RS_RESULT=$(benchmark_lib3mf_rs)
            LIB3MF_RESULT=$(benchmark_lib3mf) || true
            compare_results
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
