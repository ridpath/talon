use crate::common::TalonTestHarness;

#[test]
fn test_sha256_hash() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = "Hello World"
let hash = sha256(data)
print(len(hash))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_sha256_bytes() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = bytes("test")
let hash = sha256(data)
print(len(hash))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_md5_hash() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = "Hello World"
let hash = md5(data)
print(len(hash))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_sha1_hash() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = "test"
let hash = sha1(data)
print(len(hash))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_sha512_hash() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let data = "test"
let hash = sha512(data)
print(len(hash))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_random_bytes() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let random = random_bytes(16)
print(len(random))
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_random_int() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let r1 = random_int(100)
let r2 = random_int(100)
print(r1)
print(r2)
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_hash_collision_detection() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("hash_collision test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_weak_keys_detection() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("weak_keys test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_padding_oracle() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("padding_oracle test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_timing_attack() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("timing_attack test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_aes_padding_attack() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("aes_padding_attack test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_rsa_factorize() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("rsa_factorize test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_bleichenbacher() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("bleichenbacher test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}
