print("[*] TALON Exploitation Framework Demo")
print("[*] No binary required - showcasing API capabilities")
print("=" * 50)

print("\n[1] Type Conversion Functions")
print("-" * 50)
let addr = 0xdeadbeef
print("Address value:", hex(addr))
print("Decimal:", addr)
print("Parsed from string:", int("0x401000"))

print("\n[2] Binary Packing Operations")
print("-" * 50)
let packed_64 = p64(0x401234567890)
print("p64(0x401234567890) length:", len(packed_64), "bytes")
let unpacked = u64(packed_64)
print("Round-trip u64(packed):", hex(unpacked))

let packed_32 = p32(0xdeadbeef)
print("p32(0xdeadbeef) length:", len(packed_32), "bytes")

print("\n[3] Cyclic Pattern Generation")
print("-" * 50)
let pattern = cyclic(200)
print("Generated cyclic pattern:", len(pattern), "bytes")
print("Used for finding buffer overflow offsets")

print("\n[4] Libc Database Lookup")
print("-" * 50)
let libc = Libc("ubuntu20.04")
print("Loaded:", libc["name"])
print("Build ID:", libc["build_id"])
print("")
print("Symbol offsets:")
print("  system():", hex(libc["symbols"]["system"]))
print("  execve():", hex(libc["symbols"]["execve"]))
print("  /bin/sh string:", hex(libc["symbols"]["bin_sh"]))
print("  __malloc_hook:", hex(libc["symbols"]["__malloc_hook"]))
print("  __free_hook:", hex(libc["symbols"]["__free_hook"]))
print("")
print("One-gadgets available:", len(libc["one_gadgets"]))

print("\n[5] Libc with Base Address")
print("-" * 50)
let leaked_base = 0x7ffff7a00000
print("Simulated leaked libc base:", hex(leaked_base))
let libc_resolved = Libc({version: "ubuntu20.04", base: leaked_base})
print("Resolved system():", hex(libc_resolved["symbols"]["system"]))
print("Resolved /bin/sh:", hex(libc_resolved["symbols"]["bin_sh"]))

print("\n[6] Building ROP Payload")
print("-" * 50)
let offset = 264
let pop_rdi_gadget = 0x401234
let system_addr = libc_resolved["symbols"]["system"]
let binsh_addr = libc_resolved["symbols"]["bin_sh"]

print("Buffer offset:", offset, "bytes")
print("Gadget (pop rdi; ret):", hex(pop_rdi_gadget))
print("")

let payload = cyclic(offset)
payload = payload + p64(pop_rdi_gadget)
payload = payload + p64(binsh_addr)
payload = payload + p64(system_addr)

print("Constructed ROP chain:")
print("  1. Padding:", offset, "bytes")
print("  2. pop rdi; ret")
print("  3. /bin/sh address")
print("  4. system() address")
print("")
print("Total payload size:", len(payload), "bytes")

print("\n[7] File Operations")
print("-" * 50)
write("demo_payload.bin", payload)
print("Written payload to: demo_payload.bin")
let read_back = read("demo_payload.bin")
print("Read back:", len(read_back), "bytes")
print("Integrity check:", len(payload) == len(read_back))

print("\n[+] Demo Complete!")
print("")
print("Next steps:")
print("  1. Build vuln_binary: cd examples && make")
print("  2. Run: talon run 01_basic_overflow.talon")
print("  3. Test exploit with real binary")
