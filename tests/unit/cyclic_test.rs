use talon::cyclic_tools::*;
use talon::cyclic_pattern::{CyclicPattern};

#[test]
fn test_cyclic_generation() {
    let pattern = cyclic(100);
    assert_eq!(pattern.len(), 100);
    
    assert_eq!(&pattern[0..4], b"aaaa");
}

#[test]
fn test_cyclic_generation_sizes() {
    assert_eq!(cyclic(0).len(), 0);
    assert_eq!(cyclic(1).len(), 1);
    assert_eq!(cyclic(10).len(), 10);
    assert_eq!(cyclic(1000).len(), 1000);
    assert_eq!(cyclic(10000).len(), 10000);
}

#[test]
fn test_cyclic_uniqueness_100() {
    let pattern = cyclic(1000);
    
    let mut seen = std::collections::HashSet::new();
    for window in pattern[..500].windows(4) {
        let key = u32::from_le_bytes([window[0], window[1], window[2], window[3]]);
        assert!(!seen.contains(&key), "Duplicate pattern found at unique check!");
        seen.insert(key);
    }
}

#[test]
fn test_cyclic_find_basic() {
    let pattern = cyclic(300);
    
    let bytes = &pattern[100..104];
    let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    
    let offset = cyclic_find(value as u64);
    assert_eq!(offset, Some(100));
}

#[test]
fn test_cyclic_find_offset_264() {
    let pattern = cyclic(300);
    
    let bytes = &pattern[264..268];
    let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    
    let offset = cyclic_find(value as u64);
    assert_eq!(offset, Some(264));
}

#[test]
fn test_cyclic_find_offset_0() {
    let pattern = cyclic(300);
    
    let bytes = &pattern[0..4];
    let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    
    let offset = cyclic_find(value as u64);
    assert_eq!(offset, Some(0));
}

#[test]
fn test_cyclic_find_bytes() {
    let pattern = cyclic(300);
    let search = &pattern[100..104];
    let offset = cyclic_find_bytes(search);
    assert_eq!(offset, Some(100));
}

#[test]
fn test_cyclic_find_bytes_various_offsets() {
    let pattern = cyclic(1000);
    
    for test_offset in [0, 50, 100, 150, 200, 250] {
        let search = &pattern[test_offset..test_offset + 4];
        let found = cyclic_find_bytes(search);
        assert_eq!(found, Some(test_offset), "Failed at offset {}", test_offset);
    }
}

#[test]
fn test_cyclic_find_bytes_too_short() {
    let pattern = b"abc";
    let offset = cyclic_find_bytes(pattern);
    assert_eq!(offset, None);
}

#[test]
fn test_cyclic_find_hex() {
    let pattern = cyclic(300);
    let bytes = &pattern[72..76];
    let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let hex_str = format!("0x{:08x}", value);
    
    let offset = cyclic_find_hex(&hex_str);
    assert_eq!(offset, Some(72));
}

#[test]
fn test_cyclic_find_hex_formats() {
    let pattern = cyclic(300);
    let bytes = &pattern[100..104];
    let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    
    let hex_with_prefix = format!("0x{:08x}", value);
    let hex_no_prefix = format!("{:08x}", value);
    
    let offset1 = cyclic_find_hex(&hex_with_prefix);
    let offset2 = cyclic_find_hex(&hex_no_prefix);
    
    assert_eq!(offset1, Some(100));
    assert_eq!(offset2, Some(100));
}

#[test]
fn test_cyclic_find_hex_invalid() {
    let offset = cyclic_find_hex("not_hex");
    assert_eq!(offset, None);
}

#[test]
fn test_cyclic_custom_alphabet() {
    let custom_alpha = b"ABC";
    let pattern = cyclic_custom(50, custom_alpha);
    
    assert_eq!(pattern.len(), 50);
    
    for &byte in &pattern {
        assert!(custom_alpha.contains(&byte));
    }
}

#[test]
fn test_cyclic_custom_single_char() {
    let pattern = cyclic_custom(10, b"X");
    assert_eq!(pattern.len(), 10);
}

#[test]
fn test_cyclic_display() {
    let pattern = cyclic(40);
    let display = cyclic_display(&pattern, 16);
    
    assert!(display.contains("0000:"));
    assert!(display.contains("0010:"));
    assert!(display.contains('\n'));
}

#[test]
fn test_cyclic_display_width() {
    let pattern = cyclic(30);
    let display = cyclic_display(&pattern, 10);
    
    let lines: Vec<&str> = display.lines().collect();
    assert_eq!(lines.len(), 3);
}

#[test]
fn test_find_overflow_offset() {
    let result = find_overflow_offset("nonexistent_binary", 1000);
    assert!(result.is_err());
}

#[test]
fn test_cyclic_pattern_new() {
    let generator = CyclicPattern::new();
    let pattern = generator.generate(100);
    assert_eq!(pattern.len(), 100);
}

#[test]
fn test_cyclic_pattern_generate_various_sizes() {
    let generator = CyclicPattern::new();
    
    for size in [0, 1, 10, 100, 500, 1000] {
        let pattern = generator.generate(size);
        assert_eq!(pattern.len(), size, "Failed for size {}", size);
    }
}

#[test]
fn test_cyclic_pattern_find_offset() {
    let generator = CyclicPattern::new();
    let pattern = generator.generate(1000);
    
    let search = &pattern[200..204];
    let offset = generator.find_offset(&pattern, search);
    assert_eq!(offset, Some(200));
}

#[test]
fn test_cyclic_pattern_find_offset_from_u64() {
    let generator = CyclicPattern::new();
    let pattern = generator.generate(1000);
    
    let offset = 72;
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&pattern[offset..offset + 8]);
    let value = u64::from_le_bytes(bytes);
    
    let found = generator.find_offset_from_u64(&pattern, value);
    assert!(found.is_some());
    assert!(found.unwrap() >= offset && found.unwrap() <= offset + 4);
}

#[test]
fn test_cyclic_pattern_find_offset_from_string() {
    let generator = CyclicPattern::new();
    let pattern = generator.generate(1000);
    
    let search_str = std::str::from_utf8(&pattern[150..154]).unwrap();
    let offset = generator.find_offset_from_string(&pattern, search_str);
    assert_eq!(offset, Some(150));
}

#[test]
fn test_cyclic_find_wrapper() {
    let pattern = cyclic(1000);
    
    let search_bytes = &pattern[100..104];
    let offset = cyclic_find_bytes(search_bytes);
    assert_eq!(offset, Some(100));
}

#[test]
fn test_cyclic_find_wrapper_hex() {
    let pattern = cyclic(1000);
    
    let offset = 200;
    let bytes = &pattern[offset..offset + 4];
    let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let hex_str = format!("0x{:08x}", value);
    
    let found = cyclic_find_hex(&hex_str);
    assert_eq!(found, Some(offset));
}

#[test]
fn test_cyclic_find_wrapper_decimal() {
    let pattern = cyclic(1000);
    
    let offset = 150;
    let bytes = &pattern[offset..offset + 4];
    let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    
    let found = cyclic_find(value as u64);
    assert_eq!(found, Some(offset));
}

#[test]
fn test_cyclic_find_wrapper_empty_pattern() {
    // Test with bytes that won't be in the cyclic pattern
    let empty_bytes = vec![0xFF, 0xFF, 0xFF, 0xFF];
    let result = cyclic_find_bytes(&empty_bytes);
    assert_eq!(result, None);
}

#[test]
fn test_cyclic_find_wrapper_empty_search() {
    // Test with invalid hex string
    let result = cyclic_find_hex("");
    assert_eq!(result, None);
}

#[test]
fn test_cyclic_find_not_found() {
    // Test with value that won't be in a small cyclic pattern
    let result = cyclic_find_hex("0xffffffff");
    assert_eq!(result, None);
}

#[test]
fn test_cyclic_long_pattern() {
    let pattern = cyclic(20000);
    assert_eq!(pattern.len(), 20000);
    
    let search = &pattern[19000..19004];
    let offset = cyclic_find_bytes(search);
    assert_eq!(offset, Some(19000));
}

#[test]
fn test_cyclic_repeatability() {
    let pattern1 = cyclic(500);
    let pattern2 = cyclic(500);
    assert_eq!(pattern1, pattern2);
}

#[test]
fn test_cyclic_offset_precision() {
    let pattern = cyclic(1000);
    
    for offset in (0..=900).step_by(50) {
        let bytes = &pattern[offset..offset + 4];
        let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        let found = cyclic_find(value as u64);
        assert_eq!(found, Some(offset), "Precision test failed at offset {}", offset);
    }
}

#[test]
fn test_cyclic_boundary_conditions() {
    let pattern = cyclic(300);
    
    let bytes = &pattern[296..300];
    assert_eq!(bytes.len(), 4);
}

#[test]
fn test_cyclic_pattern_consistency() {
    let gen1 = CyclicPattern::new();
    let gen2 = CyclicPattern::new();
    
    let p1 = gen1.generate(500);
    let p2 = gen2.generate(500);
    
    assert_eq!(p1, p2);
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_cyclic_length(size in 0usize..5000) {
            let pattern = cyclic(size);
            assert_eq!(pattern.len(), size);
        }

        #[test]
        fn prop_cyclic_find_correctness(offset in 0usize..500) {
            let pattern = cyclic(1000);
            let bytes = &pattern[offset..offset + 4];
            let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            let found = cyclic_find(value as u64);
            assert_eq!(found, Some(offset));
        }

        #[test]
        fn prop_cyclic_find_bytes_correctness(offset in 0usize..500) {
            let pattern = cyclic(1000);
            let search = &pattern[offset..offset + 4];
            let found = cyclic_find_bytes(search);
            assert_eq!(found, Some(offset));
        }

        #[test]
        fn prop_cyclic_repeatability(size in 0usize..1000) {
            let p1 = cyclic(size);
            let p2 = cyclic(size);
            assert_eq!(p1, p2);
        }

        #[test]
        fn prop_cyclic_custom_length(size in 0usize..500) {
            let pattern = cyclic_custom(size, b"ABC");
            assert_eq!(pattern.len(), size);
        }

        #[test]
        fn prop_cyclic_custom_alphabet_constraint(size in 1usize..500) {
            let alphabet = b"XYZ";
            let pattern = cyclic_custom(size, alphabet);
            for &byte in &pattern {
                assert!(alphabet.contains(&byte) || byte == 0);
            }
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_exploit_workflow_simulation() {
        let buffer_size = 512;
        let pattern = cyclic(buffer_size);
        
        let crash_offset = 264;
        let crash_bytes = &pattern[crash_offset..crash_offset + 4];
        let crash_value = u32::from_le_bytes([
            crash_bytes[0],
            crash_bytes[1],
            crash_bytes[2],
            crash_bytes[3],
        ]);
        
        let found_offset = cyclic_find(crash_value as u64);
        assert_eq!(found_offset, Some(crash_offset));
        
        println!("Simulated crash at offset: {}", crash_offset);
        println!("Pattern search found offset: {}", found_offset.unwrap());
    }

    #[test]
    fn test_multiple_pattern_generation() {
        for size in [100, 200, 300, 500, 1000] {
            let pattern = cyclic(size);
            assert_eq!(pattern.len(), size);
            
            if size >= 100 {
                let test_offset = size / 2;
                let bytes = &pattern[test_offset..test_offset + 4];
                let found = cyclic_find_bytes(bytes);
                assert_eq!(found, Some(test_offset));
            }
        }
    }

    #[test]
    fn test_hex_workflow() {
        let pattern = cyclic(1000);
        
        for offset in [0, 100, 200, 300, 500] {
            let bytes = &pattern[offset..offset + 4];
            let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            
            let hex_str = format!("0x{:08x}", value);
            let found = cyclic_find_hex(&hex_str);
            assert_eq!(found, Some(offset), "Hex workflow failed at offset {}", offset);
        }
    }
}
