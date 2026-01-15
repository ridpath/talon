# Unit Tests - Parser & AST

## Overview
Comprehensive unit test suite for TALON's parser and AST modules.

## Test Files

### parser_test.rs
- **70+ test cases** covering all parsing functionality
- **7 property-based tests** using proptest for fuzzing
- Tests all grammar rules from `lang.pest`
- Covers error handling and edge cases

**Key Test Areas:**
- Variable/constant declarations
- All data structures (lists, maps, sets, bytes)
- Function definitions (sync/async, with defaults, type hints)
- Control flow (if/else, for, while, match, try/catch)
- Expressions (binary/comparison/logical ops, pack/unpack)
- Advanced features (macros, lambdas, comprehensions, pipes)
- String handling (escape sequences, unicode, multiline)
- Error conditions (syntax errors, missing keywords)

### ast_test.rs
- **70+ test cases** for AST node creation and manipulation
- Tests all AST variants and their properties
- Verifies Clone and Debug trait implementations

**Key Test Areas:**
- Type system (TypeHint variants)
- Function definitions and metadata
- Pattern matching constructs
- Control flow nodes
- Command variants
- Expression types (literals, operations, collections)
- Advanced expressions (lambdas, comprehensions, pipes)

## Running Tests

### Quick Start
```bash
# Run all unit tests
cargo test --test unit

# Run specific test file
cargo test --test unit parser_test
cargo test --test unit ast_test

# Run with output
cargo test --test unit -- --nocapture

# Run property-based tests only
cargo test --test unit prop_
```

### With Coverage
```bash
# Install tarpaulin (first time only)
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --test unit --out Html --output-dir coverage/

# Open coverage report
# Windows: start coverage\index.html
# Linux: xdg-open coverage/index.html
```

## Test Organization

```
tests/
├── unit/
│   ├── mod.rs           # Module declarations
│   ├── parser_test.rs   # Parser tests (70+ cases)
│   └── ast_test.rs      # AST tests (70+ cases)
└── common/
    └── mod.rs           # Test utilities (TalonTestHarness)
```

## Coverage Goals

| Module | Target Coverage | Test Count |
|--------|----------------|------------|
| parser.rs | >95% | 70+ |
| ast.rs | >95% | 70+ |

## Property-Based Testing

Using `proptest` for randomized input generation:
- Valid identifier patterns
- Number parsing (decimal and hex)
- String literals with escaping
- Binary/comparison operators
- Collection generation

## Dependencies

Required crates (already in Cargo.toml):
- `proptest = "1.4"` - Property-based testing
- `pretty_assertions = "1.4"` - Better assertion output

## Troubleshooting

### Cargo Not Found
If you see "cargo is not recognized as an internal or external command":
1. Install Rust: https://rustup.rs/
2. Restart your terminal
3. Verify: `cargo --version`

### Test Compilation Errors
```bash
# Check test compilation without running
cargo check --tests

# Get detailed error messages
cargo test --test unit 2>&1 | more
```

### Coverage Issues
```bash
# If tarpaulin fails, try
cargo test --test unit -- --nocapture > test_output.txt

# Then manually review which tests passed/failed
```

## Next Steps

1. **Run the tests**: `cargo test --test unit`
2. **Check coverage**: `cargo tarpaulin --test unit`
3. **Review gaps**: Look at coverage report to identify untested code paths
4. **Add tests**: If coverage < 95%, add more tests for uncovered lines

## Notes

- Tests are designed to be fast and isolated
- No network or filesystem access required (uses mocks)
- Property-based tests run 256 iterations by default
- All tests should pass on both Windows and Linux
