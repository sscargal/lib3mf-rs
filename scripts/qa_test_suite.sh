#!/bin/bash
set -u

# QA Test Suite for lib3mf-rs
#
# This script serves as the comprehensive Quality Assurance (QA) automation tool for the lib3mf-rs CLI.
# Its primary purpose is to validate the correctness, robustness, and stability of the command-line interface
# before releases. It dynamically discovers available commands to ensure full coverage and executes a mix
# of generic fuzzing tests and specialized scenario-based tests.
#
# Capabilities:
# 1. Automatic Discovery: Parses `lib3mf-cli --help` to dynamically identify and test all available subcommands.
# 2. Asset Generation: Automatically creates required test assets (RSA keys, X.509 certs, invalid 3MF/STL files).
# 3. Specialized Testing: Dedicated handlers for complex workflows like Encryption, Signing, and Diffing.
# 4. Negative Testing: Verifies that the CLI correctly rejects zero-byte files, corrupted archives, and invalid inputs.
# 5. Full Logging: Captures all STDOUT/STDERR to `commands.log` for deep debugging while keeping console output clean.

# Usage: ./scripts/qa_test_suite.sh

# Setup output handling
WORKDIR=$(pwd)
REPORT_FILE="$WORKDIR/qa_report.txt"
FAIL_COUNT=0
PASS_COUNT=0
QA_TMP_DIR=$(mktemp -d "/tmp/lib3mf_qa_XXXXXX")
CMD_LOG="$QA_TMP_DIR/commands.log"

# Cleanup trap
cleanup() {
    if [ "$FAIL_COUNT" -eq 0 ]; then
        echo "Cleaning up temporary directory: $QA_TMP_DIR"
        rm -rf "$QA_TMP_DIR"
    else
        echo "Tests failed. Artifacts preserved in: $QA_TMP_DIR"
        echo "Command Log: $CMD_LOG"
    fi
}
trap cleanup EXIT

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "Starting QA Test Suite at $(date)" > "$REPORT_FILE"
echo "Temporary Directory: $QA_TMP_DIR" >> "$REPORT_FILE"
echo "Command Log: $CMD_LOG" >> "$REPORT_FILE"
echo "------------------------------------------------" >> "$REPORT_FILE"

log_result() {
    local command="$1"
    local status="$2"
    if [ "$status" -eq 0 ]; then
        echo -e "${GREEN}[PASS]${NC} $command"
        echo "[PASS] $command" >> "$REPORT_FILE"
        ((PASS_COUNT++))
    else
        echo -e "${RED}[FAIL]${NC} $command"
        echo "[FAIL] $command" >> "$REPORT_FILE"
        ((FAIL_COUNT++))
    fi
}

run_cmd() {
    local cmd="$1"
    local msg="${2:-Running: $cmd}"
    # echo "$msg" # Suppressed to remove duplicate output
    
    {
        echo "========================================================"
        echo "TIME: $(date)"
        echo "CMD: $cmd"
        echo "--------------------------------------------------------"
    } >> "$CMD_LOG"
    
    eval "$cmd" >> "$CMD_LOG" 2>&1
    local status=$?
    
    echo "EXIT: $status" >> "$CMD_LOG"
    echo "" >> "$CMD_LOG"
    
    log_result "$cmd" $status
    return $status
}

run_negative_cmd() {
    local cmd="$1"
    local msg="${2:-Running Negative Test: $cmd}"
    # echo "$msg"
    
    {
        echo "========================================================"
        echo "TIME: $(date)"
        echo "CMD (NEGATIVE): $cmd"
        echo "--------------------------------------------------------"
    } >> "$CMD_LOG"
    
    eval "$cmd" >> "$CMD_LOG" 2>&1
    local status=$?
    
    echo "EXIT: $status" >> "$CMD_LOG"
    echo "" >> "$CMD_LOG"
    
    # Invert logic: Success (0) is FAIL, Failure (!0) is PASS
    if [ "$status" -ne 0 ]; then
        echo -e "${GREEN}[PASS]${NC} (Expected Fail) $cmd"
        echo "[PASS] (Expected Fail) $cmd" >> "$REPORT_FILE"
        ((PASS_COUNT++))
        return 0
    else
        echo -e "${RED}[FAIL]${NC} (Unexpected Success) $cmd"
        echo "[FAIL] (Unexpected Success) $cmd" >> "$REPORT_FILE"
        ((FAIL_COUNT++))
        return 1
    fi
}

echo "=== Project Validation ==="
run_cmd "cargo build" "Building Debug"
run_cmd "cargo build --release" "Building Release"

CLI_BIN_DEBUG="./target/debug/lib3mf-cli"
CLI_BIN_RELEASE="./target/release/lib3mf-cli"

# Ensure at least debug binary exists for rest of tests
if [ ! -f "$CLI_BIN_DEBUG" ]; then
    echo "Error: CLI binary not found at $CLI_BIN_DEBUG"
    exit 1
fi
CLI_BIN="$CLI_BIN_DEBUG"

run_cmd "$CLI_BIN_RELEASE --version" "Checking Release Version"
run_cmd "$CLI_BIN_DEBUG --version" "Checking Debug Version"
run_cmd "cargo clippy -- -D warnings" "Running Clippy"
run_cmd "cargo fmt --check" "Checking Format"
run_cmd "cargo test" "Running Tests"
run_cmd "cargo bench" "Running Benchmarks"

# --- Asset Generation (from test_cli.sh) ---
echo -e "${BLUE}=== Generating Test Assets ===${NC}"

# 1. Crypto Assets
KEY_FILE="$QA_TMP_DIR/private.pem"
CERT_FILE="$QA_TMP_DIR/public.crt"
echo "Generating dummy keys..."
openssl genrsa -out "$KEY_FILE" 2048 2>/dev/null
openssl req -new -x509 -key "$KEY_FILE" -out "$CERT_FILE" -days 365 -subj "/CN=lib3mf-test" 2>/dev/null

# 2. Mesh Assets
DUMMY_STL="$QA_TMP_DIR/test.stl"
DUMMY_OBJ="$QA_TMP_DIR/test.obj"
echo "Generating dummy STL/OBJ..."

# Binary STL (minimal: 1 triangle)
printf '\0%.0s' {1..80} > "$DUMMY_STL"
printf '\001\000\000\000' >> "$DUMMY_STL" # count = 1
# Normal + 3 Vertices (0,0,0; 10,0,0; 0,10,0)
printf '\0%.0s' {1..12} >> "$DUMMY_STL"
printf '\0%.0s' {1..12} >> "$DUMMY_STL"
printf '\000\000\024\101\000\000\000\000\000\000\000\000' >> "$DUMMY_STL" # 10.0, 0.0, 0.0
printf '\000\000\000\000\000\000\024\101\000\000\000\000' >> "$DUMMY_STL" # 0.0, 10.0, 0.0
printf '\000\000' >> "$DUMMY_STL" # attr byte count

cat <<EOF > "$DUMMY_OBJ"
v 0 0 0
v 10 0 0
v 0 10 0
f 1 2 3
EOF

# Define Test Assets (Real vs Dummy)
ASSET_3MF="models/Benchy.3mf"
ASSET_STL="models/Benchy.stl" 
# Use dummy if Benchy chunks missing or for variety
ASSET_STL_DUMMY="$DUMMY_STL"

# Define Known Option Values
# Updated with generated asset paths
KNOWN_VALUES=(
    "format:json,tree,text"
    "level:minimal,standard,strict,paranoid"
    "fix:degenerate,duplicates,harmonize,islands,holes,all"
    "epsilon:1e-3,1e-1"
    "key:$KEY_FILE"
    "cert:$CERT_FILE"
    "recipient:$CERT_FILE"
)

get_values_for_option() {
    local opt_name="$1"
    for entry in "${KNOWN_VALUES[@]}"; do
        local key="${entry%%:*}"
        if [[ "$opt_name" == *"$key"* ]]; then
            echo "${entry#*:}" | tr ',' ' '
            return
        fi
    done
}

# --- Negative / Stress Testing ---
echo -e "${BLUE}=== Negative & Stress Testing ===${NC}"

# 1. Zero Byte File
ZERO_FILE="$QA_TMP_DIR/zero.3mf"
touch "$ZERO_FILE"
echo "Testing Zero Byte File: $ZERO_FILE"

# Test a few commands against zero file
run_negative_cmd "$CLI_BIN stats $ZERO_FILE" "Testing stats on zero bytes"
run_negative_cmd "$CLI_BIN list $ZERO_FILE" "Testing list on zero bytes"
run_negative_cmd "$CLI_BIN validate $ZERO_FILE" "Testing validate on zero bytes"

# 2. Corrupted File
CORRUPT_FILE="$QA_TMP_DIR/corrupt.3mf"
cp "$ASSET_3MF" "$CORRUPT_FILE"
# Overwrite header with garbage
printf "TRASH_DATA_HEADER" | dd of="$CORRUPT_FILE" bs=1 count=15 conv=notrunc 2>/dev/null
echo "Testing Corrupted File: $CORRUPT_FILE"

run_negative_cmd "$CLI_BIN stats $CORRUPT_FILE" "Testing stats on corrupt file"
run_negative_cmd "$CLI_BIN list $CORRUPT_FILE" "Testing list on corrupt file"
run_negative_cmd "$CLI_BIN validate $CORRUPT_FILE" "Testing validate on corrupt file"

# --- Unit Verification Tests ---
echo -e "${BLUE}=== Unit Verification Tests ===${NC}"

# Check that stats output includes Unit Scale and Normalized Units
STATS_OUTPUT=$($CLI_BIN stats "$ASSET_3MF")
if echo "$STATS_OUTPUT" | grep -q "Scale:"; then
    log_result "Unit Scale Display" 0
else
    log_result "Unit Scale Display (Missing 'Scale:' in output)" 1
fi

if echo "$STATS_OUTPUT" | grep -q "m^2"; then
    log_result "Normalized Area Display" 0
else
    log_result "Normalized Area Display (Missing 'm^2' in output)" 1
fi

# --- Thumbnail Tests ---
echo -e "${BLUE}=== Thumbnail Tests ===${NC}"
THUMB_IMG="$QA_TMP_DIR/thumb.png"
# Create dummy PNG (minimal valid PNG signature)
printf "\x89PNG\r\n\x1a\n\0\0\0\rIHDR\0\0\0\x01\0\0\0\x01\x08\x06\0\0\0\x1f\x15\xc4\x89\0\0\0\nIDATx\x9cc\0\x01\0\0\x05\0\x01\r\n-\xb4" > "$THUMB_IMG"

THUMB_TEST_FILE="$QA_TMP_DIR/thumb_test.3mf"
run_cmd "$CLI_BIN copy $ASSET_3MF $THUMB_TEST_FILE" "Prepare 3MF for thumbnail test"

# 1. Inject Package Thumbnail
run_cmd "$CLI_BIN thumbnails $THUMB_TEST_FILE --inject $THUMB_IMG" "Inject Package Thumbnail"

# 2. Inject Object Thumbnail (Dynamic ID)
OID=$($CLI_BIN thumbnails "$THUMB_TEST_FILE" --list | grep "ID:" | head -n 1 | awk '{print $2}')
if [ -z "$OID" ]; then
    echo "No objects found in $THUMB_TEST_FILE. Skipping object injection."
else
    run_cmd "$CLI_BIN thumbnails $THUMB_TEST_FILE --inject $THUMB_IMG --oid $OID" "Inject Object Thumbnail (ID $OID)"
fi

# 3. List Thumbnails
LIST_OUT=$($CLI_BIN thumbnails $THUMB_TEST_FILE --list)
if echo "$LIST_OUT" | grep -q "Package Thumbnail: Yes"; then
    log_result "List Package Thumbnail" 0
else
    log_result "List Package Thumbnail" 1
fi

if [ -n "$OID" ]; then
    if echo "$LIST_OUT" | grep -q "Thumbnail: .*thumb_${OID}.png"; then
        log_result "List Object Thumbnail (ID $OID has thumbnail)" 0
    else
        log_result "List Object Thumbnail (Missing indication for ID $OID)" 1
    fi
fi

# 4. Extract Thumbnails
EXTRACT_DIR="$QA_TMP_DIR/thumbs_out"
run_cmd "$CLI_BIN thumbnails $THUMB_TEST_FILE --extract $EXTRACT_DIR" "Extract Thumbnails"
if [ -f "$EXTRACT_DIR/package_thumbnail.png" ]; then
    log_result "Verify Extracted Package Thumbnail" 0
else
    log_result "Verify Extracted Package Thumbnail" 1
fi

if [ -n "$OID" ]; then
    if ls "$EXTRACT_DIR"/obj_${OID}_thumbnail.png 1> /dev/null 2>&1; then
        log_result "Verify Extracted Object Thumbnail (ID $OID)" 0
    else
        log_result "Verify Extracted Object Thumbnail (ID $OID)" 1
    fi
fi

# 5. Stats Check
STATS_OUT=$($CLI_BIN stats $THUMB_TEST_FILE)
if echo "$STATS_OUT" | grep -q "Package Thumbnail: Yes"; then
    log_result "Stats Report Thumbnails" 0
else
    log_result "Stats Report Thumbnails" 1
fi

# --- Discovery & Testing Phase ---
echo -e "${BLUE}=== Discovering Commands ===${NC}"
COMMANDS=$($CLI_BIN --help | grep -E '^\s{2}[a-z]+' | awk '{print $1}')

for CMD in $COMMANDS; do
    echo -e "\n${BLUE}--- Testing Subcommand: $CMD ---${NC}"
    
    # 1. Custom Handlers for Complex Commands
    # Some commands (sign, encrypt) require specific combinations of args that automatic discovery misses.
    # We run explicit tests for these *instead* or *in addition to* generic fuzzing.
    
    case "$CMD" in
        "sign")
             CMD_LINE="$CLI_BIN sign $ASSET_3MF $QA_TMP_DIR/signed.3mf --key $KEY_FILE --cert $CERT_FILE"
             run_cmd "$CMD_LINE" "Running Custom: $CMD_LINE"
             continue ;; # Skip generic loop for sign
        "verify")
             # Pre-req: We need a signed file. If sign failed, this fails.
             if [ -f "$QA_TMP_DIR/signed.3mf" ]; then
                 CMD_LINE="$CLI_BIN verify $QA_TMP_DIR/signed.3mf"
                 run_cmd "$CMD_LINE" "Running Custom: $CMD_LINE"
             fi
             continue ;;
        "encrypt")
             CMD_LINE="$CLI_BIN encrypt $ASSET_3MF $QA_TMP_DIR/encrypted.3mf --recipient $CERT_FILE"
             run_cmd "$CMD_LINE" "Running Custom: $CMD_LINE"
             continue ;;
        "decrypt")
             if [ -f "$QA_TMP_DIR/encrypted.3mf" ]; then
                CMD_LINE="$CLI_BIN decrypt $QA_TMP_DIR/encrypted.3mf $QA_TMP_DIR/decrypted.3mf --key $KEY_FILE"
                run_cmd "$CMD_LINE" "Running Custom: $CMD_LINE"
             fi
             continue ;;
        "diff")
             CMD_LINE="$CLI_BIN diff $ASSET_3MF $ASSET_3MF"
             run_cmd "$CMD_LINE" "Running Custom: $CMD_LINE"
             continue ;;
        "help")
             CMD_LINE="$CLI_BIN help"
             run_cmd "$CMD_LINE" "Running Custom: $CMD_LINE"
             
             # Also test --help on binary
             CMD_LINE="$CLI_BIN --help"
             run_cmd "$CMD_LINE" "Running Custom: $CMD_LINE"
             continue ;;
        "extract")
             CMD_LINE="$CLI_BIN extract $ASSET_3MF 3D/3dmodel.model --output $QA_TMP_DIR/extracted.model"
             run_cmd "$CMD_LINE" "Running Custom: $CMD_LINE"
             continue ;;
        "benchmark"|"stats"|"list"|"validate"|"convert"|"repair"|"copy"|"dump"|"mn"|"rels")
             # Use generic discovery logic below
             ;;
        *)
             echo "Unknown command structure, attempting generic discovery..."
             ;;
    esac

    # 2. Generic Discovery Logic
    HELP_TEXT=$($CLI_BIN $CMD --help)
    
    HAS_FILE_ARG=false
    if echo "$HELP_TEXT" | grep -Fq -- "<FILE>"; then HAS_FILE_ARG=true; fi
    if echo "$HELP_TEXT" | grep -Fq -- "<INPUT>"; then HAS_FILE_ARG=true; fi
    
    HAS_OUTPUT_ARG=false
    if echo "$HELP_TEXT" | grep -Fq -- "<OUTPUT>"; then HAS_OUTPUT_ARG=true; fi
    
    OPTIONS=$(echo "$HELP_TEXT" | grep -o '\-\-[a-z0-9\-]\+' | sort | uniq)
    
    INPUTS=()
    if [ "$HAS_FILE_ARG" = true ]; then
        INPUTS+=("$ASSET_3MF")
        # For convert, we also want to test obj/stl input
        if [ "$CMD" = "convert" ]; then
            INPUTS+=("$DUMMY_STL")
            INPUTS+=("$DUMMY_OBJ")
        else
            INPUTS+=("$ASSET_STL")
        fi
    else
        INPUTS+=("") 
    fi

    for INPUT in "${INPUTS[@]}"; do
        # Determine output arg if needed
        TARGET_FILE=""
        if [ "$HAS_OUTPUT_ARG" = true ]; then
             # Simple logic to determine output name based on input ext
             BASE_NAME=$(basename "$INPUT")
             if [[ "$INPUT" == *.3mf ]]; then 
                TARGET_FILE="$QA_TMP_DIR/${BASE_NAME}.stl"
             elif [[ "$INPUT" == *.stl ]]; then 
                TARGET_FILE="$QA_TMP_DIR/${BASE_NAME}.3mf"
             elif [[ "$INPUT" == *.obj ]]; then
                TARGET_FILE="$QA_TMP_DIR/${BASE_NAME}.3mf"
             else
                TARGET_FILE="$QA_TMP_DIR/output.out"
             fi
             
             # For extract, it might need specific path inside 3mf?
             # extract <FILE> <PATH> [OUTPUT]
             if [ "$CMD" = "extract" ]; then
                 # Special case for extract args
                 # We can't easily auto-discover positional args beyond input/output
                 # So we skip generic loop for extract if it requires specific internal path
                 continue 
             fi
        fi
        
        # Special handling for diff: needs 2 files
        if [ "$CMD" = "diff" ]; then
            CMD_LINE="$CLI_BIN diff $INPUT $INPUT" 
            # Diffing same file passes?
            run_cmd "$CMD_LINE" "Running: $CMD_LINE"
            continue
        fi

        # Test 1: Base Command w/ Required Args
        CMD_LINE="$CLI_BIN $CMD $INPUT $TARGET_FILE"
        run_cmd "$CMD_LINE" "Running: $CMD_LINE"
        
        # Test 2: Options
        for OPT in $OPTIONS; do
             if [[ "$OPT" == "--help" || "$OPT" == "--version" ]]; then continue; fi
             
             TAKES_VAL=false
             if echo "$HELP_TEXT" | grep -Fq -- "$OPT <"; then TAKES_VAL=true; fi
             if echo "$HELP_TEXT" | grep -Fq -- "$OPT ["; then TAKES_VAL=true; fi 
             
             if [ "$TAKES_VAL" = true ]; then
                 VALS=$(get_values_for_option "$OPT")
                 if [ -z "$VALS" ]; then
                     echo "Warning: No known values for $OPT, skipping."
                 else
                     for VAL in $VALS; do
                         CMD_LINE="$CLI_BIN $CMD $OPT $VAL $INPUT $TARGET_FILE"
                         run_cmd "$CMD_LINE" "Running: $CMD_LINE"
                     done
                 fi
             else
                 CMD_LINE="$CLI_BIN $CMD $OPT $INPUT $TARGET_FILE"
                 run_cmd "$CMD_LINE" "Running: $CMD_LINE"
             fi
        done
    done
done

echo ""
echo "------------------------------------------------" >> "$REPORT_FILE"
echo "Summary:"
echo "  Passed: $PASS_COUNT"
echo "  Failed: $FAIL_COUNT"
echo "Report: $REPORT_FILE"

if [ "$FAIL_COUNT" -eq 0 ]; then
    exit 0
else
    exit 1
fi
