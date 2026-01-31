#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > 10000 {
        return;
    }

    if let Ok(format_string) = std::str::from_utf8(data) {
        let offset = (data[0] as usize % 20) + 1;
        let target_addr = u64::from_le_bytes([
            data.get(1).copied().unwrap_or(0),
            data.get(2).copied().unwrap_or(0),
            data.get(3).copied().unwrap_or(0),
            data.get(4).copied().unwrap_or(0),
            data.get(5).copied().unwrap_or(0),
            data.get(6).copied().unwrap_or(0),
            data.get(7).copied().unwrap_or(0),
            data.get(8).copied().unwrap_or(0),
        ]);

        let value = u32::from_le_bytes([
            data.get(9).copied().unwrap_or(0),
            data.get(10).copied().unwrap_or(0),
            data.get(11).copied().unwrap_or(0),
            data.get(12).copied().unwrap_or(0),
        ]);

        if let Ok(mut builder) = talon::fmtstr_tools::FormatStringBuilder::new(offset) {
            let _ = builder.leak_stack(10);
            let _ = builder.leak_address(target_addr);
            let _ = builder.write_byte(target_addr, value as u8);
            let _ = builder.write_short(target_addr, value as u16);
            let _ = builder.write_word(target_addr, value);
            let _ = builder.write_qword(target_addr, target_addr);
            let _ = builder.build();

            if format_string.len() < 500 {
                let _ = talon::format_string::parse_format_string(format_string);
                let _ = talon::format_string::validate_format_string(format_string);
            }
        }
    }
});
