#!/bin/bash
# Build static TALON binary for Linux (musl)
# Run this script in WSL or Linux environment

set -e

echo "=== TALON Static Binary Builder (Linux musl) ==="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if running in WSL/Linux
if [[ ! "$OSTYPE" =~ ^linux ]]; then
    echo -e "${RED}Error: This script must be run on Linux or WSL${NC}"
    exit 1
fi

# Install musl-tools if not present
if ! command -v musl-gcc &> /dev/null; then
    echo -e "${YELLOW}Installing musl-tools...${NC}"
    sudo apt-get update
    sudo apt-get install -y musl-tools musl-dev
fi

# Install Rust musl target if not present
if ! rustup target list | grep -q "x86_64-unknown-linux-musl (installed)"; then
    echo -e "${YELLOW}Installing Rust musl target...${NC}"
    rustup target add x86_64-unknown-linux-musl
fi

# Optional: Install ARM64 target for cross-compilation
read -p "Install ARM64 musl target? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    rustup target add aarch64-unknown-linux-musl
    
    if ! command -v cross &> /dev/null; then
        echo -e "${YELLOW}Installing cross for ARM compilation...${NC}"
        cargo install cross --git https://github.com/cross-rs/cross
    fi
fi

# Build x64 musl binary
echo ""
echo -e "${GREEN}Building x86_64 musl binary...${NC}"
cargo build --release --target x86_64-unknown-linux-musl

# Strip binary
echo -e "${GREEN}Stripping debug symbols...${NC}"
strip target/x86_64-unknown-linux-musl/release/talon

# Check binary size
SIZE=$(stat -c%s target/x86_64-unknown-linux-musl/release/talon 2>/dev/null || echo "0")
SIZE_MB=$((SIZE / 1024 / 1024))

echo ""
echo -e "${GREEN}=== Build Complete ===${NC}"
echo "Binary: target/x86_64-unknown-linux-musl/release/talon"
echo "Size: ${SIZE_MB}MB"

if [ $SIZE_MB -gt 50 ]; then
    echo -e "${YELLOW}Warning: Binary exceeds 50MB target (${SIZE_MB}MB)${NC}"
fi

# Verify static linking
echo ""
echo -e "${GREEN}=== Dependency Check ===${NC}"
ldd target/x86_64-unknown-linux-musl/release/talon || echo "Statically linked (expected)"

# Test execution
echo ""
echo -e "${GREEN}=== Testing binary ===${NC}"
target/x86_64-unknown-linux-musl/release/talon --version

# Generate checksum
echo ""
echo -e "${GREEN}=== Generating checksum ===${NC}"
sha256sum target/x86_64-unknown-linux-musl/release/talon > target/x86_64-unknown-linux-musl/release/talon.sha256
cat target/x86_64-unknown-linux-musl/release/talon.sha256

echo ""
echo -e "${GREEN}Build successful! Static binary ready for distribution.${NC}"
