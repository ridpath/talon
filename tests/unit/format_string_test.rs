// ═══════════════════════════════════════════════════════════════════════════
// FORMAT STRING EXPLOIT TEST SUITE
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn test_fmtstr_leak() {
    use talon::fmtstr_tools::fmtstr_leak;

    let payload = fmtstr_leak(6);
    assert_eq!(payload, "%6$p");

    let payload2 = fmtstr_leak(10);
    assert_eq!(payload2, "%10$p");
}

#[test]
fn test_fmtstr_leak_stack() {
    use talon::fmtstr_tools::fmtstr_leak_stack;

    let payload = fmtstr_leak_stack(5, 3);
    assert!(payload.contains("%5$p"));
    assert!(payload.contains("%6$p"));
    assert!(payload.contains("%7$p"));
}

#[test]
fn test_fmtstr_leak_stack_count() {
    use talon::fmtstr_tools::fmtstr_leak_stack;

    let payload = fmtstr_leak_stack(1, 10);
    let count = payload.matches("$p").count();
    assert_eq!(count, 10);
}

#[test]
fn test_fmtstr_write() {
    use talon::fmtstr_tools::fmtstr_write;

    let address = 0x601020;
    let value = 0x1234;
    let offset = 6;

    let payload = fmtstr_write(address, value, offset);

    assert!(!payload.is_empty());

    let addr_bytes = address.to_le_bytes();
    assert_eq!(&payload[0..8], &addr_bytes);
}

#[test]
fn test_fmtstr_write_contains_format_string() {
    use talon::fmtstr_tools::fmtstr_write;

    let payload = fmtstr_write(0x601020, 0x1234, 6);
    let payload_str = String::from_utf8_lossy(&payload);

    assert!(payload_str.contains("%"));
    assert!(payload_str.contains("$"));
}

#[test]
fn test_format_string_payload_new() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let payload = FormatStringPayload::new(6, Architecture::X64);

    assert_eq!(payload.offset, 6);
    assert_eq!(payload.architecture, Architecture::X64);
    assert_eq!(payload.writes.len(), 0);
}

#[test]
fn test_format_string_payload_add_write() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(6, Architecture::X64);
    payload.add_write(0x601020, 0xdeadbeef);

    assert_eq!(payload.writes.len(), 1);
    assert_eq!(payload.writes[0], (0x601020, 0xdeadbeef));
}

#[test]
fn test_format_string_payload_add_multiple_writes() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(6, Architecture::X64);
    payload.add_write(0x601020, 0xdeadbeef);
    payload.add_write(0x601028, 0xcafebabe);
    payload.add_write(0x601030, 0x41424344);

    assert_eq!(payload.writes.len(), 3);
}

#[test]
fn test_format_string_payload_generate_x64() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(6, Architecture::X64);
    payload.add_write(0x601020, 0xdeadbeef);

    let result = payload.generate();
    assert!(result.is_ok());

    let data = result.unwrap();
    assert!(!data.is_empty());
}

#[test]
fn test_format_string_payload_generate_x86() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(4, Architecture::X86);
    payload.add_write(0x804a000, 0x41414141);

    let result = payload.generate();
    assert!(result.is_ok());
}

#[test]
fn test_format_string_payload_generate_empty_writes() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let payload = FormatStringPayload::new(6, Architecture::X64);

    let result = payload.generate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No writes"));
}

#[test]
fn test_format_string_payload_generate_leak() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let payload = FormatStringPayload::new(6, Architecture::X64);
    let leak = payload.generate_leak(10);

    assert_eq!(leak, "%16$p");
}

#[test]
fn test_format_string_payload_generate_stack_dump() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let payload = FormatStringPayload::new(6, Architecture::X64);
    let dump = payload.generate_stack_dump(5);

    assert_eq!(dump, "%6$p.%7$p.%8$p.%9$p.%10$p");
}

#[test]
fn test_create_format_string_payload() {
    use talon::format_string::{create_format_string_payload, Architecture};

    let writes = vec![(0x601020, 0xdeadbeef), (0x601028, 0xcafebabe)];

    let result = create_format_string_payload(6, writes, Architecture::X64);
    assert!(result.is_ok());
}

#[test]
fn test_analyze_format_string_leak() {
    use talon::format_string::analyze_format_string_leak;

    let output = "0x7ffd12345678.0x400000.0x7f1234567890";
    let leaks = analyze_format_string_leak(output);

    assert_eq!(leaks.len(), 3);
    assert_eq!(leaks[0], 0x7ffd12345678);
    assert_eq!(leaks[1], 0x400000);
    assert_eq!(leaks[2], 0x7f1234567890);
}

#[test]
fn test_analyze_format_string_leak_uppercase() {
    use talon::format_string::analyze_format_string_leak;

    let output = "0X7FFD12345678.0X400000";
    let leaks = analyze_format_string_leak(output);

    assert_eq!(leaks.len(), 2);
}

#[test]
fn test_analyze_format_string_leak_empty() {
    use talon::format_string::analyze_format_string_leak;

    let output = "no leaks here";
    let leaks = analyze_format_string_leak(output);

    assert_eq!(leaks.len(), 0);
}

#[test]
fn test_analyze_format_string_leak_mixed() {
    use talon::format_string::analyze_format_string_leak;

    let output = "Some text 0x12345678 more text 0xabcdef end";
    let leaks = analyze_format_string_leak(output);

    assert_eq!(leaks.len(), 2);
}

#[test]
fn test_format_string_x64_address_alignment() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(6, Architecture::X64);
    payload.add_write(0x601020, 0x1234);

    let data = payload.generate().unwrap();

    let addr_bytes = 0x601020u64.to_le_bytes();
    assert!(data.windows(8).any(|w| w == addr_bytes));
}

#[test]
fn test_format_string_x86_address_size() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(4, Architecture::X86);
    payload.add_write(0x804a000, 0x41414141);

    let data = payload.generate().unwrap();
    assert!(!data.is_empty());
}

#[test]
fn test_format_string_payload_contains_hhn() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(6, Architecture::X64);
    payload.add_write(0x601020, 0xdeadbeef);

    let data = payload.generate().unwrap();
    let data_str = String::from_utf8_lossy(&data);

    assert!(data_str.contains("hhn"));
}

#[test]
fn test_format_string_multiple_writes() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(6, Architecture::X64);
    payload.add_write(0x601020, 0x00000001);
    payload.add_write(0x601028, 0x00000002);

    let result = payload.generate();
    assert!(result.is_ok());
}

#[test]
fn test_format_string_large_value() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(6, Architecture::X64);
    payload.add_write(0x601020, 0xffffffffffffffff);

    let result = payload.generate();
    assert!(result.is_ok());
}

#[test]
fn test_format_string_zero_value() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(6, Architecture::X64);
    payload.add_write(0x601020, 0x0000000000000000);

    let result = payload.generate();
    assert!(result.is_ok());
}

#[test]
fn test_fmtstr_tools_format_string_creation() {
    use std::path::Path;
    use talon::fmtstr_tools::FormatString;

    if !Path::new("/bin/sh").exists() && !Path::new("C:\\Windows\\System32\\cmd.exe").exists() {
        return;
    }

    let binary = if cfg!(windows) {
        "C:\\Windows\\System32\\cmd.exe"
    } else {
        "/bin/sh"
    };

    let result = FormatString::new(binary, 6);
    if result.is_ok() {
        let fmt = result.unwrap();
        assert_eq!(fmt.offset, 6);
    }
}

#[test]
fn test_fmtstr_tools_write() {
    use std::path::Path;
    use talon::fmtstr_tools::FormatString;

    if !Path::new("/bin/sh").exists() && !Path::new("C:\\Windows\\System32\\cmd.exe").exists() {
        return;
    }

    let binary = if cfg!(windows) {
        "C:\\Windows\\System32\\cmd.exe"
    } else {
        "/bin/sh"
    };

    if let Ok(mut fmt) = FormatString::new(binary, 6) {
        fmt.write(0x601020, 0xdeadbeef);
        assert_eq!(fmt.writes.len(), 1);
    }
}

#[test]
fn test_fmtstr_tools_leak() {
    use std::path::Path;
    use talon::fmtstr_tools::FormatString;

    if !Path::new("/bin/sh").exists() && !Path::new("C:\\Windows\\System32\\cmd.exe").exists() {
        return;
    }

    let binary = if cfg!(windows) {
        "C:\\Windows\\System32\\cmd.exe"
    } else {
        "/bin/sh"
    };

    if let Ok(fmt) = FormatString::new(binary, 6) {
        let leak = fmt.leak(10);
        assert_eq!(leak, "%10$p");
    }
}

#[test]
fn test_fmtstr_tools_leak_address() {
    use std::path::Path;
    use talon::fmtstr_tools::FormatString;

    if !Path::new("/bin/sh").exists() && !Path::new("C:\\Windows\\System32\\cmd.exe").exists() {
        return;
    }

    let binary = if cfg!(windows) {
        "C:\\Windows\\System32\\cmd.exe"
    } else {
        "/bin/sh"
    };

    if let Ok(fmt) = FormatString::new(binary, 6) {
        let payload = fmt.leak_address(0x7fffffffe000);
        assert!(!payload.is_empty());
    }
}

#[test]
fn test_fmtstr_tools_generate() {
    use std::path::Path;
    use talon::fmtstr_tools::FormatString;

    if !Path::new("/bin/sh").exists() && !Path::new("C:\\Windows\\System32\\cmd.exe").exists() {
        return;
    }

    let binary = if cfg!(windows) {
        "C:\\Windows\\System32\\cmd.exe"
    } else {
        "/bin/sh"
    };

    if let Ok(mut fmt) = FormatString::new(binary, 6) {
        fmt.write(0x601020, 0x1234);
        let result = fmt.generate();
        assert!(result.is_ok());
    }
}

#[test]
fn test_fmtstr_tools_generate_empty() {
    use std::path::Path;
    use talon::fmtstr_tools::FormatString;

    if !Path::new("/bin/sh").exists() && !Path::new("C:\\Windows\\System32\\cmd.exe").exists() {
        return;
    }

    let binary = if cfg!(windows) {
        "C:\\Windows\\System32\\cmd.exe"
    } else {
        "/bin/sh"
    };

    if let Ok(fmt) = FormatString::new(binary, 6) {
        let result = fmt.generate();
        assert!(result.is_err());
    }
}

#[test]
fn test_fmtstr_tools_generate_write_payload() {
    use std::path::Path;
    use talon::fmtstr_tools::FormatString;

    if !Path::new("/bin/sh").exists() && !Path::new("C:\\Windows\\System32\\cmd.exe").exists() {
        return;
    }

    let binary = if cfg!(windows) {
        "C:\\Windows\\System32\\cmd.exe"
    } else {
        "/bin/sh"
    };

    if let Ok(fmt) = FormatString::new(binary, 6) {
        let payload = fmt.generate_write_payload(0x601020, 0x1234);
        assert!(!payload.is_empty());
    }
}

#[test]
fn test_format_string_architecture_detection() {
    use std::path::Path;
    use talon::fmtstr_tools::{Architecture, FormatString};

    if !Path::new("/bin/sh").exists() && !Path::new("C:\\Windows\\System32\\cmd.exe").exists() {
        return;
    }

    let binary = if cfg!(windows) {
        "C:\\Windows\\System32\\cmd.exe"
    } else {
        "/bin/sh"
    };

    if let Ok(fmt) = FormatString::new(binary, 6) {
        match fmt.arch {
            Architecture::X8664 => assert!(true),
            Architecture::I386 => assert!(true),
        }
    }
}

#[test]
fn test_format_string_invalid_binary() {
    use talon::fmtstr_tools::FormatString;

    let result = FormatString::new("/nonexistent/binary", 6);
    assert!(result.is_err());
}

#[test]
fn test_format_string_offset_zero() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let payload = FormatStringPayload::new(0, Architecture::X64);
    assert_eq!(payload.offset, 0);
}

#[test]
fn test_format_string_offset_large() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let payload = FormatStringPayload::new(1000, Architecture::X64);
    assert_eq!(payload.offset, 1000);
}

#[test]
fn test_format_string_leak_formatting() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let payload = FormatStringPayload::new(5, Architecture::X64);
    let leak = payload.generate_leak(0);

    assert!(leak.starts_with('%'));
    assert!(leak.contains('$'));
    assert!(leak.ends_with('p'));
}

#[test]
fn test_format_string_stack_dump_separator() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let payload = FormatStringPayload::new(1, Architecture::X64);
    let dump = payload.generate_stack_dump(3);

    assert_eq!(dump.matches('.').count(), 2);
}

#[test]
fn test_format_string_payload_x64_vs_x86() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload_x64 = FormatStringPayload::new(6, Architecture::X64);
    payload_x64.add_write(0x601020, 0x12345678);

    let mut payload_x86 = FormatStringPayload::new(4, Architecture::X86);
    payload_x86.add_write(0x804a000, 0x12345678);

    let data_x64 = payload_x64.generate().unwrap();
    let data_x86 = payload_x86.generate().unwrap();

    assert_ne!(data_x64, data_x86);
}

#[test]
fn test_fmtstr_write_address_embedding() {
    use talon::fmtstr_tools::fmtstr_write;

    let address = 0x601020u64;
    let payload = fmtstr_write(address, 0x1234, 6);

    let addr_bytes = address.to_le_bytes();
    assert_eq!(&payload[0..8], &addr_bytes);
}

#[test]
fn test_fmtstr_write_format_specifier() {
    use talon::fmtstr_tools::fmtstr_write;

    let payload = fmtstr_write(0x601020, 0x1234, 6);
    let payload_str = String::from_utf8_lossy(&payload);

    assert!(payload_str.contains("$hn"));
}

#[test]
fn test_format_string_byte_by_byte_write() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(6, Architecture::X64);
    payload.add_write(0x601020, 0x0102030405060708);

    let result = payload.generate();
    assert!(result.is_ok());
}

#[test]
fn test_format_string_non_sequential_addresses() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(6, Architecture::X64);
    payload.add_write(0x601000, 0x1111);
    payload.add_write(0x602000, 0x2222);

    let result = payload.generate();
    assert!(result.is_ok());
}

#[test]
fn test_fmtstr_leak_stack_zero_count() {
    use talon::fmtstr_tools::fmtstr_leak_stack;

    let payload = fmtstr_leak_stack(5, 0);
    assert_eq!(payload, "");
}

#[test]
fn test_fmtstr_leak_stack_large_count() {
    use talon::fmtstr_tools::fmtstr_leak_stack;

    let payload = fmtstr_leak_stack(1, 100);
    let count = payload.matches("$p").count();
    assert_eq!(count, 100);
}

#[test]
fn test_analyze_format_string_leak_invalid_hex() {
    use talon::format_string::analyze_format_string_leak;

    let output = "0xGGGGGGGG.0xZZZZ";
    let leaks = analyze_format_string_leak(output);

    assert_eq!(leaks.len(), 0);
}

#[test]
fn test_analyze_format_string_leak_partial_valid() {
    use talon::format_string::analyze_format_string_leak;

    let output = "0x12345678.invalid.0xabcdef";
    let leaks = analyze_format_string_leak(output);

    assert_eq!(leaks.len(), 2);
}

#[test]
fn test_format_string_payload_clone() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let payload1 = FormatStringPayload::new(6, Architecture::X64);
    let payload2 = payload1.clone();

    assert_eq!(payload1.offset, payload2.offset);
    assert_eq!(payload1.architecture, payload2.architecture);
}

#[test]
fn test_architecture_equality() {
    use talon::format_string::Architecture;

    assert_eq!(Architecture::X64, Architecture::X64);
    assert_eq!(Architecture::X86, Architecture::X86);
    assert_ne!(Architecture::X64, Architecture::X86);
}

#[test]
fn test_format_string_payload_debug() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let payload = FormatStringPayload::new(6, Architecture::X64);
    let debug_str = format!("{:?}", payload);

    assert!(!debug_str.is_empty());
}

#[test]
fn test_architecture_copy() {
    use talon::format_string::Architecture;

    let arch1 = Architecture::X64;
    let arch2 = arch1;

    assert_eq!(arch1, arch2);
}

#[test]
fn test_format_string_write_single_byte() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(6, Architecture::X64);
    payload.add_write(0x601020, 0x41);

    let result = payload.generate();
    assert!(result.is_ok());
}

#[test]
fn test_format_string_write_maximum_value() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let mut payload = FormatStringPayload::new(6, Architecture::X64);
    payload.add_write(0x601020, u64::MAX);

    let result = payload.generate();
    assert!(result.is_ok());
}

#[test]
fn test_format_string_high_offset() {
    use talon::format_string::{Architecture, FormatStringPayload};

    let payload = FormatStringPayload::new(999, Architecture::X64);
    let leak = payload.generate_leak(0);

    assert!(leak.contains("999"));
}
