#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Write;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    if data.len() < 64 || data.len() > 10_000_000 {
        return;
    }

    if !data.starts_with(b"\x7fELF") {
        return;
    }

    if let Ok(mut file) = NamedTempFile::new() {
        if file.write_all(data).is_ok() && file.flush().is_ok() {
            if let Some(path) = file.path().to_str() {
                let _ = talon::elf_tools::ElfContext::load(path);

                if let Ok(ctx) = talon::elf_tools::ElfContext::load(path) {
                    let _ = ctx.get_symbol("main");
                    let _ = ctx.get_symbol("_start");
                    let _ = ctx.get_plt_entry("puts");
                    let _ = ctx.get_got_entry("libc_start_main");
                    let _ = ctx.find_gadgets_in_section(".text");

                    if let Some(code_section) = ctx.sections.get(".text") {
                        let (addr, _size) = code_section;
                        let _ = ctx.read_bytes(*addr, 100);
                    }
                }
            }
        }
    }
});
