// Binary Patching Example - Demonstrates semantic binary modification
//
// This example shows how to use TALON's binary patching capabilities
// NOTE: Some examples are disabled due to a known interpreter stack overflow
// issue with async cleanup. The patch operations themselves work correctly.

// Load target binary for patching
let binary = "/tmp/target_binary"
let p = Patch(binary)

// Example 1: NOP out a security check
patch_nop_out(p, 0x1234, 10)

// Example 2: Replace a function call
patch_replace_call(p, 0x5678, "custom_exit")

// Example 3: Insert assembly code
patch_insert_asm(p, 0x9abc, "xor eax, eax; ret")

// Example 4: Patch strings
patch_patch_string(p, "example.com", "evil.com")

print("[+] Binary patching examples complete")
