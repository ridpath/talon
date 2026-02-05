use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream as AsyncTcpStream;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode};

const DEFAULT_TIMEOUT_SECS: u64 = 10;
const DEFAULT_RECV_SIZE: usize = 4096;

pub struct Socket {
    stream: TcpStream,
    buffer: Vec<u8>,
    timeout: Duration,
}

impl Socket {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, String> {
        let stream = TcpStream::connect(addr).map_err(|e| format!("Connection failed: {}", e))?;

        stream
            .set_read_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))
            .map_err(|e| format!("Failed to set read timeout: {}", e))?;

        stream
            .set_write_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT_SECS)))
            .map_err(|e| format!("Failed to set write timeout: {}", e))?;

        log::info!("Connected to {:?}", stream.peer_addr());

        Ok(Socket {
            stream,
            buffer: Vec::new(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        })
    }

    pub fn send(&mut self, data: &[u8]) -> Result<(), String> {
        self.stream
            .write_all(data)
            .map_err(|e| format!("Send failed: {}", e))?;
        log::debug!("Sent {} bytes", data.len());
        Ok(())
    }

    pub fn sendline(&mut self, data: &[u8]) -> Result<(), String> {
        let mut payload = data.to_vec();
        payload.push(b'\n');
        self.send(&payload)
    }

    pub fn recv(&mut self, n: usize) -> Result<Vec<u8>, String> {
        let mut buf = vec![0u8; n];
        self.stream
            .read_exact(&mut buf)
            .map_err(|e| format!("Recv failed: {}", e))?;
        log::debug!("Received {} bytes", n);
        Ok(buf)
    }

    pub fn recvuntil(&mut self, delim: &[u8]) -> Result<Vec<u8>, String> {
        let mut result = Vec::new();
        let mut buf = [0u8; 1];

        loop {
            self.stream
                .read_exact(&mut buf)
                .map_err(|e| format!("Recv failed: {}", e))?;
            result.push(buf[0]);

            if result.len() >= delim.len() {
                let end = &result[result.len() - delim.len()..];
                if end == delim {
                    log::debug!("Received until delimiter ({} bytes)", result.len());
                    return Ok(result);
                }
            }

            if result.len() > 1_000_000 {
                return Err("Received too much data without finding delimiter".to_string());
            }
        }
    }

    pub fn recvline(&mut self) -> Result<Vec<u8>, String> {
        self.recvuntil(b"\n")
    }

    pub fn recvall(&mut self) -> Result<Vec<u8>, String> {
        let mut result = Vec::new();
        let mut buf = [0u8; DEFAULT_RECV_SIZE];

        self.stream
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        loop {
            match self.stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    result.extend_from_slice(&buf[..n]);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => break,
                Err(e) => return Err(format!("Recv failed: {}", e)),
            }
        }

        self.stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| format!("Failed to restore timeout: {}", e))?;

        log::debug!("Received all: {} bytes", result.len());
        Ok(result)
    }

    pub fn interactive(&mut self) -> Result<(), String> {
        self.interactive_with_raw_mode(false)
    }

    pub fn interactive_raw(&mut self) -> Result<(), String> {
        self.interactive_with_raw_mode(true)
    }

    pub fn interactive_with_raw_mode(&mut self, use_raw_mode: bool) -> Result<(), String> {
        log::info!("Entering interactive mode (raw_mode: {}). Press Ctrl+C to exit.", use_raw_mode);
        println!("[*] Switching to interactive mode...");

        if use_raw_mode {
            enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
        }

        let result = self.run_interactive_loop(use_raw_mode);

        if use_raw_mode {
            disable_raw_mode().ok();
        }

        result
    }

    fn run_interactive_loop(&mut self, use_raw_mode: bool) -> Result<(), String> {
        use std::io::{stdin, stdout, Write};
        use std::sync::mpsc;
        use std::thread;

        let mut read_stream = self
            .stream
            .try_clone()
            .map_err(|e| format!("Failed to clone stream: {}", e))?;

        read_stream
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        let (tx, rx) = mpsc::channel();

        let read_handle = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                if rx.try_recv().is_ok() {
                    break;
                }

                match read_stream.read(&mut buf) {
                    Ok(0) => {
                        println!("\n[*] Connection closed by remote");
                        break;
                    }
                    Ok(n) => {
                        let data = &buf[..n];
                        stdout().write_all(data).ok();
                        stdout().flush().ok();
                    }
                    Err(ref e)
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::TimedOut =>
                    {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        eprintln!("\n[!] Read error: {}", e);
                        break;
                    }
                }
            }
        });

        if use_raw_mode {
            loop {
                if event::poll(Duration::from_millis(10)).unwrap_or(false) {
                    if let Ok(Event::Key(key_event)) = event::read() {
                        if key_event.modifiers.contains(KeyModifiers::CONTROL)
                            && (key_event.code == KeyCode::Char('c') || key_event.code == KeyCode::Char('C'))
                        {
                            println!("\n[*] Ctrl+C received, exiting interactive mode");
                            break;
                        }

                        match key_event.code {
                            KeyCode::Char(c) => {
                                if self.stream.write_all(&[c as u8]).is_err() {
                                    break;
                                }
                            }
                            KeyCode::Enter => {
                                if self.stream.write_all(b"\n").is_err() {
                                    break;
                                }
                            }
                            KeyCode::Backspace => {
                                if self.stream.write_all(b"\x7f").is_err() {
                                    break;
                                }
                            }
                            KeyCode::Tab => {
                                if self.stream.write_all(b"\t").is_err() {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        self.stream.flush().ok();
                    }
                }
            }
        } else {
            let stdin = stdin();
            let mut input_buf = String::new();

            loop {
                input_buf.clear();
                match stdin.read_line(&mut input_buf) {
                    Ok(0) => {
                        println!("\n[*] Exiting interactive mode");
                        break;
                    }
                    Ok(_) => {
                        if let Err(e) = self.stream.write_all(input_buf.as_bytes()) {
                            eprintln!("[!] Write error: {}", e);
                            break;
                        }
                        self.stream.flush().ok();
                    }
                    Err(e) => {
                        eprintln!("[!] Input error: {}", e);
                        break;
                    }
                }
            }
        }

        tx.send(()).ok();
        read_handle.join().ok();

        Ok(())
    }

    pub fn close(&mut self) -> Result<(), String> {
        self.stream
            .shutdown(std::net::Shutdown::Both)
            .map_err(|e| format!("Shutdown failed: {}", e))?;
        log::info!("Connection closed");
        Ok(())
    }

    pub fn set_timeout(&mut self, seconds: u64) -> Result<(), String> {
        self.timeout = Duration::from_secs(seconds);
        self.stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;
        self.stream
            .set_write_timeout(Some(self.timeout))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;
        Ok(())
    }

    pub fn get_raw_stream(&mut self) -> &mut TcpStream {
        &mut self.stream
    }
}

pub struct ConnectionMultiplexer {
    connections: Arc<Mutex<HashMap<String, Arc<Mutex<Socket>>>>>,
    active_connection: Arc<Mutex<Option<String>>>,
}

impl ConnectionMultiplexer {
    pub fn new() -> Self {
        ConnectionMultiplexer {
            connections: Arc::new(Mutex::new(HashMap::new())),
            active_connection: Arc::new(Mutex::new(None)),
        }
    }

    pub fn add_connection(&mut self, name: String, socket: Socket) -> Result<(), String> {
        let mut conns = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        conns.insert(name.clone(), Arc::new(Mutex::new(socket)));
        
        let mut active = self.active_connection.lock()
            .map_err(|e| format!("Failed to lock active connection: {}", e))?;
        if active.is_none() {
            *active = Some(name.clone());
            log::info!("Set active connection to: {}", name);
        }
        
        Ok(())
    }

    pub fn switch_to(&mut self, name: &str) -> Result<(), String> {
        let conns = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        
        if !conns.contains_key(name) {
            return Err(format!("Connection '{}' not found", name));
        }

        let mut active = self.active_connection.lock()
            .map_err(|e| format!("Failed to lock active connection: {}", e))?;
        *active = Some(name.to_string());
        
        log::info!("Switched to connection: {}", name);
        Ok(())
    }

    pub fn get_active(&self) -> Result<Arc<Mutex<Socket>>, String> {
        let active = self.active_connection.lock()
            .map_err(|e| format!("Failed to lock active connection: {}", e))?;
        
        if let Some(name) = active.as_ref() {
            let conns = self.connections.lock()
                .map_err(|e| format!("Failed to lock connections: {}", e))?;
            
            conns.get(name)
                .cloned()
                .ok_or_else(|| format!("Active connection '{}' not found", name))
        } else {
            Err("No active connection".to_string())
        }
    }

    pub fn list_connections(&self) -> Result<Vec<String>, String> {
        let conns = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        Ok(conns.keys().cloned().collect())
    }

    pub fn remove_connection(&mut self, name: &str) -> Result<(), String> {
        let mut conns = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        
        conns.remove(name).ok_or_else(|| format!("Connection '{}' not found", name))?;
        
        let mut active = self.active_connection.lock()
            .map_err(|e| format!("Failed to lock active connection: {}", e))?;
        if active.as_ref() == Some(&name.to_string()) {
            *active = conns.keys().next().cloned();
        }
        
        log::info!("Removed connection: {}", name);
        Ok(())
    }
}

impl Default for ConnectionMultiplexer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GdbCoordinator {
    gdb_stream: Option<Arc<Mutex<Socket>>>,
    target_stream: Option<Arc<Mutex<Socket>>>,
}

impl GdbCoordinator {
    pub fn new() -> Self {
        GdbCoordinator {
            gdb_stream: None,
            target_stream: None,
        }
    }

    pub fn set_gdb_connection(&mut self, socket: Socket) {
        self.gdb_stream = Some(Arc::new(Mutex::new(socket)));
        log::info!("GDB connection registered");
    }

    pub fn set_target_connection(&mut self, socket: Socket) {
        self.target_stream = Some(Arc::new(Mutex::new(socket)));
        log::info!("Target connection registered");
    }

    pub fn sync_breakpoint(&mut self, address: u64) -> Result<(), String> {
        if let Some(gdb) = &self.gdb_stream {
            let mut gdb_sock = gdb.lock()
                .map_err(|e| format!("Failed to lock GDB socket: {}", e))?;
            
            let cmd = format!("break *{:#x}\n", address);
            gdb_sock.send(cmd.as_bytes())?;
            log::info!("Set breakpoint at {:#x}", address);
            Ok(())
        } else {
            Err("GDB connection not set".to_string())
        }
    }

    pub fn send_gdb_command(&mut self, command: &str) -> Result<Vec<u8>, String> {
        if let Some(gdb) = &self.gdb_stream {
            let mut gdb_sock = gdb.lock()
                .map_err(|e| format!("Failed to lock GDB socket: {}", e))?;
            
            gdb_sock.sendline(command.as_bytes())?;
            std::thread::sleep(Duration::from_millis(100));
            gdb_sock.recvall()
        } else {
            Err("GDB connection not set".to_string())
        }
    }
}

impl Default for GdbCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AsyncSocket {
    stream: AsyncTcpStream,
    buffer: Vec<u8>,
}

impl AsyncSocket {
    pub async fn connect<A: tokio::net::ToSocketAddrs>(addr: A) -> Result<Self, String> {
        let stream = AsyncTcpStream::connect(addr)
            .await
            .map_err(|e| format!("Async connection failed: {}", e))?;

        log::info!("Async connected to {:?}", stream.peer_addr());

        Ok(AsyncSocket {
            stream,
            buffer: Vec::new(),
        })
    }

    pub async fn send(&mut self, data: &[u8]) -> Result<(), String> {
        self.stream
            .write_all(data)
            .await
            .map_err(|e| format!("Async send failed: {}", e))?;
        log::debug!("Async sent {} bytes", data.len());
        Ok(())
    }

    pub async fn sendline(&mut self, data: &[u8]) -> Result<(), String> {
        let mut payload = data.to_vec();
        payload.push(b'\n');
        self.send(&payload).await
    }

    pub async fn recv(&mut self, n: usize) -> Result<Vec<u8>, String> {
        let mut buf = vec![0u8; n];
        self.stream
            .read_exact(&mut buf)
            .await
            .map_err(|e| format!("Async recv failed: {}", e))?;
        log::debug!("Async received {} bytes", n);
        Ok(buf)
    }

    pub async fn recvuntil(&mut self, delim: &[u8]) -> Result<Vec<u8>, String> {
        let mut result = Vec::new();
        let mut buf = [0u8; 1];

        loop {
            self.stream
                .read_exact(&mut buf)
                .await
                .map_err(|e| format!("Async recv failed: {}", e))?;
            result.push(buf[0]);

            if result.len() >= delim.len() {
                let end = &result[result.len() - delim.len()..];
                if end == delim {
                    log::debug!("Async received until delimiter ({} bytes)", result.len());
                    return Ok(result);
                }
            }

            if result.len() > 1_000_000 {
                return Err("Received too much data without finding delimiter".to_string());
            }
        }
    }

    pub async fn recvline(&mut self) -> Result<Vec<u8>, String> {
        self.recvuntil(b"\n").await
    }

    pub async fn close(&mut self) -> Result<(), String> {
        self.stream
            .shutdown()
            .await
            .map_err(|e| format!("Async shutdown failed: {}", e))?;
        log::info!("Async connection closed");
        Ok(())
    }
}

pub async fn concurrent_connections(
    addrs: Vec<String>,
) -> Result<Vec<Result<AsyncSocket, String>>, String> {
    let mut handles = vec![];

    for addr in addrs {
        let handle = tokio::spawn(async move {
            AsyncSocket::connect(addr.as_str()).await
        });
        handles.push(handle);
    }

    let mut results = vec![];
    for handle in handles {
        let result = handle.await.map_err(|e| format!("Task join error: {}", e))?;
        results.push(result);
    }

    Ok(results)
}

pub struct TerminalManager {
    original_size: Option<(u16, u16)>,
}

impl TerminalManager {
    pub fn new() -> Self {
        TerminalManager {
            original_size: Self::get_terminal_size().ok(),
        }
    }

    pub fn get_terminal_size() -> Result<(u16, u16), String> {
        terminal::size().map_err(|e| format!("Failed to get terminal size: {}", e))
    }

    pub fn enable_raw_mode() -> Result<(), String> {
        enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))
    }

    pub fn disable_raw_mode() -> Result<(), String> {
        disable_raw_mode().map_err(|e| format!("Failed to disable raw mode: {}", e))
    }

    pub fn restore(&self) -> Result<(), String> {
        Self::disable_raw_mode().ok();
        Ok(())
    }

    pub fn monitor_resize<F>(&self, mut callback: F) -> Result<(), String>
    where
        F: FnMut(u16, u16) + Send + 'static,
    {
        use std::thread;
        use std::time::Duration;

        thread::spawn(move || {
            let mut last_size = Self::get_terminal_size().ok();

            loop {
                thread::sleep(Duration::from_millis(500));

                if let Ok(current_size) = Self::get_terminal_size() {
                    if last_size != Some(current_size) {
                        log::info!("Terminal resized to {}x{}", current_size.0, current_size.1);
                        callback(current_size.0, current_size.1);
                        last_size = Some(current_size);
                    }
                }
            }
        });

        Ok(())
    }
}

impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TerminalManager {
    fn drop(&mut self) {
        self.restore().ok();
    }
}

pub struct SignalHandler {
    ctrl_c_count: Arc<Mutex<u32>>,
}

impl SignalHandler {
    pub fn new() -> Self {
        SignalHandler {
            ctrl_c_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn install_handler(&self) -> Result<(), String> {
        let count = Arc::clone(&self.ctrl_c_count);
        
        ctrlc::set_handler(move || {
            let mut c = count.lock().unwrap();
            *c += 1;
            
            if *c == 1 {
                println!("\n[*] Ctrl+C received. Press again to force exit.");
            } else {
                println!("\n[!] Force exiting...");
                std::process::exit(130);
            }
        })
        .map_err(|e| format!("Failed to set Ctrl+C handler: {}", e))?;
        
        log::info!("Signal handler installed");
        Ok(())
    }

    pub fn reset_count(&self) {
        if let Ok(mut count) = self.ctrl_c_count.lock() {
            *count = 0;
        }
    }

    pub fn get_count(&self) -> u32 {
        self.ctrl_c_count.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Process {
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
}

impl Process {
    pub fn spawn(command: &str, args: &[&str]) -> Result<Self, String> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();

        log::info!("Spawned process: {} {:?}", command, args);

        Ok(Process { stdin, stdout })
    }

    pub fn send(&mut self, data: &[u8]) -> Result<(), String> {
        if let Some(ref mut stdin) = self.stdin {
            stdin
                .write_all(data)
                .map_err(|e| format!("Write to process failed: {}", e))?;
            stdin.flush().map_err(|e| format!("Flush failed: {}", e))?;
            Ok(())
        } else {
            Err("No stdin available".to_string())
        }
    }

    pub fn sendline(&mut self, data: &[u8]) -> Result<(), String> {
        let mut payload = data.to_vec();
        payload.push(b'\n');
        self.send(&payload)
    }

    pub fn recv(&mut self, n: usize) -> Result<Vec<u8>, String> {
        if let Some(ref mut stdout) = self.stdout {
            let mut buf = vec![0u8; n];
            stdout
                .read_exact(&mut buf)
                .map_err(|e| format!("Read from process failed: {}", e))?;
            Ok(buf)
        } else {
            Err("No stdout available".to_string())
        }
    }

    pub fn recvline(&mut self) -> Result<Vec<u8>, String> {
        if let Some(ref mut stdout) = self.stdout {
            let mut line = Vec::new();
            let mut byte_buf = [0u8; 1];

            loop {
                stdout
                    .read_exact(&mut byte_buf)
                    .map_err(|e| format!("Read failed: {}", e))?;
                line.push(byte_buf[0]);

                if byte_buf[0] == b'\n' {
                    break;
                }

                if line.len() > 1_000_000 {
                    return Err("Line too long".to_string());
                }
            }

            Ok(line)
        } else {
            Err("No stdout available".to_string())
        }
    }
}

pub fn remote(host: &str, port: u16) -> Result<Socket, String> {
    let addr = format!("{}:{}", host, port);
    Socket::connect(addr)
}

pub fn process(binary: &str) -> Result<Process, String> {
    Process::spawn(binary, &[])
}

pub fn process_with_args(binary: &str, args: &[&str]) -> Result<Process, String> {
    Process::spawn(binary, args)
}

pub async fn async_remote(host: &str, port: u16) -> Result<AsyncSocket, String> {
    let addr = format!("{}:{}", host, port);
    AsyncSocket::connect(addr.as_str()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_creation() {
        assert_eq!(
            std::mem::size_of::<Socket>(),
            std::mem::size_of::<TcpStream>()
                + std::mem::size_of::<Vec<u8>>()
                + std::mem::size_of::<Duration>()
        );
    }

    #[test]
    fn test_process_creation() {
        #[cfg(unix)]
        let result = Process::spawn("echo", &["test"]);

        #[cfg(windows)]
        let result = Process::spawn("cmd", &["/C", "echo", "test"]);

        let _ = result;
    }

    #[test]
    fn test_connection_multiplexer_creation() {
        let mux = ConnectionMultiplexer::new();
        assert!(mux.list_connections().unwrap().is_empty());
    }

    #[test]
    fn test_connection_multiplexer_add() {
        let mux = ConnectionMultiplexer::new();
        assert!(mux.list_connections().unwrap().is_empty());
    }

    #[test]
    fn test_gdb_coordinator_creation() {
        let coordinator = GdbCoordinator::new();
        assert!(coordinator.gdb_stream.is_none());
        assert!(coordinator.target_stream.is_none());
    }

    #[test]
    fn test_terminal_manager_creation() {
        let tm = TerminalManager::new();
        assert!(tm.original_size.is_some() || tm.original_size.is_none());
    }

    #[test]
    fn test_terminal_size_detection() {
        let size_result = TerminalManager::get_terminal_size();
        let _ = size_result;
    }

    #[test]
    fn test_signal_handler_creation() {
        let handler = SignalHandler::new();
        assert_eq!(handler.get_count(), 0);
    }

    #[test]
    fn test_signal_handler_count() {
        let handler = SignalHandler::new();
        handler.reset_count();
        assert_eq!(handler.get_count(), 0);
    }

    #[tokio::test]
    async fn test_async_socket_type() {
        assert_eq!(
            std::mem::size_of::<AsyncSocket>(),
            std::mem::size_of::<AsyncTcpStream>() + std::mem::size_of::<Vec<u8>>()
        );
    }

    #[tokio::test]
    async fn test_concurrent_connections_empty() {
        let result = concurrent_connections(vec![]).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
