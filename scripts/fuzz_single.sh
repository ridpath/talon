#!/bin/bash

set -e

if [ "$#" -lt 1 ]; then
    echo "Usage: $0 <target> [duration_seconds]"
    echo ""
    echo "Available targets:"
    echo "  - fuzz_parser"
    echo "  - fuzz_elf_parser"
    echo "  - fuzz_pe_parser"
    echo "  - fuzz_shellcode_generator"
    echo "  - fuzz_format_string"
    echo "  - fuzz_heap_tools"
    echo "  - fuzz_packing_tools"
    echo "  - fuzz_rop_gadget_finder"
    echo "  - fuzz_rop_chain_builder"
    echo "  - fuzz_auto_solver"
    exit 1
fi

TARGET=$1
DURATION=${2:-300}

echo "Running fuzzer: $TARGET for ${DURATION}s"
cargo +nightly fuzz run "$TARGET" -- -max_total_time="$DURATION" -print_final_stats=1

if [ -d "fuzz/artifacts/$TARGET" ]; then
    echo ""
    echo "=== Artifacts found ==="
    ls -lh "fuzz/artifacts/$TARGET/"
fi
