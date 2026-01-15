#!/bin/bash

set -e

if [ "$#" -lt 1 ]; then
    echo "Usage: $0 <target>"
    exit 1
fi

TARGET=$1

echo "Generating coverage report for: $TARGET"

cargo +nightly fuzz coverage "$TARGET"

if command -v llvm-cov &> /dev/null; then
    PROFDATA="fuzz/coverage/$TARGET/coverage.profdata"
    BINARY="fuzz/target/x86_64-unknown-linux-gnu/coverage/$TARGET"
    
    if [ -f "$PROFDATA" ] && [ -f "$BINARY" ]; then
        llvm-cov show "$BINARY" \
            -instr-profile="$PROFDATA" \
            -format=html \
            -output-dir="fuzz/coverage/$TARGET/html" \
            -Xdemangler=rustfilt
        
        echo "Coverage report generated: fuzz/coverage/$TARGET/html/index.html"
        
        llvm-cov report "$BINARY" \
            -instr-profile="$PROFDATA" \
            -use-color
    fi
else
    echo "Warning: llvm-cov not found. Install LLVM tools for detailed coverage."
fi
