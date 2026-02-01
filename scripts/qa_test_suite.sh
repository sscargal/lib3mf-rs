#!/bin/bash
set -u

# QA Test Suite for lib3mf-rs
# Integrated with test_cli.sh concepts:
# - Automatic CLI discovery
# - Asset generation (Keys, Certs, Dummy Meshes)
# - Specialized tests for complex commands (encrypt, sign, etc.)

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

echo "=== Building Project ==="
cargo build || { echo "Build failed"; exit 1; }
CLI_BIN="./target/debug/lib3mf-cli"

if [ ! -f "$CLI_BIN" ]; then
    echo "Error: CLI binary not found at $CLI_BIN"
    exit 1
fi

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
