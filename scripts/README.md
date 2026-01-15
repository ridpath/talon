# TALON Scripts Directory

This directory contains utility scripts for development, testing, and automation.

## Development Tools

### Pre-commit Hooks

**`install_hooks.sh` / `install_hooks.ps1`**
- Installs Git pre-commit hooks for automatic quality checks
- Usage:
  ```bash
  # Linux/macOS
  ./scripts/install_hooks.sh
  
  # Windows
  .\scripts\install_hooks.ps1
  ```

**`pre-commit.sh` / `pre-commit.ps1`**
- Pre-commit hook that runs before each commit
- Checks: formatting, linting, compilation, fast tests, security
- Usage: Installed automatically by `install_hooks.*` or run manually:
  ```bash
  # Linux/macOS
  ./scripts/pre-commit.sh
  
  # Windows
  .\scripts\pre-commit.ps1
  ```

## Testing Scripts

### Comprehensive Testing

**`test_all_examples.sh` / `test_all_examples.ps1`**
- Runs all TALON example scripts with validation
- Generates coverage report
- Usage:
  ```bash
  ./scripts/test_all_examples.sh
  ```

**`test_shellcode_formatstring.sh` / `test_shellcode_formatstring.ps1`**
- Targeted tests for shellcode and format string modules
- Validates payload generation
- Usage:
  ```bash
  ./scripts/test_shellcode_formatstring.sh
  ```

## Fuzzing Scripts

**`run_fuzz.sh` / `run_fuzz.ps1`**
- Main fuzzing orchestrator script
- Runs all fuzz targets with configurable duration
- Usage:
  ```bash
  ./scripts/run_fuzz.sh [duration_in_seconds]
  ./scripts/run_fuzz.sh 300  # 5 minutes
  ```

**`fuzz_single.sh`**
- Run a single fuzz target
- Usage:
  ```bash
  ./scripts/fuzz_single.sh <target_name> [duration]
  ./scripts/fuzz_single.sh parser 60
  ```

**`fuzz_continuous.sh`**
- Continuous fuzzing with automatic corpus updates
- Runs indefinitely until stopped
- Usage:
  ```bash
  ./scripts/fuzz_continuous.sh
  ```

**`fuzz_regression.sh`**
- Regression testing on known corpus
- Validates no crashes on existing inputs
- Usage:
  ```bash
  ./scripts/fuzz_regression.sh
  ```

**`fuzz_differential.sh`**
- Differential fuzzing between parser implementations
- Usage:
  ```bash
  ./scripts/fuzz_differential.sh
  ```

**`fuzz_coverage.sh`**
- Generate coverage report from fuzzing
- Usage:
  ```bash
  ./scripts/fuzz_coverage.sh
  ```

**`minimize_crash.sh`**
- Minimize a crash-inducing input
- Usage:
  ```bash
  ./scripts/minimize_crash.sh <crash_file>
  ```

## Benchmarking Scripts

**`run_benchmarks.sh` / `run_benchmarks.ps1`**
- Run all performance benchmarks with Criterion
- Generates HTML reports
- Usage:
  ```bash
  # Linux/macOS
  ./scripts/run_benchmarks.sh [target]
  
  # Windows
  .\scripts\run_benchmarks.ps1 -Target parser
  ```

## Code Quality Scripts

**`generate_coverage.sh` / `generate_coverage.ps1`**
- Generate code coverage report with tarpaulin
- Outputs HTML and XML reports
- Usage:
  ```bash
  # Linux/macOS
  ./scripts/generate_coverage.sh
  
  # Windows
  .\scripts\generate_coverage.ps1
  ```

**`security_audit.sh` / `security_audit.ps1`**
- Run security audits (cargo-audit, cargo-deny)
- Checks for vulnerabilities and license issues
- Usage:
  ```bash
  # Linux/macOS
  ./scripts/security_audit.sh
  
  # Windows
  .\scripts\security_audit.ps1
  ```

## TALON Example Scripts

These are example TALON scripts demonstrating language features:

- `chain_scan.talon` - ROP chain scanning example
- `drop_and_run.talon` - Quick exploitation pattern
- `exploit.talon` - Basic exploit template
- `format_string.talon` - Format string exploitation
- `fuzz_target.talon` - Fuzzing target example
- `stack_smash.talon` - Stack smashing example
- `tcp_overflow_send.talon` - Network overflow example

## Script Naming Convention

- `.sh` - Bash scripts for Linux/macOS
- `.ps1` - PowerShell scripts for Windows
- `.talon` - TALON language scripts

## Cross-Platform Compatibility

Most scripts have both Bash (`.sh`) and PowerShell (`.ps1`) versions for cross-platform support:

**Linux/macOS:**
```bash
./scripts/script_name.sh
```

**Windows PowerShell:**
```powershell
.\scripts\script_name.ps1
```

## Requirements

### Pre-commit Hooks
- Rust toolchain (rustup, cargo)
- Git
- Optional: Python 3 with `pre-commit` package
- Optional: cargo-deny, cargo-audit

### Testing Scripts
- Rust toolchain with test dependencies
- cargo-tarpaulin (for coverage)

### Fuzzing Scripts
- cargo-fuzz
- LLVM (for libfuzzer)

### Benchmarking Scripts
- Criterion.rs (included in dev-dependencies)

## Documentation

For detailed information about specific tooling:

- [Pre-commit Hooks](../docs/PRE_COMMIT_HOOKS.md)
- [Testing](../TESTING.md)
- [Fuzzing](../docs/FUZZING.md)
- [Benchmarking](../docs/BENCHMARKING.md)
- [Code Coverage](../docs/COVERAGE.md)
- [Security Auditing](../docs/SECURITY_AUDITING.md)

## Contributing

When adding new scripts:

1. Create both `.sh` (Bash) and `.ps1` (PowerShell) versions when possible
2. Add usage examples in this README
3. Include error handling and user-friendly output
4. Make scripts executable: `chmod +x scripts/new_script.sh`
5. Follow existing code style and structure

## Maintenance

### Updating Hooks

After updating pre-commit hooks:
```bash
./scripts/install_hooks.sh  # Reinstall
```

### Cleaning Up

Remove generated files:
```bash
# Coverage reports
rm -rf coverage/

# Fuzzing artifacts
rm -rf fuzz/corpus/ fuzz/artifacts/

# Benchmark results
rm -rf target/criterion/
```

## Support

For issues with scripts:
1. Check script prerequisites
2. Review relevant documentation
3. Ensure you're in the project root directory
4. Verify file permissions (`chmod +x`)
5. Open an issue if the problem persists
