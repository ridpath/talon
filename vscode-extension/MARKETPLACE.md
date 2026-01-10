# TALON DSL - VS Code Extension Marketplace Guide

## Marketplace Listing

### Display Name
**TALON DSL - Exploit Development Language**

### Description
Exploit development with Phase 23 features: libc.rip integration, auto-offset finding, 16 exploit templates, flag automation, real GDB parsing, and interactive quick-mode helpers. CTF-focused scripting language.

### Categories
- Programming Languages
- Snippets
- Debuggers
- Other

### Tags
talon, exploit, pwn, ctf, security, pentesting, binary exploitation, reverse engineering, ctf automation, libc database, heap exploitation, rop chains

## Key Features for Marketplace

### Phase 23 - CTF Focused Features

1. **Libc.rip Integration**
   - Automatic libc version identification from leaked addresses
   - One-click search across 1000+ libc versions
   - Instant symbol resolution (system, execve, /bin/sh, etc.)
   - Downloads cached locally for offline use

2. **Auto-Offset Finding**
   - Automated buffer overflow offset detection
   - Integrates with GDB for crash analysis
   - Supports stdin, args, and file input methods
   - Extracts instruction pointer from core dumps

3. **16 Production-Ready Templates**
   - ret2libc (full 2-stage with leak)
   - Heap exploits (tcache poison, fastbin dup, house-of-*)
   - ROP techniques (SIGROP, stack pivot, one-gadget)
   - Format string exploitation
   - All templates use real TALON functions

4. **Flag Automation**
   - Auto-detect CTF flags (flag{}, HTB{}, picoCTF{}, etc.)
   - Submit to CTFd, HackTheBox, and custom platforms
   - Supports custom regex patterns
   - Recursive directory scanning

5. **Real GDB Integration**
   - Parse GDB crash output (registers, backtrace, signals)
   - Automatic offset calculation from crashes
   - Support for GDB/MI (Machine Interface)
   - SIGSEGV, SIGABRT, SIGILL detection

6. **Interactive Quick-Mode Helpers**
   - quick_shell(): Instant reverse shell guide
   - quick_rop(): Step-by-step ROP building
   - quick_pwn(): Complete exploit generation
   - quick_heap(): Heap exploitation cookbook
   - quick_fmt(): Format string reference

### 24 Code Snippets

Type these prefixes and press Tab to expand:

**Phase 23 (NEW):**
- `libc-leak` - Leak and identify libc with libc.rip
- `auto-offset` - Automatic offset finding
- `ret2libc` - Full ret2libc exploit
- `flag-search` - Flag search and submit
- `gdb-crash` - GDB crash analysis
- `quick-shell`, `quick-rop`, `quick-pwn` - Interactive helpers
- `tcache-poison`, `fastbin-dup` - Heap exploits
- `one-gadget`, `sigrop`, `stack-pivot` - Advanced ROP

**Classic:**
- `exploit-bof` - Buffer overflow
- `rop-chain` - ROP chain builder
- `exploit-fmt` - Format string
- `shellcode` - Shellcode generation
- And 10+ more...

### Visual Tools

- **Memory Visualizer**: Real-time memory layout viewer
- **ROP Chain Builder**: Drag-and-drop ROP construction
- **Live Debugger Panel**: Integrated debugging interface
- **Visual Exploit Builder**: No-code exploit generation
- **Smart AI Assistant**: AI-powered exploit suggestions
- **Interactive Tutorials**: Step-by-step learning

### Commands & Keybindings

**Command Palette:**
- TALON: Search Libc Database
- TALON: Auto-Find Buffer Offset
- TALON: Insert Exploit Template
- TALON: Search for Flags
- TALON: Analyze with GDB
- TALON: Quick Exploitation Helper

**Keybindings:**
- `F5` - Run current exploit
- `Ctrl+Shift+P` - Payload factory

## Screenshots Needed

1. **Syntax Highlighting**: Show colorized TALON code
2. **Snippet Completion**: Tab expansion demo
3. **Libc Search**: libc_search() in action
4. **Auto-Offset**: auto_offset() output
5. **Template Gallery**: Show available templates
6. **Visual Builder**: ROP chain builder screenshot
7. **Quick Helper**: quick_pwn() output

## Publishing Checklist

- [x] package.json updated to 3.1.0
- [x] README.md with Phase 23 features
- [x] 24 code snippets (13 new)
- [x] 6 new commands added
- [x] Syntax highlighting updated
- [ ] Icon.png (128x128, 256x256 recommended)
- [ ] Screenshots (5-7 images)
- [ ] LICENSE file (MIT)
- [ ] .vscodeignore configured
- [ ] Test in VS Code locally
- [ ] Build VSIX: `vsce package`
- [ ] Publish: `vsce publish`

## Publisher Setup

1. Create Publisher Account:
   - Go to https://marketplace.visualstudio.com/manage
   - Sign in with Microsoft/GitHub account
   - Create publisher (e.g., "talon-dev")

2. Get Personal Access Token:
   - Go to https://dev.azure.com/
   - User Settings > Personal Access Tokens
   - Create token with "Marketplace (Publish)" scope

3. Login with vsce:
   ```bash
   npm install -g vsce
   vsce login talon-dev
   ```

4. Package Extension:
   ```bash
   cd vscode-extension
   npm install
   npm run compile
   vsce package
   ```

5. Publish:
   ```bash
   vsce publish
   ```

## Support & Links

- **GitHub**: https://github.com/talon-lang/vscode-talon
- **Issues**: https://github.com/talon-lang/vscode-talon/issues
- **Documentation**: https://talon-lang.org/docs
- **Discord**: https://discord.gg/talon-lang

## Version History

### 3.1.0 (January 2026) - Phase 23 Release
- **NEW**: Libc.rip database integration
- **NEW**: Automatic offset finding with GDB
- **NEW**: 13 additional exploit templates
- **NEW**: Flag search and CTF platform submission
- **NEW**: Real GDB crash analysis
- **NEW**: Interactive quick-mode helpers
- 250+ built-in functions
- 24 code snippets

### 3.0.0 (December 2025)
- Visual tools for exploit development
- AI-powered generation
- Interactive tutorials
- Workspace collaboration

### 1.0.0 (Initial Release)
- Syntax highlighting
- 10 basic snippets
- Auto-completion
