# TALON Test Suite

This directory contains the comprehensive test infrastructure for TALON.

## Structure

```
tests/
├── common/
│   └── mod.rs              # Shared test utilities and harness
├── unit/                    # Unit tests for individual modules
├── integration/             # End-to-end integration tests
├── fixtures/                # Test data and fixtures
│   ├── binaries/            # Pre-compiled test binaries
│   ├── scripts/             # Sample TALON scripts
│   ├── exploits/            # Reference exploit payloads
│   └── data/                # Test data files
└── common_test.rs          # Tests for the test harness itself
```

## Test Utilities

The `common` module provides:

- **TalonTestHarness**: Main test harness for running TALON scripts
- **Vuln enum**: Vulnerability types for mock binary generation
- **Mock binary generation**: Create test binaries with known vulnerabilities
- **Assertion helpers**: Specialized assertions for exploit testing
- **Test fixtures**: Helper functions for creating test data

### Usage Example

```rust
use common::{TalonTestHarness, Vuln};

#[test]
fn test_buffer_overflow_exploit() {
    let mut harness = TalonTestHarness::new();
    
    // Create a vulnerable binary
    let vulns = vec![Vuln::BufferOverflow { offset: 72 }];
    let bin_path = harness.mock_binary("vuln_binary", &vulns);
    
    // Run exploit script
    let script = r#"
        let binary = analyze("vuln_binary")
        let payload = bytes("A" * 72) + p64(0xdeadbeef)
        print("Exploit payload created")
    "#;
    
    let result = harness.run_script(script).unwrap();
    harness.assert_exploit_success(&result).unwrap();
}
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test common_test

# Run tests with output
cargo test -- --nocapture

# Run tests with specific features
cargo test --all-features
```

## Test Utilities API

### TalonTestHarness

```rust
impl TalonTestHarness {
    pub fn new() -> Self;
    pub fn run_script(&self, code: &str) -> Result<String, String>;
    pub fn run_file(&self, path: &Path) -> Result<String, String>;
    pub fn mock_binary(&mut self, name: &str, vulns: &[Vuln]) -> PathBuf;
    pub fn create_vulnerable_c_source(&self, name: &str, vuln_type: &Vuln) -> PathBuf;
    pub fn assert_exploit_success(&self, result: &str) -> Result<(), String>;
    pub fn assert_contains(&self, haystack: &str, needle: &str) -> Result<(), String>;
    pub fn assert_not_contains(&self, haystack: &str, needle: &str) -> Result<(), String>;
    pub fn temp_dir(&self) -> &Path;
    pub fn create_test_file(&self, name: &str, content: &str) -> PathBuf;
    pub fn get_mock_binary(&self, name: &str) -> Option<&PathBuf>;
}
```

### Vulnerability Types

```rust
pub enum Vuln {
    BufferOverflow { offset: usize },
    FormatString { vuln_arg: usize },
    UseAfterFree { heap_chunk: usize },
    IntegerOverflow { width: usize },
    StackPivot { gadget_offset: usize },
}
```

### Helper Functions

```rust
pub fn assert_u64(value: u64, expected: u64);
pub fn assert_hex_str(value: &str, expected: &str);
pub fn create_rop_gadget_binary() -> Vec<u8>;
pub fn create_shellcode_test_env() -> Vec<u8>;
```

## Test Fixtures

The `fixtures` directory contains:

- **binaries/**: Pre-compiled vulnerable test programs
- **scripts/**: Sample TALON scripts for testing
- **exploits/**: Reference exploit payloads
- **data/**: Test data files (ELF headers, PE files, etc.)

️ **Security Warning**: Files in `fixtures/` are intentionally vulnerable. Never execute them outside of sandboxed test environments.

## Writing New Tests

1. Create test file in appropriate directory (`unit/` or `integration/`)
2. Import test utilities: `use common::{TalonTestHarness, Vuln};`
3. Use `TalonTestHarness` for consistent test setup
4. Add test fixtures to `fixtures/` if needed
5. Run tests to verify: `cargo test`

## Test Coverage

To generate coverage reports:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage/
```

## Continuous Integration

Tests are automatically run in CI on:
- Every push to feature branches
- Pull requests to develop/main
- Scheduled nightly builds

See `.github/workflows/ci.yml` for CI configuration.
