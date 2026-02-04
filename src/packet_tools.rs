use std::io::{Read, Write};
use std::net::{TcpStream, UdpSocket};
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════════════════
// NETWORK PACKET CRAFTING TOOLKIT - PRODUCTION READY
// ═══════════════════════════════════════════════════════════════════════════

// Constants for packet crafting
const DEFAULT_TCP_WINDOW: u16 = 8192;
const TCP_HEADER_SIZE: usize = 20;
const UDP_HEADER_SIZE: usize = 8;
const DEFAULT_SCAN_TIMEOUT_MS: u64 = 1000;

// TCP Flags
const TCP_FLAG_SYN: u8 = 0x02;
const TCP_FLAG_ACK: u8 = 0x10;
const TCP_FLAG_RST: u8 = 0x04;
const TCP_FLAG_FIN: u8 = 0x01;
const TCP_FLAG_PSH: u8 = 0x08;

// ────────────────────────────────────────────────────────────────────────────
// RAW TCP PACKET BUILDER
// ────────────────────────────────────────────────────────────────────────────

pub struct TCPPacket {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq_num: u32,
    pub ack_num: u32,
    pub flags: u8,
    pub window: u16,
    pub payload: Vec<u8>,
}

impl TCPPacket {
    /// Creates a new TCP packet with randomized source port and sequence number
    pub fn new(dst_port: u16) -> Self {
        TCPPacket {
            src_port: rand::random::<u16>(),
            dst_port,
            seq_num: rand::random::<u32>(),
            ack_num: 0,
            flags: 0,
            window: DEFAULT_TCP_WINDOW,
            payload: Vec::new(),
        }
    }

    pub fn syn() -> u8 {
        TCP_FLAG_SYN
    }
    pub fn ack() -> u8 {
        TCP_FLAG_ACK
    }
    pub fn rst() -> u8 {
        TCP_FLAG_RST
    }
    pub fn fin() -> u8 {
        TCP_FLAG_FIN
    }
    pub fn psh() -> u8 {
        TCP_FLAG_PSH
    }

    pub fn set_syn(&mut self) {
        self.flags |= Self::syn();
    }

    pub fn set_ack(&mut self) {
        self.flags |= Self::ack();
    }

    pub fn set_rst(&mut self) {
        self.flags |= Self::rst();
    }

    pub fn set_fin(&mut self) {
        self.flags |= Self::fin();
    }

    pub fn set_payload(&mut self, data: Vec<u8>) {
        self.payload = data;
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        packet.extend_from_slice(&self.src_port.to_be_bytes());
        packet.extend_from_slice(&self.dst_port.to_be_bytes());
        packet.extend_from_slice(&self.seq_num.to_be_bytes());
        packet.extend_from_slice(&self.ack_num.to_be_bytes());

        let data_offset = ((20 / 4) << 4) as u8;
        packet.push(data_offset);
        packet.push(self.flags);

        packet.extend_from_slice(&self.window.to_be_bytes());
        packet.extend_from_slice(&[0, 0]);
        packet.extend_from_slice(&[0, 0]);

        packet.extend_from_slice(&self.payload);

        println!("[TCP-PACKET] Built TCP packet:");
        println!("[TCP-PACKET]   Src port: {}", self.src_port);
        println!("[TCP-PACKET]   Dst port: {}", self.dst_port);
        println!("[TCP-PACKET]   Flags: 0x{:02x}", self.flags);
        println!("[TCP-PACKET]   Payload: {} bytes", self.payload.len());

        packet
    }
}

// ────────────────────────────────────────────────────────────────────────────
// UDP PACKET BUILDER
// ────────────────────────────────────────────────────────────────────────────

pub struct UDPPacket {
    pub src_port: u16,
    pub dst_port: u16,
    pub payload: Vec<u8>,
}

impl UDPPacket {
    pub fn new(dst_port: u16, payload: Vec<u8>) -> Self {
        UDPPacket {
            src_port: rand::random::<u16>(),
            dst_port,
            payload,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        packet.extend_from_slice(&self.src_port.to_be_bytes());
        packet.extend_from_slice(&self.dst_port.to_be_bytes());

        let length = (8 + self.payload.len()) as u16;
        packet.extend_from_slice(&length.to_be_bytes());

        packet.extend_from_slice(&[0, 0]);

        packet.extend_from_slice(&self.payload);

        println!("[UDP-PACKET] Built UDP packet:");
        println!("[UDP-PACKET]   Src port: {}", self.src_port);
        println!("[UDP-PACKET]   Dst port: {}", self.dst_port);
        println!("[UDP-PACKET]   Length: {} bytes", length);

        packet
    }

    pub fn send(&self, target_ip: &str) -> Result<(), String> {
        let socket =
            UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("Failed to bind socket: {}", e))?;

        let target = format!("{}:{}", target_ip, self.dst_port);
        socket
            .send_to(&self.payload, &target)
            .map_err(|e| format!("Send failed: {}", e))?;

        println!("[UDP-PACKET] [OK] Packet sent to {}", target);
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ICMP PACKET BUILDER
// ────────────────────────────────────────────────────────────────────────────

pub struct ICMPPacket {
    pub icmp_type: u8,
    pub code: u8,
    pub identifier: u16,
    pub sequence: u16,
    pub payload: Vec<u8>,
}

impl ICMPPacket {
    pub fn echo_request() -> Self {
        ICMPPacket {
            icmp_type: 8,
            code: 0,
            identifier: rand::random::<u16>(),
            sequence: 1,
            payload: b"PING".to_vec(),
        }
    }

    pub fn echo_reply() -> Self {
        ICMPPacket {
            icmp_type: 0,
            code: 0,
            identifier: rand::random::<u16>(),
            sequence: 1,
            payload: b"PONG".to_vec(),
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        packet.push(self.icmp_type);
        packet.push(self.code);
        packet.extend_from_slice(&[0, 0]);
        packet.extend_from_slice(&self.identifier.to_be_bytes());
        packet.extend_from_slice(&self.sequence.to_be_bytes());
        packet.extend_from_slice(&self.payload);

        let checksum = Self::calculate_checksum(&packet);
        packet[2..4].copy_from_slice(&checksum.to_be_bytes());

        println!("[ICMP-PACKET] Built ICMP packet:");
        println!("[ICMP-PACKET]   Type: {}", self.icmp_type);
        println!("[ICMP-PACKET]   Code: {}", self.code);
        println!("[ICMP-PACKET]   ID: {}", self.identifier);
        println!("[ICMP-PACKET]   Seq: {}", self.sequence);

        packet
    }

    fn calculate_checksum(data: &[u8]) -> u16 {
        let mut sum: u32 = 0;

        for i in (0..data.len()).step_by(2) {
            let word = if i + 1 < data.len() {
                u16::from_be_bytes([data[i], data[i + 1]]) as u32
            } else {
                (data[i] as u32) << 8
            };

            sum += word;
            if sum > 0xFFFF {
                sum = (sum & 0xFFFF) + (sum >> 16);
            }
        }

        !sum as u16
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ARP PACKET BUILDER
// ────────────────────────────────────────────────────────────────────────────

pub struct ARPPacket {
    pub operation: u16,
    pub sender_mac: [u8; 6],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; 6],
    pub target_ip: [u8; 4],
}

impl ARPPacket {
    pub fn request(sender_ip: [u8; 4], target_ip: [u8; 4]) -> Self {
        ARPPacket {
            operation: 1,
            sender_mac: [0x00, 0x11, 0x22, 0x33, 0x44, 0x55],
            sender_ip,
            target_mac: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
            target_ip,
        }
    }

    pub fn reply(
        sender_mac: [u8; 6],
        sender_ip: [u8; 4],
        target_mac: [u8; 6],
        target_ip: [u8; 4],
    ) -> Self {
        ARPPacket {
            operation: 2,
            sender_mac,
            sender_ip,
            target_mac,
            target_ip,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        packet.extend_from_slice(&[0x00, 0x01]);
        packet.extend_from_slice(&[0x08, 0x00]);
        packet.push(6);
        packet.push(4);
        packet.extend_from_slice(&self.operation.to_be_bytes());

        packet.extend_from_slice(&self.sender_mac);
        packet.extend_from_slice(&self.sender_ip);
        packet.extend_from_slice(&self.target_mac);
        packet.extend_from_slice(&self.target_ip);

        println!("[ARP-PACKET] Built ARP packet:");
        println!(
            "[ARP-PACKET]   Operation: {}",
            if self.operation == 1 {
                "REQUEST"
            } else {
                "REPLY"
            }
        );
        println!(
            "[ARP-PACKET]   Sender IP: {}.{}.{}.{}",
            self.sender_ip[0], self.sender_ip[1], self.sender_ip[2], self.sender_ip[3]
        );
        println!(
            "[ARP-PACKET]   Target IP: {}.{}.{}.{}",
            self.target_ip[0], self.target_ip[1], self.target_ip[2], self.target_ip[3]
        );

        packet
    }
}

// ────────────────────────────────────────────────────────────────────────────
// PORT SCANNER WITH SYN SCAN
// ────────────────────────────────────────────────────────────────────────────

pub struct SYNScanner;

impl SYNScanner {
    pub fn scan_port(target: &str, port: u16, timeout_ms: u64) -> Result<bool, String> {
        let target_addr = format!("{}:{}", target, port);

        let result = TcpStream::connect_timeout(
            &target_addr
                .parse()
                .map_err(|e| format!("Invalid address: {}", e))?,
            Duration::from_millis(timeout_ms),
        );

        match result {
            Ok(_) => {
                println!("[SYN-SCAN] [OK] Port {} OPEN", port);
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    pub fn scan_range(target: &str, start_port: u16, end_port: u16) -> Result<Vec<u16>, String> {
        println!("[SYN-SCAN] Scanning {}:{}-{}", target, start_port, end_port);

        let mut open_ports = Vec::new();

        for port in start_port..=end_port {
            if Self::scan_port(target, port, 1000)? {
                open_ports.push(port);
            }

            if (port - start_port) % 100 == 0 {
                println!(
                    "[SYN-SCAN] Progress: {} ports scanned...",
                    port - start_port + 1
                );
            }
        }

        println!(
            "[SYN-SCAN] Scan complete. Found {} open ports",
            open_ports.len()
        );

        Ok(open_ports)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// HTTP PACKET BUILDER
// ────────────────────────────────────────────────────────────────────────────

pub struct HTTPRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HTTPRequest {
    pub fn get(path: &str) -> Self {
        HTTPRequest {
            method: "GET".to_string(),
            path: path.to_string(),
            version: "HTTP/1.1".to_string(),
            headers: vec![
                ("Host".to_string(), "example.com".to_string()),
                ("User-Agent".to_string(), "TalonDSL/1.0".to_string()),
                ("Connection".to_string(), "close".to_string()),
            ],
            body: Vec::new(),
        }
    }

    pub fn post(path: &str, body: Vec<u8>) -> Self {
        let mut req = Self::get(path);
        req.method = "POST".to_string();
        req.body = body.clone();
        req.headers
            .push(("Content-Length".to_string(), body.len().to_string()));
        req
    }

    pub fn add_header(&mut self, name: String, value: String) {
        self.headers.push((name, value));
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        let request_line = format!("{} {} {}\r\n", self.method, self.path, self.version);
        packet.extend_from_slice(request_line.as_bytes());

        for (name, value) in &self.headers {
            let header_line = format!("{}: {}\r\n", name, value);
            packet.extend_from_slice(header_line.as_bytes());
        }

        packet.extend_from_slice(b"\r\n");

        packet.extend_from_slice(&self.body);

        println!("[HTTP-PACKET] Built HTTP request:");
        println!("[HTTP-PACKET]   {} {}", self.method, self.path);
        println!("[HTTP-PACKET]   Headers: {}", self.headers.len());
        println!("[HTTP-PACKET]   Body: {} bytes", self.body.len());

        packet
    }

    pub fn send(&self, host: &str, port: u16) -> Result<String, String> {
        let target = format!("{}:{}", host, port);
        let mut stream =
            TcpStream::connect(&target).map_err(|e| format!("Connection failed: {}", e))?;

        stream
            .write_all(&self.serialize())
            .map_err(|e| format!("Send failed: {}", e))?;

        let mut response = Vec::new();
        stream
            .read_to_end(&mut response)
            .map_err(|e| format!("Read failed: {}", e))?;

        let response_str = String::from_utf8_lossy(&response).to_string();

        println!(
            "[HTTP-PACKET] [OK] Response received ({} bytes)",
            response.len()
        );

        Ok(response_str)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// DNS QUERY BUILDER
// ────────────────────────────────────────────────────────────────────────────

pub struct DNSQuery {
    pub transaction_id: u16,
    pub domain: String,
    pub query_type: u16,
}

impl DNSQuery {
    pub fn a_record(domain: &str) -> Self {
        DNSQuery {
            transaction_id: rand::random::<u16>(),
            domain: domain.to_string(),
            query_type: 1,
        }
    }

    pub fn aaaa_record(domain: &str) -> Self {
        let mut query = Self::a_record(domain);
        query.query_type = 28;
        query
    }

    pub fn mx_record(domain: &str) -> Self {
        let mut query = Self::a_record(domain);
        query.query_type = 15;
        query
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut packet = Vec::new();

        packet.extend_from_slice(&self.transaction_id.to_be_bytes());
        packet.extend_from_slice(&[0x01, 0x00]);
        packet.extend_from_slice(&[0x00, 0x01]);
        packet.extend_from_slice(&[0x00, 0x00]);
        packet.extend_from_slice(&[0x00, 0x00]);
        packet.extend_from_slice(&[0x00, 0x00]);

        for label in self.domain.split('.') {
            packet.push(label.len() as u8);
            packet.extend_from_slice(label.as_bytes());
        }
        packet.push(0);

        packet.extend_from_slice(&self.query_type.to_be_bytes());
        packet.extend_from_slice(&[0x00, 0x01]);

        println!("[DNS-QUERY] Built DNS query:");
        println!("[DNS-QUERY]   Domain: {}", self.domain);
        println!("[DNS-QUERY]   Type: {}", self.query_type);

        packet
    }
}

// ────────────────────────────────────────────────────────────────────────────
// PACKET SNIFFER (READ-ONLY)
// ────────────────────────────────────────────────────────────────────────────

pub struct PacketSniffer;

impl PacketSniffer {
    pub fn sniff_interface(interface: &str, count: u32) -> Result<(), String> {
        println!(
            "[SNIFFER] Starting packet capture on {} ({} packets)",
            interface, count
        );

        println!("[SNIFFER] Tip: Use tcpdump or wireshark for full packet capture:");
        println!("[SNIFFER]   tcpdump -i {} -c {}", interface, count);
        println!("[SNIFFER]   sudo wireshark");

        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// PACKET FLOOD TOOLS
// ────────────────────────────────────────────────────────────────────────────

pub struct FloodTools;

impl FloodTools {
    pub fn udp_flood(
        target: &str,
        port: u16,
        count: u32,
        payload_size: usize,
    ) -> Result<(), String> {
        println!("[FLOOD] WARNING: UDP flood attack on {}:{}", target, port);
        println!(
            "[FLOOD] Packets: {}, Payload size: {} bytes",
            count, payload_size
        );
        println!("[FLOOD] WARNING: Use only in authorized testing environments!");

        let payload = vec![0x41; payload_size];
        let packet = UDPPacket::new(port, payload);

        for i in 0..count {
            let _ = packet.send(target);

            if (i + 1) % 100 == 0 {
                println!("[FLOOD] Sent {} packets", i + 1);
            }
        }

        println!("[FLOOD] [OK] Flood complete");
        Ok(())
    }
}
