use std::net::{TcpStream, TcpListener, UdpSocket, SocketAddr};
use std::io::{Read, Write, Result as IoResult, Error, ErrorKind};
use std::time::Duration;
use std::thread;

pub enum ScanType {
    Connect,
    Syn,
    Stealth,
}

pub fn tcp_connect(host: &str, port: u16, timeout_secs: u64) -> IoResult<TcpStream> {
    let addr = format!("{}:{}", host, port);
    let stream = TcpStream::connect_timeout(
        &addr.parse::<SocketAddr>()
            .map_err(|e| Error::new(ErrorKind::InvalidInput, e))?,
        Duration::from_secs(timeout_secs),
    )?;
    Ok(stream)
}

pub fn tcp_listen(host: &str, port: u16) -> IoResult<TcpListener> {
    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(addr)?;
    println!("[SOCKET] Listening on {}:{}", host, port);
    Ok(listener)
}

pub fn tcp_send(stream: &mut TcpStream, data: &[u8]) -> IoResult<usize> {
    stream.write(data)
}

pub fn tcp_recv(stream: &mut TcpStream, buffer: &mut [u8]) -> IoResult<usize> {
    stream.read(buffer)
}

pub fn udp_send(target: &str, port: u16, data: &[u8]) -> IoResult<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let addr = format!("{}:{}", target, port);
    socket.send_to(data, addr)?;
    Ok(())
}

pub fn udp_recv(bind_addr: &str, port: u16, buffer: &mut [u8]) -> IoResult<(usize, SocketAddr)> {
    let addr = format!("{}:{}", bind_addr, port);
    let socket = UdpSocket::bind(addr)?;
    socket.recv_from(buffer)
}

pub fn port_scan_connect(host: &str, start_port: u16, end_port: u16, timeout_ms: u64) -> Vec<u16> {
    let mut open_ports = Vec::new();
    
    for port in start_port..=end_port {
        let addr = format!("{}:{}", host, port);
        if let Ok(addr) = addr.parse::<SocketAddr>() {
            if TcpStream::connect_timeout(&addr, Duration::from_millis(timeout_ms)).is_ok() {
                println!("[SCAN] Port {} OPEN", port);
                open_ports.push(port);
            }
        }
    }
    
    open_ports
}

pub fn port_scan_parallel(host: &str, start_port: u16, end_port: u16, timeout_ms: u64, threads: usize) -> Vec<u16> {
    use std::sync::{Arc, Mutex};
    
    let open_ports = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];
    
    let ports_per_thread = (end_port - start_port + 1) / threads as u16;
    
    for i in 0..threads {
        let host = host.to_string();
        let open_ports = Arc::clone(&open_ports);
        let start = start_port + (i as u16 * ports_per_thread);
        let end = if i == threads - 1 { end_port } else { start + ports_per_thread - 1 };
        
        let handle = thread::spawn(move || {
            for port in start..=end {
                let addr = format!("{}:{}", host, port);
                if let Ok(addr) = addr.parse::<SocketAddr>() {
                    if TcpStream::connect_timeout(&addr, Duration::from_millis(timeout_ms)).is_ok() {
                        println!("[SCAN] Port {} OPEN", port);
                        open_ports.lock().unwrap().push(port);
                    }
                }
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let mut result = open_ports.lock().unwrap().clone();
    result.sort();
    result
}

pub fn banner_grab(host: &str, port: u16, timeout_secs: u64) -> IoResult<String> {
    let mut stream = tcp_connect(host, port, timeout_secs)?;
    stream.set_read_timeout(Some(Duration::from_secs(timeout_secs)))?;
    
    let mut buffer = vec![0u8; 4096];
    let n = stream.read(&mut buffer)?;
    
    Ok(String::from_utf8_lossy(&buffer[..n]).to_string())
}

pub fn proxy_connect(proxy_host: &str, proxy_port: u16, target_host: &str, target_port: u16) -> IoResult<TcpStream> {
    let mut proxy = tcp_connect(proxy_host, proxy_port, 10)?;
    
    let connect_cmd = format!("CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n", 
                              target_host, target_port, target_host, target_port);
    proxy.write_all(connect_cmd.as_bytes())?;
    
    let mut response = vec![0u8; 1024];
    let n = proxy.read(&mut response)?;
    let response_str = String::from_utf8_lossy(&response[..n]);
    
    if response_str.contains("200") {
        Ok(proxy)
    } else {
        Err(Error::other("Proxy connection failed"))
    }
}

pub fn raw_socket_send(dest_ip: &str, dest_port: u16, payload: &[u8]) -> IoResult<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    let addr = format!("{}:{}", dest_ip, dest_port);
    socket.send_to(payload, addr)?;
    Ok(())
}

pub fn reverse_shell_connect(lhost: &str, lport: u16) -> IoResult<TcpStream> {
    println!("[REVSHELL] Connecting to {}:{}", lhost, lport);
    let stream = tcp_connect(lhost, lport, 30)?;
    println!("[REVSHELL] Connected!");
    Ok(stream)
}

pub fn bind_shell_listen(lhost: &str, lport: u16) -> IoResult<(TcpListener, TcpStream)> {
    println!("[BINDSHELL] Listening on {}:{}", lhost, lport);
    let listener = tcp_listen(lhost, lport)?;
    let (stream, addr) = listener.accept()?;
    println!("[BINDSHELL] Connection from {}", addr);
    Ok((listener, stream))
}

pub fn subnet_scan(subnet: &str, port: u16, timeout_ms: u64) -> Vec<String> {
    let parts: Vec<&str> = subnet.split('.').collect();
    if parts.len() != 4 {
        return Vec::new();
    }
    
    let base_octets: Vec<u8> = parts[..3].iter()
        .filter_map(|s| s.parse::<u8>().ok())
        .collect();
    
    if base_octets.len() != 3 {
        return Vec::new();
    }
    
    let mut alive_hosts = Vec::new();
    
    for i in 1..255 {
        let host = format!("{}.{}.{}.{}", base_octets[0], base_octets[1], base_octets[2], i);
        let addr = format!("{}:{}", host, port);
        
        if let Ok(addr) = addr.parse::<SocketAddr>() {
            if TcpStream::connect_timeout(&addr, Duration::from_millis(timeout_ms)).is_ok() {
                println!("[SUBNET] Host {} is alive", host);
                alive_hosts.push(host);
            }
        }
    }
    
    alive_hosts
}

pub fn keep_alive(_stream: &mut TcpStream) -> IoResult<()> {
    Ok(())
}

pub fn set_socket_options(stream: &mut TcpStream, nodelay: bool, timeout_secs: u64) -> IoResult<()> {
    stream.set_nodelay(nodelay)?;
    stream.set_read_timeout(Some(Duration::from_secs(timeout_secs)))?;
    stream.set_write_timeout(Some(Duration::from_secs(timeout_secs)))?;
    Ok(())
}
