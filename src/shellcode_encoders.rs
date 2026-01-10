// ═══════════════════════════════════════════════════════════════════════════
// 🔒 SHELLCODE ENCODERS - BYPASS FILTERS & BAD CHARS
// ═══════════════════════════════════════════════════════════════════════════

/// Shellcode encoder with multiple encoding schemes
pub struct ShellcodeEncoder {
    pub shellcode: Vec<u8>,
    pub bad_chars: Vec<u8>,
}

impl ShellcodeEncoder {
    /// Create a new shellcode encoder
    /// 
    /// # Example
    /// ```
    /// let encoder = ShellcodeEncoder::new(shellcode);
    /// let encoded = encoder.xor_encode(0x42)?;
    /// ```
    pub fn new(shellcode: Vec<u8>) -> Self {
        ShellcodeEncoder {
            shellcode,
            bad_chars: vec![0x00, 0x0a, 0x0d], // null, newline, carriage return
        }
    }
    
    /// Set custom bad characters to avoid
    pub fn set_bad_chars(&mut self, bad_chars: Vec<u8>) {
        self.bad_chars = bad_chars;
    }
    
    /// XOR encoding with a single byte key
    pub fn xor_encode(&self, key: u8) -> Result<Vec<u8>, String> {
        log::info!("XOR encoding shellcode with key 0x{:02x}", key);
        
        // Check if key creates bad chars
        for &byte in &self.shellcode {
            let encoded = byte ^ key;
            if self.bad_chars.contains(&encoded) {
                return Err(format!("Key 0x{:02x} creates bad char 0x{:02x}", key, encoded));
            }
        }
        
        let encoded: Vec<u8> = self.shellcode.iter()
            .map(|&b| b ^ key)
            .collect();
        
        Ok(encoded)
    }
    
    /// Find a good XOR key automatically
    pub fn find_xor_key(&self) -> Option<u8> {
        for key in 1..=255u8 {
            let mut valid = true;
            for &byte in &self.shellcode {
                let encoded = byte ^ key;
                if self.bad_chars.contains(&encoded) || self.bad_chars.contains(&key) {
                    valid = false;
                    break;
                }
            }
            if valid {
                log::info!("Found valid XOR key: 0x{:02x}", key);
                return Some(key);
            }
        }
        None
    }
    
    /// Alphanumeric encoding (uppercase + lowercase + digits only)
    pub fn alphanumeric_encode(&self) -> Result<Vec<u8>, String> {
        log::info!("Alphanumeric encoding shellcode ({} bytes)", self.shellcode.len());
        
        // This is a complex encoding that converts arbitrary bytes to alphanumeric
        // Simplified version - real implementation would use proper alphanumeric encoder
        
        let mut encoded = Vec::new();
        
        for &byte in &self.shellcode {
            // Split byte into two 4-bit nibbles
            let high = (byte >> 4) & 0x0F;
            let low = byte & 0x0F;
            
            // Encode each nibble as alphanumeric
            encoded.push(Self::nibble_to_alpha(high));
            encoded.push(Self::nibble_to_alpha(low));
        }
        
        log::info!("Encoded to {} alphanumeric bytes", encoded.len());
        Ok(encoded)
    }
    
    /// Convert nibble (0-15) to alphanumeric character
    fn nibble_to_alpha(nibble: u8) -> u8 {
        match nibble {
            0..=9 => b'0' + nibble,
            10..=15 => b'A' + (nibble - 10),
            _ => b'X',
        }
    }
    
    /// Unicode encoding
    pub fn unicode_encode(&self) -> Vec<u8> {
        log::info!("Unicode encoding shellcode");
        
        let mut encoded = Vec::new();
        for &byte in &self.shellcode {
            encoded.push(byte);
            encoded.push(0x00); // Add null byte for UTF-16LE
        }
        
        encoded
    }
    
    /// URL encoding
    pub fn url_encode(&self) -> String {
        self.shellcode.iter()
            .map(|&b| format!("%{:02X}", b))
            .collect()
    }
    
    /// Base64 encoding
    pub fn base64_encode(&self) -> String {
        base64::encode(&self.shellcode)
    }
    
    /// Generate XOR decoder stub (x86-64)
    pub fn xor_decoder_stub(key: u8, shellcode_len: usize) -> Vec<u8> {
        // Simple XOR decoder in x86-64 assembly
        // This would be actual machine code in a real implementation
        
        log::info!("Generating XOR decoder stub (key=0x{:02x}, len={})", key, shellcode_len);
        
        // Placeholder - real implementation would generate actual assembly
        vec![
            0x48, 0x31, 0xc9,                     // xor rcx, rcx
            0x48, 0xb9, 0x00, 0x00, 0x00, 0x00,   // mov rcx, shellcode_len
            0x00, 0x00, 0x00, 0x00,
        ]
    }
}

// ────────────────────────────────────────────────────────────────────────────
// POLYMORPHIC SHELLCODE
// ────────────────────────────────────────────────────────────────────────────

/// Generate polymorphic shellcode by adding random NOPs
pub fn polymorphic_encode(shellcode: &[u8], nop_density: f32) -> Vec<u8> {
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    let mut result = Vec::new();
    
    // NOP equivalents on x86
    let nops = vec![
        vec![0x90],                          // nop
        vec![0x87, 0xC0],                    // xchg eax, eax
        vec![0x97],                          // xchg eax, edi
        vec![0x40],                          // inc eax (only works in 32-bit)
        vec![0x48],                          // dec eax (only works in 32-bit)
    ];
    
    for &byte in shellcode {
        // Add random NOPs based on density
        if rng.gen::<f32>() < nop_density {
            let nop = &nops[rng.gen_range(0..nops.len())];
            result.extend_from_slice(nop);
        }
        result.push(byte);
    }
    
    log::info!("Polymorphic encoding: {} -> {} bytes", shellcode.len(), result.len());
    result
}

// ────────────────────────────────────────────────────────────────────────────
// HELPER FUNCTIONS
// ────────────────────────────────────────────────────────────────────────────

/// Quick XOR encode
pub fn xor_encode(shellcode: &[u8], key: u8) -> Vec<u8> {
    shellcode.iter().map(|&b| b ^ key).collect()
}

/// Quick XOR decode (same as encode for XOR)
pub fn xor_decode(shellcode: &[u8], key: u8) -> Vec<u8> {
    xor_encode(shellcode, key)
}

/// Check if shellcode contains bad characters
pub fn contains_bad_chars(shellcode: &[u8], bad_chars: &[u8]) -> bool {
    for &byte in shellcode {
        if bad_chars.contains(&byte) {
            return true;
        }
    }
    false
}

/// Find all bad characters in shellcode
pub fn find_bad_chars(shellcode: &[u8], bad_chars: &[u8]) -> Vec<(usize, u8)> {
    let mut found = Vec::new();
    for (i, &byte) in shellcode.iter().enumerate() {
        if bad_chars.contains(&byte) {
            found.push((i, byte));
        }
    }
    found
}

/// Generate a NOP sled
pub fn nop_sled(length: usize) -> Vec<u8> {
    vec![0x90; length]
}

/// Generate a random NOP sled (polymorphic)
pub fn polymorphic_nop_sled(length: usize) -> Vec<u8> {
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    let mut sled = Vec::with_capacity(length);
    
    let nop_variants = vec![
        0x90,       // nop
        0x41,       // inc ecx
        0x4A,       // dec edx
        0x48,       // dec eax
    ];
    
    while sled.len() < length {
        let nop = nop_variants[rng.gen_range(0..nop_variants.len())];
        sled.push(nop);
    }
    
    sled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_encode() {
        let shellcode = vec![0x31, 0xc0, 0x50, 0x68];
        let key = 0x42;
        let encoded = xor_encode(&shellcode, key);
        
        assert_eq!(encoded[0], 0x31 ^ 0x42);
        assert_eq!(encoded.len(), shellcode.len());
    }

    #[test]
    fn test_xor_decode() {
        let shellcode = vec![0x31, 0xc0, 0x50, 0x68];
        let key = 0x42;
        let encoded = xor_encode(&shellcode, key);
        let decoded = xor_decode(&encoded, key);
        
        assert_eq!(decoded, shellcode);
    }

    #[test]
    fn test_contains_bad_chars() {
        let shellcode = vec![0x31, 0x00, 0x50, 0x68];
        let bad_chars = vec![0x00, 0x0a, 0x0d];
        
        assert!(contains_bad_chars(&shellcode, &bad_chars));
    }

    #[test]
    fn test_find_bad_chars() {
        let shellcode = vec![0x31, 0x00, 0x50, 0x0a];
        let bad_chars = vec![0x00, 0x0a];
        let found = find_bad_chars(&shellcode, &bad_chars);
        
        assert_eq!(found.len(), 2);
        assert_eq!(found[0], (1, 0x00));
        assert_eq!(found[1], (3, 0x0a));
    }

    #[test]
    fn test_nop_sled() {
        let sled = nop_sled(100);
        assert_eq!(sled.len(), 100);
        assert!(sled.iter().all(|&b| b == 0x90));
    }
}
