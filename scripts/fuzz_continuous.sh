#!/bin/bash

set -e

BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== TALON Continuous Fuzzing ===${NC}"
echo ""

DURATION_PER_CYCLE=${1:-3600}
MAX_CYCLES=${2:-24}

TARGETS=(
    "fuzz_parser:5"
    "fuzz_interpreter:5"
    "fuzz_elf_parser:4"
    "fuzz_pe_parser:4"
    "fuzz_shellcode_generator:4"
    "fuzz_format_string:4"
    "fuzz_heap_tools:4"
    "fuzz_packing_tools:3"
    "fuzz_rop_gadget_finder:4"
    "fuzz_rop_chain_builder:3"
    "fuzz_auto_solver:4"
    "fuzz_ast:4"
    "fuzz_exploit_chain:4"
    "fuzz_network_protocol:3"
    "fuzz_crypto_tools:3"
    "fuzz_syscall_chain:4"
    "fuzz_disassembler:4"
)

TOTAL_CRASHES=0
CYCLE=1

while [ "$CYCLE" -le "$MAX_CYCLES" ]; do
    echo -e "\n${BLUE}=== Cycle $CYCLE/$MAX_CYCLES ===${NC}"
    echo "Started: $(date)"
    
    for target_entry in "${TARGETS[@]}"; do
        IFS=':' read -r target priority <<< "$target_entry"
        
        TIME_MULTIPLIER=$priority
        TARGET_DURATION=$((DURATION_PER_CYCLE * TIME_MULTIPLIER / 4))
        
        echo -e "\n${BLUE}Fuzzing: $target (priority=$priority, duration=${TARGET_DURATION}s)${NC}"
        
        if cargo +nightly fuzz run "$target" -- -max_total_time="$TARGET_DURATION" -print_final_stats=1; then
            echo -e "${GREEN}$target: OK${NC}"
        else
            echo -e "${RED}$target: CRASHED${NC}"
            if [ -d "fuzz/artifacts/$target" ]; then
                CRASHES=$(ls fuzz/artifacts/$target/crash-* 2>/dev/null | wc -l)
                TOTAL_CRASHES=$((TOTAL_CRASHES + CRASHES))
            fi
        fi
    done
    
    echo -e "\n${YELLOW}Minimizing corpus...${NC}"
    for target_entry in "${TARGETS[@]}"; do
        IFS=':' read -r target _ <<< "$target_entry"
        cargo +nightly fuzz cmin "$target" 2>/dev/null || true
    done
    
    echo -e "\n${GREEN}Cycle $CYCLE complete${NC}"
    echo "Total crashes so far: $TOTAL_CRASHES"
    
    CYCLE=$((CYCLE + 1))
    
    if [ "$CYCLE" -le "$MAX_CYCLES" ]; then
        echo -e "\n${YELLOW}Sleeping 60s before next cycle...${NC}"
        sleep 60
    fi
done

echo -e "\n${BLUE}=== Continuous Fuzzing Complete ===${NC}"
echo "Total cycles: $MAX_CYCLES"
echo "Total crashes: $TOTAL_CRASHES"

if [ "$TOTAL_CRASHES" -gt 0 ]; then
    echo -e "${RED}Found crashes. Review fuzz/artifacts/${NC}"
    exit 1
fi

echo -e "${GREEN}No crashes detected${NC}"
