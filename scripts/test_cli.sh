#!/bin/bash
set -e

# lib3mf-cli Comprehensive Test Script
# Tests all subcommands and various option combinations.

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting lib3mf-cli integration tests...${NC}"

# Path to the binary
CLI_BIN="./target/debug/lib3mf-cli"

if [ ! -f "$CLI_BIN" ]; then
    echo -e "${RED}Error: Binary $CLI_BIN not found. Build it first with 'cargo build -p lib3mf-cli'${NC}"
    exit 1
fi

# Test models
MODELS_DIR="./models"
BENCHY="$MODELS_DIR/Benchy.3mf"
BENCHY_STL="$MODELS_DIR/Benchy.stl"
DEADPOOL="$MODELS_DIR/Deadpool_3_Mask.3mf"

if [ ! -f "$BENCHY" ]; then
    echo -e "${RED}Error: $BENCHY not found.${NC}"
    exit 1
fi

# Setup Temp Directory
TEST_DIR=$(mktemp -d)
trap 'rm -rf "$TEST_DIR"' EXIT

echo "Using temporary directory: $TEST_DIR"

# 1. Generate Dummy Crypto Assets
echo "Generating dummy keys and certificates..."
openssl genrsa -out "$TEST_DIR/private.pem" 2048
openssl req -new -x509 -key "$TEST_DIR/private.pem" -out "$TEST_DIR/public.crt" -days 365 -subj "/CN=lib3mf-test"

# 2. Generate Dummy Mesh Assets
echo "Generating dummy STL and OBJ files..."

# Binary STL (minimal: 1 triangle)
# 80 bytes header (0)
# 4 bytes count (1)
# 50 bytes triangle (normal=0,0,0, v1=0,0,0, v2=10,0,0, v3=0,10,0, attr=0)
printf '\0%.0s' {1..80} > "$TEST_DIR/test.stl"
printf '\001\000\000\000' >> "$TEST_DIR/test.stl" # count = 1
# Normal (0,0,0) - 12 bytes
printf '\0%.0s' {1..12} >> "$TEST_DIR/test.stl"
# v1 (0,0,0) - 12 bytes
printf '\0%.0s' {1..12} >> "$TEST_DIR/test.stl"
# v2 (10.0, 0.0, 0.0) - 12 bytes (10.0 in float hex: 00 00 A0 41)
# Little Endian: \x00\x00\x20\x41
printf '\000\000\024\101\000\000\000\000\000\000\000\000' >> "$TEST_DIR/test.stl" # 10.0, 0.0, 0.0
# v3 (0.0, 10.0, 0.0) - 12 bytes
printf '\000\000\000\000\000\000\024\101\000\000\000\000' >> "$TEST_DIR/test.stl" # 0.0, 10.0, 0.0
# Attr byte count (0) - 2 bytes
printf '\000\000' >> "$TEST_DIR/test.stl"

cat <<EOF > "$TEST_DIR/test.obj"
v 0 0 0
v 10 0 0
v 0 10 0
f 1 2 3
EOF

# --- Subcommand Tests ---

test_cmd() {
    echo -e "Testing: ${GREEN}$@${NC}"
    "$CLI_BIN" "$@" > /dev/null
}

# Stats
test_cmd stats "$BENCHY"
test_cmd stats "$BENCHY" --format json

# List
test_cmd list "$BENCHY"
test_cmd list "$BENCHY" --format json
test_cmd list "$BENCHY" --format tree

# Rels
test_cmd rels "$BENCHY"
test_cmd rels "$BENCHY" --format json

# Dump
test_cmd dump "$BENCHY"
test_cmd dump "$BENCHY" --format json

# Extract
test_cmd extract "$BENCHY" "3D/3dmodel.model" --output "$TEST_DIR/extracted.model"
test_cmd extract "$BENCHY" "3D/3dmodel.model" # to stdout

# Copy
test_cmd copy "$BENCHY" "$TEST_DIR/copy.3mf"

# Convert
test_cmd convert "$BENCHY" "$TEST_DIR/from_3mf.stl"
test_cmd convert "$BENCHY" "$TEST_DIR/from_3mf.obj"

if [ -f "$BENCHY_STL" ]; then
    # Check if it's a real STL or actually a ZIP/3MF
    if head -c 4 "$BENCHY_STL" | grep -q "PK"; then
        echo "Note: $BENCHY_STL appears to be a 3MF/ZIP renamed to STL. Using mock for STL conversion test."
        test_cmd convert "$TEST_DIR/test.stl" "$TEST_DIR/to_3mf_mock_stl.3mf"
    else
        test_cmd convert "$BENCHY_STL" "$TEST_DIR/to_3mf_real_stl.3mf"
    fi
else
    test_cmd convert "$TEST_DIR/test.stl" "$TEST_DIR/to_3mf_mock_stl.3mf"
fi
test_cmd convert "$TEST_DIR/test.obj" "$TEST_DIR/to_3mf_obj.3mf"

# Validate
test_cmd validate "$BENCHY"
test_cmd validate "$BENCHY" --level minimal
test_cmd validate "$BENCHY" --level standard
test_cmd validate "$BENCHY" --level strict

# Repair
test_cmd repair "$BENCHY" "$TEST_DIR/repaired.3mf"

# Sign / Verify
# Note: Verification might be a placeholder in current impl but we test the CLI interface
test_cmd sign "$BENCHY" "$TEST_DIR/signed.3mf" --key "$TEST_DIR/private.pem" --cert "$TEST_DIR/public.crt"
test_cmd verify "$TEST_DIR/signed.3mf"

# Encrypt / Decrypt
test_cmd encrypt "$BENCHY" "$TEST_DIR/encrypted.3mf" --recipient "$TEST_DIR/public.crt"
test_cmd decrypt "$TEST_DIR/encrypted.3mf" "$TEST_DIR/decrypted.3mf" --key "$TEST_DIR/private.pem"

# Benchmark
test_cmd benchmark "$BENCHY"

# Diff
# Create a slightly different model by copying one
test_cmd diff "$BENCHY" "$DEADPOOL"

echo -e "${GREEN}All CLI integration tests passed successfully!${NC}"
