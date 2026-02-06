# TALON Static Build Scripts

Build scripts for creating static TALON binaries for Linux, Windows, and macOS.

## Quick Start

### Linux (WSL/Native)

```bash
# Make scripts executable (first time only)
chmod +x scripts/build_static.sh
chmod +x scripts/test_static_builds.sh

# Build static binary
bash scripts/build_static.sh

# Test build configuration
bash scripts/test_static_builds.sh
```

### Windows

```powershell
# Build static binary
.\scripts\build_static.ps1

# Make shell scripts executable in Git (for WSL usage)
.\scripts\make_executable.bat
```

### macOS

```bash
# Make script executable (first time only)
chmod +x scripts/build_static_macos.sh

# Build universal binary
bash scripts/build_static_macos.sh
```

## Available Scripts

### `build_static.sh` (Linux)

Automated build script for Linux musl static binaries.

**Features:**
- Checks for musl-tools installation
- Installs Rust musl target
- Builds x86_64 static binary
- Optional ARM64 cross-compilation
- Verifies static linking
- Generates SHA256 checksum
- Tests binary execution

**Output:** `target/x86_64-unknown-linux-musl/release/talon`

**Usage:**
```bash
bash scripts/build_static.sh
```

### `build_static.ps1` (Windows)

Automated build script for Windows MSVC static binaries.

**Features:**
- Checks for Rust/MSVC installation
- Installs MSVC target
- Optional 32-bit (i686) build
- Builds with static CRT
- Dependency analysis (dumpbin)
- Generates SHA256 checksum
- Tests binary execution

**Output:** `target\x86_64-pc-windows-msvc\release\talon.exe`

**Usage:**
```powershell
.\scripts\build_static.ps1
```

### `build_static_macos.sh` (macOS)

Automated build script for macOS universal binaries.

**Features:**
- Checks for Homebrew dependencies
- Builds x86_64 (Intel) binary
- Builds aarch64 (Apple Silicon) binary
- Creates universal binary with `lipo`
- Verifies universal binary
- Generates SHA256 checksum
- Tests binary execution

**Output:** `target/universal/release/talon`

**Usage:**
```bash
bash scripts/build_static_macos.sh
```

### `test_static_builds.sh`

Comprehensive test suite for build configuration.

**Tests:**
1. Profile configuration in Cargo.toml
2. Target configuration in .cargo/config.toml
3. Dependency analysis for static linking
4. Build test (Linux musl)
5. CI/CD workflow verification
6. Build script verification

**Usage:**
```bash
bash scripts/test_static_builds.sh
```

**Expected output:**
```
=== TALON Static Build Test Suite ===
✓ PASS: Rust installed
✓ PASS: Release profile configured
✓ PASS: Linux musl target configured
...
=== Test Summary ===
Passed: 25
Failed: 0

All critical tests passed!
```

### `make_executable.bat` (Windows helper)

Makes shell scripts executable in Git on Windows.

**Usage:**
```batch
.\scripts\make_executable.bat
```

This updates Git's index to mark scripts as executable, allowing them to run in WSL or Git Bash.

## Build Targets

### Linux musl (Fully Static)

**Target:** `x86_64-unknown-linux-musl`

**Requirements:**
- musl-tools
- musl-dev
- Rust musl target

**Install:**
```bash
sudo apt-get install musl-tools musl-dev
rustup target add x86_64-unknown-linux-musl
```

**Verify static linking:**
```bash
ldd target/x86_64-unknown-linux-musl/release/talon
# Expected: "not a dynamic executable" or "statically linked"
```

### Linux ARM64 (Cross-compilation)

**Target:** `aarch64-unknown-linux-musl`

**Requirements:**
- cross tool

**Install:**
```bash
cargo install cross --git https://github.com/cross-rs/cross
rustup target add aarch64-unknown-linux-musl
```

**Build:**
```bash
cross build --release --target aarch64-unknown-linux-musl
```

### Windows MSVC (Static CRT)

**Target:** `x86_64-pc-windows-msvc`

**Requirements:**
- Visual Studio Build Tools with C++ support
- Rust MSVC target

**Install:**
```powershell
# Via Visual Studio Installer or Chocolatey
choco install visualstudio2022buildtools --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools"

# Rust target
rustup target add x86_64-pc-windows-msvc
```

**Verify dependencies:**
```powershell
dumpbin /DEPENDENTS target\x86_64-pc-windows-msvc\release\talon.exe
# Expected: Only system DLLs (KERNEL32, ADVAPI32, WS2_32)
# Should NOT show: VCRUNTIME140.dll, MSVCP140.dll
```

### macOS Universal

**Targets:**
- `x86_64-apple-darwin` (Intel)
- `aarch64-apple-darwin` (Apple Silicon)

**Requirements:**
- Homebrew
- Rust targets

**Install:**
```bash
brew install capstone protobuf
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

**Verify universal binary:**
```bash
lipo -info target/universal/release/talon
# Expected: "Architectures in the fat file: talon are: x86_64 arm64"
```

## CI/CD Integration

### GitHub Actions Workflow

**File:** `.github/workflows/build-matrix.yml`

**Triggers:**
- Push to `main` branch
- Pull requests to `main`
- Version tags (`v*`)
- Manual dispatch

**Build matrix:**
- Linux x64 musl
- Linux ARM64 musl
- Windows x64 MSVC
- Windows x86 MSVC
- macOS x64
- macOS ARM64
- macOS Universal (combined)

**Verification:**
- Static linking test on Alpine Linux
- Binary execution test
- Size verification (<50MB target)
- SHA256 checksum generation

**Artifacts:**
- Static binaries (30-day retention)
- SHA256 checksums
- GitHub release (on version tags)

## Troubleshooting

### "musl-gcc not found" (Linux)

**Solution:**
```bash
sudo apt-get update
sudo apt-get install musl-tools musl-dev
```

### "error: linker `x86_64-linux-musl-gcc` not found" (Linux)

**Solution:**
Ensure musl-tools is installed and the Rust target is added:
```bash
sudo apt-get install musl-tools
rustup target add x86_64-unknown-linux-musl
```

### "VCRUNTIME140.dll not found" (Windows)

**Cause:** Static CRT not enabled.

**Solution:**
Verify `.cargo/config.toml` has:
```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

Rebuild with:
```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

### Binary size exceeds 50MB

**Solutions:**

1. **Use size-optimized profile:**
   ```bash
   cargo build --profile release-small --target <target>
   ```

2. **Ensure stripping is enabled** (check Cargo.toml):
   ```toml
   [profile.release]
   strip = true
   ```

3. **Manually strip symbols:**
   ```bash
   # Linux/macOS
   strip target/<target>/release/talon
   
   # Windows (requires Windows SDK)
   # Stripping handled automatically
   ```

4. **Disable optional features:**
   ```bash
   cargo build --release --target <target> --no-default-features
   ```

### Cross-compilation fails

**For ARM64 on x64 host:**

Use `cross` tool instead of `cargo`:
```bash
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target aarch64-unknown-linux-musl
```

## Binary Verification

### Linux

```bash
# Check static linking
ldd target/x86_64-unknown-linux-musl/release/talon

# Check size
ls -lh target/x86_64-unknown-linux-musl/release/talon

# Test execution
target/x86_64-unknown-linux-musl/release/talon --version

# Verify on Alpine (Docker)
docker run --rm -v $(pwd):/work -w /work alpine:latest \
    sh -c "./target/x86_64-unknown-linux-musl/release/talon --version"
```

### Windows

```powershell
# Check dependencies
dumpbin /DEPENDENTS target\x86_64-pc-windows-msvc\release\talon.exe

# Check size
Get-Item target\x86_64-pc-windows-msvc\release\talon.exe | Select Length

# Test execution
target\x86_64-pc-windows-msvc\release\talon.exe --version
```

### macOS

```bash
# Check architecture
lipo -info target/universal/release/talon

# Check dependencies
otool -L target/universal/release/talon

# Check size
ls -lh target/universal/release/talon

# Test execution (both architectures)
target/universal/release/talon --version
```

## Performance

### Build times

| Target | Average Time | LTO Enabled |
|--------|--------------|-------------|
| Linux x64 musl | 5-10 min | Yes (fat) |
| Windows x64 MSVC | 8-12 min | Yes (fat) |
| macOS Universal | 15-20 min | Yes (fat) |

**Note:** First build is slower due to dependency compilation. Subsequent builds use Cargo's incremental compilation.

### Optimization levels

**`release` profile:**
- `opt-level = 3` (maximum optimization)
- `lto = "fat"` (full link-time optimization)
- `codegen-units = 1` (single codegen unit)
- Target size: ~40-50MB

**`release-small` profile:**
- `opt-level = "z"` (optimize for size)
- `lto = "fat"`
- `codegen-units = 1`
- Target size: ~30-40MB

## References

- [Static Binary Builds Documentation](../docs/STATIC_BUILDS.md)
- [Cargo Build Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [Rust Cross-Compilation](https://rust-lang.github.io/rustup/cross-compilation.html)
- [musl-libc](https://www.musl-libc.org/)
