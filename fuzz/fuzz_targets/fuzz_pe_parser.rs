#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Write;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    if data.len() < 64 || data.len() > 10_000_000 {
        return;
    }

    if !data.starts_with(b"MZ") {
        return;
    }

    if let Ok(mut file) = NamedTempFile::new() {
        if file.write_all(data).is_ok() && file.flush().is_ok() {
            if let Some(path) = file.path().to_str() {
                let _ = talon::binary_analyzer::analyze_pe(path);

                use pelite::PeFile;
                if let Ok(pe) = PeFile::from_bytes(data) {
                    let _ = pe.exports();
                    let _ = pe.imports();
                    let _ = pe.resources();
                    let _ = pe.base_relocs();

                    if let Ok(sections) = pe.section_headers() {
                        for section in sections {
                            let _ = section.name();
                            let _ = section.virtual_range();
                        }
                    }
                }
            }
        }
    }
});
