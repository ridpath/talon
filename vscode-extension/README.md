# TALON VS Code Extension

Comprehensive IDE support for the TALON exploit development language.

## Features

### Language Support
- Syntax highlighting for .talon files
- IntelliSense (autocomplete) for 138+ builtin functions
- Hover documentation with examples and mini-tutorials
- Signature help for function parameters
- Error diagnostics and warnings
- Code snippets for common exploit patterns

### Debugging
- Debug Adapter Protocol (DAP) integration
- Set breakpoints in TALON scripts
- Step through exploit code
- Inspect variables and memory state
- GDB integration for target binaries

### Visual Tools
- **Memory Visualizer**: View process memory layout
- **ROP Chain Builder**: Visual ROP gadget selection
- **Smart Assistant**: AI-powered exploit suggestions
- **Findings Panel**: Track discovered vulnerabilities
- **Interactive Tutorials**: Learn TALON interactively

### Commands
- `TALON: Run Exploit` (F5) - Execute current script
- `TALON: Load Template` - Insert exploit templates
- `TALON: Payload Factory` - Transform and obfuscate payloads
- `TALON: Live Process Attach` - Attach to running process
- `TALON: Checksec` - Analyze binary protections
- `TALON: Auto-Find Buffer Offset` - Cyclic pattern analysis
- `TALON: Search Libc Database` - libc.rip integration
- `TALON: Analyze with GDB` - Interactive debugging

## Installation

### Prerequisites

#### Windows
1. Install Rust toolchain:
   ```powershell
   winget install Rustlang.Rustup
   ```
2. Install Visual Studio Build Tools:
   ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
   ```
3. Restart terminal to refresh PATH

#### Linux (Ubuntu/Debian)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install build essentials
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev
```

#### Linux (Fedora/RHEL/CentOS)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install development tools
sudo dnf groupinstall "Development Tools"
sudo dnf install openssl-devel pkg-config
```

#### macOS
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Xcode Command Line Tools
xcode-select --install
```

### Build TALON Language Server

#### All Platforms
```bash
# Clone repository
git clone https://github.com/ridpath/talon.git
cd talon

# Build TALON compiler and language server
cargo build --release

# Verify installation
cargo test
```

### Install VS Code Extension

#### From Source
```bash
cd vscode-extension

# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Package extension (optional)
npm install -g vsce
vsce package

# Install .vsix file in VS Code
code --install-extension talon-language-*.vsix
```

#### From Marketplace (when published)
1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "TALON"
4. Click Install

## Configuration

### Extension Settings
- `talon.languageServer.path`: Path to TALON language server binary
- `talon.gdb.path`: Path to GDB executable (default: "gdb")
- `talon.docker.enabled`: Use Docker for sandboxed execution
- `talon.telemetry.enabled`: Send anonymous usage statistics

### Example settings.json
```json
{
  "talon.languageServer.path": "/path/to/talon-lsp",
  "talon.gdb.path": "/usr/bin/gdb",
  "talon.docker.enabled": true
}
```

## Usage

### Creating an Exploit
1. Create a new file: `exploit.talon`
2. Start typing - autocomplete will suggest functions
3. Use Command Palette (Ctrl+Shift+P) > "TALON: Load Template"
4. Press F5 to run the exploit

### Example Script
```talon
# Buffer overflow exploit
let offset = 264
let binary = "./vulnerable_app"

# Find ROP gadgets
let pop_rdi = rop_find(binary, "pop rdi; ret")[0]
let system_plt = analyze(binary).plt.system

# Build payload
let payload = cyclic(offset)
payload = payload + p64(pop_rdi)
payload = payload + p64(0x400123)  # "/bin/sh" string
payload = payload + p64(system_plt)

# Connect and exploit
let conn = connect("target.com", 1337)
send(conn, payload)
interactive(conn)
```

### Debugging
1. Set breakpoints by clicking left of line numbers
2. Press F5 or use Command Palette > "Debug: Start Debugging"
3. Use Debug toolbar to step through code
4. Inspect variables in Debug panel

## Keyboard Shortcuts

| Shortcut | Command |
|----------|---------|
| F5 | Run Exploit |
| Ctrl+Shift+P | Payload Factory |
| Ctrl+Shift+L | Load Template |
| Ctrl+Shift+M | Memory Visualizer |
| Ctrl+Shift+R | ROP Chain Builder |
| Ctrl+Shift+A | Smart Assistant |

## Troubleshooting

### Language Server Not Starting
1. Verify Rust installation: `rustc --version`
2. Build language server: `cargo build --release`
3. Check extension logs: Output > TALON Language Server

### Debugger Not Working
1. Verify GDB installation: `gdb --version`
2. Check debug configuration in `.vscode/launch.json`
3. Ensure target binary has debug symbols

### Autocomplete Not Working
1. Reload VS Code window (Ctrl+Shift+P > "Reload Window")
2. Check for TypeScript errors in Developer Tools (Help > Toggle Developer Tools)
3. Verify extension is activated: Extensions panel should show "TALON" as active

### Performance Issues
1. Disable unused extensions
2. Increase VS Code memory limit in settings
3. Use Docker mode for large binaries

## Development

### Building from Source
```bash
cd vscode-extension

# Install dependencies
npm install

# Watch mode (auto-compile on changes)
npm run watch

# Run extension in debug mode
# Press F5 in VS Code (Extension Development Host)
```

### Testing
```bash
# Run extension tests
npm test

# Lint TypeScript
npm run lint
```

### Project Structure
```
vscode-extension/
├── src/
│   ├── extension.ts          # Main extension entry point
│   ├── server.ts              # Language server client
│   ├── debugAdapter.ts        # Debug adapter implementation
│   ├── commands/              # VS Code commands
│   └── visualizers/           # Custom UI panels
├── syntaxes/
│   └── talon.tmLanguage.json  # Syntax highlighting
├── snippets/
│   └── talon.json            # Code snippets
├── package.json              # Extension manifest
└── tsconfig.json             # TypeScript config
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `npm test`
5. Submit a pull request

## License

MIT License

## Links

- [TALON Repository](https://github.com/ridpath/talon)
- [Documentation](https://docs.talon-lang.org)
- [Issue Tracker](https://github.com/ridpath/talon/issues)
- [Discord Community](https://discord.gg/talon)

## Credits

Developed by the TALON team for the offensive security research community.
