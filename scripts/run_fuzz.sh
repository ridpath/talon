#!/bin/bash

set -e

TARGETS=(
    "fuzz_parser"
    "fuzz_interpreter"
    "fuzz_ast"
    "fuzz_elf_parser"
    "fuzz_pe_parser"
    "fuzz_shellcode_generator"
    "fuzz_format_string"
    "fuzz_heap_tools"
    "fuzz_packing_tools"
    "fuzz_rop_gadget_finder"
    "fuzz_rop_chain_builder"
    "fuzz_auto_solver"
    "fuzz_exploit_chain"
    "fuzz_network_protocol"
    "fuzz_crypto_tools"
    "fuzz_syscall_chain"
    "fuzz_disassembler"
)

DURATION=${1:-300}
TARGET=${2:-""}

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║           TALON Fuzzing Test Suite                      ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════╝${NC}"
echo ""

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}ERROR: cargo not found. Please install Rust toolchain.${NC}"
    exit 1
fi

if ! cargo fuzz --version &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-fuzz...${NC}"
    cargo install cargo-fuzz
fi

if [ -n "$TARGET" ]; then
    if [[ ! " ${TARGETS[@]} " =~ " ${TARGET} " ]]; then
        echo -e "${RED}ERROR: Unknown target '$TARGET'${NC}"
        echo "Available targets: ${TARGETS[*]}"
        exit 1
    fi
    TARGETS=("$TARGET")
fi

TOTAL_CRASHES=0
FAILED_TARGETS=()

for target in "${TARGETS[@]}"; do
    echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Running: $target (${DURATION}s)${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    START_TIME=$(date +%s)
    
    if cargo +nightly fuzz run "$target" -- -max_total_time="$DURATION" -print_final_stats=1; then
        END_TIME=$(date +%s)
        ELAPSED=$((END_TIME - START_TIME))
        echo -e "${GREEN}✓ $target completed in ${ELAPSED}s (no crashes)${NC}"
    else
        EXIT_CODE=$?
        END_TIME=$(date +%s)
        ELAPSED=$((END_TIME - START_TIME))
        
        echo -e "${RED}✗ $target failed after ${ELAPSED}s (exit code: $EXIT_CODE)${NC}"
        FAILED_TARGETS+=("$target")
        
        if [ -d "fuzz/artifacts/$target" ]; then
            CRASH_COUNT=$(ls fuzz/artifacts/$target/crash-* 2>/dev/null | wc -l)
            if [ "$CRASH_COUNT" -gt 0 ]; then
                TOTAL_CRASHES=$((TOTAL_CRASHES + CRASH_COUNT))
                echo -e "${RED}  Found $CRASH_COUNT crash artifact(s):${NC}"
                ls -lh "fuzz/artifacts/$target/"
                
                echo -e "\n${YELLOW}  Sample crash (first artifact):${NC}"
                FIRST_CRASH=$(ls fuzz/artifacts/$target/crash-* 2>/dev/null | head -n 1)
                if [ -n "$FIRST_CRASH" ]; then
                    hexdump -C "$FIRST_CRASH" | head -n 20
                fi
            fi
        fi
    fi
done

echo -e "\n${BLUE}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                  Fuzzing Summary                         ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "Total targets tested: ${#TARGETS[@]}"
echo -e "Failed targets: ${#FAILED_TARGETS[@]}"
echo -e "Total crashes: $TOTAL_CRASHES"

if [ ${#FAILED_TARGETS[@]} -gt 0 ]; then
    echo -e "\n${RED}Failed targets:${NC}"
    for failed in "${FAILED_TARGETS[@]}"; do
        echo -e "  - $failed"
    done
fi

if [ "$TOTAL_CRASHES" -gt 0 ]; then
    echo -e "\n${RED}CRITICAL: Found $TOTAL_CRASHES crash(es)!${NC}"
    echo -e "${YELLOW}Please review artifacts in fuzz/artifacts/ directory${NC}"
    exit 1
else
    echo -e "\n${GREEN}✓ All fuzz tests passed successfully!${NC}"
    exit 0
fi
