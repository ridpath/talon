# ═══════════════════════════════════════════════════════════════════════════
# NATURAL LANGUAGE EXAMPLES - English to TALON
# ═══════════════════════════════════════════════════════════════════════════
# This file shows how to think about exploits in plain English
# and translate them to TALON code

print("🗣️  Natural Language to TALON Translation Guide")
print("═══════════════════════════════════════════════════")

# ─────────────────────────────────────────────────────────────────────────────
# EXAMPLE 1: "Send a buffer overflow to crash the program"
# ─────────────────────────────────────────────────────────────────────────────
print("\n📝 EXAMPLE 1: Buffer Overflow")
print("English: Send 500 bytes to overflow the buffer")
print("TALON:")
print("  let overflow = cyclic(500)")
print("  send overflow")

# ─────────────────────────────────────────────────────────────────────────────
# EXAMPLE 2: "Find gadgets to build a ROP chain"
# ─────────────────────────────────────────────────────────────────────────────
print("\n📝 EXAMPLE 2: ROP Chain")
print("English: Find 'pop rdi; ret' and call system()")
print("TALON:")
print("  let pop_rdi = 0x401234")
print("  let system_addr = 0x7ffff7...")
print("  let chain = [pop_rdi, bin_sh, system_addr]")

# ─────────────────────────────────────────────────────────────────────────────
# EXAMPLE 3: "Leak a memory address from the output"
# ─────────────────────────────────────────────────────────────────────────────
print("\n📝 EXAMPLE 3: Memory Leak")
print("English: Extract address from position 10 in response")
print("TALON:")
print("  let response = receive()")
print("  let parts = split(response, ' ')")
print("  let leaked_addr = parts[10]")

# ─────────────────────────────────────────────────────────────────────────────
# EXAMPLE 4: "Try the exploit multiple times"
# ─────────────────────────────────────────────────────────────────────────────
print("\n📝 EXAMPLE 4: Brute Force / Retry")
print("English: Try exploit 100 times with different values")
print("TALON:")
print("  for attempt in 0..100")
print("      let payload = build_payload(attempt)")
print("      send payload")
print("      try")
print("          let response = receive()")
print("          if response == 'success'")
print("              break")
print("          end")
print("      catch err")
print("          continue")
print("      end")
print("  end")

# ─────────────────────────────────────────────────────────────────────────────
# EXAMPLE 5: "Parse the binary to find useful addresses"
# ─────────────────────────────────────────────────────────────────────────────
print("\n📝 EXAMPLE 5: Binary Analysis")
print("English: Find all references to '/bin/sh' in libc")
print("TALON:")
print("  # Use built-in tools (when integrated)")
print("  # analyze binary 'libc.so.6'")
print("  # find_string '/bin/sh'")

# ─────────────────────────────────────────────────────────────────────────────
# COMMON PATTERNS
# ─────────────────────────────────────────────────────────────────────────────
print("\n\n📚 COMMON PATTERNS:")
print("────────────────────────────────────────")

print("\n1️⃣  CONNECT AND SEND:")
print("   connect to 'target.com' on port 1337")
print("   send payload")

print("\n2️⃣  RECEIVE AND PARSE:")
print("   let data = receive()")
print("   let parts = split(data, ',')")

print("\n3️⃣  BUILD PAYLOAD:")
print("   let payload = padding + p64(address) + shellcode")

print("\n4️⃣  CONDITIONAL LOGIC:")
print("   if leaked_addr > 0x7f0000000000")
print("       print('Looks like a libc address!')")
print("   end")

print("\n5️⃣  LOOPS FOR AUTOMATION:")
print("   for i in 0..256")
print("       try_byte_value(i)")
print("   end")

print("\n═══════════════════════════════════════════════════")
print("💡 TIP: Start with small scripts and build up!")
print("   - Test each step separately")
print("   - Use print() to debug values")
print("   - Add try/catch for errors")
print("═══════════════════════════════════════════════════")
