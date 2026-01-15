# TALON Fuzzing Suite

This directory contains the fuzzing infrastructure for TALON.

## Quick Start

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run all fuzzers (5 minutes each)
../scripts/run_fuzz.sh 300

# Run specific fuzzer
cargo +nightly fuzz run fuzz_parser -- -max_total_time=300
```

## Directory Structure

```
fuzz/
├── Cargo.toml              # Fuzz harness configuration
├── fuzz_targets/           # Fuzz target implementations
│   ├── fuzz_parser.rs
│   ├── fuzz_elf_parser.rs
│   ├── fuzz_pe_parser.rs
│   ├── fuzz_shellcode_generator.rs
│   ├── fuzz_format_string.rs
│   ├── fuzz_heap_tools.rs
│   ├── fuzz_packing_tools.rs
│   ├── fuzz_rop_gadget_finder.rs
│   ├── fuzz_rop_chain_builder.rs
│   └── fuzz_auto_solver.rs
├── corpus/                 # Seed inputs for each target
│   ├── parser/
│   ├── elf_parser/
│   ├── pe_parser/
│   ├── shellcode/
│   ├── format_string/
│   ├── heap_tools/
│   ├── packing_tools/
│   ├── rop_gadget/
│   ├── rop_chain/
│   └── auto_solver/
└── artifacts/              # Crash artifacts (gitignored)
```

## Fuzz Targets

| Target | Component | Priority | Notes |
|--------|-----------|----------|-------|
| `fuzz_parser` | TALON DSL Parser | ⭐⭐⭐⭐⭐ | Critical - test first |
| `fuzz_interpreter` | Script Interpreter | ⭐⭐⭐⭐⭐ | Critical - runtime execution |
| `fuzz_ast` | AST Operations | ⭐⭐⭐⭐ | AST validation/optimization |
| `fuzz_elf_parser` | ELF Binary Analysis | ⭐⭐⭐⭐ | Tests goblin parsing |
| `fuzz_pe_parser` | PE Binary Analysis | ⭐⭐⭐⭐ | Tests pelite parsing |
| `fuzz_shellcode_generator` | Shellcode Library | ⭐⭐⭐⭐ | Multi-arch coverage |
| `fuzz_format_string` | Format String Exploits | ⭐⭐⭐⭐ | Tests payload generation |
| `fuzz_heap_tools` | Heap Exploitation | ⭐⭐⭐⭐ | Tests allocator logic |
| `fuzz_packing_tools` | Packing/Encoding | ⭐⭐⭐ | Tests primitive operations |
| `fuzz_rop_gadget_finder` | ROP Gadget Search | ⭐⭐⭐⭐ | Multi-arch disassembly |
| `fuzz_rop_chain_builder` | ROP Chain Assembly | ⭐⭐⭐ | Tests chain construction |
| `fuzz_auto_solver` | Automated ROP | ⭐⭐⭐⭐ | End-to-end automation |
| `fuzz_exploit_chain` | Exploit Orchestration | ⭐⭐⭐⭐ | Multi-stage chains |
| `fuzz_network_protocol` | Network Protocols | ⭐⭐⭐ | Protocol parsing |
| `fuzz_crypto_tools` | Cryptographic Primitives | ⭐⭐⭐ | Hash/encryption |
| `fuzz_syscall_chain` | Syscall Analysis | ⭐⭐⭐⭐ | SECCOMP bypass |
| `fuzz_disassembler` | Disassembly Engine | ⭐⭐⭐⭐ | Multi-arch CFG |

## Usage Examples

### Run Single Target (5 minutes)

```bash
cargo +nightly fuzz run fuzz_parser -- -max_total_time=300
```

### Run with Multiple Jobs (Parallel)

```bash
cargo +nightly fuzz run fuzz_parser -- -jobs=8 -max_total_time=600
```

### Minimize Corpus

```bash
cargo +nightly fuzz cmin fuzz_parser
```

### Reproduce Crash

```bash
cargo +nightly fuzz run fuzz_parser artifacts/fuzz_parser/crash-abc123
```

### Coverage Report

```bash
cargo +nightly fuzz coverage fuzz_parser
```

## CI/CD Integration

Fuzzing runs automatically via GitHub Actions:
- Daily at 2:00 AM UTC
- 5 minutes per target
- Crash artifacts uploaded for analysis

See `.github/workflows/fuzzing.yml` for details.

## Adding New Fuzz Targets

1. Create new target in `fuzz_targets/`:

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Your fuzzing logic here
    let _ = talon::your_module::your_function(data);
});
```

2. Add to `Cargo.toml`:

```toml
[[bin]]
name = "fuzz_your_target"
path = "fuzz_targets/fuzz_your_target.rs"
test = false
doc = false
```

3. Create corpus directory:

```bash
mkdir -p corpus/your_target
echo "seed_input" > corpus/your_target/seed1.bin
```

4. Test:

```bash
cargo +nightly fuzz run fuzz_your_target -- -max_total_time=60
```

## Performance Tips

- Use `-jobs=N` for parallelization
- Start with short runs (5 min) to catch obvious bugs
- Use `-rss_limit_mb=2048` to reduce memory usage
- Minimize corpus regularly with `cargo fuzz cmin`

## Troubleshooting

**Problem**: `cargo-fuzz` not found  
**Solution**: `cargo install cargo-fuzz && rustup install nightly`

**Problem**: Out of memory  
**Solution**: `cargo +nightly fuzz run <target> -- -rss_limit_mb=1024`

**Problem**: Slow fuzzing (<100 exec/s)  
**Solution**: Check for early-exit logic, reduce input validation overhead

## Full Documentation

See `docs/FUZZING.md` for comprehensive documentation.
