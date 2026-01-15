#!/bin/bash
# Security audit script for TALON
# Runs comprehensive security checks using cargo-audit and cargo-deny

set -e

echo "=================================="
echo "TALON Security Audit"
echo "=================================="
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo is not installed"
    echo "Please install Rust: https://rustup.rs/"
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Install cargo-audit if not present
if ! command -v cargo-audit &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-audit...${NC}"
    cargo install cargo-audit
fi

# Install cargo-deny if not present
if ! command -v cargo-deny &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-deny...${NC}"
    cargo install cargo-deny
fi

echo ""
echo "=================================="
echo "1. Cargo Audit - Vulnerability Scan"
echo "=================================="
echo ""

# Run cargo audit
if cargo audit --deny warnings; then
    echo -e "${GREEN}✓ No known vulnerabilities found${NC}"
    AUDIT_PASS=true
else
    echo -e "${RED}✗ Security vulnerabilities detected${NC}"
    AUDIT_PASS=false
fi

echo ""
echo "=================================="
echo "2. Cargo Deny - License & Supply Chain"
echo "=================================="
echo ""

# Run cargo deny checks
DENY_PASS=true

echo -e "${BLUE}Checking advisories...${NC}"
if cargo deny check advisories; then
    echo -e "${GREEN}✓ No advisory issues${NC}"
else
    echo -e "${RED}✗ Advisory issues detected${NC}"
    DENY_PASS=false
fi

echo ""
echo -e "${BLUE}Checking licenses...${NC}"
if cargo deny check licenses; then
    echo -e "${GREEN}✓ All licenses approved${NC}"
else
    echo -e "${RED}✗ License issues detected${NC}"
    DENY_PASS=false
fi

echo ""
echo -e "${BLUE}Checking bans...${NC}"
if cargo deny check bans; then
    echo -e "${GREEN}✓ No banned dependencies${NC}"
else
    echo -e "${RED}✗ Banned dependencies detected${NC}"
    DENY_PASS=false
fi

echo ""
echo -e "${BLUE}Checking sources...${NC}"
if cargo deny check sources; then
    echo -e "${GREEN}✓ All sources approved${NC}"
else
    echo -e "${RED}✗ Source issues detected${NC}"
    DENY_PASS=false
fi

echo ""
echo "=================================="
echo "3. Dependency Tree Analysis"
echo "=================================="
echo ""

# Show dependency tree for critical security dependencies
echo -e "${BLUE}Critical security dependencies:${NC}"
cargo tree -p openssl -p rustls -p ring -p webpki 2>/dev/null || echo "No TLS dependencies found"

echo ""
echo "=================================="
echo "4. Outdated Dependencies Check"
echo "=================================="
echo ""

# Check for outdated dependencies
if command -v cargo-outdated &> /dev/null; then
    cargo outdated --root-deps-only
else
    echo -e "${YELLOW}Skipping (cargo-outdated not installed)${NC}"
    echo "Install with: cargo install cargo-outdated"
fi

echo ""
echo "=================================="
echo "Security Audit Summary"
echo "=================================="
echo ""

# Summary
if [ "$AUDIT_PASS" = true ] && [ "$DENY_PASS" = true ]; then
    echo -e "${GREEN}✓ All security checks passed${NC}"
    echo ""
    echo "Your project has no known vulnerabilities and complies with"
    echo "security policies defined in deny.toml"
    exit 0
else
    echo -e "${RED}✗ Security issues detected${NC}"
    echo ""
    echo "Please review the errors above and:"
    echo "1. Update vulnerable dependencies"
    echo "2. Review and approve licenses if acceptable"
    echo "3. Remove or replace banned dependencies"
    echo "4. Verify source registry configurations"
    exit 1
fi
