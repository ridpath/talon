#!/bin/bash

set -e

BLUE='\033[0;34m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}=== TALON Differential Fuzzing ===${NC}"
echo ""

PARSER_FUZZERS=("fuzz_parser" "fuzz_ast" "fuzz_interpreter")

echo -e "${BLUE}Running differential fuzzing on parser subsystem...${NC}"

DURATION=${1:-600}

for fuzzer in "${PARSER_FUZZERS[@]}"; do
    echo -e "\n${BLUE}Starting $fuzzer (${DURATION}s)${NC}"
    cargo +nightly fuzz run "$fuzzer" -- -max_total_time="$DURATION" -print_final_stats=1 &
done

wait

echo -e "\n${GREEN}Differential fuzzing complete${NC}"

if [ -d "fuzz/artifacts" ]; then
    TOTAL_CRASHES=$(find fuzz/artifacts -name "crash-*" 2>/dev/null | wc -l)
    if [ "$TOTAL_CRASHES" -gt 0 ]; then
        echo -e "${RED}Found $TOTAL_CRASHES crashes across parser subsystem${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}No crashes detected${NC}"
