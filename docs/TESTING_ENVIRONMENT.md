# TALON Testing Environment Setup

## Overview
This guide provides comprehensive instructions for setting up testing environments for TALON development, QA validation, and security research. Multiple environment configurations are documented to support different testing scenarios.

**Target Audience**: QA Engineers, Security Researchers, CI/CD Administrators  
**Estimated Setup Time**: 30-90 minutes (depending on environment)

---

## Table of Contents
1. [Core Requirements](#core-requirements)
2. [Development Environment](#development-environment)
3. [Testing Environment](#testing-environment)
4. [Security Testing Environment](#security-testing-environment)
5. [CI/CD Environment](#cicd-environment)
6. [Cross-Platform Environments](#cross-platform-environments)
7. [IDE Integration Environment](#ide-integration-environment)
8. [Troubleshooting](#troubleshooting)

---

## Core Requirements

### Minimum Hardware Requirements
| Resource | Minimum | Recommended | CI/CD |
|----------|---------|-------------|-------|
| **CPU** | 2 cores | 4+ cores | 8+ cores |
| **RAM** | 4 GB | 8 GB | 16 GB |
| **Disk** | 10 GB free | 20 GB free | 50 GB free |
| **OS** | Linux/Windows 10+ | Linux/Windows 11 | Linux containers |

### Software Prerequisites

#### All Platforms
- **Rust**: 1.70.0 or later (stable toolchain)
- **Git**: 2.30.0 or later
- **Text Editor**: VS Code, Vim, or Emacs (optional)

#### Linux-Specific
- **Build Tools**: GCC 9+, Make, pkg-config
- **Libraries**: libssl-dev, build-essential
- **Optional**: Docker, QEMU, GDB, Valgrind

#### Windows-Specific
- **Build Tools**: Visual Studio Build Tools 2019+ or MinGW-w64
- **Optional**: WSL2, Docker Desktop

---

## Development Environment

### Linux (Ubuntu/Debian) Setup
**Duration**: 15-20 minutes

#### Step 1: Install System Dependencies
```bash
# Update package lists
sudo apt-get update

# Install build essentials
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git \
    gcc \
    g++ \
    make \
    cmake

# Install optional tools
sudo apt-get install -y \
    gdb \
    valgrind \
    strace \
    ltrace \
    binutils \
    nasm
```

#### Step 2: Install Rust
```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source environment (or restart terminal)
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version

# Install additional components
rustup component add clippy rustfmt
rustup toolchain install nightly  # For fuzzing
```

#### Step 3: Clone and Build TALON
```bash
# Clone repository
git clone https://github.com/ridpath/talon.git
cd talon

# Build in release mode
cargo build --release

# Verify build
./target/release/talon --version

# Run quick test
cargo test --lib -- --test-threads=1
```

#### Step 4: Install Development Tools
```bash
# Install cargo extensions
cargo install cargo-edit          # cargo add, cargo rm
cargo install cargo-watch          # Auto-rebuild on changes
cargo install cargo-tree           # Dependency visualization
cargo install cargo-audit          # Security vulnerability scanning
cargo install cargo-tarpaulin      # Code coverage (Linux only)
cargo install cargo-criterion      # Benchmarking viewer

# Install fuzzing tools (nightly)
cargo +nightly install cargo-fuzz
```

#### Step 5: Verify Environment
```bash
# Run comprehensive tests
./scripts/verify_environment.sh  # If script exists

# Or manual verification
cargo test --all-features
cargo clippy --all-targets --all-features
cargo fmt --check
cargo audit
```

---

### Linux (Fedora/RHEL/CentOS) Setup
**Duration**: 15-20 minutes

#### Step 1: Install System Dependencies
```bash
# Install development tools
sudo dnf groupinstall "Development Tools"

# Install required libraries
sudo dnf install -y \
    openssl-devel \
    pkg-config \
    cmake \
    git \
    curl

# Install optional tools
sudo dnf install -y \
    gdb \
    valgrind \
    strace \
    binutils \
    nasm
```

#### Step 2-5: Follow Ubuntu Steps 2-5
(Same as Ubuntu after system dependencies installed)

---

### macOS Setup
**Duration**: 20-30 minutes

#### Step 1: Install Xcode Command Line Tools
```bash
# Install Xcode CLI tools
xcode-select --install

# Verify installation
gcc --version
make --version
```

#### Step 2: Install Homebrew (Optional but Recommended)
```bash
# Install Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install useful tools
brew install git
brew install cmake
brew install pkg-config
brew install openssl
```

#### Step 3-5: Follow Ubuntu Steps 2-5
(Rust installation and build process identical)

---

### Windows Setup (Native)
**Duration**: 25-35 minutes

#### Step 1: Install Visual Studio Build Tools

**Option A: Using winget (Windows 10/11)**
```powershell
# Install Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools `
    --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

# Install Git
winget install Git.Git

# Restart terminal to refresh PATH
```

**Option B: Manual Installation**
1. Download VS Build Tools: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
2. Run installer, select "Desktop development with C++"
3. Install Git: https://git-scm.com/download/win
4. Restart terminal

#### Step 2: Install Rust
```powershell
# Download and run rustup-init.exe
# https://rustup.rs

# Or using winget
winget install Rustlang.Rustup

# Restart terminal

# Verify installation
rustc --version
cargo --version

# Install components
rustup component add clippy rustfmt
```

#### Step 3: Clone and Build TALON
```powershell
# Clone repository
git clone https://github.com/ridpath/talon.git
cd talon

# Build in release mode
cargo build --release

# Verify build
.\target\release\talon.exe --version

# Run quick test
cargo test --lib
```

#### Step 4: Install Development Tools
```powershell
# Install cargo extensions
cargo install cargo-edit
cargo install cargo-watch
cargo install cargo-tree
cargo install cargo-audit

# Note: cargo-tarpaulin not available on Windows
# Use Docker or WSL2 for coverage

# Install fuzzing tools (nightly)
rustup toolchain install nightly
cargo +nightly install cargo-fuzz
```

---

### Windows Setup (MinGW Alternative)
**Duration**: 25-35 minutes

#### Step 1: Install MinGW-w64
```powershell
# Download MinGW-w64 from:
# https://github.com/niXman/mingw-builds-binaries/releases

# Extract to C:\mingw64

# Add to PATH
[Environment]::SetEnvironmentVariable(
    "Path",
    "$env:Path;C:\mingw64\bin",
    "User"
)

# Verify
gcc --version
```

#### Step 2: Configure Rust for GNU Toolchain
```powershell
# Install Rust
rustup-init.exe

# Set GNU toolchain as default
rustup default stable-x86_64-pc-windows-gnu

# Verify
rustc --version --verbose
# Should show: host: x86_64-pc-windows-gnu
```

#### Step 3-4: Follow Windows Native Steps 3-4

---

### Windows Setup (WSL2)
**Duration**: 30-40 minutes

#### Step 1: Enable WSL2
```powershell
# Enable WSL (requires admin)
wsl --install

# Or manual:
dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart
dism.exe /online /enable-feature /featurename:VirtualMachinePlatform /all /norestart

# Restart computer

# Set WSL2 as default
wsl --set-default-version 2
```

#### Step 2: Install Linux Distribution
```powershell
# Install Ubuntu 22.04
wsl --install -d Ubuntu-22.04

# Launch
wsl

# Update packages (inside WSL)
sudo apt-get update && sudo apt-get upgrade -y
```

#### Step 3: Follow Linux Setup
Inside WSL2 terminal, follow the Ubuntu setup instructions above.

#### Step 4: VS Code Integration
```powershell
# Install VS Code
winget install Microsoft.VisualStudio.Code

# Install Remote-WSL extension
code --install-extension ms-vscode-remote.remote-wsl

# Open project in WSL
wsl
cd talon
code .
```

---

## Testing Environment

### Automated Testing Setup
**Duration**: 10-15 minutes

#### Test Dependencies
```bash
# Add test dependencies to Cargo.toml
# (These should already be in the project)

# Verify test configuration
cat Cargo.toml | grep -A 10 "\[dev-dependencies\]"

# Expected:
# proptest = "1.0"
# criterion = "0.5"
# mockall = "0.11"
# assert_cmd = "2.0"
# tempfile = "3.5"
```

#### Test Execution Scripts

**Linux/macOS** (`scripts/run_tests.sh`):
```bash
#!/bin/bash
set -e

echo "Running TALON test suite..."

# Unit tests
echo "==> Unit tests"
cargo test --lib --all-features -- --test-threads=1

# Integration tests
echo "==> Integration tests"
cargo test --test '*' --all-features

# Doc tests
echo "==> Doc tests"
cargo test --doc

# Specific test categories
echo "==> Parser tests"
cargo test --test parser_test

echo "==> ROP tests"
cargo test --test rop_test

echo "==> Heap tests"
cargo test --test heap_test

echo "==> Format string tests"
cargo test --test format_string_test

echo "==> Exploit chain tests"
cargo test --test exploit_chain_test

echo "All tests passed!"
```

**Windows** (`scripts\run_tests.ps1`):
```powershell
Write-Host "Running TALON test suite..." -ForegroundColor Green

# Unit tests
Write-Host "==> Unit tests" -ForegroundColor Cyan
cargo test --lib --all-features -- --test-threads=1

# Integration tests
Write-Host "==> Integration tests" -ForegroundColor Cyan
cargo test --test '*' --all-features

# Doc tests
Write-Host "==> Doc tests" -ForegroundColor Cyan
cargo test --doc

Write-Host "All tests passed!" -ForegroundColor Green
```

#### Coverage Setup (Linux Only)
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin \
    --out Html \
    --output-dir coverage \
    --all-features \
    --timeout 300

# View report
firefox coverage/index.html  # or your browser
```

---

### Benchmarking Environment
**Duration**: 10 minutes

#### Setup Benchmarks
```bash
# Verify benchmark configuration
ls benches/

# Expected files:
# - parser_bench.rs
# - interpreter_bench.rs
# - binary_analysis_bench.rs
# - rop_bench.rs

# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench parser

# Save baseline
cargo bench -- --save-baseline main

# Compare with baseline later
cargo bench -- --baseline main
```

#### Benchmark Results Storage
```bash
# Results stored in target/criterion/
ls target/criterion/

# Each benchmark has:
# - report/index.html (visual report)
# - base/estimates.json (data)
# - change/estimates.json (comparison if baseline exists)
```

---

### Fuzzing Environment
**Duration**: 15-20 minutes

#### Setup Fuzzing
```bash
# Install nightly toolchain
rustup toolchain install nightly

# Install cargo-fuzz
cargo +nightly install cargo-fuzz

# Verify fuzz targets
ls fuzz/fuzz_targets/

# Expected targets:
# - fuzz_parser.rs
# - fuzz_elf_parser.rs
# - fuzz_pe_parser.rs
# - fuzz_shellcode.rs
# - fuzz_format_string.rs
# - fuzz_heap_tools.rs
# - fuzz_rop_finder.rs
# - fuzz_packing.rs
# - fuzz_interpreter.rs
# - fuzz_exploit_chain.rs
```

#### Run Fuzzing Campaign
```bash
# Quick fuzz (5 minutes per target)
./scripts/run_fuzz.sh 300

# Or manual:
cargo +nightly fuzz run fuzz_parser -- -max_total_time=300

# Run with corpus
cargo +nightly fuzz run fuzz_elf_parser fuzz/corpus/elf/

# Check for crashes
ls fuzz/artifacts/fuzz_parser/

# Reproduce crash
cargo +nightly fuzz run fuzz_parser fuzz/artifacts/fuzz_parser/crash-...
```

---

## Security Testing Environment

### Sandboxed Environment Setup
**Duration**: 20-30 minutes

#### Docker Container Environment
```bash
# Pull base image
docker pull ubuntu:22.04

# Create Dockerfile for testing
cat > Dockerfile.test <<EOF
FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    gdb \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:\${PATH}"

# Working directory
WORKDIR /talon

# Default command
CMD ["/bin/bash"]
EOF

# Build testing image
docker build -f Dockerfile.test -t talon-test:latest .

# Run tests in container
docker run -it --rm -v $(pwd):/talon talon-test:latest bash -c "
    cd /talon
    cargo build --release
    cargo test --all-features
"
```

#### Isolated Network Testing
```bash
# Create isolated network
docker network create --driver bridge talon-test-net

# Run test server
docker run -d \
    --name test-server \
    --network talon-test-net \
    -p 8888:8888 \
    alpine:latest \
    nc -l -p 8888

# Run TALON tests
docker run -it --rm \
    --network talon-test-net \
    -v $(pwd):/talon \
    talon-test:latest \
    bash -c "cd /talon && cargo test --test network_test"

# Cleanup
docker stop test-server
docker network rm talon-test-net
```

---

### Vulnerable Test Binary Environment
**Duration**: 15 minutes

#### Create Test Binaries
```bash
# Create test directory
mkdir -p tests/binaries
cd tests/binaries

# Buffer overflow test binary
cat > buffer_overflow.c <<EOF
#include <stdio.h>
#include <string.h>

void vulnerable_function(char *input) {
    char buffer[64];
    strcpy(buffer, input);  // Vulnerable!
    printf("Input: %s\n", buffer);
}

int main(int argc, char **argv) {
    if (argc < 2) {
        printf("Usage: %s <input>\n", argv[0]);
        return 1;
    }
    vulnerable_function(argv[1]);
    return 0;
}
EOF

# Compile with vulnerabilities enabled
gcc buffer_overflow.c -o buffer_overflow \
    -fno-stack-protector \
    -z execstack \
    -no-pie

# Format string test binary
cat > format_string.c <<EOF
#include <stdio.h>

int main(int argc, char **argv) {
    if (argc < 2) {
        printf("Usage: %s <input>\n", argv[0]);
        return 1;
    }
    printf(argv[1]);  // Vulnerable!
    printf("\n");
    return 0;
}
EOF

# Compile
gcc format_string.c -o format_string \
    -fno-stack-protector \
    -no-pie

# Heap UAF test binary
cat > heap_uaf.c <<EOF
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char data[32];
    void (*callback)(char *);
} Object;

void print_data(char *data) {
    printf("Data: %s\n", data);
}

int main() {
    Object *obj = malloc(sizeof(Object));
    obj->callback = print_data;
    strcpy(obj->data, "Hello");
    
    free(obj);  // Free object
    
    // Use after free!
    obj->callback(obj->data);
    
    return 0;
}
EOF

# Compile
gcc heap_uaf.c -o heap_uaf -fno-stack-protector

cd ../..
```

#### Verify Test Binaries
```bash
# Check security features
checksec tests/binaries/buffer_overflow
# Expected: No PIE, No Canary, NX disabled

# Test ROP gadget finding
talon analyze tests/binaries/buffer_overflow

# Test exploit generation
talon run examples/01_buffer_overflow_rop.talon
```

---

## CI/CD Environment

### GitHub Actions Setup
**Duration**: 10 minutes

#### Local Act Setup (Optional - Test CI Locally)
```bash
# Install act (GitHub Actions local runner)
# Linux
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash

# macOS
brew install act

# Windows
choco install act-cli

# Test workflow locally
act -l  # List workflows
act -j test  # Run 'test' job
act push  # Simulate push event
```

#### Verify CI Configuration
```bash
# Check workflow files
ls .github/workflows/

# Expected files:
# - ci.yml (main CI)
# - security.yml (security audit)
# - fuzzing.yml (fuzzing campaign)
# - benchmarks.yml (performance)

# Validate syntax
# (requires actionlint tool)
actionlint .github/workflows/*.yml
```

---

## Cross-Platform Environments

### Multi-Platform Testing Matrix
**Duration**: Variable (depends on platform availability)

#### Linux Matrix
```bash
# Ubuntu 20.04 (LTS)
docker run -it --rm -v $(pwd):/talon ubuntu:20.04 bash
apt-get update && apt-get install -y build-essential curl
# ... install Rust and build

# Ubuntu 22.04 (LTS)
docker run -it --rm -v $(pwd):/talon ubuntu:22.04 bash
# ... same as above

# Ubuntu 24.04 (Latest LTS)
docker run -it --rm -v $(pwd):/talon ubuntu:24.04 bash
# ... same as above

# Fedora
docker run -it --rm -v $(pwd):/talon fedora:latest bash
dnf groupinstall "Development Tools" -y
# ... install Rust and build
```

#### Windows Matrix
```powershell
# Windows 10
# (Use physical/VM machine)

# Windows 11
# (Use physical/VM machine)

# Windows Server 2019
# (For CI/CD)

# Windows Server 2022
# (For CI/CD)
```

---

## IDE Integration Environment

### VS Code Extension Development
**Duration**: 20 minutes

#### Setup Extension Development
```bash
# Navigate to extension directory
cd vscode-extension

# Install Node.js dependencies
npm install

# Compile TypeScript
npm run compile

# Watch for changes (during development)
npm run watch
```

#### Test Extension Locally
```bash
# Method 1: Launch from VS Code
# - Open vscode-extension folder in VS Code
# - Press F5 (Start Debugging)
# - New VS Code window opens with extension loaded

# Method 2: Package and install
npm run package
code --install-extension talon-vscode-0.1.0.vsix

# Verify installation
code --list-extensions | grep talon
```

#### LSP Server Testing
```bash
# Build LSP server
cd ..  # Back to project root
cargo build --release --bin talon-lsp

# Verify binary
./target/release/talon-lsp --version

# Test LSP manually (requires LSP client)
# Use VS Code extension for integration testing
```

---

### Vim/Neovim Setup (Optional)
**Duration**: 10 minutes

#### Install Syntax Highlighting
```bash
# Create syntax directory
mkdir -p ~/.vim/syntax
mkdir -p ~/.vim/ftdetect

# Copy syntax files
cp syntax/talon.vim ~/.vim/syntax/
cp syntax/ftdetect/talon.vim ~/.vim/ftdetect/

# Or for Neovim
mkdir -p ~/.config/nvim/syntax
mkdir -p ~/.config/nvim/ftdetect
cp syntax/talon.vim ~/.config/nvim/syntax/
cp syntax/ftdetect/talon.vim ~/.config/nvim/ftdetect/

# Test
vim test.talon
```

---

## Troubleshooting

### Common Environment Issues

#### Issue: Cargo not found after Rust installation
**Symptoms**: `cargo: command not found`

**Solutions**:
```bash
# Linux/macOS
source $HOME/.cargo/env
# Or add to ~/.bashrc or ~/.zshrc:
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc

# Windows (PowerShell)
# Restart terminal or manually add to PATH:
# C:\Users\<username>\.cargo\bin
```

#### Issue: Linker errors during build
**Symptoms**: `error: linker 'cc' not found`

**Solutions**:
```bash
# Linux (Ubuntu/Debian)
sudo apt-get install build-essential

# Linux (Fedora)
sudo dnf groupinstall "Development Tools"

# macOS
xcode-select --install

# Windows
# Install Visual Studio Build Tools
# Or use MinGW
```

#### Issue: OpenSSL not found
**Symptoms**: `Could not find directory of OpenSSL installation`

**Solutions**:
```bash
# Linux (Ubuntu/Debian)
sudo apt-get install libssl-dev pkg-config

# Linux (Fedora)
sudo dnf install openssl-devel pkg-config

# macOS
brew install openssl
# May need to set environment variable:
export OPENSSL_DIR=$(brew --prefix openssl)

# Windows
# Should work with vcpkg integration in VS Build Tools
# Or set OPENSSL_DIR environment variable
```

#### Issue: Tests fail with timeout
**Symptoms**: `test ... has been running for over 60 seconds`

**Solutions**:
```bash
# Increase timeout
cargo test -- --test-threads=1 --nocapture

# Or run specific test
cargo test slow_test_name -- --exact --ignored

# Check system resources (CPU, RAM)
```

#### Issue: Fuzzing fails to start
**Symptoms**: `cargo fuzz: command not found`

**Solutions**:
```bash
# Ensure nightly toolchain installed
rustup toolchain install nightly

# Install cargo-fuzz with nightly
cargo +nightly install cargo-fuzz

# Run with nightly explicitly
cargo +nightly fuzz run fuzz_parser
```

#### Issue: Docker permissions error (Linux)
**Symptoms**: `permission denied while trying to connect to Docker daemon`

**Solutions**:
```bash
# Add user to docker group
sudo usermod -aG docker $USER

# Log out and back in, or:
newgrp docker

# Or use sudo (not recommended for regular use)
sudo docker run ...
```

#### Issue: VS Code extension won't activate
**Symptoms**: Extension installed but not active on `.talon` files

**Solutions**:
1. Check extension is enabled (Extensions panel)
2. Verify file association: Open `.talon` file and check language mode (bottom-right)
3. Reload window: `Ctrl+Shift+P` → "Reload Window"
4. Check extension logs: `Ctrl+Shift+P` → "Developer: Show Logs"
5. Rebuild extension:
   ```bash
   cd vscode-extension
   npm run compile
   ```

---

## Environment Verification Checklist

### Development Environment
- [ ] Rust toolchain installed (stable 1.70+)
- [ ] Nightly toolchain installed (for fuzzing)
- [ ] Build tools installed (gcc/MSVC)
- [ ] TALON builds successfully
- [ ] Basic tests pass
- [ ] Clippy and rustfmt work

### Testing Environment
- [ ] Unit tests execute
- [ ] Integration tests execute
- [ ] Benchmarks run
- [ ] Fuzzing works (nightly)
- [ ] Coverage reports generate (Linux)

### Security Environment
- [ ] Docker installed and working
- [ ] Test binaries compile
- [ ] Sandboxed execution works
- [ ] Network isolation functional

### IDE Environment
- [ ] VS Code extension compiles
- [ ] Extension installs successfully
- [ ] LSP features work (highlighting, autocomplete)
- [ ] Debugger attaches

### CI/CD Environment
- [ ] GitHub Actions workflows validated
- [ ] Act (local testing) works (optional)
- [ ] Artifacts generate correctly

---

## Next Steps

After environment setup:
1. Read [`TESTING.md`](TESTING.md) for testing guidelines
2. Review [`MANUAL_TESTING.md`](MANUAL_TESTING.md) for manual test procedures
3. Check [`QA_CHECKLIST.md`](QA_CHECKLIST.md) for release validation
4. See [`CONTRIBUTING.md`](../CONTRIBUTING.md) for development workflow

---

## Environment Maintenance

### Regular Updates
```bash
# Update Rust toolchain
rustup update

# Update cargo tools
cargo install-update -a  # Requires cargo-update

# Update dependencies
cargo update

# Audit for vulnerabilities
cargo audit

# Clean build artifacts
cargo clean
```

### Periodic Checks
- **Weekly**: `cargo audit` for security vulnerabilities
- **Monthly**: Update Rust toolchain and dev dependencies
- **Before Release**: Full environment verification from scratch

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-15  
**Maintained By**: TALON Development Team
