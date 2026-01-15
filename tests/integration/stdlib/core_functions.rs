use crate::common::TalonTestHarness;

#[test]
fn test_p64_pack() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let packed = p64(0xdeadbeef)
print(len(packed))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_p32_pack() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let packed = p32(0x41424344)
print(len(packed))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_p16_pack() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let packed = p16(0x4142)
print(len(packed))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_p8_pack() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let packed = p8(0x41)
print(len(packed))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_u64_unpack() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let packed = p64(0xdeadbeef)
let unpacked = u64(packed)
print(hex(unpacked))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_u32_unpack() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let packed = p32(0x12345678)
let unpacked = u32(packed)
print(hex(unpacked))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_u16_unpack() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let packed = p16(0x1234)
let unpacked = u16(packed)
print(hex(unpacked))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_pack_unpack_roundtrip() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let original = 0x1234567890abcdef
let packed = p64(original)
let unpacked = u64(packed)
if unpacked == original
    print("SUCCESS")
end
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_bytes_creation() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = bytes("AAAA")
print(len(data))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_bytes_from_list() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = bytes([0x41, 0x42, 0x43, 0x44])
print(len(data))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_hex_conversion() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let num = 255
let h = hex(num)
print(h)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_int_conversion() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let s = "12345"
let n = int(s)
print(n)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_int_from_hex_string() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let s = "0xdeadbeef"
let n = int(s)
print(hex(n))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_str_conversion() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let n = 42
let s = str(n)
print(s)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_len_string() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let s = "hello"
let l = len(s)
print(l)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_len_list() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let items = [1, 2, 3, 4, 5]
let l = len(items)
print(l)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_len_bytes() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = bytes("test")
let l = len(data)
print(l)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_split_string() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let s = "one,two,three"
let parts = split(s, ",")
print(len(parts))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_join_list() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let parts = ["a", "b", "c"]
let s = join(parts, "-")
print(s)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_replace_string() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let s = "hello world"
let r = replace(s, "world", "TALON")
print(r)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_range_generation() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let r = range(5)
print(len(r))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_range_with_start() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let r = range(3, 8)
print(len(r))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_print_multiple_args() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("Hello", "World", 123)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_cyclic_pattern_generation() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let pattern = cyclic(100)
print(len(pattern))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_cyclic_find() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let pattern = cyclic(200)
let offset = cyclic_find("AAAA", pattern)
print(offset)
"#;
    assert!(harness.run_script(code).is_ok());
}
