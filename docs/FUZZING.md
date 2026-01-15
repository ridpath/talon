# TALON Fuzzing Infrastructure

## Overview

TALON includes comprehensive fuzzing infrastructure using **cargo-fuzz** (libFuzzer) to ensure robustness and security. The fuzzing suite targets critical components including parsers, binary analysis tools, exploit primitives, and encoding/packing utilities.

## Prerequisites

### Install Rust Nightly

```bash
# Install nightly toolchain
rustup install nightly

# Set nightly as default (optional)
rustup default nightly
```

### Install cargo-fuzz

```bash
cargo install cargo-fuzz
```

## Fuzz Targets

TALON includes 17 comprehensive fuzz targets covering all critical subsystems:

### 1. **fuzz_parser** (Critical)
- **Component**: TALON DSL parser
- **Purpose**: Ensures parser handles malformed/edge-case TALON scripts without crashes
- **Coverage**: Syntax parsing, AST construction, error handling
- **Priority**: ⭐⭐⭐⭐⭐ (Highest)

### 2. **fuzz_elf_parser**
- **Component**: ELF binary analysis (goblin-based)
- **Purpose**: Validates robustness against malformed/malicious ELF binaries
- **Coverage**: Symbol resolution, section parsing, PLT/GOT extraction
- **Priority**: ⭐⭐⭐⭐

### 3. **fuzz_pe_parser**
- **Component**: PE binary analysis (pelite-based)
- **Purpose**: Ensures safe parsing of Windows executables
- **Coverage**: Export/import tables, sections, resources
- **Priority**: ⭐⭐⭐⭐

### 4. **fuzz_shellcode_generator**
- **Component**: Shellcode generation and encoding
- **Purpose**: Tests shellcode library for all architectures and types
- **Coverage**: x86/x64/ARM/ARM64, XOR/alphanumeric encoding
- **Priority**: ⭐⭐⭐⭐

### 5. **fuzz_format_string**
- **Component**: Format string exploit builder
- **Purpose**: Validates format string parsing and payload generation
- **Coverage**: Leak/write primitives, offset calculation
- **Priority**: ⭐⭐⭐⭐

### 6. **fuzz_heap_tools**
- **Component**: Heap exploitation utilities
- **Purpose**: Tests heap analysis for various allocators
- **Coverage**: Tcache/fastbin/unsorted bin, chunk metadata validation
- **Priority**: ⭐⭐⭐⭐

### 7. **fuzz_packing_tools**
- **Component**: Packing and encoding primitives
- **Purpose**: Validates pack/unpack operations and encoding
- **Coverage**: p8/p16/p32/p64, base64, hex, URL encoding
- **Priority**: ⭐⭐⭐

### 8. **fuzz_rop_gadget_finder**
- **Component**: ROP gadget discovery
- **Purpose**: Tests gadget finding for multiple architectures
- **Coverage**: x86/x64/ARM/ARM64 disassembly and pattern matching
- **Priority**: ⭐⭐⭐⭐

### 9. **fuzz_rop_chain_builder**
- **Component**: ROP chain construction
- **Purpose**: Validates chain building logic
- **Coverage**: Gadget chaining, constraint solving
- **Priority**: ⭐⭐⭐

### 10. **fuzz_auto_solver**
- **Component**: Automated ROP solver
- **Purpose**: Tests end-to-end ROP automation
- **Coverage**: Strategy selection, constraint handling, goal solving
- **Priority**: ⭐⭐⭐⭐

### 11. **fuzz_interpreter**
- **Component**: TALON script interpreter
- **Purpose**: Tests runtime execution and evaluation
- **Coverage**: Variable binding, function calls, control flow, timeout handling
- **Priority**: ⭐⭐⭐⭐⭐ (Critical)

### 12. **fuzz_ast**
- **Component**: Abstract Syntax Tree operations
- **Purpose**: Tests AST construction, serialization, optimization
- **Coverage**: AST validation, type checking, JSON serialization
- **Priority**: ⭐⭐⭐⭐

### 13. **fuzz_exploit_chain**
- **Component**: Multi-stage exploit orchestration
- **Purpose**: Tests exploit chaining framework
- **Coverage**: Stage dependencies, checkpoints, dry-run validation
- **Priority**: ⭐⭐⭐⭐

### 14. **fuzz_network_protocol**
- **Component**: Network protocol handlers
- **Purpose**: Tests protocol parsing and encoding
- **Coverage**: TCP, UDP, HTTP, WebSocket, packet analysis
- **Priority**: ⭐⭐⭐

### 15. **fuzz_crypto_tools**
- **Component**: Cryptographic primitives
- **Purpose**: Tests hashing and encryption functions
- **Coverage**: SHA256, MD5, AES, XOR, base64, hex encoding
- **Priority**: ⭐⭐⭐

### 16. **fuzz_syscall_chain**
- **Component**: Syscall chain analysis
- **Purpose**: Tests syscall validation and ROP generation
- **Coverage**: Syscall chains, SECCOMP bypass detection
- **Priority**: ⭐⭐⭐⭐

### 17. **fuzz_disassembler**
- **Component**: Multi-architecture disassembly
- **Purpose**: Tests disassembly engine
- **Coverage**: x86/x64/ARM/ARM64, function detection, CFG analysis
- **Priority**: ⭐⭐⭐⭐

## Running Fuzzers

### Quick Start (5 minutes)

```bash
# Run all fuzzers for 5 minutes each
./scripts/run_fuzz.sh 300

# Or on Windows
.\scripts\run_fuzz.ps1 -Duration 300
```

### Run Specific Target

```bash
# Fuzz the parser for 10 minutes
./scripts/fuzz_single.sh fuzz_parser 600

# Or using cargo-fuzz directly
cargo +nightly fuzz run fuzz_parser -- -max_total_time=600
```

### Run Single Target (Windows)

```powershell
.\scripts\run_fuzz.ps1 -Duration 600 -Target fuzz_parser
```

### Extended Fuzzing (Recommended for CI/CD)

```bash
# Run all targets for 1 hour each
./scripts/run_fuzz.sh 3600

# Overnight fuzzing (8 hours per target)
./scripts/run_fuzz.sh 28800
```

## Continuous Fuzzing (CI/CD)

### GitHub Actions

The fuzzing workflow runs automatically:
- **Schedule**: Daily at 2 AM UTC
- **Duration**: 5 minutes per target
- **Trigger**: Manual via workflow_dispatch

### Trigger Manual Fuzzing

```bash
# Via GitHub CLI
gh workflow run fuzzing.yml -f duration=600 -f target=fuzz_parser

# Via GitHub UI
# Actions → Continuous Fuzzing → Run workflow
```

## Analyzing Results

### Check for Crashes

```bash
# List crash artifacts
ls -lh fuzz/artifacts/

# View crash details
hexdump -C fuzz/artifacts/fuzz_parser/crash-abc123 | head -n 30
```

### Reproduce Crashes

```bash
# Reproduce a specific crash
cargo +nightly fuzz run fuzz_parser fuzz/artifacts/fuzz_parser/crash-abc123

# Minimize crash input
cargo +nightly fuzz cmin fuzz_parser
```

### Minimize Crash Corpus

```bash
./scripts/minimize_crash.sh fuzz_parser fuzz/artifacts/fuzz_parser/crash-abc123
```

## Corpus Management

### Seed Corpus

Initial corpus files are stored in `fuzz/corpus/<target>/`:
- `parser/`: Valid TALON scripts, ROP exploits, complex syntax
- `elf_parser/`: Minimal ELF binaries, section variations
- `pe_parser/`: Windows PE samples
- `format_string/`: Format string patterns
- `shellcode/`: Architecture-specific shellcode
- `heap_tools/`: Chunk metadata patterns
- `packing_tools/`: Packed binary data

### Add New Corpus Files

```bash
# Add a new test case to parser corpus
echo 'let x = [1, 2, 3]' > fuzz/corpus/parser/array_test.talon

# Add malformed ELF
cp suspicious.elf fuzz/corpus/elf_parser/
```

### Export Corpus

```bash
# Export minimized corpus for archival
cargo +nightly fuzz cmin fuzz_parser
tar -czf parser_corpus_$(date +%Y%m%d).tar.gz fuzz/corpus/parser/
```

## Performance Tuning

### Increase Fuzzing Throughput

```bash
# Use multiple jobs (parallelization)
cargo +nightly fuzz run fuzz_parser -- -jobs=8 -max_total_time=3600

# Reduce memory limit for faster restarts
cargo +nightly fuzz run fuzz_parser -- -rss_limit_mb=2048
```

### Coverage-Guided Fuzzing

```bash
# Enable coverage tracking (slower but more thorough)
cargo +nightly fuzz run fuzz_parser -- -use_value_profile=1
```

## Integration with Other Tools

### AFL++ Integration (Advanced)

For even more powerful fuzzing:

```bash
# Build AFL++ compatible binary
cargo afl build --release

# Run with AFL++
cargo afl fuzz -i fuzz/corpus/parser/ -o fuzz/afl_output/ target/release/talon_afl_parser
```

### Honggfuzz (Alternative Fuzzer)

```bash
# Install honggfuzz
cargo install honggfuzz

# Create honggfuzz target (manual setup required)
# See: https://github.com/rust-fuzz/honggfuzz-rs
```

## Debugging Crashes

### Enable Debug Symbols

```bash
# Build with debug symbols
cargo +nightly fuzz build fuzz_parser --dev

# Run with backtrace
RUST_BACKTRACE=full cargo +nightly fuzz run fuzz_parser crash_file
```

### Use AddressSanitizer

Fuzzing automatically uses AddressSanitizer (ASan) to detect:
- Buffer overflows
- Use-after-free
- Memory leaks
- Double-free
- Stack/heap overflow

### Reproduce in Debugger

```bash
# Build debug binary
cargo +nightly build --bin talon --features test-utils

# Run under GDB
gdb --args target/debug/talon < crash_input.bin
```

## Best Practices

### 1. **Start with Critical Components**
- Prioritize `fuzz_parser` (most critical)
- Then `fuzz_elf_parser`, `fuzz_shellcode_generator`

### 2. **Incremental Fuzzing**
- Run short sessions first (5 min) to catch obvious bugs
- Increase duration for deeper testing (1+ hours)

### 3. **Corpus Maintenance**
- Regularly minimize corpus with `cargo fuzz cmin`
- Archive interesting corpus for regression testing

### 4. **CI/CD Integration**
- Run quick fuzzing (5 min) on every PR
- Extended fuzzing (1 hour) on nightly builds
- Archive crash artifacts for analysis

### 5. **Crash Triage**
- Fix parser crashes immediately (critical)
- Document binary parser crashes (may be malformed input)
- Test fixes by adding crash input to regression corpus

## Troubleshooting

### "cargo-fuzz not found"

```bash
cargo install cargo-fuzz
rustup install nightly
```

### "linker error" or "sanitizer not supported"

```bash
# Ensure nightly toolchain is installed
rustup install nightly

# Update LLVM/Clang (Linux)
sudo apt-get install llvm clang

# macOS
brew install llvm
```

### "Out of memory"

```bash
# Reduce RSS limit
cargo +nightly fuzz run <target> -- -rss_limit_mb=1024
```

### "Too slow / No coverage increase"

```bash
# Use multiple cores
cargo +nightly fuzz run <target> -- -jobs=8

# Reduce complexity of input validation
# (Check if target has early-exit for invalid inputs)
```

## Advanced Configuration

### Custom libFuzzer Options

```bash
# Dictionary-based fuzzing
cargo +nightly fuzz run fuzz_parser -- -dict=fuzz/dict/parser.dict

# Energy schedule (exploration vs exploitation)
cargo +nightly fuzz run fuzz_parser -- -energy=1

# Reduce noise in logs
cargo +nightly fuzz run fuzz_parser -- -print_final_stats=1 -print_pcs=0
```

### Structured Fuzzing

For complex inputs, consider using `arbitrary` crate:

```rust
use arbitrary::Arbitrary;

#[derive(Arbitrary, Debug)]
struct StructuredInput {
    command: CommandType,
    offset: u64,
    data: Vec<u8>,
}

fuzz_target!(|input: StructuredInput| {
    // Use structured input
});
```

## Metrics and Reporting

### Coverage Metrics

```bash
# Generate coverage report (requires source-based coverage)
cargo +nightly fuzz coverage fuzz_parser
```

### Fuzzing Statistics

libFuzzer reports:
- **exec/s**: Executions per second (throughput)
- **cov**: Total edge coverage
- **corp**: Corpus size
- **units**: Total inputs executed

**Target**: >1000 exec/s for optimal fuzzing

## Security Considerations

### Fuzzing in Production

- **Never** run fuzzers on production systems
- Use isolated containers/VMs for fuzzing
- Monitor resource usage (CPU/RAM/disk)

### Handling Sensitive Crash Data

- Crash files may contain sensitive payloads
- Review before committing to version control
- Use `.gitignore` to exclude `fuzz/artifacts/`

## Resources

- [libFuzzer Documentation](https://llvm.org/docs/LibFuzzer.html)
- [cargo-fuzz Book](https://rust-fuzz.github.io/book/cargo-fuzz.html)
- [Fuzzing Best Practices](https://google.github.io/fuzzing/docs/)
- [AFL++ Documentation](https://aflplus.plus/)

## Support

For fuzzing-related issues:
1. Check existing crash artifacts
2. Review fuzzing logs in CI/CD
3. Open an issue with crash reproduction steps
4. Include crash file and target name
