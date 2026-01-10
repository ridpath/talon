# TALON Language Support for VS Code

Language support for the TALON Exploit Development Language with Phase 23 CTF-focused features.

## Phase 23 Features

### New Capabilities:
- **Libc.rip Integration**: Automatic libc identification from leaked addresses
- **Auto-Offset Finding**: Automated buffer overflow offset detection with GDB
- **16 Exploit Templates**: Production-ready templates (ret2libc, heap exploits, ROP chains)
- **Flag Automation**: Search and submit flags to CTF platforms (CTFd, HTB, custom)
- **Real GDB Parsing**: Crash analysis with register extraction and backtrace
- **Interactive Helpers**: Quick-mode guides for common exploitation tasks

## Features

- **Integrated Debugger**: Full GDB integration with source-level debugging
  - Set breakpoints by clicking line numbers
  - Step through TALON code (maps to assembly)
  - Live register and memory inspection
  - Variable hover evaluation
- **Visual Memory Viewer**: Interactive memory visualization
  - Stack/Heap/Mappings tabs
  - Color-coded patterns (cyclic, ROP gadgets, shellcode)
  - Memory search (find strings like /bin/sh)
  - Telescope feature (follow pointer chains)
- **ROP Chain Builder**: Drag-and-drop gadget construction
  - Auto-scan binaries for ROP gadgets
  - Filter by null bytes and categories
  - One-click code generation
  - Address offset calculator for ASLR
- **pwndbg-style Commands**: Familiar tooling integration
  - checksec - Binary protection analysis
  - vmmap - Memory mapping viewer
  - search - Find patterns in memory
  - telescope - Follow pointer chains
- **Syntax Highlighting**: Full TextMate grammar with 250+ built-in functions
- **Auto-completion**: Smart completions with Phase 23 functions
- **Code Snippets**: 24+ templates including all Phase 23 exploits
- **AI Assistant**: Smart exploit generation and analysis
- **Interactive Tutorials**: Step-by-step exploitation guides

## Syntax Highlighting

The extension provides comprehensive syntax highlighting for:
- Keywords (`let`, `if`, `for`, `while`, `function`, etc.)
- Built-in functions (exploitation, networking, crypto, etc.)
- String literals with escape sequence support
- Comments (line and block)
- Numbers (decimal, hex, binary)
- Operators

## Code Snippets

### Phase 23 Snippets (NEW):
- **libc-leak**: Leak and identify libc with libc.rip
- **auto-offset**: Automatic buffer overflow offset finding
- **ret2libc**: Full ret2libc exploit with leak and libc identification
- **flag-search**: Search for flags and submit to CTF platforms
- **gdb-crash**: Analyze crashes with GDB
- **quick-shell**: Interactive shell helper guide
- **quick-rop**: Interactive ROP building guide
- **quick-pwn**: Complete exploit generation guide
- **tcache-poison**: Tcache poisoning heap exploit
- **fastbin-dup**: Fastbin duplication
- **one-gadget**: One gadget RCE exploitation
- **sigrop**: SIGROP (Sigreturn-Oriented Programming)
- **stack-pivot**: Stack pivot exploitation

### Classic Snippets:
- **exploit-bof**: Buffer overflow exploit
- **rop-chain**: Return-oriented programming chain
- **exploit-fmt**: Format string exploit
- **remote**: Remote connection template
- **shellcode**: Shellcode generation
- **http-get**: HTTP GET request
- **port-scan**: Port scanning
- **hash**: Cryptographic hashing
- **b64encode**: Base64 encoding
- **compress**: Data compression

## Installation

### From VSIX
1. Download the `.vsix` file
2. Open VS Code
3. Press `Ctrl+Shift+P` (Windows/Linux) or `Cmd+Shift+P` (Mac)
4. Type "Install from VSIX"
5. Select the downloaded file

### From Marketplace
1. Open VS Code
2. Go to Extensions (`Ctrl+Shift+X`)
3. Search for "TALON DSL"
4. Click Install

## Usage

1. Create a new file with `.talon` extension
2. Start typing to see syntax highlighting
3. Use `Ctrl+Space` for auto-completion
4. Type snippet prefixes and press `Tab` to expand

## Example

```talon
// Buffer overflow exploit example
let offset = 264
let payload = cyclic(offset) + p64(0xdeadbeef)

let conn = connect("target.com", 1337)
send(conn, payload)
interactive(conn)
```

## Requirements

- VS Code 1.70.0 or higher

## Commands

Access via Command Palette (`Ctrl+Shift+P` or `Cmd+Shift+P`):

### Debug Commands (NEW):
- **TALON: Start Debugging** (F5) - Launch TALON script with GDB debugger
- **TALON: Checksec** - Analyze binary protections (NX, PIE, Canary, RELRO)
- **TALON: Vmmap** - Display memory mappings in visualizer
- **TALON: Search Memory** - Find strings or hex patterns in memory
- **TALON: Telescope** - Follow pointer chains from an address

### Visual Tools:
- **TALON: Show Memory Visualizer** - Live stack/heap/memory viewer
  - Tabs for Stack, Heap, Mappings, Search, Telescope
  - Auto-highlight cyclic patterns, ROP gadgets, shellcode
  - Interactive hex dump with annotations
- **TALON: Show ROP Chain Builder** - Visual ROP chain construction
  - Scan binary for gadgets automatically
  - Drag-and-drop interface
  - One-click code generation
  - Null byte detection

### Phase 23 Commands:
- **TALON: Search Libc Database** - Search libc.rip for version identification
- **TALON: Auto-Find Buffer Offset** - Automatically find overflow offset
- **TALON: Insert Exploit Template** - Insert production-ready exploit template
- **TALON: Search for Flags** - Scan output for CTF flags
- **TALON: Analyze with GDB** - Run binary under GDB and parse crashes
- **TALON: Quick Exploitation Helper** - Display interactive helper guides

### Core Commands:
- **TALON: Run Exploit** (F5) - Execute current TALON script
- **TALON: Smart AI Assistant** - AI-powered exploit generation
- **TALON: Interactive Tutorials** - Step-by-step learning guides

## Keybindings

- **F5**: Start debugging (or run exploit if not debugging)
- **F10**: Step over (during debugging)
- **F11**: Step into (during debugging)
- **Shift+F11**: Step out (during debugging)
- **Ctrl+Shift+P**: Payload factory (weaponize PoC)

## Release Notes

### 3.1.0 (Phase 23 - January 2026)
- **NEW**: Libc.rip integration snippets and commands
- **NEW**: Auto-offset finding with GDB integration
- **NEW**: 13 additional exploit templates (ret2libc, heap exploits, SIGROP, etc.)
- **NEW**: Flag search and CTF platform submission
- **NEW**: Real GDB crash analysis and parsing
- **NEW**: Interactive quick-mode helper guides
- **IMPROVED**: Syntax highlighting for 250+ built-in functions
- **IMPROVED**: Auto-completion with all Phase 23 features
- **Total Snippets**: 24 (up from 10)

### 3.0.0
- Visual tools for exploit development
- AI-powered exploit generation
- Interactive tutorials system
- Research findings panel
- Workspace collaboration
- Complete syntax highlighting

### 1.0.0
- Initial release
- Complete syntax highlighting
- 10 code snippets
- Auto-completion support
- Bracket matching and auto-closing

## Contributing

Found a bug or want a feature? [Open an issue](https://github.com/talon-lang/vscode-talon/issues)

## License

MIT
