# Test Fixtures

This directory contains test fixtures used by the TALON test suite.

## Structure

- `binaries/` - Pre-compiled test binaries with known vulnerabilities
- `scripts/` - Sample TALON scripts for integration testing
- `exploits/` - Reference exploit payloads
- `data/` - Test data files (ELF, PE, shellcode, etc.)

## Usage

Test fixtures are loaded by the test harness in `tests/common/mod.rs`.

## Security Notice

**DO NOT** execute binaries in this directory outside of sandboxed test environments.
These are intentionally vulnerable programs for testing purposes only.
