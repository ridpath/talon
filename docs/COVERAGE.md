# TALON Code Coverage Guide

This guide explains how to generate, analyze, and improve code coverage for the TALON project.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Coverage Tools](#coverage-tools)
- [Running Coverage](#running-coverage)
- [Understanding Reports](#understanding-reports)
- [Coverage Targets](#coverage-targets)
- [Improving Coverage](#improving-coverage)
- [CI/CD Integration](#cicd-integration)
- [Troubleshooting](#troubleshooting)

## Overview

TALON uses `cargo-tarpaulin` for code coverage analysis, with a target of **≥80% line coverage** across the codebase. Coverage reports are automatically generated in CI/CD pipelines and uploaded to Codecov.

### Why 80%?

- **Core modules** (parser, interpreter, AST): Target 95%+ coverage
- **Exploitation modules** (ROP, heap, shellcode): Target 85%+ coverage
- **Binary analysis** (ELF, PE parsing): Target 80%+ coverage
- **LSP/IDE integration**: Target 75%+ coverage
- **Overall project**: Target 80%+ coverage

## Quick Start

### Prerequisites

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin
```

### Generate Coverage (Linux/macOS)

```bash
# Quick coverage (stdout only, 1 minute timeout)
./scripts/generate_coverage.sh quick

# Comprehensive coverage (HTML/XML/LCOV/JSON, 5 minute timeout)
./scripts/generate_coverage.sh comprehensive

# CI mode (XML only, fail if <80%)
./scripts/generate_coverage.sh ci
```

### Generate Coverage (Windows)

```powershell
# Quick coverage
.\scripts\generate_coverage.ps1 -Profile quick

# Comprehensive coverage
.\scripts\generate_coverage.ps1 -Profile comprehensive

# CI mode
.\scripts\generate_coverage.ps1 -Profile ci
```

## Coverage Tools

### cargo-tarpaulin

Primary coverage tool for Rust projects.

```bash
# Install
cargo install cargo-tarpaulin

# Basic usage
cargo tarpaulin --out Html --output-dir coverage/

# With all features
cargo tarpaulin --all-features --workspace --out Html

# Include doctests
cargo tarpaulin --run-types Tests,Doctests
```

### Configuration

Coverage settings are defined in `tarpaulin.toml`:

```toml
[config]
out = ["Html", "Xml", "Lcov", "Json"]
run-types = ["Tests", "Doctests"]
all-features = true
workspace = true
timeout = 300
fail-under = 80.0
```

## Running Coverage

### Profile Options

#### Quick Profile
- **Output**: Stdout, HTML
- **Timeout**: 60 seconds
- **Use case**: Rapid feedback during development

```bash
./scripts/generate_coverage.sh quick
```

#### Comprehensive Profile
- **Output**: HTML, XML, LCOV, JSON
- **Timeout**: 300 seconds (5 minutes)
- **Run types**: Tests + Doctests
- **Use case**: Detailed analysis, local development

```bash
./scripts/generate_coverage.sh comprehensive
```

#### CI Profile
- **Output**: XML only (Cobertura format)
- **Timeout**: 300 seconds
- **Fail threshold**: 80%
- **Use case**: Automated CI/CD pipelines

```bash
./scripts/generate_coverage.sh ci
```

### Manual Coverage Commands

```bash
# HTML report only
cargo tarpaulin --out Html --output-dir coverage/

# Multiple formats
cargo tarpaulin --out Html --out Xml --out Lcov --output-dir coverage/

# Specific package
cargo tarpaulin --package talon --out Html

# Exclude tests from coverage
cargo tarpaulin --exclude-files '**/test_*.rs' '**/*_test.rs'

# With features
cargo tarpaulin --all-features --out Html

# Verbose output
cargo tarpaulin --verbose --out Html
```

## Understanding Reports

### HTML Report

The HTML report (`tarpaulin-report.html`) provides:

- **Overall coverage percentage**
- **File-by-file breakdown**
- **Line-by-line coverage visualization**
  - 🟢 Green: Executed lines
  - 🔴 Red: Uncovered lines
  - ⚪ Gray: Non-executable lines

### XML Report (Cobertura)

The XML report (`cobertura.xml`) is used by:
- Codecov integration
- CI/CD pipelines
- IDE coverage plugins

Structure:
```xml
<coverage line-rate="0.85" branch-rate="0.75">
  <packages>
    <package name="talon">
      <classes>
        <class name="src/parser.rs" line-rate="0.92">
          <lines>
            <line number="1" hits="5"/>
            <line number="2" hits="0"/>
          </lines>
        </class>
      </classes>
    </package>
  </packages>
</coverage>
```

### JSON Report

The JSON report (`tarpaulin-report.json`) provides programmatic access:

```json
{
  "files": {
    "src/parser.rs": {
      "covered": 450,
      "coverable": 500,
      "percentage": 90.0
    }
  }
}
```

### LCOV Report

LCOV format (`lcov.info`) for integration with:
- VS Code coverage extensions
- Other LCOV-compatible tools

## Coverage Targets

### Component Coverage Goals

| Component | Target | Current | Priority |
|-----------|--------|---------|----------|
| Core Interpreter | 95% | TBD | Critical |
| Parser & AST | 95% | TBD | Critical |
| Builtins | 90% | TBD | High |
| ROP Tools | 85% | TBD | High |
| Heap Tools | 85% | TBD | High |
| Shellcode | 85% | TBD | High |
| Binary Analysis | 80% | TBD | Medium |
| LSP Server | 75% | TBD | Medium |
| Exploit Chaining | 80% | TBD | High |

### Excluded from Coverage

- Test files (`tests/**/*`, `**/*_test.rs`)
- Benchmarks (`benches/**/*`)
- Examples (`examples/**/*`)
- Scripts (`scripts/**/*`)
- VS Code extension (`vscode-extension/**/*`)
- Fuzz targets (`fuzz/**/*`)

## Improving Coverage

### Identify Uncovered Code

1. **Generate comprehensive report**:
   ```bash
   ./scripts/generate_coverage.sh comprehensive
   ```

2. **Open HTML report**:
   ```bash
   open coverage/reports/latest/tarpaulin-report.html
   ```

3. **Look for red lines** in critical modules

### Common Coverage Gaps

#### 1. Error Handling Paths

```rust
// Uncovered error path
fn parse_number(s: &str) -> Result<i64, ParseError> {
    s.parse().map_err(|_| ParseError::InvalidNumber) // ❌ Error path not tested
}
```

**Fix**: Add error test cases:
```rust
#[test]
fn test_parse_number_invalid() {
    assert!(parse_number("not_a_number").is_err());
}
```

#### 2. Edge Cases

```rust
// Uncovered edge case
fn divide(a: i64, b: i64) -> Option<i64> {
    if b == 0 {
        None // ❌ Not tested
    } else {
        Some(a / b)
    }
}
```

**Fix**: Test edge cases:
```rust
#[test]
fn test_divide_by_zero() {
    assert_eq!(divide(10, 0), None);
}
```

#### 3. Complex Branches

```rust
// Uncovered branch
match token {
    Token::If => { /* tested */ },
    Token::While => { /* tested */ },
    Token::For => { /* ❌ not tested */ },
    _ => { /* tested */ }
}
```

**Fix**: Add branch coverage:
```rust
#[test]
fn test_for_loop_parsing() {
    let result = parse("for i in 0..10 { }");
    assert!(result.is_ok());
}
```

#### 4. Doctests

Add executable documentation:
```rust
/// Parse a hexadecimal string to bytes
///
/// # Examples
///
/// ```
/// use talon::hex_to_bytes;
/// let bytes = hex_to_bytes("deadbeef").unwrap();
/// assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
/// ```
pub fn hex_to_bytes(s: &str) -> Result<Vec<u8>, HexError> {
    // implementation
}
```

### Coverage Improvement Workflow

1. **Identify low-coverage modules**:
   ```bash
   # Generate coverage report
   ./scripts/generate_coverage.sh comprehensive
   
   # Find modules below 80%
   grep -A5 'line-rate' coverage/reports/latest/cobertura.xml | grep '0\.[0-7]'
   ```

2. **Prioritize critical modules**:
   - Core interpreter
   - Parser and AST
   - Exploitation primitives

3. **Write targeted tests**:
   - Focus on uncovered lines
   - Test error paths
   - Cover edge cases

4. **Re-run coverage**:
   ```bash
   ./scripts/generate_coverage.sh quick
   ```

5. **Iterate until target met**

## CI/CD Integration

### GitHub Actions

Coverage is automatically generated in the CI pipeline (`.github/workflows/ci.yml`):

```yaml
coverage:
  name: Code Coverage
  runs-on: ubuntu-latest
  steps:
    - name: Checkout code
      uses: actions/checkout@v4
    
    - name: Install Rust toolchain
      uses: dtolnay/rust-toolchain@stable
    
    - name: Install tarpaulin
      run: cargo install cargo-tarpaulin
    
    - name: Generate coverage
      run: cargo tarpaulin --verbose --all-features --workspace --timeout 300 --out xml
    
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v4
      with:
        files: ./cobertura.xml
        fail_ci_if_error: false
```

### Codecov Integration

Configuration in `codecov.yml`:

```yaml
coverage:
  status:
    project:
      default:
        target: 80%
        threshold: 2%
    patch:
      default:
        target: 80%
        threshold: 5%
```

### Coverage Badge

Add to `README.md`:

```markdown
[![codecov](https://codecov.io/gh/YOUR_ORG/talon/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_ORG/talon)
```

## Troubleshooting

### Coverage Generation Fails

**Problem**: `cargo tarpaulin` crashes or times out

**Solutions**:
1. Increase timeout:
   ```bash
   cargo tarpaulin --timeout 600
   ```

2. Run tests serially:
   ```bash
   cargo tarpaulin --test-threads 1
   ```

3. Exclude problematic tests:
   ```bash
   cargo tarpaulin --exclude-files 'src/problematic.rs'
   ```

### Low Coverage on Binary Analysis

**Problem**: ELF/PE parsing shows low coverage

**Cause**: Complex binary formats, many edge cases

**Solutions**:
1. Create test binaries with known features
2. Use fixtures in `tests/fixtures/`
3. Mock external dependencies
4. Focus on critical paths first

### Doctests Not Counted

**Problem**: Doctests don't increase coverage

**Cause**: Not running with `--run-types Doctests`

**Solution**:
```bash
cargo tarpaulin --run-types Tests,Doctests
```

### Platform-Specific Code

**Problem**: Windows-only code not covered on Linux CI

**Solutions**:
1. Use conditional compilation tests:
   ```rust
   #[cfg(target_os = "windows")]
   #[test]
   fn test_windows_feature() { }
   ```

2. Run coverage on multiple platforms in CI

3. Use mocking for platform-specific APIs

### Coverage Report Not Generated

**Problem**: No HTML report after running tarpaulin

**Solutions**:
1. Check output directory:
   ```bash
   cargo tarpaulin --out Html --output-dir ./coverage
   ```

2. Verify file permissions:
   ```bash
   chmod 755 coverage/
   ```

3. Check disk space:
   ```bash
   df -h
   ```

## Best Practices

### 1. Test Critical Paths First

Prioritize coverage for:
- User-facing APIs
- Security-critical functions
- Error handling paths

### 2. Avoid Testing for Coverage Sake

Don't write meaningless tests just to increase numbers:
```rust
// ❌ Bad: Meaningless test
#[test]
fn test_getter() {
    let x = Foo::new();
    let _ = x.get_value(); // Just calling to increase coverage
}

// ✅ Good: Tests behavior
#[test]
fn test_value_persists_after_creation() {
    let x = Foo::new_with_value(42);
    assert_eq!(x.get_value(), 42);
}
```

### 3. Use Property-Based Testing

Increase coverage with property tests:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_any_number(n: i64) {
        let s = n.to_string();
        let parsed = parse_number(&s).unwrap();
        assert_eq!(parsed, n);
    }
}
```

### 4. Mock External Dependencies

Use mocking to test I/O-heavy code:
```rust
#[cfg(test)]
use mockall::predicate::*;
#[cfg(test)]
use mockall::*;

#[automock]
trait FileSystem {
    fn read_file(&self, path: &str) -> Result<Vec<u8>>;
}
```

### 5. Benchmark Coverage Generation

Track coverage generation performance:
```bash
time ./scripts/generate_coverage.sh comprehensive
```

## Resources

- [cargo-tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [Codecov Documentation](https://docs.codecov.com/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Property-Based Testing](https://github.com/proptest-rs/proptest)

## Summary

- **Target**: ≥80% line coverage
- **Tool**: cargo-tarpaulin
- **CI**: Automated coverage in GitHub Actions
- **Reports**: HTML, XML, LCOV, JSON
- **Scripts**: `./scripts/generate_coverage.sh` (Linux/macOS), `.\scripts\generate_coverage.ps1` (Windows)
- **Focus**: Critical paths, error handling, edge cases

For questions or issues, see [CONTRIBUTING.md](../CONTRIBUTING.md) or open an issue on GitHub.
