// ═══════════════════════════════════════════════════════════════════════════
// PACKING/UNPACKING TOOLKIT - WORLD-CLASS EXPLOIT DEV
// ═══════════════════════════════════════════════════════════════════════════
// Provides pwntools-style pack/unpack primitives for exploit development

/// Pack a 64-bit value to little-endian bytes
pub fn pack64(value: u64) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

/// Pack a 64-bit value to big-endian bytes
pub fn pack64_be(value: u64) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

/// Unpack little-endian bytes to 64-bit value
pub fn unpack64(bytes: &[u8]) -> Result<u64, String> {
    if bytes.len() < 8 {
        return Err(format!("Need 8 bytes for u64, got {}", bytes.len()));
    }
    let arr: [u8; 8] = bytes[0..8]
        .try_into()
        .map_err(|_| "Failed to convert to array".to_string())?;
    Ok(u64::from_le_bytes(arr))
}

/// Unpack big-endian bytes to 64-bit value
pub fn unpack64_be(bytes: &[u8]) -> Result<u64, String> {
    if bytes.len() < 8 {
        return Err(format!("Need 8 bytes for u64, got {}", bytes.len()));
    }
    let arr: [u8; 8] = bytes[0..8]
        .try_into()
        .map_err(|_| "Failed to convert to array".to_string())?;
    Ok(u64::from_be_bytes(arr))
}

/// Pack a 32-bit value to little-endian bytes
pub fn pack32(value: u32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

/// Pack a 32-bit value to big-endian bytes
pub fn pack32_be(value: u32) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

/// Unpack little-endian bytes to 32-bit value
pub fn unpack32(bytes: &[u8]) -> Result<u32, String> {
    if bytes.len() < 4 {
        return Err(format!("Need 4 bytes for u32, got {}", bytes.len()));
    }
    let arr: [u8; 4] = bytes[0..4]
        .try_into()
        .map_err(|_| "Failed to convert to array".to_string())?;
    Ok(u32::from_le_bytes(arr))
}

/// Unpack big-endian bytes to 32-bit value
pub fn unpack32_be(bytes: &[u8]) -> Result<u32, String> {
    if bytes.len() < 4 {
        return Err(format!("Need 4 bytes for u32, got {}", bytes.len()));
    }
    let arr: [u8; 4] = bytes[0..4]
        .try_into()
        .map_err(|_| "Failed to convert to array".to_string())?;
    Ok(u32::from_be_bytes(arr))
}

/// Pack a 16-bit value to little-endian bytes
pub fn pack16(value: u16) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

/// Pack a 16-bit value to big-endian bytes
pub fn pack16_be(value: u16) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

/// Unpack little-endian bytes to 16-bit value
pub fn unpack16(bytes: &[u8]) -> Result<u16, String> {
    if bytes.len() < 2 {
        return Err(format!("Need 2 bytes for u16, got {}", bytes.len()));
    }
    let arr: [u8; 2] = bytes[0..2]
        .try_into()
        .map_err(|_| "Failed to convert to array".to_string())?;
    Ok(u16::from_le_bytes(arr))
}

/// Unpack big-endian bytes to 16-bit value
pub fn unpack16_be(bytes: &[u8]) -> Result<u16, String> {
    if bytes.len() < 2 {
        return Err(format!("Need 2 bytes for u16, got {}", bytes.len()));
    }
    let arr: [u8; 2] = bytes[0..2]
        .try_into()
        .map_err(|_| "Failed to convert to array".to_string())?;
    Ok(u16::from_be_bytes(arr))
}

/// Pack a single byte
pub fn pack8(value: u8) -> Vec<u8> {
    vec![value]
}

/// Unpack a single byte
pub fn unpack8(bytes: &[u8]) -> Result<u8, String> {
    if bytes.is_empty() {
        return Err("Need 1 byte for u8".to_string());
    }
    Ok(bytes[0])
}

// ────────────────────────────────────────────────────────────────────────────
// ADVANCED PACKING UTILITIES
// ────────────────────────────────────────────────────────────────────────────

/// Pack multiple values as a struct
pub fn pack_struct(format: &str, values: &[u64]) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let mut val_idx = 0;

    for ch in format.chars() {
        if val_idx >= values.len() {
            return Err("Not enough values for format string".to_string());
        }

        match ch {
            'Q' => {
                // 64-bit little-endian
                result.extend_from_slice(&pack64(values[val_idx]));
                val_idx += 1;
            }
            'I' => {
                // 32-bit little-endian
                result.extend_from_slice(&pack32(values[val_idx] as u32));
                val_idx += 1;
            }
            'H' => {
                // 16-bit little-endian
                result.extend_from_slice(&pack16(values[val_idx] as u16));
                val_idx += 1;
            }
            'B' => {
                // 8-bit
                result.push(values[val_idx] as u8);
                val_idx += 1;
            }
            _ => return Err(format!("Unknown format character: {}", ch)),
        }
    }

    Ok(result)
}

/// Flat pack a list of values (all as u64 little-endian)
pub fn flat_pack(values: &[u64]) -> Vec<u8> {
    let mut result = Vec::new();
    for &val in values {
        result.extend_from_slice(&pack64(val));
    }
    result
}

/// Create a buffer of repeated pattern
pub fn cyclic_buffer(size: usize, pattern: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(size);
    while result.len() < size {
        for &byte in pattern {
            if result.len() >= size {
                break;
            }
            result.push(byte);
        }
    }
    result
}

/// Convert hex string to bytes
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.trim_start_matches("0x");
    hex::decode(hex).map_err(|e| format!("Hex decode error: {}", e))
}

/// Convert bytes to hex string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack64() {
        assert_eq!(
            pack64(0xdeadbeef),
            vec![0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(
            pack64(0x4142434445464748),
            vec![0x48, 0x47, 0x46, 0x45, 0x44, 0x43, 0x42, 0x41]
        );
    }

    #[test]
    fn test_unpack64() {
        let bytes = vec![0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(unpack64(&bytes).unwrap(), 0xdeadbeef);
    }

    #[test]
    fn test_pack32() {
        assert_eq!(pack32(0x41424344), vec![0x44, 0x43, 0x42, 0x41]);
    }

    #[test]
    fn test_unpack32() {
        let bytes = vec![0x44, 0x43, 0x42, 0x41];
        assert_eq!(unpack32(&bytes).unwrap(), 0x41424344);
    }

    #[test]
    fn test_pack16() {
        assert_eq!(pack16(0x4142), vec![0x42, 0x41]);
    }

    #[test]
    fn test_pack_struct() {
        let result = pack_struct("QIH", &[0x4142434445464748, 0x41424344, 0x4142]).unwrap();
        assert_eq!(result.len(), 8 + 4 + 2);
    }

    #[test]
    fn test_flat_pack() {
        let result = flat_pack(&[0x41, 0x42, 0x43]);
        assert_eq!(result.len(), 24); // 3 * 8 bytes
    }
}
