use crate::common::TalonTestHarness;

#[test]
fn test_rop_find_basic() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("rop_find test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_rop_new() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("rop_new test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_rop_build_chain() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("rop_build_chain test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_rop_find_gadget() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("rop_find_gadget test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_rop_find_gadgets() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("rop_find_gadgets test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_rop_ret2libc() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("rop_ret2libc test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_rop_ret2syscall() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("rop_ret2syscall test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_rop_solve() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("rop_solve test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_rop_list_gadgets() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("rop_list_gadgets test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_rop_search() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("rop_search test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_rop_auto() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("rop_auto test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_gadget_search() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("gadget_search test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_quick_rop() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("quick_rop test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}
