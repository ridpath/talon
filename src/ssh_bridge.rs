use ssh2::{Channel, Session};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::collections::HashMap;

pub type SshConnectionId = u64;

#[derive(Debug)]
pub enum SshError {
    ConnectionFailed(String),
    AuthenticationFailed(String),
    ChannelError(String),
    IoError(String),
    NotInteractive(String),
}

impl std::fmt::Display for SshError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SshError::ConnectionFailed(msg) => write!(f, "SSH connection failed: {}", msg),
            SshError::AuthenticationFailed(msg) => write!(f, "SSH authentication failed: {}", msg),
            SshError::ChannelError(msg) => write!(f, "SSH channel error: {}", msg),
            SshError::IoError(msg) => write!(f, "SSH I/O error: {}", msg),
            SshError::NotInteractive(msg) => write!(f, "SSH session not in interactive mode: {}", msg),
        }
    }
}

impl std::error::Error for SshError {}

pub struct SshConnection {
    session: Arc<Mutex<Session>>,
    tcp_stream: Arc<Mutex<Option<TcpStream>>>,
    interactive_channel: Arc<Mutex<Option<Channel>>>,
    host: String,
    port: u16,
    username: String,
}

impl SshConnection {
    pub fn connect(host: &str, port: u16, username: &str, password: &str) -> Result<Self, SshError> {
        let tcp = TcpStream::connect(format!("{}:{}", host, port))
            .map_err(|e| SshError::ConnectionFailed(format!("Failed to connect to {}:{}: {}", host, port, e)))?;
        
        tcp.set_read_timeout(Some(Duration::from_secs(10)))
            .map_err(|e| SshError::IoError(format!("Failed to set read timeout: {}", e)))?;
        
        let mut sess = Session::new()
            .map_err(|e| SshError::ConnectionFailed(format!("Failed to create SSH session: {}", e)))?;
        
        sess.set_tcp_stream(tcp.try_clone()
            .map_err(|e| SshError::IoError(format!("Failed to clone TCP stream: {}", e)))?);
        
        sess.handshake()
            .map_err(|e| SshError::ConnectionFailed(format!("SSH handshake failed: {}", e)))?;
        
        sess.userauth_password(username, password)
            .map_err(|e| SshError::AuthenticationFailed(format!("Password authentication failed for user '{}': {}", username, e)))?;
        
        if !sess.authenticated() {
            return Err(SshError::AuthenticationFailed("Authentication failed".to_string()));
        }
        
        Ok(SshConnection {
            session: Arc::new(Mutex::new(sess)),
            tcp_stream: Arc::new(Mutex::new(Some(tcp))),
            interactive_channel: Arc::new(Mutex::new(None)),
            host: host.to_string(),
            port,
            username: username.to_string(),
        })
    }
    
    pub fn connect_with_key(host: &str, port: u16, username: &str, private_key_path: &str, passphrase: Option<&str>) -> Result<Self, SshError> {
        let tcp = TcpStream::connect(format!("{}:{}", host, port))
            .map_err(|e| SshError::ConnectionFailed(format!("Failed to connect to {}:{}: {}", host, port, e)))?;
        
        tcp.set_read_timeout(Some(Duration::from_secs(10)))
            .map_err(|e| SshError::IoError(format!("Failed to set read timeout: {}", e)))?;
        
        let mut sess = Session::new()
            .map_err(|e| SshError::ConnectionFailed(format!("Failed to create SSH session: {}", e)))?;
        
        sess.set_tcp_stream(tcp.try_clone()
            .map_err(|e| SshError::IoError(format!("Failed to clone TCP stream: {}", e)))?);
        
        sess.handshake()
            .map_err(|e| SshError::ConnectionFailed(format!("SSH handshake failed: {}", e)))?;
        
        let pp = passphrase.unwrap_or("");
        sess.userauth_pubkey_file(username, None, std::path::Path::new(private_key_path), Some(pp))
            .map_err(|e| SshError::AuthenticationFailed(format!("Public key authentication failed: {}", e)))?;
        
        if !sess.authenticated() {
            return Err(SshError::AuthenticationFailed("Authentication failed".to_string()));
        }
        
        Ok(SshConnection {
            session: Arc::new(Mutex::new(sess)),
            tcp_stream: Arc::new(Mutex::new(Some(tcp))),
            interactive_channel: Arc::new(Mutex::new(None)),
            host: host.to_string(),
            port,
            username: username.to_string(),
        })
    }
    
    pub fn connect_pty(host: &str, port: u16, username: &str, password: &str, rows: u32, cols: u32) -> Result<Self, SshError> {
        let conn = Self::connect(host, port, username, password)?;
        
        let sess = conn.session.lock().unwrap();
        let mut channel = sess.channel_session()
            .map_err(|e| SshError::ChannelError(format!("Failed to create channel: {}", e)))?;
        
        channel.request_pty("xterm", None, Some((cols, rows, 0, 0)))
            .map_err(|e| SshError::ChannelError(format!("Failed to request PTY: {}", e)))?;
        
        channel.shell()
            .map_err(|e| SshError::ChannelError(format!("Failed to start shell: {}", e)))?;
        
        drop(sess);
        
        *conn.interactive_channel.lock().unwrap() = Some(channel);
        
        Ok(conn)
    }
    
    pub fn execute(&self, command: &str) -> Result<String, SshError> {
        let sess = self.session.lock().unwrap();
        let mut channel = sess.channel_session()
            .map_err(|e| SshError::ChannelError(format!("Failed to create channel: {}", e)))?;
        
        channel.exec(command)
            .map_err(|e| SshError::ChannelError(format!("Failed to execute command: {}", e)))?;
        
        let mut output = String::new();
        channel.read_to_string(&mut output)
            .map_err(|e| SshError::IoError(format!("Failed to read command output: {}", e)))?;
        
        channel.wait_close()
            .map_err(|e| SshError::ChannelError(format!("Failed to close channel: {}", e)))?;
        
        Ok(output)
    }
    
    pub fn upload(&self, local_path: &str, remote_path: &str) -> Result<(), SshError> {
        let local_data = std::fs::read(local_path)
            .map_err(|e| SshError::IoError(format!("Failed to read local file '{}': {}", local_path, e)))?;
        
        let sess = self.session.lock().unwrap();
        let mut remote_file = sess.scp_send(
            std::path::Path::new(remote_path),
            0o644,
            local_data.len() as u64,
            None
        ).map_err(|e| SshError::ChannelError(format!("Failed to create remote file '{}': {}", remote_path, e)))?;
        
        remote_file.write_all(&local_data)
            .map_err(|e| SshError::IoError(format!("Failed to write to remote file: {}", e)))?;
        
        remote_file.send_eof()
            .map_err(|e| SshError::ChannelError(format!("Failed to send EOF: {}", e)))?;
        
        remote_file.wait_eof()
            .map_err(|e| SshError::ChannelError(format!("Failed to wait for EOF: {}", e)))?;
        
        remote_file.close()
            .map_err(|e| SshError::ChannelError(format!("Failed to close remote file: {}", e)))?;
        
        remote_file.wait_close()
            .map_err(|e| SshError::ChannelError(format!("Failed to wait for channel close: {}", e)))?;
        
        Ok(())
    }
    
    pub fn download(&self, remote_path: &str, local_path: &str) -> Result<(), SshError> {
        let sess = self.session.lock().unwrap();
        let (mut remote_file, _stat) = sess.scp_recv(std::path::Path::new(remote_path))
            .map_err(|e| SshError::ChannelError(format!("Failed to open remote file '{}': {}", remote_path, e)))?;
        
        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents)
            .map_err(|e| SshError::IoError(format!("Failed to read remote file: {}", e)))?;
        
        remote_file.send_eof()
            .map_err(|e| SshError::ChannelError(format!("Failed to send EOF: {}", e)))?;
        
        remote_file.wait_eof()
            .map_err(|e| SshError::ChannelError(format!("Failed to wait for EOF: {}", e)))?;
        
        remote_file.close()
            .map_err(|e| SshError::ChannelError(format!("Failed to close channel: {}", e)))?;
        
        remote_file.wait_close()
            .map_err(|e| SshError::ChannelError(format!("Failed to wait for channel close: {}", e)))?;
        
        drop(sess);
        
        std::fs::write(local_path, contents)
            .map_err(|e| SshError::IoError(format!("Failed to write local file '{}': {}", local_path, e)))?;
        
        Ok(())
    }
    
    pub fn forward_local(&self, local_port: u16, remote_host: &str, remote_port: u16) -> Result<(), SshError> {
        let sess = self.session.lock().unwrap();
        
        let mut channel = sess.channel_direct_tcpip(remote_host, remote_port, None)
            .map_err(|e| SshError::ChannelError(format!("Failed to create port forward: {}", e)))?;
        
        drop(sess);
        
        let listener = std::net::TcpListener::bind(format!("127.0.0.1:{}", local_port))
            .map_err(|e| SshError::IoError(format!("Failed to bind local port {}: {}", local_port, e)))?;
        
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                if let Ok(mut local_stream) = stream {
                    let mut buf = vec![0u8; 4096];
                    loop {
                        match local_stream.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => {
                                if channel.write_all(&buf[..n]).is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                        
                        match channel.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => {
                                if local_stream.write_all(&buf[..n]).is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    pub fn interactive_start(&self) -> Result<(), SshError> {
        let channel_lock = self.interactive_channel.lock().unwrap();
        if channel_lock.is_none() {
            return Err(SshError::NotInteractive("Connection was not opened with PTY. Use connect_ssh_pty()".to_string()));
        }
        Ok(())
    }
    
    pub fn interactive_send(&self, data: &[u8]) -> Result<(), SshError> {
        let mut channel_lock = self.interactive_channel.lock().unwrap();
        if let Some(ref mut channel) = *channel_lock {
            channel.write_all(data)
                .map_err(|e| SshError::IoError(format!("Failed to send data: {}", e)))?;
            channel.flush()
                .map_err(|e| SshError::IoError(format!("Failed to flush channel: {}", e)))?;
            Ok(())
        } else {
            Err(SshError::NotInteractive("No interactive channel active".to_string()))
        }
    }
    
    pub fn interactive_recv(&self, timeout_ms: u64) -> Result<String, SshError> {
        let mut channel_lock = self.interactive_channel.lock().unwrap();
        if let Some(ref mut channel) = *channel_lock {
            let mut buffer = Vec::new();
            let mut chunk = [0u8; 4096];
            
            let start = std::time::Instant::now();
            loop {
                if start.elapsed() >= Duration::from_millis(timeout_ms) {
                    break;
                }
                
                match channel.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        buffer.extend_from_slice(&chunk[..n]);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                        break;
                    }
                    Err(e) => {
                        return Err(SshError::IoError(format!("Failed to read from channel: {}", e)));
                    }
                }
            }
            
            String::from_utf8(buffer)
                .map_err(|e| SshError::IoError(format!("Invalid UTF-8 in response: {}", e)))
        } else {
            Err(SshError::NotInteractive("No interactive channel active".to_string()))
        }
    }
    
    pub fn interactive_close(&self) -> Result<(), SshError> {
        let mut channel_lock = self.interactive_channel.lock().unwrap();
        if let Some(mut channel) = channel_lock.take() {
            channel.close()
                .map_err(|e| SshError::ChannelError(format!("Failed to close channel: {}", e)))?;
            channel.wait_close()
                .map_err(|e| SshError::ChannelError(format!("Failed to wait for channel close: {}", e)))?;
            Ok(())
        } else {
            Ok(())
        }
    }
    
    pub fn get_host(&self) -> &str {
        &self.host
    }
    
    pub fn get_port(&self) -> u16 {
        self.port
    }
    
    pub fn get_username(&self) -> &str {
        &self.username
    }
}

impl Drop for SshConnection {
    fn drop(&mut self) {
        let _ = self.interactive_close();
    }
}

pub struct SshRegistry {
    connections: HashMap<SshConnectionId, SshConnection>,
    next_id: SshConnectionId,
}

impl SshRegistry {
    pub fn new() -> Self {
        SshRegistry {
            connections: HashMap::new(),
            next_id: 1,
        }
    }
    
    pub fn add(&mut self, connection: SshConnection) -> SshConnectionId {
        let id = self.next_id;
        self.next_id += 1;
        self.connections.insert(id, connection);
        id
    }
    
    pub fn get(&self, id: SshConnectionId) -> Option<&SshConnection> {
        self.connections.get(&id)
    }
    
    pub fn get_mut(&mut self, id: SshConnectionId) -> Option<&mut SshConnection> {
        self.connections.get_mut(&id)
    }
    
    pub fn remove(&mut self, id: SshConnectionId) -> Option<SshConnection> {
        self.connections.remove(&id)
    }
    
    pub fn list(&self) -> Vec<SshConnectionId> {
        self.connections.keys().copied().collect()
    }
}

impl Default for SshRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ssh_registry_creation() {
        let registry = SshRegistry::new();
        assert_eq!(registry.list().len(), 0);
        assert_eq!(registry.next_id, 1);
    }
    
    #[test]
    fn test_ssh_error_display() {
        let err = SshError::ConnectionFailed("test error".to_string());
        assert_eq!(format!("{}", err), "SSH connection failed: test error");
        
        let err = SshError::AuthenticationFailed("auth failed".to_string());
        assert_eq!(format!("{}", err), "SSH authentication failed: auth failed");
        
        let err = SshError::ChannelError("channel error".to_string());
        assert_eq!(format!("{}", err), "SSH channel error: channel error");
        
        let err = SshError::IoError("io error".to_string());
        assert_eq!(format!("{}", err), "SSH I/O error: io error");
        
        let err = SshError::NotInteractive("not interactive".to_string());
        assert_eq!(format!("{}", err), "SSH session not in interactive mode: not interactive");
    }
}
