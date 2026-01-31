# TALON Testing Guide

This document provides comprehensive guidance for testing the TALON scripting language project.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Test Organization](#test-organization)
- [Running Tests](#running-tests)
- [Writing Tests](#writing-tests)
- [Code Coverage](#code-coverage)
- [Fuzzing](#fuzzing)
- [Benchmarking](#benchmarking)
- [Continuous Integration](#continuous-integration)
- [Troubleshooting](#troubleshooting)

---

## Overview

TALON employs a comprehensive testing strategy encompassing:

- **Unit Tests**: Testing individual functions and modules in isolation
- **Integration Tests**: Testing component interactions and end-to-end workflows
- **Fuzz Tests**: Discovering edge cases and vulnerabilities through automated input generation
- **Benchmarks**: Measuring performance and preventing regressions
- **Doc Tests**: Ensuring documentation examples remain correct

### Testing Philosophy

1. **Zero-Error Tolerance**: All code must compile and pass tests before merging
2. **Security-First**: Every security-critical function must have comprehensive tests
3. **Performance Awareness**: Benchmarks guard against performance regressions
4. **Coverage Goals**: Maintain >80% line coverage, >90% for security-critical modules

---

## Quick Start

```bash
# Run all tests
cargo test --all-features

# Run with output
cargo test --all-features -- --nocapture

# Run specific test
cargo test test_name

# Run all tests in a file
cargo test --test parser_test

# Run tests matching a pattern
cargo test rop_

# Run with multiple threads
cargo test --all-features -- --test-threads=4
```

### Windows-Specific

```powershell
# PowerShell
cargo test --all-features

# Command Prompt
cargo test --all-features
```

---

## Test Organization

```
tests/
├── common/                   # Shared test utilities
│   └── mod.rs               # TalonTestHarness, MockBinary, assertions
├── fixtures/                # Test data and binaries
│   ├── binaries/           # ELF/PE test files
│   ├── data/               # Input/output test data
│   ├── exploits/           # Exploit test cases
│   └── scripts/            # TALON script fixtures
├── unit/                    # Unit tests
│   ├── interpreter/        # Interpreter module tests
│   ├── parser_test.rs      # Parser tests
│   ├── ast_test.rs         # AST tests
│   ├── rop_test.rs         # ROP tools tests
│   ├── heap_test.rs        # Heap exploitation tests
│   ├── shellcode_test.rs   # Shellcode generation tests
│   ├── binary_analysis_test.rs
│   ├── format_string_test.rs
│   ├── lsp_test.rs
│   ├── packing_test.rs
│   ├── encoding_test.rs
│   └── cyclic_test.rs
└── integration/             # Integration tests
    ├── stdlib/             # Standard library tests
    │   ├── core_test.rs
    │   ├── crypto_test.rs
    │   ├── rop_test.rs
    │   └── ...
    ├── exploit_chain_test.rs
    ├── lsp_integration_test.rs
    ├── example_runner_test.rs
    └── stdlib_test.rs
```

### Test Categories

**Unit Tests** (`tests/unit/`)
- Test individual functions and modules
- Mock external dependencies
- Fast execution (<1ms per test)
- Examples: parser, AST, packing utilities

**Integration Tests** (`tests/integration/`)
- Test module interactions
- End-to-end workflows
- Standard library coverage
- Example scripts validation

**Fuzz Tests** (`fuzz/`)
- Automated input generation
- Edge case discovery
- Security vulnerability detection
- See `docs/FUZZING.md`

**Benchmarks** (`benches/`)
- Performance measurement
- Regression detection
- See `docs/BENCHMARKING.md`

---

## Running Tests

### Basic Test Execution

```bash
# All tests (recommended for CI)
cargo test --all-features

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --tests

# Specific test file
cargo test --test parser_test
cargo test --test rop_test

# Specific test function
cargo test test_parse_expression
cargo test test_rop_chain_building

# Tests matching pattern
cargo test heap_
cargo test format_string
```

### Test Output Control

```bash
# Show println! output
cargo test -- --nocapture

# Show all test names
cargo test -- --show-output

# Quiet mode (errors only)
cargo test --quiet

# Run ignored tests
cargo test -- --ignored

# Run all tests including ignored
cargo test -- --include-ignored
```

### Parallel Execution

```bash
# Run tests in parallel (default)
cargo test

# Single-threaded (useful for debugging)
cargo test -- --test-threads=1

# Specific thread count
cargo test -- --test-threads=4
```

### Platform-Specific Testing

```bash
# Linux-specific tests
cargo test --features linux-specific

# Windows-specific tests
cargo test --features windows-specific

# Cross-platform (default)
cargo test --all-features
```

---

## Writing Tests

### Unit Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = "test input";
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }
    
    #[test]
    #[should_panic(expected = "error message")]
    fn test_error_handling() {
        function_that_should_panic();
    }
    
    #[test]
    fn test_result_error() -> Result<(), Box<dyn std::error::Error>> {
        let result = fallible_function()?;
        assert!(result.is_valid());
        Ok(())
    }
}
```

### Integration Test Structure

```rust
// tests/integration/my_feature_test.rs
use talon::*;

#[test]
fn test_end_to_end_workflow() {
    let input = "
    let payload = p64(0xdeadbeef)
    print(hex(u64(payload)))
    ";
    
    let result = execute_talon_script(input);
    assert_eq!(result, "0xdeadbeef");
}
```

### Using Test Utilities

```rust
use tests::common::*;

#[test]
fn test_with_mock_binary() {
    let harness = TalonTestHarness::new();
    let binary = harness.create_mock_elf(vec![
        0x55, 0x48, 0x89, 0xe5  // push rbp; mov rbp, rsp
    ]);
    
    let gadgets = find_rop_gadgets(&binary);
    assert!(!gadgets.is_empty());
}

#[test]
fn test_with_assertions() {
    let script = "let x = p64(0x1234)";
    assert_talon_compiles!(script);
    assert_talon_output!(script, "");
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_packing_roundtrip(value in 0u64..u64::MAX) {
        let packed = p64(value);
        let unpacked = u64_from_bytes(&packed);
        prop_assert_eq!(value, unpacked);
    }
    
    #[test]
    fn test_parser_never_panics(s in "\\PC*") {
        // Should never panic, even on invalid input
        let _ = parse_talon_script(&s);
    }
}
```

### Testing Async Code

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Testing with Fixtures

```rust
#[test]
fn test_with_fixture() {
    let binary_path = "tests/fixtures/binaries/test_binary.elf";
    let binary = std::fs::read(binary_path).unwrap();
    
    let analysis = analyze_binary(&binary);
    assert_eq!(analysis.architecture, "x86_64");
}
```

---

## Code Coverage

### Using Tarpaulin

```bash
# Install
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --out Html --all-features

# Generate multiple formats
cargo tarpaulin --out Xml --out Html --all-features

# Upload to Codecov
cargo tarpaulin --out Xml --all-features
bash <(curl -s https://codecov.io/bash)
```

### Using llvm-cov

```bash
# Install
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov

# Generate coverage
cargo llvm-cov --html --all-features

# Open report
cargo llvm-cov --open
```

### Coverage Scripts

```bash
# Linux/macOS
./scripts/generate_coverage.sh

# Windows
.\scripts\generate_coverage.ps1
```

### Coverage Goals

| Component | Target Coverage |
|-----------|----------------|
| Parser | >95% |
| Interpreter | >85% |
| ROP Tools | >90% |
| Heap Tools | >90% |
| Binary Analysis | >85% |
| Shellcode Generation | >80% |
| Standard Library | >80% |
| Overall | >80% |

---

## Fuzzing

### Quick Fuzzing

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run all fuzz targets (5 minutes each)
./scripts/run_fuzz.sh 300

# Run specific target
cargo +nightly fuzz run fuzz_parser

# Run with corpus
cargo +nightly fuzz run fuzz_parser corpus/parser/
```

### Fuzz Targets

- `fuzz_parser` - TALON DSL parser
- `fuzz_elf_parser` - ELF binary parser
- `fuzz_pe_parser` - PE binary parser
- `fuzz_shellcode` - Shellcode generator
- `fuzz_format_string` - Format string exploits
- `fuzz_heap_tools` - Heap manipulation
- `fuzz_rop_finder` - ROP gadget finder
- `fuzz_packing` - Packing/encoding tools

See **`docs/FUZZING.md`** for complete documentation.

---

## Benchmarking

### Running Benchmarks

```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench parser
cargo bench rop

# Save baseline
cargo bench -- --save-baseline main

# Compare to baseline
cargo bench -- --baseline main
```

### Benchmark Scripts

```bash
# Linux/macOS
./scripts/run_benchmarks.sh

# Windows
.\scripts\run_benchmarks.ps1
```

See **`docs/BENCHMARKING.md`** for complete documentation.

---

## Continuous Integration

### GitHub Actions Workflows

**`.github/workflows/ci.yml`**
- Builds on Linux and Windows
- Runs full test suite
- Generates coverage reports
- Uploads to Codecov

**`.github/workflows/security.yml`**
- Runs cargo-audit
- Runs cargo-deny
- Security vulnerability scanning

**`.github/workflows/fuzzing.yml`**
- Daily fuzzing campaigns
- Artifact preservation

**`.github/workflows/benchmarks.yml`**
- Performance tracking
- Regression detection

### CI Test Commands

```bash
# Exact commands used in CI
cargo test --all-features --verbose
cargo test --doc
cargo clippy -- -D warnings
cargo fmt -- --check
```

---

## Troubleshooting

### Common Issues

**Tests Fail to Compile**
```bash
# Clean and rebuild
cargo clean
cargo build --tests

# Update dependencies
cargo update
```

**Tests Hang**
```bash
# Run with timeout
cargo test -- --test-threads=1 --nocapture
```

**Flaky Tests**
```bash
# Run multiple times
for i in {1..10}; do cargo test test_name || break; done
```

**Out of Memory**
```bash
# Reduce parallelism
cargo test -- --test-threads=2
```

### Platform-Specific Issues

**Windows: Linker Errors**
```powershell
# Install Visual Studio Build Tools
winget install Microsoft.VisualStudio.2022.BuildTools

# Or use MinGW
rustup default stable-x86_64-pc-windows-gnu
```

**Linux: Missing Dependencies**
```bash
sudo apt-get install build-essential pkg-config libssl-dev
```

**macOS: Missing Developer Tools**
```bash
xcode-select --install
```

### Test Data Issues

**Missing Fixtures**
```bash
# Fixtures are tracked in Git
git checkout tests/fixtures/
```

**Binary Fixtures Corrupted**
```bash
# Regenerate test binaries
./tests/fixtures/scripts/generate_test_binaries.sh
```

---

## Best Practices

### Test Naming

```rust
// Good
#[test]
fn test_parse_function_definition() { }

#[test]
fn test_rop_chain_with_pie_binary() { }

// Avoid
#[test]
fn test1() { }

#[test]
fn it_works() { }
```

### Test Independence

```rust
// Good - each test is independent
#[test]
fn test_feature_a() {
    let state = create_fresh_state();
    // test logic
}

#[test]
fn test_feature_b() {
    let state = create_fresh_state();
    // test logic
}

// Avoid - tests share state
static mut SHARED_STATE: i32 = 0;
```

### Assertion Messages

```rust
// Good
assert_eq!(
    actual, expected,
    "ROP chain length mismatch: expected {} gadgets, got {}",
    expected, actual
);

// Avoid
assert_eq!(actual, expected);
```

### Test Documentation

```rust
/// Tests that the parser correctly handles nested function calls
/// with multiple argument types including strings, integers, and
/// function references.
#[test]
fn test_parser_nested_function_calls() {
    // Test implementation
}
```

---

## Additional Resources

- **Fuzzing Guide**: `docs/FUZZING.md`
- **Benchmarking Guide**: `docs/BENCHMARKING.md`
- **Coverage Guide**: `docs/COVERAGE.md`
- **Security Auditing**: `docs/SECURITY_AUDITING.md`
- **Contributing Guide**: `CONTRIBUTING.md`
- **QA Checklist**: `docs/QA_CHECKLIST.md`
- **Manual Testing**: `docs/MANUAL_TESTING.md`

---

## Summary

TALON's testing infrastructure ensures:

 **Correctness**: Comprehensive unit and integration tests  
 **Security**: Fuzzing and security audits  
 **Performance**: Benchmarks and profiling  
 **Reliability**: CI/CD automation  
 **Coverage**: >80% code coverage target  

For questions or issues, consult the troubleshooting section or open a GitHub issue.
