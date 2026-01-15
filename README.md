<!--
TALON DSL: research-oriented exploit development language,
binary analysis scripting, reproducible exploitation framework,
compiled DSL for offensive security research, reverse engineering automation,
kernel exploit prototyping, smart contract auditing language
-->
<!-- TALON scripting language, TALON DSL, offensive security scripting language, exploit development language, exploit centric DSL, security native programming language, hacking DSL, red team scripting language, CTF scripting language, exploit automation language, binary exploitation DSL, pwn scripting language, reverse engineering DSL, exploit compiler, exploit interpreter, domain specific language for hacking, English like exploit language, human readable exploit scripting, native exploit compilation, Rust LLVM exploit compiler, LLVM based exploit language, native code exploit generation, shellcode generation language, ROP scripting language, return oriented programming DSL, heap exploitation language, kernel exploitation scripting, Windows exploitation DSL, Linux exploitation DSL, format string exploitation language, automatic offset discovery, GDB assisted exploitation, libc database integration, libc.rip integration, exploit template engine, interactive exploit helpers, quick exploit prototyping, CTF automation tooling, red team lab tooling, adversarial research language, security research DSL, exploit research platform, binary analysis tooling, ELF analysis, PE analysis, Mach-O analysis, integrated disassembler helpers, fuzzing language, binary fuzzing DSL, kernel fuzzing research, exploit fuzzing automation, vulnerability research language, exploit chain modeling, post exploitation scripting, lateral movement research tooling, privilege escalation research, kernel CVE exploitation framework, exploit proof of concept language, exploit PoC automation, exploit scaffolding language, modular standard library, security focused standard library, plugin based exploit framework, extensible exploit language, native Rust plugin system, dynamic module loading, exploit module framework, IDE assisted exploitation, VS Code exploit extension, exploit debugger integration, visual exploit tooling, interactive REPL exploitation, exploit REPL environment, command line exploit tooling, cross platform exploit language, Windows Linux exploit tooling, offensive security compiler, red team engineering toolkit, exploit engineering platform, CTF competition tooling, alpha stage exploit language, experimental exploit DSL, research only offensive tooling, ethical security research language -->

# TALON - Scripting Language for Offensive Security
<img src="talon.png" alt="Talon scripting language logo" width="50%">


TALON is a security native, exploit centric, English like domain specific language designed for exploit developers, CTF competitors, red teamers, reverse engineers, and security researchers.

This repository contains the TALON compiler, interpreter, standard library, exploit tooling, plugin system, and IDE integrations.

> **ALPHA NOTICE**: TALON is under **active development**. Expect breaking changes, experimental syntax, and rapid iteration. Not yet production-safe. Ideal for CTF competitions, red team labs, research, or prototyping offensive techniques.

[![CI](https://github.com/ridpath/talon/workflows/CI/badge.svg)](https://github.com/ridpath/talon/actions/workflows/ci.yml)
[![Security Audit](https://github.com/ridpath/talon/workflows/Security%20Audit/badge.svg)](https://github.com/ridpath/talon/actions/workflows/security.yml)
[![codecov](https://codecov.io/gh/ridpath/talon/branch/main/graph/badge.svg)](https://codecov.io/gh/ridpath/talon)
![status: alpha](https://img.shields.io/badge/status-alpha-yellow)
![platform: windows+linux](https://img.shields.io/badge/platform-windows%20%7C%20linux-blue)
---

## Key Capabilities

- Human-readable exploit DSL
- Native compilation via Rust/LLVM
- Built-in exploitation primitives (ROP, heap, kernel, format strings)
- Libc database integration (libc.rip)
- Automatic buffer overflow offset discovery (GDB)
- Exploit templates and interactive quick helpers
- Integrated binary analysis (ELF, PE, Mach-O)
- Modular standard library (138+ modules)
- Plugin system (TALON modules + native Rust plugins)
- VS Code IDE extension with debugger and visual tools

---

## Installation

### Prerequisites

#### Linux (Ubuntu/Debian)
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install build essentials (required for linking)
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev
```

#### Linux (Fedora/RHEL/CentOS)
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install development tools
sudo dnf groupinstall "Development Tools"
sudo dnf install openssl-devel pkg-config
```

#### macOS
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Xcode Command Line Tools (if not already installed)
xcode-select --install
```

#### Windows

**Option 1: Using winget (Windows 10/11)**
```powershell
# Install Rust
winget install Rustlang.Rustup

# Install Visual Studio Build Tools (required for linking)
winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

# Restart terminal to refresh PATH
```

**Option 2: Manual Installation**
1. Download and install Rust from https://rustup.rs
2. Download and install Visual Studio Build Tools: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
   - Select "Desktop development with C++" workload
3. Restart your terminal

**Option 3: Using MinGW (alternative to MSVC)**
```powershell
# Download MinGW-w64 from: https://github.com/niXman/mingw-builds-binaries/releases
# Extract to C:\mingw64 and add C:\mingw64\bin to PATH

# Then configure Rust to use GNU toolchain:
rustup default stable-x86_64-pc-windows-gnu
```

### Build from Source

#### Linux/macOS
```bash
git clone https://github.com/ridpath/talon.git
cd talon
cargo build --release
./target/release/talon --version
./target/release/talon repl
```

#### Windows (PowerShell)
```powershell
git clone https://github.com/ridpath/talon.git
cd talon
cargo build --release
.\target\release\talon.exe --version
.\target\release\talon.exe repl
```

#### Windows (CMD)
```cmd
git clone https://github.com/ridpath/talon.git
cd talon
cargo build --release
target\release\talon.exe --version
target\release\talon.exe repl
```

### Quick Install (All Platforms)

After installing prerequisites above:
```bash
# Linux/macOS
cargo install --path .

# Windows
cargo install --path .

# Then run from anywhere:
talon --version
talon repl
```

---

## Core CLI Commands

```bash
talon run <script.talon>
talon build <script.talon>
talon repl
talon analyze <binary>
talon fuzz <binary>
talon kernel exploit <CVE-ID>
talon audit <contract.sol>
talon web scan <url>
talon template <name> <args>
```

---

## Interactive Helpers

```talon
quick_shell("host", port)
quick_rop("./binary")
quick_pwn("./binary", "host", port)
quick_heap()
quick_fmt()
```

---

## Plugin System

### Custom TALON Modules

```talon
define function my_helper(arg)
    print(arg)
end
```

### Native Rust Plugins

```talon
load_plugin("plugins/my_plugin.so")
```

---

## VS Code Extension

See `README.md in vscode-extensions directory` for IDE features, debugging, and visual tools.

---

## Testing & Quality Assurance

### Fuzzing

TALON includes comprehensive fuzzing infrastructure using cargo-fuzz (libFuzzer):

```bash
# Quick fuzzing (5 minutes per target)
./scripts/run_fuzz.sh 300

# Run specific target
cargo +nightly fuzz run fuzz_parser -- -max_total_time=600

# Windows
.\scripts\run_fuzz.ps1 -Duration 300
```

**Fuzz Targets**:
- Parser (TALON DSL)
- ELF/PE binary parsers
- Shellcode generator
- Format string exploits
- Heap tools
- ROP gadget finder
- Packing/encoding tools

See `docs/FUZZING.md` for complete documentation.

### Running Tests

```bash
# Run all tests
cargo test --all-features

# Run specific test suite
cargo test --test parser_test
cargo test --test stdlib_test

# With coverage
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --all-features
```

### Continuous Integration

All commits are tested via GitHub Actions:
- Build on Linux + Windows
- Full test suite
- Security audit (cargo-audit)
- Daily fuzzing campaigns

---

## License

MIT License

## Built-in Functions Reference

A short quick-reference is included in this repo:

- `BUILTIN_FUNCTIONS_REFERENCE.md` - Quick reference for core language helpers (collections, conversion, file I/O, packing/unpacking, and common utilities)

For the full catalog (exploitation, analysis, fuzzing, kernel, web, blockchain, forensics, templates, and helpers), see the "Built-in Functions" section in this README and the in-tool docs:

- `talon repl` then `help()` or `help(search: "keyword")`
- `man talon` and `man talon-<topic>`


<!--
TALON DSL: CTF exploitation language, competitive hacking framework,
ret2libc automation, format string exploitation, heap exploitation toolkit,
GDB-assisted offset discovery, libc identification via leaks,
CTFd and Hack The Box workflow tooling, exploit templates for competitions
-->
