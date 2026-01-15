#![no_main]

use libfuzzer_sys::fuzz_target;
use talon::rop_tools::{AutoROPSolver, ROPGoal, ROPStrategy, Constraint};

fuzz_target!(|data: &[u8]| {
    if data.len() < 32 {
        return;
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
    
    let gadget_offset = 0x500;
    if elf.len() > gadget_offset + 100 {
        elf[gadget_offset] = 0x5f;
        elf[gadget_offset + 1] = 0xc3;
        elf[gadget_offset + 10] = 0x5e;
        elf[gadget_offset + 11] = 0xc3;
        elf[gadget_offset + 20] = 0x0f;
        elf[gadget_offset + 21] = 0x05;
    }
    
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    if let Ok(mut file) = NamedTempFile::new() {
        if file.write_all(&elf).is_ok() && file.flush().is_ok() {
            if let Some(path) = file.path().to_str() {
                if let Ok(mut solver) = AutoROPSolver::new(path) {
                    solver.libc_base = Some(0x7ffff7a00000);
                    
                    if data[0] & 0x01 != 0 {
                        solver.add_constraint(Constraint::NoNullBytes);
                    }
                    
                    if data[0] & 0x02 != 0 {
                        solver.add_constraint(Constraint::MaxLength(256));
                    }
                    
                    let goal = match data[1] % 3 {
                        0 => ROPGoal::System("/bin/sh".to_string()),
                        1 => ROPGoal::Execve("/bin/sh".to_string(), vec![]),
                        _ => ROPGoal::Mprotect(0x600000, 0x1000, 7),
                    };
                    
                    let strategies = vec![ROPStrategy::Ret2Libc, ROPStrategy::Ret2Syscall];
                    
                    let _ = solver.solve(goal, strategies);
                }
            }
        }
    }
});
