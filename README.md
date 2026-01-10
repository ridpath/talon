# TALON — A Human‑Readable Scripting Language for Offensive Security

TALON is a security‑native, exploit‑centric, English‑like DSL designed for exploit developers, CTF competitors, red teamers, reverse engineers, and malware analysts.

**Status:** Alpha (active development; breaking changes expected)

---

## Quick Start

```bash
git clone https://github.com/ridpath/talon.git
cd talon
cargo build --release

./target/release/talon run examples/buffer_overflow.talon
./target/release/talon repl
```

---

## CLI Reference (Verified)

TALON uses a subcommand‑based CLI. Always use `--help` to discover what is available in your build.

```bash
talon --help
talon run --help
talon build --help
talon repl --help
talon analyze --help
talon fuzz --help
talon kernel --help
talon audit --help
talon web --help
talon template --help
```

If an interactive wizard exists, it will appear as its own subcommand in `talon --help`.

---

## Quick Helpers (REPL & Scripts)

Quick helpers are TALON functions, not CLI commands. Use them inside scripts or the REPL.

```talon
quick_shell("10.10.14.5", 1337)
quick_rop("./vuln")
quick_leak(conn)
quick_pwn("./vuln", "10.10.14.5", 1337)
quick_heap()
quick_fmt()
```

Launch REPL:

```bash
talon repl
```

---

## Templates

```bash
talon template ret2libc 10.10.14.5 1337 > exploit.talon
talon run exploit.talon
```

---

## Documentation & Help

### Man Pages
```bash
man talon
man talon-<topic>
```

### REPL Help
```talon
help()
help(search: "libc")
help(search: "rop")
help(search: "fmt")
```

---

## Modules & Plugin System

### Module Paths
- `talon_std/` — standard library
- `~/.talon/modules/` — user modules

```talon
import "custom_exploit"
```

### Native Plugins

```talon
load_plugin("plugins/my_plugin.so")
```

Extensions:
- Linux: `.so`
- macOS: `.dylib`
- Windows: `.dll`

---

## Filesystem Layout

- `~/.talon/libc/` — libc cache (libc.rip)
- `~/.talon/modules/` — user modules
- `~/.talon/cache/` — internal cache (if enabled)

---

## Common Workflows

### Analyze Binary
```bash
talon analyze ./vuln_binary
```

### Find Offset
```talon
let offset = auto_offset("./vuln")
print(offset)
```

### Debug Crash
```talon
let info = gdb_run("./vuln")
print(info.signal)
print(hex(info.rip))
```

---

## License

MIT License.

---

## Security Notice

For authorized security testing, CTFs, education, and research only.
