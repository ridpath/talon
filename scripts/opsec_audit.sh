#!/bin/bash
# OpSec Sanitization Audit Script
# Verifies that release binaries are properly sanitized

set -e

BINARY_PATH="${1:-target/release/talon}"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================"
echo "TALON OpSec Sanitization Audit"
echo "========================================"
echo ""

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}[FAIL]${NC} Binary not found: $BINARY_PATH"
    echo "Build the release binary first:"
    echo "  cargo build --release"
    exit 1
fi

echo "Auditing binary: $BINARY_PATH"
echo ""

# Initialize counters
PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0

# Test 1: Check for Zenflow references
echo -n "Test 1: Checking for 'Zenflow' references... "
if strings "$BINARY_PATH" | grep -qi "zenflow"; then
    echo -e "${RED}[FAIL]${NC}"
    echo "  Found Zenflow references in binary:"
    strings "$BINARY_PATH" | grep -i "zenflow"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo -e "${GREEN}[PASS]${NC}"
    PASS_COUNT=$((PASS_COUNT + 1))
fi

# Test 2: Check for interactivetalon references
echo -n "Test 2: Checking for 'interactivetalon' references... "
if strings "$BINARY_PATH" | grep -qi "interactivetalon"; then
    echo -e "${RED}[FAIL]${NC}"
    echo "  Found interactivetalon references in binary:"
    strings "$BINARY_PATH" | grep -i "interactivetalon"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo -e "${GREEN}[PASS]${NC}"
    PASS_COUNT=$((PASS_COUNT + 1))
fi

# Test 3: Check for excessive src/ paths
echo -n "Test 3: Checking for source file paths... "
SRC_COUNT=$(strings "$BINARY_PATH" | grep -c "src/" || true)
if [ "$SRC_COUNT" -gt 50 ]; then
    echo -e "${RED}[FAIL]${NC}"
    echo "  Found $SRC_COUNT instances of 'src/' (threshold: 50)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
elif [ "$SRC_COUNT" -gt 10 ]; then
    echo -e "${YELLOW}[WARN]${NC}"
    echo "  Found $SRC_COUNT instances of 'src/' (consider reducing)"
    WARN_COUNT=$((WARN_COUNT + 1))
else
    echo -e "${GREEN}[PASS]${NC} ($SRC_COUNT instances)"
    PASS_COUNT=$((PASS_COUNT + 1))
fi

# Test 4: Verify binary is stripped
echo -n "Test 4: Verifying binary is stripped... "
if file "$BINARY_PATH" | grep -q "not stripped"; then
    echo -e "${RED}[FAIL]${NC}"
    echo "  Binary contains symbols (not stripped)"
    FAIL_COUNT=$((FAIL_COUNT + 1))
else
    echo -e "${GREEN}[PASS]${NC}"
    PASS_COUNT=$((PASS_COUNT + 1))
fi

# Test 5: Check binary size
echo -n "Test 5: Checking binary size... "
BINARY_SIZE=$(stat -c%s "$BINARY_PATH" 2>/dev/null || stat -f%z "$BINARY_PATH" 2>/dev/null || echo 0)
BINARY_SIZE_MB=$((BINARY_SIZE / 1024 / 1024))
if [ "$BINARY_SIZE_MB" -gt 50 ]; then
    echo -e "${YELLOW}[WARN]${NC}"
    echo "  Binary size: ${BINARY_SIZE_MB}MB (target: <50MB)"
    WARN_COUNT=$((WARN_COUNT + 1))
else
    echo -e "${GREEN}[PASS]${NC} (${BINARY_SIZE_MB}MB)"
    PASS_COUNT=$((PASS_COUNT + 1))
fi

# Test 6: Check for debug sections (Linux/macOS)
if command -v objdump &> /dev/null; then
    echo -n "Test 6: Checking for debug sections... "
    DEBUG_SECTIONS=$(objdump -h "$BINARY_PATH" 2>/dev/null | grep -cE '\.debug|\.dwarf' || true)
    if [ "$DEBUG_SECTIONS" -gt 0 ]; then
        echo -e "${YELLOW}[WARN]${NC}"
        echo "  Found $DEBUG_SECTIONS debug sections (may be metadata only)"
        WARN_COUNT=$((WARN_COUNT + 1))
    else
        echo -e "${GREEN}[PASS]${NC}"
        PASS_COUNT=$((PASS_COUNT + 1))
    fi
else
    echo "Test 6: Skipping (objdump not available)"
fi

# Test 7: Check for common sensitive strings
echo -n "Test 7: Checking for sensitive strings... "
SENSITIVE_FOUND=0
for keyword in "TODO" "FIXME" "XXX" "HACK" "password" "secret" "api_key"; do
    if strings "$BINARY_PATH" | grep -qi "$keyword"; then
        if [ "$SENSITIVE_FOUND" -eq 0 ]; then
            echo -e "${YELLOW}[WARN]${NC}"
            SENSITIVE_FOUND=1
        fi
        echo "  Found potentially sensitive keyword: $keyword"
    fi
done
if [ "$SENSITIVE_FOUND" -eq 0 ]; then
    echo -e "${GREEN}[PASS]${NC}"
    PASS_COUNT=$((PASS_COUNT + 1))
else
    WARN_COUNT=$((WARN_COUNT + 1))
fi

# Test 8: Check for panic messages with file paths
echo -n "Test 8: Checking for panic messages with file paths... "
PANIC_PATHS=$(strings "$BINARY_PATH" | grep -E "panicked at.*:[0-9]+:[0-9]+" | wc -l || echo 0)
if [ "$PANIC_PATHS" -gt 5 ]; then
    echo -e "${RED}[FAIL]${NC}"
    echo "  Found $PANIC_PATHS panic messages with file:line:col info"
    FAIL_COUNT=$((FAIL_COUNT + 1))
elif [ "$PANIC_PATHS" -gt 0 ]; then
    echo -e "${YELLOW}[WARN]${NC}"
    echo "  Found $PANIC_PATHS panic messages with location info"
    WARN_COUNT=$((WARN_COUNT + 1))
else
    echo -e "${GREEN}[PASS]${NC}"
    PASS_COUNT=$((PASS_COUNT + 1))
fi

# Summary
echo ""
echo "========================================"
echo "Audit Summary"
echo "========================================"
echo -e "${GREEN}Passed:${NC} $PASS_COUNT"
echo -e "${YELLOW}Warnings:${NC} $WARN_COUNT"
echo -e "${RED}Failed:${NC} $FAIL_COUNT"
echo ""

if [ "$FAIL_COUNT" -gt 0 ]; then
    echo -e "${RED}OpSec audit FAILED${NC}"
    echo "Review failures above and rebuild with proper sanitization."
    exit 1
elif [ "$WARN_COUNT" -gt 0 ]; then
    echo -e "${YELLOW}OpSec audit passed with warnings${NC}"
    echo "Review warnings and consider additional sanitization if needed."
    exit 0
else
    echo -e "${GREEN}OpSec audit PASSED${NC}"
    echo "Binary is properly sanitized for production release."
    exit 0
fi
