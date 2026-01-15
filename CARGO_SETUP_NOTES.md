# Cargo Installation and Build System Setup

## Current Status

### Rust Toolchain Installed
- **rustc**: 1.92.0 (ded5c06cf 2025-12-08)
- **cargo**: 1.92.0 (344c4567c 2025-10-21)
- **Location**: `%USERPROFILE%\.cargo\bin`

### Toolchains Available
- `stable-x86_64-pc-windows-msvc` (requires Visual Studio C++ Build Tools)
- `stable-x86_64-pc-windows-gnu` (requires MinGW-w64 GCC) - **ACTIVE**

## Issue Identified

The project requires a C linker to compile. Two options:

### Option 1: Install Visual Studio Build Tools (MSVC)
**Recommended for Windows native development**

Download and install: https://visualstudio.microsoft.com/downloads/
- Select "Desktop development with C++"
- Minimum components:
  - MSVC v143 - VS 2022 C++ x64/x86 build tools
  - Windows 10/11 SDK

After installation:
```cmd
rustup default stable-x86_64-pc-windows-msvc
cargo build --release
```

### Option 2: Install MinGW-w64 (GNU toolchain)
**Alternative for cross-platform compatibility**

1. Download MinGW-w64:
   - https://github.com/brechtsanders/winlibs_mingw/releases/
   - Get: `winlibs-x86_64-posix-seh-gcc-14.2.0-mingw-w64ucrt-12.0.0-r1.zip`

2. Extract to `C:\mingw64`

3. Add to PATH:
   ```cmd
   setx PATH "%PATH%;C:\mingw64\bin"
   ```

4. Build project:
   ```cmd
   rustup default stable-x86_64-pc-windows-gnu
   cargo build --release
   ```

### Option 3: Use WSL (Windows Subsystem for Linux)
**Best for Linux-targeted development**

```bash
wsl --install
wsl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
cargo build --release
```

## Current Configuration

Rust is installed with GNU toolchain active:
```cmd
rustup default stable-x86_64-pc-windows-gnu
```

**Status**: Awaiting linker installation (MSVC or MinGW-w64)

## Testing After Setup

Once linker is installed:

```cmd
cd C:\Users\Chogyam\.zenflow\worktrees\new-task-7d4f

# Check if tests compile
cargo check --tests

# Run all tests
cargo test --all

# Run stdlib tests specifically
cargo test --test stdlib

# Build release binary
cargo build --release
```

## Verification

```cmd
# Verify toolchain
rustup show

# Verify linker
where link.exe     # For MSVC
where gcc.exe      # For MinGW

# Test compilation
cargo build --release
```

## .gitignore Status

`.zenflow/` is properly ignored at line 92 of `.gitignore`:
```gitignore
# Temporary and workflow directories
.zenflow/
-p/
*.tmp
*.temp
*.bak
*.backup
```

## Next Steps

1. Choose and install linker (MSVC or MinGW-w64)
2. Verify compilation: `cargo check --tests`
3. Run test suite: `cargo test --all`
4. Build release: `cargo build --release`

## Standard Library Tests Created

163 tests across 12 modules covering 288 stdlib functions:
- tests/integration/stdlib/core_functions.rs (28 tests)
- tests/integration/stdlib/crypto_functions.rs (14 tests)
- tests/integration/stdlib/encoding_functions.rs (12 tests)
- tests/integration/stdlib/rop_functions.rs (13 tests)
- tests/integration/stdlib/io_functions.rs (13 tests)
- tests/integration/stdlib/heap_functions.rs (12 tests)
- tests/integration/stdlib/kernel_functions.rs (12 tests)
- tests/integration/stdlib/network_functions.rs (11 tests)
- tests/integration/stdlib/web_functions.rs (13 tests)
- tests/integration/stdlib/fuzzing_functions.rs (6 tests)
- tests/integration/stdlib/debugging_functions.rs (13 tests)
- tests/integration/stdlib/exploit_functions.rs (16 tests)

All tests ready to run once linker is configured.
