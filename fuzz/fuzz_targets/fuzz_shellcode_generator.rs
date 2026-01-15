#![no_main]

use libfuzzer_sys::fuzz_target;
use talon::shellcode_library::{ShellcodeLibrary, Architecture, ShellcodeType};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    
    let arch = match data[0] % 4 {
        0 => Architecture::X64,
        1 => Architecture::X86,
        2 => Architecture::ARM,
        3 => Architecture::ARM64,
        _ => Architecture::X64,
    };
    
    let mut lib = ShellcodeLibrary::new(arch);
    
    let shellcode_type = if data.len() > 1 {
        match data[1] % 5 {
            0 => ShellcodeType::Shell,
            1 => ShellcodeType::ReverseShell,
            2 => ShellcodeType::BindShell,
            3 => ShellcodeType::Execve,
            4 => ShellcodeType::ReadFlag,
            _ => ShellcodeType::Shell,
        }
    } else {
        ShellcodeType::Shell
    };
    
    let _ = lib.generate(shellcode_type);
    
    if data.len() > 10 {
        let port = u16::from_le_bytes([data[2], data[3]]);
        let ip_bytes = if data.len() > 6 {
            [data[4], data[5], data[6], data[7]]
        } else {
            [127, 0, 0, 1]
        };
        let ip = format!("{}.{}.{}.{}", ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);
        
        let _ = lib.reverse_shell(&ip, port);
        let _ = lib.bind_shell(port);
    }
    
    if data.len() > 20 {
        if let Ok(cmd) = std::str::from_utf8(&data[20..std::cmp::min(data.len(), 120)]) {
            let _ = lib.exec(cmd);
        }
    }
    
    let avoid_bytes = if data.len() > 8 {
        data[8..std::cmp::min(data.len(), 18)].to_vec()
    } else {
        vec![0x00]
    };
    
    if let Ok(shellcode) = lib.generate(shellcode_type) {
        let _ = talon::shellcode_encoders::encode_xor(&shellcode, 0x42);
        let _ = talon::shellcode_encoders::encode_alphanumeric(&shellcode);
        let _ = talon::shellcode_encoders::avoid_badchars(&shellcode, &avoid_bytes);
    }
});
