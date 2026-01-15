#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }
    
    let num_syscalls = (data[0] as usize % 20) + 1;
    let mut syscalls = Vec::new();
    let mut offset = 1;
    
    for _ in 0..num_syscalls {
        if offset + 7 > data.len() {
            break;
        }
        
        let syscall_num = u64::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
            data[offset+4], data[offset+5], data[offset+6], 0
        ]);
        
        syscalls.push(syscall_num & 0x1FF);
        offset += 7;
    }
    
    if !syscalls.is_empty() {
        let _ = talon::syscall_tools::validate_chain(&syscalls);
        let _ = talon::syscall_tools::build_syscall_rop(&syscalls);
        let _ = talon::syscall_tools::detect_seccomp_bypass(&syscalls);
    }
});
