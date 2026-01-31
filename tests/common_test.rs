mod common;

use common::{
    assert_u64, create_rop_gadget_binary, create_shellcode_test_env, TalonTestHarness, Vuln,
};
use std::fs;

#[test]
fn test_harness_basic_functionality() {
    let harness = TalonTestHarness::new();
    assert!(harness.temp_dir().exists());
}

#[test]
fn test_mock_binary_with_buffer_overflow() {
    let mut harness = TalonTestHarness::new();
    let vulns = vec![Vuln::BufferOverflow { offset: 72 }];
    let bin_path = harness.mock_binary("vuln_bof", &vulns);

    assert!(bin_path.exists());
    let content = fs::read(&bin_path).unwrap();

    assert_eq!(&content[0..4], &[0x7f, 0x45, 0x4c, 0x46]);
    assert!(content.len() >= 256);
}

#[test]
fn test_mock_binary_with_format_string() {
    let mut harness = TalonTestHarness::new();
    let vulns = vec![Vuln::FormatString { vuln_arg: 1 }];
    let bin_path = harness.mock_binary("vuln_fmt", &vulns);

    assert!(bin_path.exists());
}

#[test]
fn test_mock_binary_with_uaf() {
    let mut harness = TalonTestHarness::new();
    let vulns = vec![Vuln::UseAfterFree { heap_chunk: 128 }];
    let bin_path = harness.mock_binary("vuln_uaf", &vulns);

    assert!(bin_path.exists());
}

#[test]
fn test_create_vulnerable_c_source_buffer_overflow() {
    let harness = TalonTestHarness::new();
    let vuln = Vuln::BufferOverflow { offset: 64 };
    let source_path = harness.create_vulnerable_c_source("bof_test", &vuln);

    assert!(source_path.exists());
    let content = fs::read_to_string(&source_path).unwrap();
    assert!(content.contains("strcpy"));
    assert!(content.contains("buffer[64]"));
}

#[test]
fn test_create_vulnerable_c_source_format_string() {
    let harness = TalonTestHarness::new();
    let vuln = Vuln::FormatString { vuln_arg: 2 };
    let source_path = harness.create_vulnerable_c_source("fmt_test", &vuln);

    assert!(source_path.exists());
    let content = fs::read_to_string(&source_path).unwrap();
    assert!(content.contains("printf(argv[2])"));
}

#[test]
fn test_assert_helpers() {
    let harness = TalonTestHarness::new();

    let result = harness.assert_contains("exploit successful", "successful");
    assert!(result.is_ok());

    let result = harness.assert_contains("exploit failed", "successful");
    assert!(result.is_err());

    let result = harness.assert_not_contains("exploit successful", "failed");
    assert!(result.is_ok());

    let result = harness.assert_not_contains("exploit failed", "failed");
    assert!(result.is_err());
}

#[test]
fn test_exploit_success_assertion() {
    let harness = TalonTestHarness::new();

    let result = harness.assert_exploit_success("Shell spawned successfully");
    assert!(result.is_ok());

    let result = harness.assert_exploit_success("error: segmentation fault");
    assert!(result.is_err());

    let result = harness.assert_exploit_success("failed to connect");
    assert!(result.is_err());
}

#[test]
fn test_create_test_file() {
    let harness = TalonTestHarness::new();
    let content = "test payload data";
    let file_path = harness.create_test_file("payload.bin", content);

    assert!(file_path.exists());
    let read_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(read_content, content);
}

#[test]
fn test_get_mock_binary() {
    let mut harness = TalonTestHarness::new();
    let vulns = vec![Vuln::BufferOverflow { offset: 100 }];
    harness.mock_binary("test_bin", &vulns);

    let retrieved = harness.get_mock_binary("test_bin");
    assert!(retrieved.is_some());

    let non_existent = harness.get_mock_binary("nonexistent");
    assert!(non_existent.is_none());
}

#[test]
fn test_rop_gadget_binary_generation() {
    let binary = create_rop_gadget_binary();

    assert!(binary.len() >= 1024);
    assert_eq!(&binary[0..4], &[0x7f, 0x45, 0x4c, 0x46]);

    assert!(binary.windows(2).any(|w| w == [0x5f, 0xc3]));
    assert!(binary.windows(2).any(|w| w == [0x5e, 0xc3]));
}

#[test]
fn test_shellcode_test_env() {
    let shellcode = create_shellcode_test_env();

    assert!(!shellcode.is_empty());
    assert_eq!(shellcode[0], 0x48);
    assert!(shellcode.len() > 10);
}

#[test]
fn test_assert_u64() {
    assert_u64(0xdeadbeef, 0xdeadbeef);
}

#[test]
#[should_panic]
fn test_assert_u64_fail() {
    assert_u64(0xdeadbeef, 0xcafebabe);
}

#[test]
fn test_multiple_vulnerabilities() {
    let mut harness = TalonTestHarness::new();
    let vulns = vec![
        Vuln::BufferOverflow { offset: 64 },
        Vuln::FormatString { vuln_arg: 1 },
        Vuln::StackPivot {
            gadget_offset: 0x1000,
        },
    ];
    let bin_path = harness.mock_binary("multi_vuln", &vulns);

    assert!(bin_path.exists());
    let content = fs::read(&bin_path).unwrap();
    assert!(content.len() >= 256);
}

#[test]
fn test_temp_dir_isolation() {
    let harness1 = TalonTestHarness::new();
    let harness2 = TalonTestHarness::new();

    assert_ne!(harness1.temp_dir(), harness2.temp_dir());
}

#[test]
fn test_run_script_basic() {
    let harness = TalonTestHarness::new();
    let code = r#"
let x = 42
print(x)
"#;

    let result = harness.run_script(code);
    assert!(result.is_ok());
}

#[test]
fn test_run_file() {
    let harness = TalonTestHarness::new();
    let script_path = harness.create_test_file("test.talon", "print('hello')");

    let result = harness.run_file(&script_path);
    assert!(result.is_ok());
}

#[test]
fn test_run_file_nonexistent() {
    let harness = TalonTestHarness::new();
    let fake_path = harness.temp_dir().join("nonexistent.talon");

    let result = harness.run_file(&fake_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("does not exist"));
}
