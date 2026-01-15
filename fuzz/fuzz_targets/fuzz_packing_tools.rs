#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    
    for endian in &["little", "big"] {
        if data.len() >= 1 {
            let _ = talon::packing_tools::pack_u8(data[0]);
        }
        
        if data.len() >= 2 {
            let val = u16::from_le_bytes([data[0], data[1]]);
            let _ = talon::packing_tools::pack_u16(val, endian);
            let _ = talon::packing_tools::unpack_u16(&data[0..2], endian);
        }
        
        if data.len() >= 4 {
            let val = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            let _ = talon::packing_tools::pack_u32(val, endian);
            let _ = talon::packing_tools::unpack_u32(&data[0..4], endian);
        }
        
        if data.len() >= 8 {
            let val = u64::from_le_bytes([
                data[0], data[1], data[2], data[3],
                data[4], data[5], data[6], data[7],
            ]);
            let _ = talon::packing_tools::pack_u64(val, endian);
            let _ = talon::packing_tools::unpack_u64(&data[0..8], endian);
        }
    }
    
    if data.len() <= 10000 {
        let _ = talon::packing_tools::flat(&[data]);
        
        if let Ok(s) = std::str::from_utf8(data) {
            let _ = talon::encoding_tools::url_encode(s);
            let _ = talon::encoding_tools::url_decode(s);
            let _ = talon::encoding_tools::base64_encode(data);
            let _ = talon::encoding_tools::hex_encode(data);
        }
    }
    
    if let Ok(hex_str) = std::str::from_utf8(data) {
        let _ = talon::encoding_tools::base64_decode(hex_str);
        let _ = talon::encoding_tools::hex_decode(hex_str);
    }
    
    if data.len() > 2 {
        let pattern_len = (data[0] as usize % 20) + 1;
        let _ = talon::cyclic_tools::cyclic(pattern_len * 100);
        
        if data.len() >= 4 {
            let search_val = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
            let _ = talon::cyclic_tools::cyclic_find(search_val, pattern_len);
        }
    }
});
