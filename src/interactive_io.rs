use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════════════════
// INTERACTIVE I/O - PWNTOOLS-STYLE SOCKET CONTEXT
// ═══════════════════════════════════════════════════════════════════════════

const DEFAULT_TIMEOUT_SECS: u64 = 10;
const DEFAULT_RECV_SIZE: usize = 4096;

/// Socket connection wrapper with pwntools-style interface
pub struct Socket {
    stream: TcpStream,
    buffer: Vec<u8>,
    timeout: Duration,
}

impl Socket {
    /// Connect to a remote host
    ///
    /// # Example
    /// ```
    /// let mut conn = Socket::connect("192.168.1.1:9001")?;
    /// conn.sendline(b"Hello");
    /// let response = conn.recvline()?;
    /// ```
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

    /// Send data
    pub fn send(&mut self, data: &[u8]) -> Result<(), String> {
        self.stream
            .write_all(data)
            .map_err(|e| format!("Send failed: {}", e))?;
        log::debug!("Sent {} bytes", data.len());
        Ok(())
    }

    /// Send data with newline
    pub fn sendline(&mut self, data: &[u8]) -> Result<(), String> {
        let mut payload = data.to_vec();
        payload.push(b'\n');
        self.send(&payload)
    }

    /// Receive exactly n bytes
    pub fn recv(&mut self, n: usize) -> Result<Vec<u8>, String> {
        let mut buf = vec![0u8; n];
        self.stream
            .read_exact(&mut buf)
            .map_err(|e| format!("Recv failed: {}", e))?;
        log::debug!("Received {} bytes", n);
        Ok(buf)
    }

    /// Receive until a delimiter is found
    pub fn recvuntil(&mut self, delim: &[u8]) -> Result<Vec<u8>, String> {
        let mut result = Vec::new();
        let mut buf = [0u8; 1];

        loop {
            self.stream
                .read_exact(&mut buf)
                .map_err(|e| format!("Recv failed: {}", e))?;
            result.push(buf[0]);

            // Check if we found the delimiter
            if result.len() >= delim.len() {
                let end = &result[result.len() - delim.len()..];
                if end == delim {
                    log::debug!("Received until delimiter ({} bytes)", result.len());
                    return Ok(result);
                }
            }

            // Safety limit
            if result.len() > 1_000_000 {
                return Err("Received too much data without finding delimiter".to_string());
            }
        }
    }

    /// Receive a single line
    pub fn recvline(&mut self) -> Result<Vec<u8>, String> {
        self.recvuntil(b"\n")
    }

    /// Receive all available data (non-blocking after timeout)
    pub fn recvall(&mut self) -> Result<Vec<u8>, String> {
        let mut result = Vec::new();
        let mut buf = [0u8; DEFAULT_RECV_SIZE];

        self.stream
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        loop {
            match self.stream.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    result.extend_from_slice(&buf[..n]);
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => break,
                Err(e) => return Err(format!("Recv failed: {}", e)),
            }
        }

        // Restore original timeout
        self.stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| format!("Failed to restore timeout: {}", e))?;

        log::debug!("Received all: {} bytes", result.len());
        Ok(result)
    }

    /// Interactive mode - bidirectional I/O with remote
    pub fn interactive(&mut self) -> Result<(), String> {
        use std::io::{stdin, stdout};
        use std::sync::mpsc;
        use std::thread;
        use std::time::Duration;

        log::info!("Entering interactive mode. Press Ctrl+C to exit.");
        println!("[*] Switching to interactive mode...");

        // Clone stream for reading thread
        let mut read_stream = self
            .stream
            .try_clone()
            .map_err(|e| format!("Failed to clone stream: {}", e))?;

        // Set non-blocking read timeout
        read_stream
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        // Create channel for graceful shutdown
        let (tx, rx) = mpsc::channel();

        // Spawn thread to read from remote
        let read_handle = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                // Check for shutdown signal
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
                        // Timeout, continue loop
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        eprintln!("\n[!] Read error: {}", e);
                        break;
                    }
                }
            }
        });

        // Read from stdin and send to remote
        let stdin = stdin();
        let mut input_buf = String::new();

        loop {
            input_buf.clear();
            match stdin.read_line(&mut input_buf) {
                Ok(0) => {
                    // EOF (Ctrl+D)
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

        // Signal read thread to exit
        tx.send(()).ok();
        read_handle.join().ok();

        Ok(())
    }

    /// Clean shutdown
    pub fn close(&mut self) -> Result<(), String> {
        self.stream
            .shutdown(std::net::Shutdown::Both)
            .map_err(|e| format!("Shutdown failed: {}", e))?;
        log::info!("Connection closed");
        Ok(())
    }

    /// Set timeout
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
}

// ────────────────────────────────────────────────────────────────────────────
// PROCESS INTERACTION
// ────────────────────────────────────────────────────────────────────────────

/// Process wrapper for local exploitation
pub struct Process {
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
}

impl Process {
    /// Spawn a local process
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

    /// Send data to process
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

    /// Send line to process
    pub fn sendline(&mut self, data: &[u8]) -> Result<(), String> {
        let mut payload = data.to_vec();
        payload.push(b'\n');
        self.send(&payload)
    }

    /// Receive from process
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

    /// Receive line from process
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

                // Safety limit
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

// ────────────────────────────────────────────────────────────────────────────
// HELPER FUNCTIONS
// ────────────────────────────────────────────────────────────────────────────

/// Quick connect helper
pub fn remote(host: &str, port: u16) -> Result<Socket, String> {
    let addr = format!("{}:{}", host, port);
    Socket::connect(addr)
}

/// Quick local process helper
pub fn process(binary: &str) -> Result<Process, String> {
    Process::spawn(binary, &[])
}

/// Quick local process with args
pub fn process_with_args(binary: &str, args: &[&str]) -> Result<Process, String> {
    Process::spawn(binary, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_creation() {
        // This would require a test server
        // For now, just test that the struct exists
        assert_eq!(
            std::mem::size_of::<Socket>(),
            std::mem::size_of::<TcpStream>()
                + std::mem::size_of::<Vec<u8>>()
                + std::mem::size_of::<Duration>()
        );
    }

    #[test]
    fn test_process_creation() {
        // Test spawning a simple process (echo on Unix, cmd on Windows)
        #[cfg(unix)]
        let result = Process::spawn("echo", &["test"]);

        #[cfg(windows)]
        let result = Process::spawn("cmd", &["/C", "echo", "test"]);

        // May fail in test environment, but struct should work
        let _ = result;
    }
}
