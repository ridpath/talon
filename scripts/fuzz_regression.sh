#!/bin/bash

set -e

echo "=== TALON Fuzzing Regression Suite ==="
echo ""

REGRESSION_DIR="fuzz/regression"
mkdir -p "$REGRESSION_DIR"

if [ ! -d "fuzz/artifacts" ]; then
    echo "No artifacts directory found. Run fuzzing first."
    exit 0
fi

CRASH_COUNT=0

for target in fuzz/artifacts/*; do
    if [ ! -d "$target" ]; then
        continue
    fi
    
    TARGET_NAME=$(basename "$target")
    echo "Checking $TARGET_NAME..."
    
    for crash in "$target"/crash-*; do
        if [ ! -f "$crash" ]; then
            continue
        fi
        
        CRASH_COUNT=$((CRASH_COUNT + 1))
        CRASH_NAME=$(basename "$crash")
        REGRESSION_FILE="$REGRESSION_DIR/${TARGET_NAME}_${CRASH_NAME}"
        
        if [ ! -f "$REGRESSION_FILE" ]; then
            echo "  New crash: $CRASH_NAME"
            cp "$crash" "$REGRESSION_FILE"
        else
            echo "  Known crash: $CRASH_NAME"
        fi
    done
done

echo ""
echo "Regression test: Running all known crashes..."

FAILED=0

for regression_file in "$REGRESSION_DIR"/*; do
    if [ ! -f "$regression_file" ]; then
        continue
    fi
    
    FILENAME=$(basename "$regression_file")
    TARGET_NAME=$(echo "$FILENAME" | cut -d'_' -f1-3)
    
    echo -n "Testing $FILENAME... "
    
    if cargo +nightly fuzz run "$TARGET_NAME" "$regression_file" 2>/dev/null; then
        echo "FIXED"
    else
        echo "STILL CRASHES"
        FAILED=$((FAILED + 1))
    fi
done

echo ""
echo "Total regression tests: $(ls -1 "$REGRESSION_DIR" 2>/dev/null | wc -l)"
echo "Still failing: $FAILED"

if [ "$FAILED" -gt 0 ]; then
    exit 1
fi
