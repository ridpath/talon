#!/bin/bash
# Test All Examples Script
# Runs all .talon example scripts with timeout and resource limits
# Usage: ./scripts/test_all_examples.sh [--verbose] [--timeout SECONDS]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
EXAMPLES_DIR="$PROJECT_ROOT/examples"
TIMEOUT=30
VERBOSE=0
FAILED_COUNT=0
PASSED_COUNT=0
SKIPPED_COUNT=0

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose|-v)
            VERBOSE=1
            shift
            ;;
        --timeout|-t)
            TIMEOUT="$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --verbose, -v         Show script output"
            echo "  --timeout, -t SECS    Set timeout per script (default: 30)"
            echo "  --help, -h            Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

if [ ! -d "$EXAMPLES_DIR" ]; then
    echo -e "${RED}Error: Examples directory not found: $EXAMPLES_DIR${NC}"
    exit 1
fi

if ! command -v timeout &> /dev/null; then
    echo -e "${YELLOW}Warning: 'timeout' command not found. Scripts will run without timeout.${NC}"
    TIMEOUT_CMD=""
else
    TIMEOUT_CMD="timeout ${TIMEOUT}s"
fi

TALON_BIN="$PROJECT_ROOT/target/debug/talon"
if [ ! -f "$TALON_BIN" ]; then
    echo -e "${BLUE}Building talon binary...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --quiet
    
    if [ ! -f "$TALON_BIN" ]; then
        echo -e "${RED}Error: Failed to build talon binary${NC}"
        exit 1
    fi
fi

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  TALON Example Script Validation${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "Examples directory: $EXAMPLES_DIR"
echo -e "Timeout per script: ${TIMEOUT}s"
echo -e "Verbose output: $([ $VERBOSE -eq 1 ] && echo 'Yes' || echo 'No')"
echo -e "${BLUE}───────────────────────────────────────────────────────────${NC}"
echo ""

FAILED_SCRIPTS=()

for script in "$EXAMPLES_DIR"/*.talon; do
    if [ ! -f "$script" ]; then
        continue
    fi
    
    script_name=$(basename "$script")
    printf "Testing: %-45s ... " "$script_name"
    
    OUTPUT_FILE=$(mktemp)
    
    if [ -n "$TIMEOUT_CMD" ]; then
        if $TIMEOUT_CMD "$TALON_BIN" run "$script" > "$OUTPUT_FILE" 2>&1; then
            echo -e "${GREEN}✓ PASS${NC}"
            PASSED_COUNT=$((PASSED_COUNT + 1))
        else
            EXIT_CODE=$?
            if [ $EXIT_CODE -eq 124 ]; then
                echo -e "${RED}✗ FAIL (timeout)${NC}"
            else
                echo -e "${RED}✗ FAIL (exit code: $EXIT_CODE)${NC}"
            fi
            FAILED_COUNT=$((FAILED_COUNT + 1))
            FAILED_SCRIPTS+=("$script_name")
            
            if [ $VERBOSE -eq 1 ]; then
                echo -e "${YELLOW}Output:${NC}"
                cat "$OUTPUT_FILE" | head -20
                echo ""
            fi
        fi
    else
        if "$TALON_BIN" run "$script" > "$OUTPUT_FILE" 2>&1; then
            echo -e "${GREEN}✓ PASS${NC}"
            PASSED_COUNT=$((PASSED_COUNT + 1))
        else
            echo -e "${RED}✗ FAIL${NC}"
            FAILED_COUNT=$((FAILED_COUNT + 1))
            FAILED_SCRIPTS+=("$script_name")
            
            if [ $VERBOSE -eq 1 ]; then
                echo -e "${YELLOW}Output:${NC}"
                cat "$OUTPUT_FILE" | head -20
                echo ""
            fi
        fi
    fi
    
    rm -f "$OUTPUT_FILE"
done

echo -e "${BLUE}───────────────────────────────────────────────────────────${NC}"
echo -e "Summary:"
echo -e "  ${GREEN}Passed:${NC}  $PASSED_COUNT"
echo -e "  ${RED}Failed:${NC}  $FAILED_COUNT"
echo -e "  ${YELLOW}Skipped:${NC} $SKIPPED_COUNT"

if [ $FAILED_COUNT -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed examples:${NC}"
    for failed_script in "${FAILED_SCRIPTS[@]}"; do
        echo -e "  - $failed_script"
    done
    echo ""
    echo -e "${RED}Test suite failed with $FAILED_COUNT error(s)${NC}"
    exit 1
else
    echo ""
    echo -e "${GREEN}All examples passed successfully!${NC}"
    exit 0
fi
