#!/bin/bash
# Shellcode & Format String Test Runner
# Validates all payload generation and exploitation primitives

set -e

echo "=========================================="
echo "TALON Shellcode & Format String Tests"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
TOTAL_TESTS=99
PASSED=0
FAILED=0

# Function to run test and capture result
run_test() {
    local test_name=$1
    echo -n "Running $test_name... "
    
    if cargo test "$test_name" --quiet 2>&1 > /dev/null; then
        echo -e "${GREEN}✓ PASS${NC}"
        ((PASSED++))
    else
        echo -e "${RED}✗ FAIL${NC}"
        ((FAILED++))
    fi
}

# Banner
echo -e "${YELLOW}Phase 1: Shellcode Tests (46 tests)${NC}"
echo "=========================================="

# Run shellcode tests
cargo test shellcode_test --quiet

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ All shellcode tests passed${NC}"
    PASSED=$((PASSED + 46))
else
    echo -e "${RED}✗ Some shellcode tests failed${NC}"
    FAILED=$((FAILED + 46))
fi

echo ""
echo -e "${YELLOW}Phase 2: Format String Tests (53 tests)${NC}"
echo "=========================================="

# Run format string tests
cargo test format_string_test --quiet

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ All format string tests passed${NC}"
    PASSED=$((PASSED + 53))
else
    echo -e "${RED}✗ Some format string tests failed${NC}"
    FAILED=$((FAILED + 53))
fi

echo ""
echo "=========================================="
echo "Test Results Summary"
echo "=========================================="
echo -e "Total Tests:  $TOTAL_TESTS"
echo -e "${GREEN}Passed:       $PASSED${NC}"

if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Failed:       $FAILED${NC}"
else
    echo -e "Failed:       0"
fi

echo "=========================================="

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ ALL TESTS PASSED!${NC}"
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    echo ""
    echo "To debug failures, run:"
    echo "  cargo test shellcode_test -- --nocapture"
    echo "  cargo test format_string_test -- --nocapture"
    exit 1
fi
