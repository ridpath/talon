# TALON DSL - Built-in Functions Quick Reference

Complete guide to commonly used built-in functions.

## Collection Functions

### `len(collection)`
Returns the length/size of a collection.

Supported types: List, String, Bytes, Map, Set

```talon
len([1, 2, 3, 4, 5])         # -> 5
len("hello world")           # -> 11
len(p64(0xdeadbeef))         # -> 8
len({a: 1, b: 2, c: 3})      # -> 3
```

## Sequence Generation

### `range(end)`
### `range(start, end)`
Generates a sequence of numbers.

```talon
range(5)                     # -> [0, 1, 2, 3, 4]
range(3, 8)                  # -> [3, 4, 5, 6, 7]
range(10, 15)                # -> [10, 11, 12, 13, 14]

for i in range(10)
    print("Iteration", i)
end

let nums = range(1, 100)
let sum = 0
for n in nums
    sum = sum + n
end
```

## Type Conversions

### `hex(number)`
Converts a number to a hexadecimal string.

```talon
hex(255)                     # -> "0xff"
hex(0x08048000)              # -> "0x8048000"
hex(4195222)                 # -> "0x400686"

let addr = 0x401000
print("Address:", hex(addr))
```

### `int(string)`
Parses a string to an integer (supports hex and decimal).

```talon
int("12345")                 # -> 12345
int("0xdeadbeef")            # -> 3735928559
int("0xFF")                  # -> 255

let user_input = "0x400000"
let base_addr = int(user_input)
```

### `bytes(value)`
Converts various types to byte arrays.

```talon
bytes("hello")                    # -> [104, 101, 108, 108, 111]
bytes([72, 101, 108, 108, 111])   # -> [72, 101, 108, 108, 111]
bytes(65)                         # -> [65] ('A')

let payload = bytes("AAAA") + p64(ret_addr)
```

### `str(value)`
Converts any value to its string representation.

```talon
str(12345)                   # -> "12345"
str(0xdead)                  # -> "57005"
str([1, 2, 3])               # -> "[1, 2, 3]"

let data = bytes("hello")
let text = str(data)

let msg = "Port: " + str(4444)
```

## File I/O

### `read(filepath)`
Reads file contents as bytes.

```talon
let shellcode = read("shellcode.bin")
let config = str(read("config.txt"))

let payload = read("payload.bin")
send(conn, payload)
```

### `write(filepath, data)`
Writes data to a file (creates or overwrites).

Returns: Number of bytes written

```talon
write("output.txt", "Hello World!")

let payload = cyclic(100) + p64(0xdeadbeef)
write("exploit.bin", payload)

write("result.txt", 12345)
```

## String Manipulation

### `split(string, delimiter)`
Splits a string into a list.

```talon
split("one,two,three", ",")
split("192.168.1.1", ".")

let csv = "10,20,30,40"
let numbers = split(csv, ",")
```

### `join(list, separator)`
Joins a list into a string.

```talon
join(["a", "b", "c"], "-")
join([1, 2, 3], ",")

let parts = ["exploit", "py"]
let filename = join(parts, ".")
```

### `replace(string, old, new)`
Replaces all occurrences of a substring.

```talon
replace("hello world", "world", "TALON")
replace("192.168.1.1", ".", "_")

let template = str(read("template.txt"))
let exploit = replace(template, "{{TARGET}}", "192.168.1.100")
```

## Output

### `print(value1, value2, ...)`
Prints values to stdout (space-separated).

```talon
print("Hello World")
print("Address:", hex(0x400000))
print("Size:", len(payload), "bytes")
print("Target:", host, "Port:", port)
```

## Binary Packing

### `p64(number)` / `p32(number)` / `p16(number)` / `p8(number)`
Packs numbers to little-endian bytes.

```talon
p64(0xdeadbeef)
p32(0x08048000)
p16(0x1234)
p8(0x41)

let rop = p64(pop_rdi) + p64(bin_sh) + p64(system)
```

### `u64(bytes)` / `u32(bytes)` / `u16(bytes)` / `u8(bytes)`
Unpacks bytes to numbers (little-endian).

```talon
u64([0xef, 0xbe, 0xad, 0xde, 0, 0, 0, 0])
u32([0x00, 0x80, 0x04, 0x08])

let leaked = recv(conn, 8)
let addr = u64(leaked)
print("Leaked address:", hex(addr))
```

## Binary Analysis

### `Elf(path)`
Loads and analyzes an ELF binary, returning an object with all relevant information.

Returns: Map with `plt`, `got`, `symbols`, `nx`, `pie`, `canary`, `relro`, `fortify`, `base_addr`, `path`

```talon
let elf = Elf("./vuln")
let main = elf["symbols"]["main"]
let puts_plt = elf["plt"]["puts"]
let libc_got = elf["got"]["__libc_start_main"]

print("[*] Binary base:", hex(elf["base_addr"]))
print("[*] PIE:", elf["pie"], "| NX:", elf["nx"], "| Canary:", elf["canary"])
```

### `analyze(path)`
Alias for `Elf()`. Loads and analyzes an ELF binary.

```talon
let binary = analyze("./target")
print("Entry point:", hex(binary["symbols"]["_start"]))
```

### `checksec(path)`
Displays security features of a binary.

```talon
checksec("./vuln")
```

## Libc Database

### `Libc(version)`
Loads libc offsets for a specific version. Returns offsets relative to base address 0.

Returns: Map with `symbols`, `one_gadgets`, `name`, `build_id`, `base`

Available versions: `ubuntu18.04`, `ubuntu20.04`, `ubuntu22.04`, `debian10`

```talon
let libc = Libc("ubuntu20.04")
let system_offset = libc["symbols"]["system"]
let binsh_offset = libc["symbols"]["bin_sh"]
let one_gadgets = libc["one_gadgets"]

print("[*] Libc:", libc["name"])
print("[*] Build ID:", libc["build_id"])
print("[*] One-gadgets:", one_gadgets)
```

### `Libc({version, base})`
Loads libc with a specific base address. Returns absolute addresses.

```talon
let libc_base = leaked_addr - 0x21b10
let libc_resolved = Libc({version: "ubuntu20.04", base: libc_base})
let system_addr = libc_resolved["symbols"]["system"]
let binsh_addr = libc_resolved["symbols"]["bin_sh"]

print("[*] system():", hex(system_addr))
print("[*] /bin/sh:", hex(binsh_addr))
```

Available symbols: `system`, `execve`, `sh`, `bin_sh`, `dup2`, `read`, `write`, `open`, `mprotect`, `__malloc_hook`, `__free_hook`, `__realloc_hook`

## ROP Gadgets

### `ROP(elf_obj | path)`
Creates a ROP chain builder with automatic gadget discovery.

Returns: Map with `binary`, `gadgets`, `gadget_count`

```talon
let elf = Elf("./vuln")
let rop = ROP(elf)

print("[*] Found", rop["gadget_count"], "gadgets")
```

### `find(rop_obj, pattern)`
Searches for gadgets matching a pattern (case-insensitive).

Returns: Address of first matching gadget

```talon
let rop = ROP("./binary")
let pop_rdi = find(rop, "pop rdi; ret")
let pop_rsi = find(rop, "pop rsi")
let ret = find(rop, "ret")

let payload = cyclic(264) + p64(pop_rdi) + p64(binsh) + p64(system)
```

### `quick_rop(path)`
Quick ROP gadget finder. Returns ROP object with all gadgets.

```talon
let rop = quick_rop("./vuln")
let gadget = find(rop, "pop rax")
```

## Exploit Patterns

### `cyclic(length)`
Generates a De Bruijn sequence for finding buffer overflow offsets.

```talon
let pattern = cyclic(200)
send(conn, pattern)

let offset = cyclic_find("daab")
print("Offset:", offset)
```

## CTF Exploitation Functions

### `shellcode(arch, type)`
Retrieves pre-built shellcode from the integrated shellcode database.

Returns: Map with `bytes` (shellcode bytes), `size` (length in bytes), `description` (shellcode purpose)

Supported architectures: `x86`, `x64`, `arm`, `arm64`, `mips`
Supported types: `execve`, `shell`, `setuid`, `read_flag`, `reverse_shell`, `bind_shell`

```talon
let sc = shellcode("x64", "execve")
print("[*] Shellcode:", sc["description"])
print("[*] Size:", sc["size"], "bytes")
send(conn, sc["bytes"])

let arm_shell = shellcode("arm", "shell")
let payload = padding + arm_shell["bytes"]
```

### `fmtstr_write(offset, writes)`
Generates a format string payload for arbitrary memory writes.

Parameters:
- `offset` - Format string offset (position on stack)
- `writes` - Map of {address: value} pairs to write

Returns: String containing the format string payload

```talon
let offset = 6
let got_exit = 0x0804a020
let system_addr = 0xf7e50da0

let payload = fmtstr_write(offset, {got_exit: system_addr})
send(conn, payload)

let multiple_writes = fmtstr_write(8, {
    0x0804a020: 0xdeadbeef,
    0x0804a024: 0xcafebabe
})
```

### `find_fmt_offset(conn, marker)` (stdlib function)
Automatically discovers the format string offset by sending test patterns.

**Note**: This is a stdlib helper function. Include `stdlib/ctf_helpers.talon` to use it.

Parameters:
- `conn` - Active connection handle
- `marker` - Test marker value (e.g., 0x41414141)

Returns: Integer offset where format string reads from stack

```talon
include "stdlib/ctf_helpers.talon"

let conn = connect("pwn.chal.ctf", 9999)
let offset = find_fmt_offset(conn, 0x41414141)
print("[+] Format offset:", offset)

let leak_payload = "%{}$p".format(offset + 2)
send(conn, leak_payload)
```

## Network I/O

### `connect(host, port)`
Establishes a TCP connection.

Returns: Connection handle

```talon
let conn = connect("pwn.chal.ctf", 1337)
let local = connect("127.0.0.1", 4444)
```

### `send(conn, data)`
Sends data over a connection.

```talon
send(conn, "AAAA")
send(conn, p64(0xdeadbeef))
send(conn, payload)
```

### `recv(conn, size)`
Receives specified number of bytes.

Returns: Bytes received

```talon
let data = recv(conn, 1024)
let leak = recv(conn, 8)
let addr = u64(leak)
```

### `recvuntil(conn, delimiter)`
Receives data until a delimiter is found.

Returns: All data up to and including the delimiter

```talon
let banner = recvuntil(conn, "\n")
let prompt = recvuntil(conn, "> ")
let response = recvuntil(conn, "Enter choice:")
```

### `interactive(conn)`
Drops into an interactive shell with bidirectional I/O.

```talon
send(conn, payload)
interactive(conn)
```

## Function Categories Summary

| Category | Functions |
|----------|-----------|
| Collections | `len()` |
| Sequences | `range()` |
| Type Conversion | `hex()`, `int()`, `bytes()`, `str()` |
| File I/O | `read()`, `write()` |
| String Ops | `split()`, `join()`, `replace()` |
| Output | `print()` |
| Binary Pack | `p64()`, `p32()`, `p16()`, `p8()` |
| Binary Unpack | `u64()`, `u32()`, `u16()`, `u8()` |
| Binary Analysis | `Elf()`, `analyze()`, `checksec()` |
| Libc Database | `Libc()` |
| ROP Gadgets | `ROP()`, `find()`, `quick_rop()` |
| Exploit Patterns | `cyclic()`, `cyclic_find()` |
| CTF Exploitation | `shellcode()`, `fmtstr_write()`, `find_fmt_offset()` (stdlib) |
| Network I/O | `connect()`, `send()`, `recv()`, `recvuntil()`, `interactive()` |

Note: TALON includes many more built-ins across exploitation, analysis, fuzzing, and tooling. See the main README "Built-in Functions" section for the full category listing.
