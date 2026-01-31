// ═══════════════════════════════════════════════════════════════════════════
// PACKING/UNPACKING TOOLKIT - WORLD-CLASS EXPLOIT DEV
// ═══════════════════════════════════════════════════════════════════════════
// Provides pwntools-style pack/unpack primitives for exploit development

/// Pack a 64-bit value to little-endian bytes.
///
/// # Examples
///
/// ```
/// use talon::packing_tools::pack64;
///
/// let packed = pack64(0xdeadbeef);
/// assert_eq!(packed, vec![0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00]);
///
/// let addr = pack64(0x00007ffff7a0d000);
/// assert_eq!(addr.len(), 8);
/// ```
pub fn pack64(value: u64) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

/// Pack a 64-bit value to big-endian bytes.
///
/// # Examples
///
/// ```
/// use talon::packing_tools::pack64_be;
///
/// let packed = pack64_be(0xdeadbeef);
/// assert_eq!(packed, vec![0x00, 0x00, 0x00, 0x00, 0xde, 0xad, 0xbe, 0xef]);
/// ```
pub fn pack64_be(value: u64) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

/// Unpack little-endian bytes to 64-bit value.
///
/// # Examples
///
/// ```
/// use talon::packing_tools::unpack64;
///
/// let bytes = vec![0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00];
/// let value = unpack64(&bytes).unwrap();
/// assert_eq!(value, 0xdeadbeef);
///
/// let short_bytes = vec![0x01, 0x02];
/// assert!(unpack64(&short_bytes).is_err());
/// ```
pub fn unpack64(bytes: &[u8]) -> Result<u64, String> {
    if bytes.len() < 8 {
        return Err(format!("Need 8 bytes for u64, got {}", bytes.len()));
    }
    let arr: [u8; 8] = bytes[0..8].try_into()
        .map_err(|_| "Failed to convert to array".to_string())?;
    Ok(u64::from_le_bytes(arr))
}

/// Unpack big-endian bytes to 64-bit value.
///
/// # Examples
///
/// ```
/// use talon::packing_tools::unpack64_be;
///
/// let bytes = vec![0x00, 0x00, 0x00, 0x00, 0xde, 0xad, 0xbe, 0xef];
/// let value = unpack64_be(&bytes).unwrap();
/// assert_eq!(value, 0xdeadbeef);
/// ```
pub fn unpack64_be(bytes: &[u8]) -> Result<u64, String> {
    if bytes.len() < 8 {
        return Err(format!("Need 8 bytes for u64, got {}", bytes.len()));
    }
    let arr: [u8; 8] = bytes[0..8].try_into()
        .map_err(|_| "Failed to convert to array".to_string())?;
    Ok(u64::from_be_bytes(arr))
}

/// Pack a 32-bit value to little-endian bytes.
///
/// # Examples
///
/// ```
/// use talon::packing_tools::pack32;
///
/// let packed = pack32(0x12345678);
/// assert_eq!(packed, vec![0x78, 0x56, 0x34, 0x12]);
/// ```
pub fn pack32(value: u32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

/// Pack a 32-bit value to big-endian bytes.
///
/// # Examples
///
/// ```
/// use talon::packing_tools::pack32_be;
///
/// let packed = pack32_be(0x12345678);
/// assert_eq!(packed, vec![0x12, 0x34, 0x56, 0x78]);
/// ```
pub fn pack32_be(value: u32) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

/// Unpack little-endian bytes to 32-bit value.
///
/// # Examples
///
/// ```
/// use talon::packing_tools::unpack32;
///
/// let bytes = vec![0x78, 0x56, 0x34, 0x12];
/// let value = unpack32(&bytes).unwrap();
/// assert_eq!(value, 0x12345678);
/// ```
pub fn unpack32(bytes: &[u8]) -> Result<u32, String> {
    if bytes.len() < 4 {
        return Err(format!("Need 4 bytes for u32, got {}", bytes.len()));
    }
    let arr: [u8; 4] = bytes[0..4].try_into()
        .map_err(|_| "Failed to convert to array".to_string())?;
    Ok(u32::from_le_bytes(arr))
}

/// Unpack big-endian bytes to 32-bit value.
///
/// # Examples
///
/// ```
/// use talon::packing_tools::unpack32_be;
///
/// let bytes = vec![0x12, 0x34, 0x56, 0x78];
/// let value = unpack32_be(&bytes).unwrap();
/// assert_eq!(value, 0x12345678);
/// ```
pub fn unpack32_be(bytes: &[u8]) -> Result<u32, String> {
    if bytes.len() < 4 {
        return Err(format!("Need 4 bytes for u32, got {}", bytes.len()));
    }
    let arr: [u8; 4] = bytes[0..4].try_into()
        .map_err(|_| "Failed to convert to array".to_string())?;
    Ok(u32::from_be_bytes(arr))
}

/// Pack a 16-bit value to little-endian bytes.
///
/// # Examples
///
/// ```
/// use talon::packing_tools::pack16;
///
/// let packed = pack16(0x1234);
/// assert_eq!(packed, vec![0x34, 0x12]);
/// ```
pub fn pack16(value: u16) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

/// Pack a 16-bit value to big-endian bytes.
///
/// # Examples
///
/// ```
/// use talon::packing_tools::pack16_be;
///
/// let packed = pack16_be(0x1234);
/// assert_eq!(packed, vec![0x12, 0x34]);
/// ```
pub fn pack16_be(value: u16) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}

/// Unpack little-endian bytes to 16-bit value.
///
/// # Examples
///
/// ```
/// use talon::packing_tools::unpack16;
///
/// let bytes = vec![0x34, 0x12];
/// let value = unpack16(&bytes).unwrap();
/// assert_eq!(value, 0x1234);
/// ```
pub fn unpack16(bytes: &[u8]) -> Result<u16, String> {
    if bytes.len() < 2 {
        return Err(format!("Need 2 bytes for u16, got {}", bytes.len()));
    }
    let arr: [u8; 2] = bytes[0..2].try_into()
        .map_err(|_| "Failed to convert to array".to_string())?;
    Ok(u16::from_le_bytes(arr))
}

/// Unpack big-endian bytes to 16-bit value.
///
/// # Examples
///
/// ```
/// use talon::packing_tools::unpack16_be;
///
/// let bytes = vec![0x12, 0x34];
/// let value = unpack16_be(&bytes).unwrap();
/// assert_eq!(value, 0x1234);
/// ```
pub fn unpack16_be(bytes: &[u8]) -> Result<u16, String> {
    if bytes.len() < 2 {
        return Err(format!("Need 2 bytes for u16, got {}", bytes.len()));
    }
    let arr: [u8; 2] = bytes[0..2].try_into()
        .map_err(|_| "Failed to convert to array".to_string())?;
    Ok(u16::from_be_bytes(arr))
}

/// Pack a single byte.
///
/// # Examples
///
/// ```
/// use talon::packing_tools::pack8;
///
/// let packed = pack8(0x42);
/// assert_eq!(packed, vec![0x42]);
/// ```
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
// PWNTOOLS-STYLE ALIASES
// ────────────────────────────────────────────────────────────────────────────

/// Pwntools-style alias for pack64
pub fn p64(value: u64) -> Vec<u8> {
    pack64(value)
}

/// Pwntools-style alias for unpack64
pub fn u64(bytes: &[u8]) -> Result<u64, String> {
    unpack64(bytes)
}

/// Pwntools-style alias for pack32
pub fn p32(value: u32) -> Vec<u8> {
    pack32(value)
}

/// Pwntools-style alias for unpack32
pub fn u32(bytes: &[u8]) -> Result<u32, String> {
    unpack32(bytes)
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
            'Q' => { // 64-bit little-endian
                result.extend_from_slice(&pack64(values[val_idx]));
                val_idx += 1;
            }
            'I' => { // 32-bit little-endian
                result.extend_from_slice(&pack32(values[val_idx] as u32));
                val_idx += 1;
            }
            'H' => { // 16-bit little-endian
                result.extend_from_slice(&pack16(values[val_idx] as u16));
                val_idx += 1;
            }
            'B' => { // 8-bit
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

/// Assemble code to bytes
pub fn assemble(code: &str, arch: &str) -> Result<Vec<u8>, String> {
    match arch {
        "x64" | "x86_64" => {
            let mut result = Vec::new();
            for line in code.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                    continue;
                }
                
                let instr = line.split_whitespace().next().unwrap_or("");
                match instr {
                    "nop" => result.push(0x90),
                    "ret" => result.push(0xc3),
                    "syscall" => result.extend_from_slice(&[0x0f, 0x05]),
                    "int3" => result.push(0xcc),
                    "push" => {
                        let operand = line.split_whitespace().nth(1).ok_or("Missing operand")?;
                        match operand {
                            "rax" => result.push(0x50),
                            "rcx" => result.push(0x51),
                            "rdx" => result.push(0x52),
                            "rbx" => result.push(0x53),
                            "rsp" => result.push(0x54),
                            "rbp" => result.push(0x55),
                            "rsi" => result.push(0x56),
                            "rdi" => result.push(0x57),
                            _ => {
                                if let Ok(val) = operand.trim_start_matches("0x").parse::<u8>() {
                                    result.extend_from_slice(&[0x6a, val]);
                                } else {
                                    return Err(format!("Unknown push operand: {}", operand));
                                }
                            }
                        }
                    }
                    "pop" => {
                        let operand = line.split_whitespace().nth(1).ok_or("Missing operand")?;
                        match operand {
                            "rax" => result.push(0x58),
                            "rcx" => result.push(0x59),
                            "rdx" => result.push(0x5a),
                            "rbx" => result.push(0x5b),
                            "rsp" => result.push(0x5c),
                            "rbp" => result.push(0x5d),
                            "rsi" => result.push(0x5e),
                            "rdi" => result.push(0x5f),
                            _ => return Err(format!("Unknown pop operand: {}", operand)),
                        }
                    }
                    "xor" => {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            let dst = parts[1].trim_end_matches(',');
                            let src = parts[2];
                            if dst == "eax" && src == "eax" {
                                result.extend_from_slice(&[0x31, 0xc0]);
                            } else if dst == "ecx" && src == "ecx" {
                                result.extend_from_slice(&[0x31, 0xc9]);
                            } else if dst == "edx" && src == "edx" {
                                result.extend_from_slice(&[0x31, 0xd2]);
                            } else if dst == "rax" && src == "rax" {
                                result.extend_from_slice(&[0x48, 0x31, 0xc0]);
                            } else if dst == "rcx" && src == "rcx" {
                                result.extend_from_slice(&[0x48, 0x31, 0xc9]);
                            } else if dst == "rdx" && src == "rdx" {
                                result.extend_from_slice(&[0x48, 0x31, 0xd2]);
                            } else {
                                return Err(format!("Unsupported xor: {} {}", dst, src));
                            }
                        }
                    }
                    _ => return Err(format!("Unsupported instruction: {}", instr)),
                }
            }
            Ok(result)
        }
        _ => Err(format!("Unsupported architecture: {}", arch)),
    }
}

/// Disassemble bytes to assembly
pub fn disassemble(bytes: &[u8], arch: &str, addr: u64) -> Result<String, String> {
    use capstone::prelude::*;
    
    let cs = match arch {
        "x64" | "x86_64" => Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build()
            .map_err(|e| format!("Capstone init error: {:?}", e))?,
        "x86" | "i386" => Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode32)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build()
            .map_err(|e| format!("Capstone init error: {:?}", e))?,
        _ => return Err(format!("Unsupported architecture: {}", arch)),
    };
    
    let insns = cs.disasm_all(bytes, addr)
        .map_err(|e| format!("Disassembly error: {:?}", e))?;
    
    let mut result = String::new();
    for insn in insns.iter() {
        result.push_str(&format!("0x{:x}: {} {}\n",
            insn.address(),
            insn.mnemonic().unwrap_or(""),
            insn.op_str().unwrap_or("")
        ));
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack64() {
        assert_eq!(pack64(0xdeadbeef), vec![0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(pack64(0x4142434445464748), vec![0x48, 0x47, 0x46, 0x45, 0x44, 0x43, 0x42, 0x41]);
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
