# TALON DSL - Built-in Functions Quick Reference

**Complete guide to all built-in functions** ⚡

---

## 📏 Collection Functions

### `len(collection)`
Returns the length/size of a collection.

**Supported Types**: List, String, Bytes, Map, Set

```talon
len([1, 2, 3, 4, 5])         # → 5
len("hello world")           # → 11
len(p64(0xdeadbeef))         # → 8
len({a: 1, b: 2, c: 3})      # → 3
```

---

## 🔢 Sequence Generation

### `range(end)`
### `range(start, end)`
Generates a sequence of numbers.

```talon
range(5)                     # → [0, 1, 2, 3, 4]
range(3, 8)                  # → [3, 4, 5, 6, 7]
range(10, 15)                # → [10, 11, 12, 13, 14]

# Use in loops
for i in range(10)
    print("Iteration", i)
end

# Use in expressions
let nums = range(1, 100)
let sum = 0
for n in nums
    sum = sum + n
end
```

---

## 🔄 Type Conversions

### `hex(number)`
Converts a number to a hexadecimal string.

```talon
hex(255)                     # → "0xff"
hex(0x08048000)              # → "0x8048000"
hex(4195222)                 # → "0x400686"

let addr = 0x401000
print("Address:", hex(addr)) # → "Address: 0x401000"
```

---

### `int(string)`
Parses a string to an integer (supports hex and decimal).

```talon
int("12345")                 # → 12345
int("0xdeadbeef")            # → 3735928559
int("0xFF")                  # → 255

let user_input = "0x400000"
let base_addr = int(user_input)
```

---

### `bytes(value)`
Converts various types to byte arrays.

```talon
# From string
bytes("hello")               # → [104, 101, 108, 108, 111]

# From list of numbers
bytes([72, 101, 108, 108, 111])  # → [72, 101, 108, 108, 111]

# From single number
bytes(65)                    # → [65]  ('A')

# Usage example
let payload = bytes("AAAA") + p64(ret_addr)
```

---

### `str(value)`
Converts any value to its string representation.

```talon
str(12345)                   # → "12345"
str(0xdead)                  # → "57005"
str([1, 2, 3])               # → "[1, 2, 3]"

# Convert bytes to string
let data = bytes("hello")
let text = str(data)         # → "hello"

# Convert numbers for concatenation
let msg = "Port: " + str(4444)
```

---

## 📁 File I/O

### `read(filepath)`
Reads file contents as bytes.

```talon
# Read binary file
let shellcode = read("shellcode.bin")

# Read text file (convert to string)
let config = str(read("config.txt"))

# Read exploit payload
let payload = read("payload.bin")
send(conn, payload)
```

---

### `write(filepath, data)`
Writes data to a file (creates or overwrites).

```talon
# Write string
write("output.txt", "Hello World!")

# Write bytes
let payload = cyclic(100) + p64(0xdeadbeef)
write("exploit.bin", payload)

# Write numbers (auto-converted to string)
write("result.txt", 12345)

# Chaining operations
let exploit = read("template.bin")
# ... modify exploit ...
write("final_exploit.bin", exploit)
```

**Returns**: Number of bytes written

---

## 🔤 String Manipulation

### `split(string, delimiter)`
Splits a string into a list.

```talon
split("one,two,three", ",")  # → ["one", "two", "three"]
split("192.168.1.1", ".")    # → ["192", "168", "1", "1"]

let csv = "10,20,30,40"
let numbers = split(csv, ",")
```

---

### `join(list, separator)`
Joins a list into a string.

```talon
join(["a", "b", "c"], "-")   # → "a-b-c"
join([1, 2, 3], ",")         # → "1,2,3"

let parts = ["exploit", "py"]
let filename = join(parts, ".")  # → "exploit.py"
```

---

### `replace(string, old, new)`
Replaces all occurrences of a substring.

```talon
replace("hello world", "world", "TALON")  # → "hello TALON"
replace("192.168.1.1", ".", "_")          # → "192_168_1_1"

let template = read("template.txt")
let exploit = replace(template, "{{TARGET}}", "192.168.1.100")
```

---

## 🖨️ Output

### `print(value1, value2, ...)`
Prints values to stdout (space-separated).

```talon
print("Hello World")
print("Address:", hex(0x400000))
print("Size:", len(payload), "bytes")

# Multiple arguments
print("Target:", host, "Port:", port)
```

---

## 📦 Binary Packing

### `p64(number)` / `p32(number)` / `p16(number)` / `p8(number)`
Packs numbers to little-endian bytes.

```talon
p64(0xdeadbeef)              # → [0xef, 0xbe, 0xad, 0xde, 0, 0, 0, 0]
p32(0x08048000)              # → [0x00, 0x80, 0x04, 0x08]
p16(0x1234)                  # → [0x34, 0x12]
p8(0x41)                     # → [0x41]

# Build ROP chain
let rop = p64(pop_rdi) + p64(bin_sh) + p64(system)
```

---

### `u64(bytes)` / `u32(bytes)` / `u16(bytes)` / `u8(bytes)`
Unpacks bytes to numbers (little-endian).

```talon
u64([0xef, 0xbe, 0xad, 0xde, 0, 0, 0, 0])  # → 3735928559
u32([0x00, 0x80, 0x04, 0x08])              # → 134512640

# Parse leaked address
let leaked = recv(conn, 8)
let addr = u64(leaked)
print("Leaked address:", hex(addr))
```

---

## 🔗 Practical Examples

### Example 1: Generate Padding
```talon
let offset = 264
let padding = bytes("A") * offset
# or use cyclic pattern
let padding = cyclic(offset)
```

### Example 2: Build Exploit Payload
```talon
let offset = 264
let ret_addr = 0x08048ABC
let shellcode = shellcode(arch: "x64", payload: "execve")

let payload = cyclic(offset) + p64(ret_addr) + shellcode
write("payload.bin", payload)
```

### Example 3: Parse CSV Configuration
```talon
let config_data = str(read("targets.csv"))
let lines = split(config_data, "\n")

for line in lines
    let parts = split(line, ",")
    let host = parts[0]
    let port = int(parts[1])
    print("Target:", host, "Port:", port)
end
```

### Example 4: Brute Force with Range
```talon
let base_addr = 0x400000

for offset in range(0, 256)
    let test_addr = base_addr + offset
    print("Trying:", hex(test_addr))
    # ... test exploit ...
end
```

### Example 5: File Operations
```talon
# Read, modify, write pattern
let original = read("shellcode.bin")
let modified = original + p64(0xdeadbeef)
write("modified_shellcode.bin", modified)

# Check sizes
print("Original size:", len(original))
print("Modified size:", len(modified))
```

---

## 🎓 Tips & Tricks

### Chaining Conversions
```talon
# String → Bytes → Modify → String
let text = "hello"
let data = bytes(text)
# ... modify data ...
let result = str(data)
```

### Dynamic File Names
```talon
for i in range(10)
    let filename = "payload_" + str(i) + ".bin"
    write(filename, cyclic(100 + i * 10))
end
```

### Hex Literal Shortcuts
```talon
# All these work!
let addr1 = 0x08048ABC
let addr2 = int("0x08048ABC")
let addr3 = 134515388

# Prefer hex literals for readability
let rop_chain = [
    0x400686,  # pop rdi; ret
    0x400687,  # pop rsi; ret
    0x400285   # ret
]
```

---

## 📚 Function Categories Summary

| Category | Functions |
|----------|-----------|
| **Collections** | `len()` |
| **Sequences** | `range()` |
| **Type Conversion** | `hex()`, `int()`, `bytes()`, `str()` |
| **File I/O** | `read()`, `write()` |
| **String Ops** | `split()`, `join()`, `replace()` |
| **Output** | `print()` |
| **Binary Pack** | `p64()`, `p32()`, `p16()`, `p8()` |
| **Binary Unpack** | `u64()`, `u32()`, `u16()`, `u8()` |

---

**Total Built-in Functions**: 18+

All functions are fully tested and production-ready! ✅
