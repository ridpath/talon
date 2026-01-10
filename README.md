# TALON — A Human-Readable Scripting Language for Offensive Security

TALON is a security native, exploit centric, English like domain specific language designed for exploit developers, CTF competitors, red teamers, reverse engineers, and security researchers.

This repository contains the TALON compiler, interpreter, standard library, exploit tooling, plugin system, and IDE integrations.

> **ALPHA NOTICE**: TALON is under **active development**. Expect breaking changes, experimental syntax, and rapid iteration. Not yet production-safe. Ideal for CTF competitions, red team labs, research, or prototyping offensive techniques.

![status: alpha](https://img.shields.io/badge/status-alpha-yellow)
![compilation: passing](https://img.shields.io/badge/build-0%20errors-brightgreen)
![platform: windows+linux](https://img.shields.io/badge/platform-windows%20%7C%20linux-blue)
---

## Key Capabilities

- Human-readable exploit DSL
- Native compilation via Rust/LLVM
- Built-in exploitation primitives (ROP, heap, kernel, format strings)
- Libc database integration (libc.rip)
- Automatic buffer overflow offset discovery (GDB)
- Exploit templates and interactive quick helpers
- Integrated binary analysis (ELF, PE, Mach-O)
- Modular standard library (138+ modules)
- Plugin system (TALON modules + native Rust plugins)
- VS Code IDE extension with debugger and visual tools

---

## Quick Start

```bash
git clone https://github.com/ridpath/talon.git
cd talon
cargo build --release
./target/release/talon repl
```

---

## Core CLI Commands

```bash
talon run <script.talon>
talon build <script.talon>
talon repl
talon analyze <binary>
talon fuzz <binary>
talon kernel exploit <CVE-ID>
talon audit <contract.sol>
talon web scan <url>
talon template <name> <args>
```

---

## Interactive Helpers

```talon
quick_shell("host", port)
quick_rop("./binary")
quick_pwn("./binary", "host", port)
quick_heap()
quick_fmt()
```

---

## Plugin System

### Custom TALON Modules

```talon
define function my_helper(arg)
    print(arg)
end
```

### Native Rust Plugins

```talon
load_plugin("plugins/my_plugin.so")
```

---

## VS Code Extension

See `README.md in vscode-extensions directory` for IDE features, debugging, and visual tools.

---

## License

MIT License

## Built-in Functions Reference

A short quick-reference is included in this repo:

- `BUILTIN_FUNCTIONS_REFERENCE.md` - Quick reference for core language helpers (collections, conversion, file I/O, packing/unpacking, and common utilities)

For the full catalog (exploitation, analysis, fuzzing, kernel, web, blockchain, forensics, templates, and helpers), see the "Built-in Functions" section in this README and the in-tool docs:

- `talon repl` then `help()` or `help(search: "keyword")`
- `man talon` and `man talon-<topic>`
