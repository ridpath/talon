#![no_main]

use libfuzzer_sys::fuzz_target;
use talon::rop_gadget_finder::{ROPGadgetFinder, Architecture};

fuzz_target!(|data: &[u8]| {
    if data.len() < 10 {
        return;
    }

    if data.len() > 1024 * 1024 {
        return;
    }

    let arch = match data[0] % 4 {
        0 => Architecture::X64,
        1 => Architecture::X86,
        2 => Architecture::ARM,
        3 => Architecture::ARM64,
        _ => Architecture::X64,
    };

    if let Ok(mut finder) = ROPGadgetFinder::new(arch) {
        let code = &data[1..];
        let base_addr = 0x400000;

        let _ = finder.analyze_bytes(code, base_addr);

        if !finder.gadgets.is_empty() {
            let _ = finder.find_gadgets_by_pattern("pop");
            let _ = finder.find_gadgets_by_pattern("ret");
            let _ = finder.get_best_gadgets(10);
        }
    }
});
