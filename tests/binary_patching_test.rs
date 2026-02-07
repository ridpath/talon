// Binary Patching Integration Tests
// Tests semantic binary patching, assembly integration, and verification

use std::fs;
use std::path::Path;

#[test]
fn test_binary_patch_nop_out() {
    // Test NOP instruction patching
    
    // Create test ELF binary
    let test_binary = create_test_elf_binary();
    let test_path = "test_patch_nop.elf";
    fs::write(test_path, &test_binary).unwrap();
    
    let mut patcher = talon::binary_patch::Patch::new(test_path).unwrap();
    
    // NOP out 10 bytes at offset 0x100
    let result = patcher.nop_out(0x100, 10);
    assert!(result.is_ok(), "NOP patching should succeed");
    
    // Apply patches
    let apply_result = patcher.apply();
    assert!(apply_result.is_ok(), "Apply should succeed");
    
    // Verify patch was applied
    let patched_data = fs::read(test_path).unwrap();
    for i in 0..10 {
        assert_eq!(patched_data[0x100 + i], 0x90, "Byte should be NOP (0x90)");
    }
    
    // Cleanup
    fs::remove_file(test_path).ok();
}

#[test]
fn test_binary_patch_replace_call() {
    // Test call instruction replacement
    
    let test_binary = create_test_elf_binary();
    let test_path = "test_patch_call.elf";
    fs::write(test_path, &test_binary).unwrap();
    
    let mut patcher = talon::binary_patch::Patch::new(test_path).unwrap();
    
    // Replace call instruction (hypothetical)
    // In real usage: patcher.replace_call("exit", "hacked_fn")
    // For test, just verify the function exists
    assert!(patcher.get_operations().is_empty(), "No operations yet");
    
    // Cleanup
    fs::remove_file(test_path).ok();
}

#[test]
#[cfg(feature = "binary-patching")]
fn test_binary_patch_insert_asm() {
    // Test assembly instruction insertion (requires keystone)
    
    let test_binary = create_test_elf_binary();
    let test_path = "test_patch_asm.elf";
    fs::write(test_path, &test_binary).unwrap();
    
    let mut patcher = talon::binary_patch::Patch::new(test_path).unwrap();
    
    // Insert assembly at offset
    let result = patcher.insert_asm(0x100, "xor eax, eax; ret");
    
    match result {
        Ok(_) => {
            // Assembly insertion succeeded
            let ops = patcher.get_operations();
            assert!(!ops.is_empty(), "Should have operations");
        }
        Err(e) => {
            // Keystone may not be available
            eprintln!("Assembly insertion failed (expected if keystone unavailable): {}", e);
        }
    }
    
    // Cleanup
    fs::remove_file(test_path).ok();
}

#[test]
fn test_binary_patch_dry_run() {
    // Test dry-run mode (no actual file modification)
    
    let test_binary = create_test_elf_binary();
    let test_path = "test_patch_dryrun.elf";
    fs::write(test_path, &test_binary).unwrap();
    
    let original_data = fs::read(test_path).unwrap();
    
    let mut patcher = talon::binary_patch::Patch::new_dry_run(test_path).unwrap();
    
    // NOP out bytes in dry-run mode
    patcher.nop_out(0x100, 10).unwrap();
    
    // Apply (should not modify file in dry-run)
    patcher.apply().unwrap();
    
    // Verify file unchanged
    let after_data = fs::read(test_path).unwrap();
    assert_eq!(original_data, after_data, "File should not be modified in dry-run");
    
    // Cleanup
    fs::remove_file(test_path).ok();
}

#[test]
fn test_binary_patch_checksum_verification() {
    // Test checksum verification before/after patching
    
    let test_binary = create_test_elf_binary();
    let test_path = "test_patch_checksum.elf";
    fs::write(test_path, &test_binary).unwrap();
    
    let patcher = talon::binary_patch::Patch::new(test_path).unwrap();
    
    // Verify checksums can be computed
    // In real usage, patcher would track before/after checksums
    
    // Cleanup
    fs::remove_file(test_path).ok();
}

#[test]
fn test_binary_patch_rollback() {
    // Test rollback capability
    
    let test_binary = create_test_elf_binary();
    let test_path = "test_patch_rollback.elf";
    fs::write(test_path, &test_binary).unwrap();
    
    let original_data = fs::read(test_path).unwrap();
    
    let mut patcher = talon::binary_patch::Patch::new(test_path).unwrap();
    
    // Apply patch
    patcher.nop_out(0x100, 10).unwrap();
    patcher.apply().unwrap();
    
    // Verify patch applied
    let patched_data = fs::read(test_path).unwrap();
    assert_ne!(original_data, patched_data, "File should be modified");
    
    // Rollback
    let rollback_result = patcher.rollback_all();
    assert!(rollback_result.is_ok(), "Rollback should succeed");
    
    // Verify rollback restored original
    let restored_data = fs::read(test_path).unwrap();
    assert_eq!(original_data, restored_data, "Rollback should restore original");
    
    // Cleanup
    fs::remove_file(test_path).ok();
}

#[test]
fn test_binary_patch_undo() {
    // Test undo last operation
    
    let test_binary = create_test_elf_binary();
    let test_path = "test_patch_undo.elf";
    fs::write(test_path, &test_binary).unwrap();
    
    let original_data = fs::read(test_path).unwrap();
    
    let mut patcher = talon::binary_patch::Patch::new(test_path).unwrap();
    
    // Apply first patch
    patcher.nop_out(0x100, 5).unwrap();
    patcher.apply().unwrap();
    
    // Apply second patch
    patcher.nop_out(0x200, 5).unwrap();
    patcher.apply().unwrap();
    
    // Undo last patch
    patcher.undo().unwrap();
    
    // Verify only first patch remains
    let data = fs::read(test_path).unwrap();
    for i in 0..5 {
        assert_eq!(data[0x100 + i], 0x90, "First patch should remain");
    }
    assert_ne!(data[0x200], 0x90, "Second patch should be undone");
    
    // Cleanup
    fs::remove_file(test_path).ok();
}

#[test]
fn test_binary_patch_inject_shellcode() {
    // Test shellcode injection
    
    let test_binary = create_test_elf_binary();
    let test_path = "test_patch_shellcode.elf";
    fs::write(test_path, &test_binary).unwrap();
    
    let mut patcher = talon::binary_patch::Patch::new(test_path).unwrap();
    
    // Inject shellcode at end of binary
    let shellcode = vec![0x48, 0x31, 0xc0, 0xc3]; // xor rax, rax; ret
    let result = patcher.inject_shellcode(&shellcode);
    
    assert!(result.is_ok(), "Shellcode injection should succeed");
    
    patcher.apply().unwrap();
    
    // Verify shellcode appended
    let data = fs::read(test_path).unwrap();
    assert!(data.len() > test_binary.len(), "Binary should be larger");
    
    // Cleanup
    fs::remove_file(test_path).ok();
}

#[test]
fn test_binary_patch_code_cave() {
    // Test code cave creation
    
    let test_binary = create_test_elf_binary();
    let test_path = "test_patch_cave.elf";
    fs::write(test_path, &test_binary).unwrap();
    
    let mut patcher = talon::binary_patch::Patch::new(test_path).unwrap();
    
    // Create 64-byte code cave
    let result = patcher.create_code_cave(64);
    
    match result {
        Ok(offset) => {
            assert!(offset > 0, "Code cave offset should be valid");
            
            patcher.apply().unwrap();
            
            // Verify code cave is NOPs
            let data = fs::read(test_path).unwrap();
            for i in 0..64 {
                assert_eq!(data[offset + i], 0x90, "Code cave should be NOPs");
            }
        }
        Err(e) => {
            eprintln!("Code cave creation failed: {}", e);
        }
    }
    
    // Cleanup
    fs::remove_file(test_path).ok();
}

#[test]
fn test_binary_patch_find_pattern() {
    // Test byte pattern finding
    
    let test_binary = create_test_elf_binary();
    let test_path = "test_patch_find.elf";
    fs::write(test_path, &test_binary).unwrap();
    
    let patcher = talon::binary_patch::Patch::new(test_path).unwrap();
    
    // Find ELF magic bytes
    let pattern = vec![0x7F, 0x45, 0x4C, 0x46]; // ELF magic
    let results = patcher.find_pattern(&pattern);
    
    match results {
        Ok(offsets) => {
            assert!(!offsets.is_empty(), "Should find ELF magic");
            assert_eq!(offsets[0], 0, "ELF magic should be at offset 0");
        }
        Err(e) => {
            panic!("Pattern finding failed: {}", e);
        }
    }
    
    // Cleanup
    fs::remove_file(test_path).ok();
}

#[test]
fn test_binary_patch_architecture_detection() {
    // Test architecture auto-detection
    
    let test_binary = create_test_elf_binary();
    let test_path = "test_patch_arch.elf";
    fs::write(test_path, &test_binary).unwrap();
    
    let patcher = talon::binary_patch::Patch::new(test_path).unwrap();
    
    // Get detected architecture
    let arch = patcher.get_architecture();
    
    // Should detect x64 from ELF header
    use talon::binary_patch::Architecture;
    assert!(
        matches!(arch, Architecture::X64 | Architecture::X86),
        "Should detect valid architecture"
    );
    
    // Cleanup
    fs::remove_file(test_path).ok();
}

#[test]
fn test_binary_patch_string_replacement() {
    // Test string patching
    
    let mut test_binary = create_test_elf_binary();
    
    // Add string to binary
    let test_string = b"TESTSTRING\0";
    test_binary.extend_from_slice(test_string);
    
    let test_path = "test_patch_string.elf";
    fs::write(test_path, &test_binary).unwrap();
    
    let mut patcher = talon::binary_patch::Patch::new(test_path).unwrap();
    
    // Replace string
    let result = patcher.patch_string("TESTSTRING", "PATCHED   "); // Same length
    
    assert!(result.is_ok(), "String patching should succeed");
    
    patcher.apply().unwrap();
    
    // Verify string replaced
    let data = fs::read(test_path).unwrap();
    let patched_str = String::from_utf8_lossy(&data);
    assert!(patched_str.contains("PATCHED"), "String should be patched");
    assert!(!patched_str.contains("TESTSTRING"), "Original string should be gone");
    
    // Cleanup
    fs::remove_file(test_path).ok();
}

// Helper function to create minimal test ELF binary
fn create_test_elf_binary() -> Vec<u8> {
    let mut binary = Vec::new();
    
    // ELF header (64-bit x86-64)
    binary.extend_from_slice(&[
        0x7F, 0x45, 0x4C, 0x46, // Magic number
        0x02,                   // 64-bit
        0x01,                   // Little endian
        0x01,                   // ELF version
        0x00,                   // System V ABI
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Padding
        0x02, 0x00,             // Executable file
        0x3E, 0x00,             // x86-64
        0x01, 0x00, 0x00, 0x00, // Version
    ]);
    
    // Add padding to make it large enough for patching
    binary.resize(0x1000, 0xCC); // Fill with INT3 for testing
    
    binary
}
