use crate::common::TalonTestHarness;

#[test]
fn test_http_get() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("http_get test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_http_post() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("http_post test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_http_request() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("http_request test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_web_scan() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("web_scan test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_js_spray() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("js_spray test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_type_confuse() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("type_confuse test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_uaf_dom() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("uaf_dom test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_sandbox_escape() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("sandbox_escape test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_jit_exploit() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("jit_exploit test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_docker_escape() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("docker_escape test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_kube_escape() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("kube_escape test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_metadata_exploit() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("metadata_exploit test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_iam_escalate() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("iam_escalate test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}
