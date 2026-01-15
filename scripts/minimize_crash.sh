#!/bin/bash

set -e

if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <target> <crash_file>"
    echo ""
    echo "Example:"
    echo "  $0 fuzz_parser fuzz/artifacts/fuzz_parser/crash-abc123"
    exit 1
fi

TARGET=$1
CRASH_FILE=$2

if [ ! -f "$CRASH_FILE" ]; then
    echo "Error: Crash file not found: $CRASH_FILE"
    exit 1
fi

echo "Minimizing crash for target: $TARGET"
echo "Crash file: $CRASH_FILE"
echo ""

cargo +nightly fuzz cmin "$TARGET"

echo ""
echo "Minimization complete!"
echo "Minimized corpus saved to: fuzz/corpus/$TARGET/"
