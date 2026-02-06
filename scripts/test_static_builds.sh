#!/bin/bash
# Test static builds for TALON
# Verifies build configuration, static linking, and binary execution

set -e

echo "=== TALON Static Build Test Suite ==="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

FAILED_TESTS=0
PASSED_TESTS=0

# Test helper functions
pass() {
    echo -e "${GREEN}✓ PASS:${NC} $1"
    PASSED_TESTS=$((PASSED_TESTS + 1))
}

fail() {
    echo -e "${RED}✗ FAIL:${NC} $1"
    FAILED_TESTS=$((FAILED_TESTS + 1))
}

warn() {
    echo -e "${YELLOW}⚠ WARNING:${NC} $1"
}

info() {
    echo -e "${BLUE}ℹ INFO:${NC} $1"
}

# Check environment
echo -e "${BLUE}=== Environment Check ===${NC}"

if [[ "$OSTYPE" =~ ^linux ]]; then
    info "Running on Linux"
    PLATFORM="linux"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    info "Running on macOS"
    PLATFORM="macos"
else
    warn "Running on unsupported platform: $OSTYPE"
    PLATFORM="unknown"
fi

# Check Rust installation
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    pass "Rust installed: $RUST_VERSION"
else
    fail "Rust not installed"
    exit 1
fi

# Check cargo
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    pass "Cargo installed: $CARGO_VERSION"
else
    fail "Cargo not installed"
    exit 1
fi

echo ""

# Test 1: Verify Cargo.toml has profile settings
echo -e "${BLUE}=== Test 1: Profile Configuration ===${NC}"

if grep -q "\[profile.release\]" Cargo.toml; then
    pass "Release profile configured in Cargo.toml"
else
    fail "Release profile not found in Cargo.toml"
fi

if grep -q "\[profile.release-small\]" Cargo.toml; then
    pass "Release-small profile configured in Cargo.toml"
else
    fail "Release-small profile not found in Cargo.toml"
fi

if grep -q "lto = \"fat\"" Cargo.toml; then
    pass "Link-time optimization enabled"
else
    warn "LTO not configured"
fi

echo ""

# Test 2: Verify .cargo/config.toml has target settings
echo -e "${BLUE}=== Test 2: Target Configuration ===${NC}"

if [ -f .cargo/config.toml ]; then
    pass ".cargo/config.toml exists"
    
    if grep -q "\[target.x86_64-unknown-linux-musl\]" .cargo/config.toml; then
        pass "Linux musl target configured"
    else
        fail "Linux musl target not configured"
    fi
    
    if grep -q "\[target.x86_64-pc-windows-msvc\]" .cargo/config.toml; then
        pass "Windows MSVC target configured"
    else
        fail "Windows MSVC target not configured"
    fi
    
    if grep -q "target-feature=+crt-static" .cargo/config.toml; then
        pass "Static CRT enabled for targets"
    else
        warn "Static CRT not configured"
    fi
else
    fail ".cargo/config.toml not found"
fi

echo ""

# Test 3: Check for problematic dependencies
echo -e "${BLUE}=== Test 3: Dependency Analysis ===${NC}"

info "Analyzing dependencies for static linking compatibility..."

# Check if any dependencies use dynamic linking
if grep -q "cdylib" Cargo.toml; then
    warn "cdylib dependency found (may require dynamic linking)"
fi

# Check for OpenSSL (can be problematic)
if grep -q "openssl" Cargo.lock 2>/dev/null; then
    warn "OpenSSL dependency detected (may require static linking configuration)"
fi

# Verify optional features are properly gated
if grep -q "keystone-engine.*optional = true" Cargo.toml; then
    pass "keystone-engine is optional (good for static builds)"
else
    warn "keystone-engine dependency may be required"
fi

if grep -q "z3.*optional = true" Cargo.toml; then
    pass "z3 is optional"
else
    warn "z3 dependency may be required"
fi

echo ""

# Test 4: Build test (if on Linux)
if [ "$PLATFORM" = "linux" ]; then
    echo -e "${BLUE}=== Test 4: Build Test (Linux musl) ===${NC}"
    
    # Check for musl-tools
    if command -v musl-gcc &> /dev/null; then
        pass "musl-gcc available"
        
        # Check if musl target is installed
        if rustup target list | grep -q "x86_64-unknown-linux-musl (installed)"; then
            pass "musl target installed"
            
            info "Attempting test build (this may take several minutes)..."
            
            # Build in release mode
            if cargo build --release --target x86_64-unknown-linux-musl 2>&1 | tee /tmp/build.log | grep -q "Finished"; then
                pass "Build succeeded"
                
                # Check if binary exists
                if [ -f target/x86_64-unknown-linux-musl/release/talon ]; then
                    pass "Binary created: target/x86_64-unknown-linux-musl/release/talon"
                    
                    # Check size
                    SIZE=$(stat -c%s target/x86_64-unknown-linux-musl/release/talon)
                    SIZE_MB=$((SIZE / 1024 / 1024))
                    info "Binary size: ${SIZE_MB}MB"
                    
                    if [ $SIZE_MB -lt 50 ]; then
                        pass "Binary size within 50MB target"
                    else
                        warn "Binary size exceeds 50MB target (${SIZE_MB}MB)"
                    fi
                    
                    # Test static linking
                    if ldd target/x86_64-unknown-linux-musl/release/talon 2>&1 | grep -q "not a dynamic executable"; then
                        pass "Binary is statically linked"
                    elif ! ldd target/x86_64-unknown-linux-musl/release/talon &> /dev/null; then
                        pass "Binary is statically linked (no ldd output)"
                    else
                        warn "Binary may have dynamic dependencies:"
                        ldd target/x86_64-unknown-linux-musl/release/talon
                    fi
                    
                    # Test execution
                    if target/x86_64-unknown-linux-musl/release/talon --version &> /dev/null; then
                        VERSION=$(target/x86_64-unknown-linux-musl/release/talon --version)
                        pass "Binary executes: $VERSION"
                    else
                        fail "Binary execution failed"
                    fi
                else
                    fail "Binary not created"
                fi
            else
                fail "Build failed (check /tmp/build.log)"
            fi
        else
            warn "musl target not installed (run: rustup target add x86_64-unknown-linux-musl)"
        fi
    else
        warn "musl-gcc not available (run: sudo apt-get install musl-tools)"
    fi
fi

echo ""

# Test 5: GitHub Actions workflow verification
echo -e "${BLUE}=== Test 5: CI/CD Configuration ===${NC}"

if [ -f .github/workflows/build-matrix.yml ]; then
    pass "Build matrix workflow exists"
    
    # Check for all platforms
    if grep -q "x86_64-unknown-linux-musl" .github/workflows/build-matrix.yml; then
        pass "Linux x64 musl build configured"
    else
        fail "Linux x64 musl build not configured"
    fi
    
    if grep -q "aarch64-unknown-linux-musl" .github/workflows/build-matrix.yml; then
        pass "Linux ARM64 musl build configured"
    else
        warn "Linux ARM64 musl build not configured"
    fi
    
    if grep -q "x86_64-pc-windows-msvc" .github/workflows/build-matrix.yml; then
        pass "Windows x64 MSVC build configured"
    else
        fail "Windows x64 MSVC build not configured"
    fi
    
    if grep -q "x86_64-apple-darwin" .github/workflows/build-matrix.yml; then
        pass "macOS x64 build configured"
    else
        fail "macOS x64 build not configured"
    fi
    
    if grep -q "aarch64-apple-darwin" .github/workflows/build-matrix.yml; then
        pass "macOS ARM64 build configured"
    else
        warn "macOS ARM64 build not configured"
    fi
    
    # Check for verification job
    if grep -q "verify-static" .github/workflows/build-matrix.yml; then
        pass "Static verification job configured"
    else
        warn "Static verification job not found"
    fi
    
    # Check for Alpine container test
    if grep -q "alpine:latest" .github/workflows/build-matrix.yml; then
        pass "Alpine container test configured"
    else
        warn "Alpine container test not configured"
    fi
else
    fail "Build matrix workflow not found"
fi

echo ""

# Test 6: Build scripts verification
echo -e "${BLUE}=== Test 6: Build Scripts ===${NC}"

if [ -f scripts/build_static.sh ]; then
    pass "Linux build script exists"
    
    if [ -x scripts/build_static.sh ]; then
        pass "Linux build script is executable"
    else
        warn "Linux build script not executable (run: chmod +x scripts/build_static.sh)"
    fi
else
    fail "Linux build script not found"
fi

if [ -f scripts/build_static.ps1 ]; then
    pass "Windows build script exists"
else
    fail "Windows build script not found"
fi

if [ -f scripts/build_static_macos.sh ]; then
    pass "macOS build script exists"
    
    if [ -x scripts/build_static_macos.sh ]; then
        pass "macOS build script is executable"
    else
        warn "macOS build script not executable (run: chmod +x scripts/build_static_macos.sh)"
    fi
else
    fail "macOS build script not found"
fi

echo ""

# Summary
echo -e "${BLUE}=== Test Summary ===${NC}"
echo -e "Passed: ${GREEN}${PASSED_TESTS}${NC}"
echo -e "Failed: ${RED}${FAILED_TESTS}${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo ""
    echo -e "${GREEN}All critical tests passed!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}Some tests failed. Review output above.${NC}"
    exit 1
fi
