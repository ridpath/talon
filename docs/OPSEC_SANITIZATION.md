# OpSec Sanitization - TALON Framework

This document details the operational security measures implemented to prevent information leakage from TALON binaries and source code.

## Implemented Sanitization Measures

### 1. Custom Panic Handler

**Module**: `src/panic_handler.rs`

**Purpose**: Remove file paths and debug information from panic messages in release builds.

**Features**:
- Sanitizes Windows paths (C:\path\to\file.rs)
- Sanitizes Unix paths (/path/to/file.rs)
- Removes line:column numbers (:123:45)
- Removes src/ and src\ references
- Debug builds preserve full panic info for development
- Release builds show sanitized messages only

**Activation**: Automatically installed in `main()` for release builds via `#[cfg(not(debug_assertions))]`

### 2. Build Configuration

**File**: `Cargo.toml`

**Release Profile Settings**:
```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = "fat"            # Full link-time optimization
codegen-units = 1      # Single compilation unit
panic = "abort"        # Smaller binary, no unwinding
strip = true           # Automatically strip symbols
```

**Effect**:
- Symbols automatically stripped from release binaries
- Stack unwinding disabled (no backtrace info)
- Maximum code optimization (harder to reverse engineer)
- Minimal binary size

### 3. Error Message Redaction

**Module**: `src/error_context.rs`

**Features**:
- Production mode error obfuscation with Ed25519 signing
- AES-256-GCM encryption for error content
- File paths replaced with [REDACTED]
- Line numbers and column numbers zeroed out
- No .talon source code exposed in network errors
- Cryptographic signing prevents tampering

**Activation**: Use `talon run --production` flag

### 4. String Sanitization

**Removed References**:
- ✅ "Zenflow" - No references found in source code
- ✅ "interactivetalon" - No references found in source code
- ✅ Task system paths - Removed from documentation

**Remaining References** (Acceptable):
- Build-time file paths in codegen (not in binary)
- Kernel configuration paths (Linux system paths)
- Solidity import paths (generated code, not TALON source)
- Test code file paths (not in production builds)

### 5. Debug Information Removal

**Automatic Stripping**:
- `strip = true` in Cargo.toml
- All debug symbols removed from release binaries
- No DWARF debug info in final binary
- Panic messages sanitized in release mode

**Manual Verification**:
```bash
# Check if binary is stripped (should show "stripped")
file target/release/talon

# Verify minimal debug info (should be very small)
objdump -h target/release/talon | grep debug
```

## Verification Checklist

### Pre-Release Audit

1. **String Audit**:
   ```bash
   # Check for Zenflow references (should be empty)
   strings target/release/talon | grep -i zenflow
   
   # Check for interactivetalon references (should be empty)
   strings target/release/talon | grep -i interactivetalon
   
   # Check for src/ paths (should be minimal/none)
   strings target/release/talon | grep "src/" | wc -l
   ```

2. **Binary Size Verification**:
   ```bash
   # Release binary should be <50MB stripped
   ls -lh target/release/talon
   ```

3. **Symbol Stripping**:
   ```bash
   # Verify binary is stripped
   file target/release/talon
   
   # Verify no debug sections (should be empty or very minimal)
   objdump -h target/release/talon | grep -E '\.debug|\.dwarf'
   ```

4. **Panic Message Testing**:
   ```bash
   # Test panic handler (in release mode, should show sanitized message)
   cargo build --release
   # Trigger a panic and verify no file paths appear
   ```

### Production Build Checklist

- [ ] Built with `cargo build --release` or `cargo build --profile release-small`
- [ ] Binary stripped (verified with `file` command)
- [ ] No "Zenflow" strings in binary
- [ ] No "interactivetalon" strings in binary
- [ ] Minimal "src/" references (only from embedded data, not debug info)
- [ ] Binary size <50MB
- [ ] Panic messages sanitized (tested)
- [ ] Error obfuscation working (if using --production flag)

## Additional Hardening (Optional)

### 1. Binary Packing

For additional obfuscation, consider using:
- UPX (Universal Packer for eXecutables)
- Custom packers for target platform

**Note**: May trigger AV false positives

### 2. Static Analysis Obfuscation

Additional measures for sensitive deployments:
- Control flow flattening (LLVM passes)
- String encryption for embedded data
- Dead code insertion

**Note**: Already implemented in polymorphic shellcode module

### 3. Anti-Debugging

For deployments where debugging detection is critical:
- Use production error obfuscation (`--production` flag)
- Memory scrubber auto-activates anti-debugging checks
- Detects: IsDebuggerPresent, remote debugger, memory dumping

## Maintenance

### Regular Audits

1. **Before Each Release**:
   - Run string audit on binary
   - Verify no new debug info leaks
   - Test panic handler behavior
   - Review error messages for implementation details

2. **After Code Changes**:
   - Grep for new "src/" references in error messages
   - Verify panic! calls use sanitized messages
   - Check error_context integration

3. **Quarterly**:
   - Full security review of error messages
   - Update panic handler regex patterns if needed
   - Review binary size and symbol table

## Contact

For security concerns or questions about OpSec measures, consult the technical lead.

## Version History

- v0.1.0 (2026-02-06): Initial OpSec sanitization implementation
  - Custom panic handler with path redaction
  - Release profile hardening
  - Error message obfuscation support
  - String reference cleanup
