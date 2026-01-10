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

Note: TALON includes many more built-ins across exploitation, analysis, fuzzing, and tooling. See the main README "Built-in Functions" section for the full category listing.
