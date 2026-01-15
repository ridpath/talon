#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 || data.len() % 8 != 0 {
        return;
    }
    
    if data.len() > 1024 {
        return;
    }
    
    let mut addresses = Vec::new();
    for chunk in data.chunks_exact(8) {
        let addr = u64::from_le_bytes([
            chunk[0], chunk[1], chunk[2], chunk[3],
            chunk[4], chunk[5], chunk[6], chunk[7],
        ]);
        addresses.push(addr);
    }
    
    let mut elf = Vec::new();
    elf.extend_from_slice(&[0x7f, 0x45, 0x4c, 0x46, 0x02, 0x01, 0x01, 0x00]);
    elf.extend_from_slice(&[0x00; 8]);
    elf.extend_from_slice(&[0x02, 0x00, 0x3e, 0x00]);
    elf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    elf.extend_from_slice(&[0x40, 0x00, 0x38, 0x00, 0x01, 0x00, 0x00, 0x00]);
    
    while elf.len() < 4096 {
        elf.push(0x90);
    }
    
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    if let Ok(mut file) = NamedTempFile::new() {
        if file.write_all(&elf).is_ok() && file.flush().is_ok() {
            if let Some(path) = file.path().to_str() {
                if let Ok(rop) = talon::rop_tools::RopChain::new(path) {
                    let _ = rop.build_chain(&addresses);
                    let _ = rop.find_gadgets("pop");
                    let _ = rop.find_common_gadgets();
                }
            }
        }
    }
});
