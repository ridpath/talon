use crate::common::TalonTestHarness;

#[test]
fn test_base64_encode() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = "Hello World"
let encoded = base64_encode(data)
print(encoded)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_base64_decode() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let encoded = "SGVsbG8gV29ybGQ="
let decoded = base64_decode(encoded)
print(decoded)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_base64_roundtrip() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let original = "Test Data"
let encoded = base64_encode(original)
let decoded = base64_decode(encoded)
print(decoded)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_url_encode() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = "hello world"
let encoded = url_encode(data)
print(encoded)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_url_decode() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let encoded = "hello%20world"
let decoded = url_decode(encoded)
print(decoded)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_gzip_compress() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = "Test data for compression"
let compressed = gzip_compress(data)
print(len(compressed))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_gzip_decompress() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = "Test data"
let compressed = gzip_compress(data)
let decompressed = gzip_decompress(compressed)
print(decompressed)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_zlib_compress() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = "Test data for zlib"
let compressed = zlib_compress(data)
print(len(compressed))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_zlib_decompress() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = "Test data"
let compressed = zlib_compress(data)
let decompressed = zlib_decompress(compressed)
print(decompressed)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_hex_encoding() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let num = 0xdeadbeef
let h = hex(num)
print(h)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_regex_find() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let text = "The port is 8080"
let matches = regex_find(text, "[0-9]+")
print(matches)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_regex_replace() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let text = "hello 123 world 456"
let replaced = regex_replace(text, "[0-9]+", "X")
print(replaced)
"#;
    assert!(harness.run_script(code).is_ok());
}
