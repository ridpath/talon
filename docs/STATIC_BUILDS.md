# TALON Static Binary Builds

Comprehensive guide for building static TALON binaries for distribution across Linux, Windows, and macOS platforms.

## Build Matrix

TALON supports building fully static binaries for the following platforms:

| Platform | Target Triple | Binary Size Target | Static Linking |
|----------|---------------|-------------------|----------------|
| Linux x64 | `x86_64-unknown-linux-musl` | <50MB | Full (musl) |
| Linux ARM64 | `aarch64-unknown-linux-musl` | <50MB | Full (musl) |
| Windows x64 | `x86_64-pc-windows-msvc` | <50MB | Static CRT |
| Windows x86 | `i686-pc-windows-msvc` | <50MB | Static CRT |
| macOS x64 | `x86_64-apple-darwin` | <50MB | System libs |
| macOS ARM64 | `aarch64-apple-darwin` | <50MB | System libs |
| macOS Universal | Combined x64+ARM64 | <100MB | System libs |

## Quick Start

### Linux (WSL/Native)

```bash
# Run automated build script
bash scripts/build_static.sh

# Binary output: target/x86_64-unknown-linux-musl/release/talon
```

### Windows

```powershell
# Run automated build script
.\scripts\build_static.ps1

# Binary output: target\x86_64-pc-windows-msvc\release\talon.exe
```

### macOS

```bash
# Run automated build script
bash scripts/build_static_macos.sh

# Universal binary output: target/universal/release/talon
```

## Manual Build Instructions

### Linux musl (Static)

**Prerequisites:**
```bash
# Ubuntu/Debian
sudo apt-get install musl-tools musl-dev build-essential libssl-dev pkg-config

# Install Rust target
rustup target add x86_64-unknown-linux-musl
```

**Build:**
```bash
cargo build --release --target x86_64-unknown-linux-musl

# Strip debug symbols
strip target/x86_64-unknown-linux-musl/release/talon

# Verify static linking (should show "statically linked" or no output)
ldd target/x86_64-unknown-linux-musl/release/talon
```

**Cross-compile for ARM64:**
```bash
# Install cross tool
cargo install cross --git https://github.com/cross-rs/cross

# Build ARM64
cross build --release --target aarch64-unknown-linux-musl
```

### Windows MSVC (Static CRT)

**Prerequisites:**
```powershell
# Install Visual Studio Build Tools with C++ support
# Or install via Chocolatey:
choco install visualstudio2022buildtools --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools"

# Install protobuf compiler
choco install protoc

# Install Rust target
rustup target add x86_64-pc-windows-msvc
```

**Build:**
```powershell
cargo build --release --target x86_64-pc-windows-msvc

# Check dependencies (should only show system DLLs)
dumpbin /DEPENDENTS target\x86_64-pc-windows-msvc\release\talon.exe
```

### macOS Universal Binary

**Prerequisites:**
```bash
# Install Homebrew dependencies
brew install capstone protobuf

# Install Rust targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

**Build:**
```bash
# Build Intel binary
cargo build --release --target x86_64-apple-darwin

# Build Apple Silicon binary
cargo build --release --target aarch64-apple-darwin

# Create universal binary
lipo -create \
    target/x86_64-apple-darwin/release/talon \
    target/aarch64-apple-darwin/release/talon \
    -output talon-universal

# Verify
lipo -info talon-universal
```

## Dependency Static Linking

### Fully Static Dependencies

These dependencies compile into the binary with no external requirements:

- **Core Rust crates**: All pure Rust dependencies
- **z3**: Uses `static-link-z3` feature flag
- **AES/crypto**: Pure Rust implementations (ring, aes-gcm, etc.)
- **Parsing**: pest, pest_derive
- **Networking**: Pure Rust implementations

### C Library Dependencies

Some dependencies require C libraries. Static linking strategy:

| Dependency | Library | Static Linking Strategy |
|------------|---------|------------------------|
| `capstone` | libcapstone | Statically linked via musl on Linux |
| `ssh2` | libssh2 | Statically linked via musl on Linux |
| `openssl` | OpenSSL | Vendored or statically linked (musl) |
| `keystone-engine` | keystone | Optional feature, static when enabled |
| `yara` | libyara | Optional feature, static when enabled |

### musl Static Linking

On Linux, musl-libc enables full static linking including C dependencies:

```bash
# Example: ssh2 with static libssh2
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-musl
```

Configuration in `.cargo/config.toml`:
```toml
[target.x86_64-unknown-linux-musl]
rustflags = ["-C", "target-feature=+crt-static", "-C", "link-arg=-static"]
```

### Windows Static CRT

Windows uses static C runtime via MSVC:

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

This embeds the C runtime into the binary, eliminating VCRUNTIME DLL dependencies.

## Build Profiles

### `release` (Default)

Optimized for performance:
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

### `release-small` (Size-optimized)

For minimal binary size:
```bash
cargo build --profile release-small --target x86_64-unknown-linux-musl
```

Configuration:
```toml
[profile.release-small]
opt-level = "z"  # Optimize for size
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

## CI/CD Build Matrix

GitHub Actions workflow automatically builds for all platforms:

```yaml
# .github/workflows/build-matrix.yml
# Triggered on: push to main, tags, pull requests
# Outputs: Static binaries for 6+ platforms
# Verification: Tests on Alpine Linux (no glibc)
```

**Workflow steps:**
1. Build matrix for all targets
2. Cross-compile ARM/alternate architectures
3. Strip debug symbols
4. Verify static linking (Linux: ldd, Windows: dumpbin)
5. Test binary execution
6. Generate SHA256 checksums
7. Upload artifacts (30-day retention)
8. Create GitHub release (on version tags)

## Verification

### Linux Static Verification

Test on minimal Alpine Linux container (no glibc):

```bash
# Run in Docker
docker run --rm -v $(pwd):/work -w /work alpine:latest sh -c "
    chmod +x target/x86_64-unknown-linux-musl/release/talon
    ./target/x86_64-unknown-linux-musl/release/talon --version
"
```

Expected: Binary runs successfully with no dependency errors.

### Windows Dependency Check

```powershell
# Check DLL dependencies
dumpbin /DEPENDENTS target\x86_64-pc-windows-msvc\release\talon.exe

# Expected output (only system DLLs):
# KERNEL32.dll
# ADVAPI32.dll
# WS2_32.dll
# (no VCRUNTIME or other C runtime DLLs)
```

### macOS Dependency Check

```bash
# Check dynamic libraries
otool -L target/universal/release/talon

# Expected: Only system frameworks (/usr/lib/libSystem, /System/Library/Frameworks)
```

## Binary Size Optimization

Target: <50MB stripped binary

### Current Size Analysis

```bash
# Check size
ls -lh target/x86_64-unknown-linux-musl/release/talon

# Breakdown by section
size target/x86_64-unknown-linux-musl/release/talon
```

### Size Reduction Strategies

1. **LTO (Link-Time Optimization)**: Already enabled (`lto = "fat"`)
2. **Code generation units**: Reduced to 1 for maximum optimization
3. **Strip symbols**: Automated via `strip = true`
4. **Opt-level "z"**: Use `release-small` profile for size-critical builds
5. **Feature flags**: Disable optional features not needed for distribution

### Size Comparison

| Build Type | Estimated Size |
|------------|----------------|
| Debug | ~200MB |
| Release | ~40-50MB |
| Release-small | ~30-40MB |
| Stripped release | ~25-35MB |

## Distribution

### Artifact Naming Convention

```
talon-<platform>-<arch>[.ext]
talon-<platform>-<arch>.sha256
```

Examples:
- `talon-linux-x64` (musl static)
- `talon-linux-arm64` (musl static)
- `talon-windows-x64.exe` (MSVC static CRT)
- `talon-windows-x86.exe` (MSVC static CRT)
- `talon-macos-x64` (Intel native)
- `talon-macos-arm64` (Apple Silicon native)
- `talon-macos-universal` (x64 + ARM64 fat binary)

### Checksums

SHA256 checksums generated for all binaries:

```bash
# Linux/macOS
sha256sum talon-linux-x64 > talon-linux-x64.sha256

# Windows
certutil -hashfile talon-windows-x64.exe SHA256 > talon-windows-x64.exe.sha256
```

## Troubleshooting

### "cannot find -lssl" (Linux)

**Cause**: OpenSSL development headers not installed.

**Solution**:
```bash
sudo apt-get install libssl-dev pkg-config
```

### "linker `x86_64-linux-musl-gcc` not found" (Linux)

**Cause**: musl-tools not installed.

**Solution**:
```bash
sudo apt-get install musl-tools musl-dev
```

### "VCRUNTIME140.dll not found" (Windows)

**Cause**: Static CRT not enabled or wrong build target.

**Solution**:
```powershell
# Ensure using MSVC target (not GNU)
cargo build --release --target x86_64-pc-windows-msvc

# Verify .cargo/config.toml has static CRT flag
```

### Binary size exceeds 50MB

**Causes**:
- Debug symbols not stripped
- LTO not enabled
- Multiple codegen units

**Solutions**:
```bash
# Use release profile (should be automatic)
cargo build --release --target <target>

# Manually strip if needed
strip target/<target>/release/talon

# Use size-optimized profile
cargo build --profile release-small --target <target>
```

### "undefined reference to symbol" (Linux cross-compile)

**Cause**: Cross-compilation toolchain missing or incorrect.

**Solution**:
```bash
# Use cross tool instead of cargo
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target aarch64-unknown-linux-musl
```

## References

- [Rust Cross-Compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [musl-libc](https://www.musl-libc.org/)
- [cross tool](https://github.com/cross-rs/cross)
- [GitHub Actions Workflows](.github/workflows/build-matrix.yml)
