#!/usr/bin/env bash
# Pre-commit hook for TALON development
# This runs automatically before each commit when installed

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         TALON Pre-Commit Checks                  ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════╝${NC}"
echo ""

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
    fi
}

# Function to run a check
run_check() {
    local name="$1"
    shift
    echo -e "${YELLOW}▶${NC} Running: $name"
    
    if "$@"; then
        print_status 0 "$name passed"
        return 0
    else
        print_status 1 "$name failed"
        return 1
    fi
}

FAILED=0

# 1. Check for Rust toolchain
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}ERROR: cargo not found. Install Rust toolchain first.${NC}"
    exit 1
fi

# 2. Format check
echo ""
echo -e "${BLUE}[1/7] Checking code formatting...${NC}"
if ! run_check "cargo fmt" cargo fmt --all -- --check; then
    echo -e "${YELLOW}→ Run 'cargo fmt' to fix formatting${NC}"
    FAILED=1
fi

# 3. Clippy lints
echo ""
echo -e "${BLUE}[2/7] Running Clippy lints...${NC}"
if ! run_check "cargo clippy" cargo clippy --all-features --all-targets -- -D warnings; then
    echo -e "${YELLOW}→ Fix clippy warnings before committing${NC}"
    FAILED=1
fi

# 4. Compilation check
echo ""
echo -e "${BLUE}[3/7] Checking compilation...${NC}"
if ! run_check "cargo check" cargo check --all-features; then
    echo -e "${YELLOW}→ Fix compilation errors before committing${NC}"
    FAILED=1
fi

# 5. Fast unit tests (skip slow integration tests)
echo ""
echo -e "${BLUE}[4/7] Running fast unit tests...${NC}"
if ! run_check "cargo test (fast)" cargo test --lib --bins --all-features -- --test-threads=4; then
    echo -e "${YELLOW}→ Fix failing tests before committing${NC}"
    FAILED=1
fi

# 6. Security check (if cargo-deny is installed)
echo ""
echo -e "${BLUE}[5/7] Security audit...${NC}"
if command -v cargo-deny &> /dev/null; then
    if ! run_check "cargo deny" cargo deny check advisories; then
        echo -e "${YELLOW}→ Address security advisories${NC}"
        FAILED=1
    fi
else
    echo -e "${YELLOW}⚠${NC} cargo-deny not installed (optional), skipping..."
fi

# 7. Check for forbidden patterns
echo ""
echo -e "${BLUE}[6/7] Checking for forbidden patterns...${NC}"

# Get staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM)

if [ -n "$STAGED_FILES" ]; then
    # Check for large files
    for FILE in $STAGED_FILES; do
        if [ -f "$FILE" ]; then
            SIZE=$(stat -f%z "$FILE" 2>/dev/null || stat -c%s "$FILE" 2>/dev/null || echo 0)
            if [ "$SIZE" -gt 1048576 ]; then  # 1MB
                echo -e "${RED}✗${NC} Large file detected: $FILE (${SIZE} bytes)"
                FAILED=1
            fi
        fi
    done
    
    # Check for sensitive patterns
    if echo "$STAGED_FILES" | grep -qE '\.(key|pem|crt|p12|pfx)$'; then
        echo -e "${RED}✗${NC} Private key files detected in commit"
        FAILED=1
    fi
    
    if echo "$STAGED_FILES" | grep -qE '\.(exploit|payload)$'; then
        echo -e "${RED}✗${NC} Exploit artifacts detected in commit"
        FAILED=1
    fi
    
    # Check for secrets in file content (basic check)
    for FILE in $STAGED_FILES; do
        if [ -f "$FILE" ] && file "$FILE" | grep -q text; then
            if grep -qiE '(api[_-]?key|secret[_-]?key|password\s*=|token\s*=)' "$FILE"; then
                echo -e "${YELLOW}⚠${NC} Potential secret in: $FILE"
                echo -e "${YELLOW}  Review carefully before committing${NC}"
            fi
        fi
    done
    
    if [ "$FAILED" -eq 0 ]; then
        print_status 0 "No forbidden patterns detected"
    fi
else
    echo -e "${YELLOW}⚠${NC} No staged files to check"
fi

# 8. Check for debug statements
echo ""
echo -e "${BLUE}[7/7] Checking for debug statements...${NC}"
if echo "$STAGED_FILES" | grep -q '\.rs$'; then
    DEBUG_FOUND=0
    for FILE in $STAGED_FILES; do
        if echo "$FILE" | grep -q '\.rs$' && [ -f "$FILE" ]; then
            if grep -nE '(println!|dbg!|eprintln!)' "$FILE" | grep -v '//' | grep -v 'tests/' | grep -v '#\[cfg\(test\)\]' &>/dev/null; then
                if [ "$DEBUG_FOUND" -eq 0 ]; then
                    echo -e "${YELLOW}⚠${NC} Debug statements found (review before commit):"
                    DEBUG_FOUND=1
                fi
                grep -nE '(println!|dbg!|eprintln!)' "$FILE" | grep -v '//' | grep -v 'tests/' | head -3
            fi
        fi
    done
    
    if [ "$DEBUG_FOUND" -eq 0 ]; then
        print_status 0 "No debug statements found"
    fi
fi

# Final result
echo ""
echo -e "${BLUE}╔══════════════════════════════════════════════════╗${NC}"
if [ "$FAILED" -eq 0 ]; then
    echo -e "${GREEN}║  ✓ All pre-commit checks passed!                ║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════╝${NC}"
    exit 0
else
    echo -e "${RED}║  ✗ Pre-commit checks failed                     ║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${YELLOW}Fix the issues above or use:${NC}"
    echo -e "${YELLOW}  git commit --no-verify${NC} (not recommended)"
    echo ""
    exit 1
fi
