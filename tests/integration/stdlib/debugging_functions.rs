use crate::common::TalonTestHarness;

#[test]
fn test_disasm() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
let code_bytes = bytes([0x48, 0x31, 0xc0])
print("disasm test")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_cfg() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("cfg test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_taint() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("taint test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_emulate() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("emulate test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_gdb_run() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("gdb_run test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_debug_attach() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("debug_attach test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_debug_step() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("debug_step test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_debug_continue() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("debug_continue test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_debug_read_mem() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("debug_read_mem test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_debug_write_mem() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("debug_write_mem test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_debug_read_reg() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("debug_read_reg test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_debug_write_reg() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("debug_write_reg test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_breakpoint() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("breakpoint test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}
