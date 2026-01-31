#![no_main]

use libfuzzer_sys::fuzz_target;
use talon::disassembler::{Disassembler, Architecture};

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > 100_000 {
        return;
    }

    let arch = match data[0] % 4 {
        0 => Architecture::X64,
        1 => Architecture::X86,
        2 => Architecture::ARM,
        3 => Architecture::ARM64,
        _ => Architecture::X64,
    };

    let code = if data.len() > 1 { &data[1..] } else { data };

    if let Ok(disasm) = Disassembler::new(arch) {
        let base_addr = 0x400000;
        let _ = disasm.disassemble(code, base_addr);
        let _ = disasm.find_functions(code, base_addr);
        let _ = disasm.find_strings(code);
        let _ = disasm.analyze_control_flow(code, base_addr);

        if code.len() < 1000 {
            let _ = disasm.find_syscalls(code);
            let _ = disasm.find_dangerous_functions(code);
        }
    }
});
