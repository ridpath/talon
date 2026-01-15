use crate::common::TalonTestHarness;

#[test]
fn test_ethernet_packet() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("ethernet test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_ip_packet() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("ip_packet test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_tcp_packet() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("tcp_packet test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_udp_packet() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("udp_packet test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_icmp_packet() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("icmp_packet test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_arp_packet() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("arp_packet test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_dns_query() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("dns_query test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_dns_resolve() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("dns_resolve test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_port_scan() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("port_scan test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_tls_handshake() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("tls_handshake test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}

#[test]
fn test_network_proxy() {
    let mut harness = TalonTestHarness::new();
    let code = r#"
print("network_proxy test placeholder")
"#;
    assert!(harness.run_script(code).is_ok());
}
