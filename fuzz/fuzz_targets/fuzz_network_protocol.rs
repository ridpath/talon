#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 4 {
        return;
    }

    let protocol = match data[0] % 4 {
        0 => "tcp",
        1 => "udp",
        2 => "http",
        3 => "websocket",
        _ => "tcp",
    };

    if data.len() > 10 {
        if let Ok(payload_str) = std::str::from_utf8(&data[4..]) {
            let _ = talon::network_tools::parse_packet(payload_str, protocol);
            let _ = talon::network_tools::encode_payload(payload_str.as_bytes(), protocol);
        }
    }

    if data.len() >= 8 {
        let port = u16::from_le_bytes([data[1], data[2]]);
        if port > 0 && port < 65535 {
            let _ = talon::network_tools::validate_port(port);
        }
    }

    if data.len() > 4 {
        let _ = talon::network_tools::detect_protocol(&data[4..]);
    }
});
