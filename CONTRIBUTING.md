# Contributing to TALON

Thank you for your interest in contributing to TALON! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [How to Contribute](#how-to-contribute)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Issue Reporting](#issue-reporting)
- [Community](#community)

---

## Code of Conduct

### Our Commitment

TALON is a security research tool designed for:
- Authorized penetration testing
- CTF competitions
- Security research
- Educational purposes
- Defensive security development

### Prohibited Use

**Do NOT**:
- Use TALON for unauthorized access to systems
- Contribute exploits for active, unpatched vulnerabilities
- Share techniques designed to evade detection for malicious purposes
- Engage in illegal activities

### Professional Standards

- Be respectful and constructive
- Assume good intentions
- Provide helpful feedback
- Focus on security research, not malicious use

---

## Getting Started

### Prerequisites

Before contributing, ensure you have:

1. **Rust Toolchain** (stable)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Development Tools**
   - **Linux**: `build-essential`, `pkg-config`, `libssl-dev`
   - **Windows**: Visual Studio Build Tools or MinGW-w64
   - **macOS**: Xcode Command Line Tools

3. **Git**
   ```bash
   git --version
   ```

4. **Optional Tools**
   ```bash
   cargo install cargo-tarpaulin  # Code coverage
   cargo install cargo-fuzz       # Fuzzing
   cargo install cargo-criterion  # Benchmarking
   cargo install cargo-audit      # Security auditing
   cargo install cargo-deny       # License checking
   ```

### Fork and Clone

```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/YOUR_USERNAME/talon.git
cd talon

# Add upstream remote
git remote add upstream https://github.com/ridpath/talon.git

# Create a feature branch
git checkout -b feature/my-contribution
```

---

## Development Setup

### Build the Project

```bash
# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run the REPL
cargo run -- repl

# Run a script
cargo run -- run examples/basic_rop.talon
```

### Run Tests

```bash
# All tests
cargo test --all-features

# Specific test suite
cargo test --test parser_test

# Watch mode (requires cargo-watch)
cargo watch -x test
```

### Install Pre-commit Hooks

```bash
# Linux/macOS
./scripts/install_hooks.sh

# Windows
.\scripts\install_hooks.ps1
```

This ensures:
- Code formatting (`cargo fmt`)
- Linting (`cargo clippy`)
- Tests pass (`cargo test`)

---

## How to Contribute

### Types of Contributions

** Bug Fixes**
- Fix parsing errors
- Resolve runtime issues
- Correct documentation errors

** New Features**
- Add exploitation primitives
- Implement new standard library functions
- Create exploit templates
- Improve binary analysis capabilities

** Documentation**
- Improve README
- Add examples
- Write tutorials
- Document stdlib functions

** Testing**
- Add unit tests
- Create integration tests
- Write fuzz targets
- Improve coverage

** Performance**
- Optimize hot paths
- Reduce allocations
- Improve parsing speed
- Benchmark critical functions

** Security**
- Report vulnerabilities (see `SECURITY.md`)
- Add security checks
- Improve input validation

### Contribution Workflow

1. **Check Existing Issues**
   - Search for related issues
   - Comment on the issue you want to work on
   - Get assigned or acknowledge you're working on it

2. **Create a Branch**
   ```bash
   git checkout -b feature/descriptive-name
   ```

3. **Make Changes**
   - Write clean, readable code
   - Follow coding standards (see below)
   - Add tests for new functionality
   - Update documentation

4. **Test Locally**
   ```bash
   cargo test --all-features
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

5. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat: add ROP chain auto-solver"
   ```
   
   Use [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` New feature
   - `fix:` Bug fix
   - `docs:` Documentation
   - `test:` Adding tests
   - `perf:` Performance improvement
   - `refactor:` Code restructuring
   - `style:` Code formatting
   - `chore:` Maintenance tasks

6. **Push to Your Fork**
   ```bash
   git push origin feature/descriptive-name
   ```

7. **Create Pull Request**
   - Go to GitHub
   - Click "New Pull Request"
   - Fill out the template
   - Link related issues

---

## Coding Standards

### Rust Style Guide

**Follow Rust Best Practices**:
- Use `rustfmt` for formatting
- Use `clippy` for linting
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings
```

### Code Style

**Naming Conventions**:
```rust
// Good
fn find_rop_gadgets(binary: &[u8]) -> Vec<RopGadget> { }
struct BinaryAnalyzer { }
const MAX_GADGET_LENGTH: usize = 16;

// Avoid
fn FindROPGadgets(b: &[u8]) -> Vec<RopGadget> { }
struct binary_analyzer { }
const max_gadget_length: usize = 16;
```

**Error Handling**:
```rust
// Good - use Result for recoverable errors
fn parse_elf(data: &[u8]) -> Result<ElfFile, ParseError> {
    if data.len() < 4 {
        return Err(ParseError::TooShort);
    }
    // ...
}

// Avoid - don't panic in library code
fn parse_elf(data: &[u8]) -> ElfFile {
    if data.len() < 4 {
        panic!("ELF too short!");  // Bad!
    }
    // ...
}
```

**Documentation**:
```rust
/// Finds ROP gadgets in the provided binary.
///
/// # Arguments
///
/// * `binary` - Raw bytes of the binary to analyze
/// * `arch` - Target architecture (x86_64, arm64, etc.)
///
/// # Returns
///
/// A vector of `RopGadget` structures containing addresses and instructions.
///
/// # Examples
///
/// ```
/// use talon::rop_tools::find_rop_gadgets;
///
/// let binary = std::fs::read("./binary").unwrap();
/// let gadgets = find_rop_gadgets(&binary, "x86_64");
/// assert!(!gadgets.is_empty());
/// ```
pub fn find_rop_gadgets(binary: &[u8], arch: &str) -> Vec<RopGadget> {
    // Implementation
}
```

**Comments**:
```rust
// Good - explain WHY, not WHAT
// Skip null bytes because they terminate strings in strcpy vulnerabilities
let filtered = shellcode.iter().filter(|&&b| b != 0).copied().collect();

// Avoid - obvious comments
// Loop through shellcode
for byte in shellcode { }
```

### TALON DSL Style

**Consistent Naming**:
```talon
# Good - clear, descriptive names
let rop_chain = p64(pop_rdi) + p64(bin_sh) + p64(system)
let leak_offset = 72

# Avoid - unclear abbreviations
let rc = p64(pr) + p64(bs) + p64(s)
let o = 72
```

**Readable Structure**:
```talon
# Good - logical grouping
# Stage 1: Leak libc
let leak_payload = bytes("A" * 72) + p64(puts_plt) + p64(main)
conn.send(leak_payload)
let libc_leak = u64(conn.recv(8))

# Stage 2: Calculate offsets
let libc_base = libc_leak - 0x21910
let system = libc_base + 0x4f440

# Stage 3: Execute shell
let shell_payload = bytes("A" * 72) + p64(system)
conn.send(shell_payload)
```

---

## Testing Requirements

### All Contributions Must Include Tests

**Bug Fixes**:
- Add a regression test demonstrating the bug
- Verify the fix resolves the issue

**New Features**:
- Unit tests for individual functions
- Integration tests for workflows
- Property-based tests for complex logic

**Performance Changes**:
- Benchmark comparison (before/after)
- Verify no regression in other areas

### Test Coverage

- Aim for **>80% coverage** overall
- **>90% coverage** for security-critical code
- **100% coverage** for exploitation primitives

```bash
# Check coverage
cargo tarpaulin --out Html

# Open report
firefox tarpaulin-report.html  # Linux
open tarpaulin-report.html     # macOS
start tarpaulin-report.html    # Windows
```

### Writing Good Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptive_name() {
        // Arrange - set up test conditions
        let input = create_test_input();
        
        // Act - perform the operation
        let result = function_under_test(input);
        
        // Assert - verify expectations
        assert_eq!(result.status, Status::Success);
        assert!(result.data.len() > 0);
    }

    #[test]
    fn test_error_handling() {
        let result = function_with_invalid_input();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Expected error message"
        );
    }
}
```

See **`TESTING.md`** for comprehensive testing guidance.

---

## Pull Request Process

### Before Submitting

-  All tests pass: `cargo test --all-features`
-  No linting errors: `cargo clippy -- -D warnings`
-  Code is formatted: `cargo fmt`
-  Documentation is updated
-  CHANGELOG.md is updated (for notable changes)
-  Commit messages follow Conventional Commits

### PR Description Template

```markdown
## Description
Brief description of changes

## Motivation
Why is this change necessary?

## Changes
- Added X feature
- Fixed Y bug
- Refactored Z module

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed
- [ ] All tests pass locally

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex logic
- [ ] Documentation updated
- [ ] No new warnings introduced
- [ ] Security implications considered

## Related Issues
Closes #123
Related to #456
```

### Review Process

1. **Automated Checks**: CI must pass
2. **Code Review**: At least one maintainer approval
3. **Testing**: Reviewers may request additional tests
4. **Iteration**: Address feedback, push updates
5. **Merge**: Maintainer merges when approved

### Merge Requirements

-  CI passes (Linux + Windows)
-  Code coverage maintained (>80%)
-  At least 1 approving review
-  No unresolved review comments
-  Branch up-to-date with main

---

## Issue Reporting

### Bug Reports

**Use the Bug Report Template**:

```markdown
## Bug Description
Clear description of the bug

## Steps to Reproduce
1. Run `talon run script.talon`
2. Observe error

## Expected Behavior
Should parse successfully

## Actual Behavior
Parser crashes with error: ...

## Environment
- OS: Windows 11
- Rust version: 1.75.0
- TALON version: 0.1.0

## Additional Context
Stack trace, logs, screenshots
```

### Feature Requests

```markdown
## Feature Description
Add support for ARM64 ROP gadgets

## Motivation
Many CTF challenges now use ARM architecture

## Proposed Solution
Extend ROP finder with ARM64 disassembly

## Alternatives Considered
Manual gadget specification

## Additional Context
Related to issue #789
```

### Security Vulnerabilities

**DO NOT** open public issues for security vulnerabilities.

See **`SECURITY.md`** for responsible disclosure process.

---

## Community

### Communication Channels

- **GitHub Issues**: Bug reports, feature requests
- **GitHub Discussions**: Questions, ideas, showcase
- **Pull Requests**: Code contributions

### Getting Help

**Before Asking**:
1. Check documentation (`docs/`, `README.md`)
2. Search existing issues
3. Read `TESTING.md` and `CONTRIBUTING.md`

**Where to Ask**:
- Technical questions → GitHub Discussions
- Bug reports → GitHub Issues
- Security concerns → See `SECURITY.md`

### Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` (auto-generated)
- Release notes
- GitHub contributors page

---

## Development Tips

### Useful Commands

```bash
# Watch mode - rebuild on file changes
cargo watch -x build

# Run tests with output
cargo test -- --nocapture

# Run single test
cargo test test_name -- --exact

# Generate documentation
cargo doc --open

# Check for outdated dependencies
cargo outdated

# Security audit
cargo audit

# Benchmarks
cargo bench

# Fuzzing
cargo +nightly fuzz run fuzz_parser
```

### IDE Setup

**VS Code**:
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.rustfmt.enableRangeFormatting": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true
  }
}
```

**IntelliJ/CLion**:
- Install Rust plugin
- Enable `cargo fmt` on save
- Enable `cargo clippy` checks

### Debugging

```bash
# Run with debug logging
RUST_LOG=debug cargo run -- run script.talon

# Run with backtrace
RUST_BACKTRACE=1 cargo run -- run script.talon

# Use rust-gdb/rust-lldb
rust-gdb target/debug/talon
```

---

## Advanced Topics

### Adding a New Standard Library Function

1. **Define in `interpreter.rs`**:
```rust
"my_function" => {
    let arg = evaluate_expr(&args[0], state)?;
    let result = my_function_impl(arg);
    Ok(TalonValue::String(result))
}
```

2. **Implement logic**:
```rust
fn my_function_impl(input: TalonValue) -> String {
    // Implementation
}
```

3. **Add tests**:
```rust
#[test]
fn test_my_function() {
    let script = r#"
    let result = my_function("input")
    assert(result == "expected")
    "#;
    assert_talon_executes!(script);
}
```

4. **Document**:
```rust
/// my_function(input: str) -> str
///
/// Does something useful with the input.
///
/// # Examples
///
/// ```talon
/// let result = my_function("test")
/// print(result)
/// ```
```

5. **Update `BUILTIN_FUNCTIONS_REFERENCE.md`**

### Adding a New Exploitation Primitive

1. Create module in `src/` (e.g., `src/my_primitive.rs`)
2. Implement core logic with comprehensive error handling
3. Add unit tests in `tests/unit/my_primitive_test.rs`
4. Add integration tests in `tests/integration/stdlib/my_primitive_test.rs`
5. Expose via interpreter in `interpreter.rs`
6. Document in `docs/` and `BUILTIN_FUNCTIONS_REFERENCE.md`
7. Add example script in `examples/`

### Adding Binary Format Support

1. Create parser in `src/` (e.g., `src/macho_parser.rs`)
2. Implement `BinaryFormat` trait
3. Add fuzz target in `fuzz/`
4. Add unit tests with real binaries in `tests/fixtures/binaries/`
5. Update `binary_analyzer.rs` to support new format

---

## Release Process

(For maintainers)

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create release branch: `release/v0.1.0`
4. Run full test suite + fuzzing
5. Tag release: `git tag -a v0.1.0 -m "Release v0.1.0"`
6. Push tag: `git push origin v0.1.0`
7. GitHub Actions builds and publishes release

---

## Questions?

If you have questions not covered in this guide:

1. Check `TESTING.md`, `README.md`, and `docs/`
2. Search existing GitHub issues and discussions
3. Open a GitHub Discussion
4. Tag maintainers if urgent

---

## Thank You!

Your contributions make TALON better for the security research community. Whether it's code, documentation, bug reports, or ideas—every contribution matters.

Happy hacking! 
