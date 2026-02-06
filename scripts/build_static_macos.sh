#!/bin/bash
# Build static TALON binary for macOS (universal binary)
# Run this script on macOS

set -e

echo "=== TALON Static Binary Builder (macOS) ==="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}Error: This script must be run on macOS${NC}"
    exit 1
fi

# Install Homebrew dependencies
if ! command -v brew &> /dev/null; then
    echo -e "${RED}Error: Homebrew not installed. Install from https://brew.sh/${NC}"
    exit 1
fi

echo -e "${YELLOW}Installing build dependencies...${NC}"
brew install capstone protobuf || true

# Install Rust targets
echo -e "${YELLOW}Installing Rust targets...${NC}"
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build x86_64 (Intel) binary
echo ""
echo -e "${GREEN}Building x86_64 (Intel) binary...${NC}"
cargo build --release --target x86_64-apple-darwin

# Build aarch64 (Apple Silicon) binary
echo ""
echo -e "${GREEN}Building aarch64 (Apple Silicon) binary...${NC}"
cargo build --release --target aarch64-apple-darwin

# Create universal binary
echo ""
echo -e "${GREEN}Creating universal binary...${NC}"
mkdir -p target/universal/release

lipo -create \
    target/x86_64-apple-darwin/release/talon \
    target/aarch64-apple-darwin/release/talon \
    -output target/universal/release/talon

# Make executable
chmod +x target/universal/release/talon

# Verify universal binary
echo ""
echo -e "${GREEN}=== Universal Binary Info ===${NC}"
lipo -info target/universal/release/talon
file target/universal/release/talon

# Strip debug symbols
echo ""
echo -e "${GREEN}Stripping debug symbols...${NC}"
strip target/universal/release/talon

# Check size
SIZE=$(stat -f%z target/universal/release/talon 2>/dev/null || echo "0")
SIZE_MB=$((SIZE / 1024 / 1024))

echo ""
echo -e "${GREEN}=== Build Complete ===${NC}"
echo "Binary: target/universal/release/talon"
echo "Size: ${SIZE_MB}MB"

if [ $SIZE_MB -gt 50 ]; then
    echo -e "${YELLOW}Warning: Binary exceeds 50MB target (${SIZE_MB}MB)${NC}"
fi

# Check for dynamic dependencies
echo ""
echo -e "${GREEN}=== Dependency Check ===${NC}"
otool -L target/universal/release/talon

# Test execution
echo ""
echo -e "${GREEN}=== Testing binary ===${NC}"
target/universal/release/talon --version

# Generate checksum
echo ""
echo -e "${GREEN}=== Generating checksum ===${NC}"
shasum -a 256 target/universal/release/talon > target/universal/release/talon.sha256
cat target/universal/release/talon.sha256

echo ""
echo -e "${GREEN}Build successful! Universal binary ready for distribution.${NC}"
