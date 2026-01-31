// Interactive shell for live exploitation
// Provides bidirectional communication with target processes

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct InteractiveShell {
    stream: Arc<Mutex<TcpStream>>,
    running: Arc<Mutex<bool>>,
}

impl InteractiveShell {
    pub fn new(stream: TcpStream) -> Result<Self, String> {
        stream
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| format!("Failed to set read timeout: {}", e))?;

        Ok(InteractiveShell {
            stream: Arc::new(Mutex::new(stream)),
            running: Arc::new(Mutex::new(false)),
        })
    }

    pub fn start(&mut self) -> Result<(), String> {
        *self.running.lock().unwrap() = true;

        println!("[*] Starting interactive shell");
        println!("[*] Press Ctrl+C to exit");
        println!("{}", "-".repeat(50));

        let stream_clone = Arc::clone(&self.stream);
        let running_clone = Arc::clone(&self.running);

        let recv_thread = thread::spawn(move || {
            let mut buffer = [0u8; 4096];

            while *running_clone.lock().unwrap() {
                if let Ok(mut stream) = stream_clone.lock() {
                    match stream.read(&mut buffer) {
                        Ok(0) => {
                            println!("\n[*] Connection closed by remote host");
                            *running_clone.lock().unwrap() = false;
                            break;
                        }
                        Ok(n) => {
                            let data = &buffer[..n];
                            print!("{}", String::from_utf8_lossy(data));
                            std::io::stdout().flush().ok();
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(10));
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                            thread::sleep(Duration::from_millis(10));
                        }
                        Err(e) => {
                            eprintln!("\n[!] Read error: {}", e);
                            *running_clone.lock().unwrap() = false;
                            break;
                        }
                    }
                }
            }
        });

        let stdin = std::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        while *self.running.lock().unwrap() {
            line.clear();

            match reader.read_line(&mut line) {
                Ok(0) => {
                    *self.running.lock().unwrap() = false;
                    break;
                }
                Ok(_) => {
                    if let Ok(mut stream) = self.stream.lock() {
                        if let Err(e) = stream.write_all(line.as_bytes()) {
                            eprintln!("[!] Write error: {}", e);
                            *self.running.lock().unwrap() = false;
                            break;
                        }
                        stream.flush().ok();
                    }
                }
                Err(e) => {
                    eprintln!("[!] Input error: {}", e);
                    *self.running.lock().unwrap() = false;
                    break;
                }
            }
        }

        *self.running.lock().unwrap() = false;
        recv_thread.join().ok();

        println!("\n{}", "-".repeat(50));
        println!("[*] Interactive shell closed");

        Ok(())
    }

    pub fn send_raw(&self, data: &[u8]) -> Result<(), String> {
        let mut stream = self.stream.lock().unwrap();
        stream
            .write_all(data)
            .map_err(|e| format!("Failed to send data: {}", e))?;
        stream
            .flush()
            .map_err(|e| format!("Failed to flush stream: {}", e))?;
        Ok(())
    }

    pub fn recv_until(&self, delimiter: &[u8], timeout: Duration) -> Result<Vec<u8>, String> {
        let mut buffer = Vec::new();
        let mut single_byte = [0u8; 1];
        let start = std::time::Instant::now();

        let mut stream = self.stream.lock().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        loop {
            if start.elapsed() > timeout {
                return Err("Timeout waiting for delimiter".to_string());
            }

            match stream.read(&mut single_byte) {
                Ok(0) => {
                    return Err("Connection closed".to_string());
                }
                Ok(_) => {
                    buffer.push(single_byte[0]);

                    if buffer.len() >= delimiter.len() {
                        let end_slice = &buffer[buffer.len() - delimiter.len()..];
                        if end_slice == delimiter {
                            return Ok(buffer);
                        }
                    }
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    return Err(format!("Read error: {}", e));
                }
            }
        }
    }

    pub fn recv_n_bytes(&self, n: usize, timeout: Duration) -> Result<Vec<u8>, String> {
        let mut buffer = vec![0u8; n];
        let mut total_read = 0;
        let start = std::time::Instant::now();

        let mut stream = self.stream.lock().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|e| format!("Failed to set timeout: {}", e))?;

        while total_read < n {
            if start.elapsed() > timeout {
                return Err(format!("Timeout: received {}/{} bytes", total_read, n));
            }

            match stream.read(&mut buffer[total_read..]) {
                Ok(0) => {
                    return Err("Connection closed".to_string());
                }
                Ok(bytes_read) => {
                    total_read += bytes_read;
                }
                Err(ref e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut =>
                {
                    thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(e) => {
                    return Err(format!("Read error: {}", e));
                }
            }
        }

        Ok(buffer)
    }

    pub fn sendline(&self, line: &str) -> Result<(), String> {
        let data = format!("{}\n", line);
        self.send_raw(data.as_bytes())
    }

    pub fn recvline(&self, timeout: Duration) -> Result<String, String> {
        let data = self.recv_until(b"\n", timeout)?;
        Ok(String::from_utf8_lossy(&data).to_string())
    }
}

pub fn create_interactive_shell(host: &str, port: u16) -> Result<InteractiveShell, String> {
    let addr = format!("{}:{}", host, port);
    let stream =
        TcpStream::connect(&addr).map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;

    InteractiveShell::new(stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn test_interactive_shell_creation() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let server_thread = thread::spawn(move || {
            let (_stream, _addr) = listener.accept().unwrap();
            thread::sleep(Duration::from_millis(100));
        });

        thread::sleep(Duration::from_millis(50));

        let result = create_interactive_shell("127.0.0.1", addr.port());
        assert!(result.is_ok());

        server_thread.join().ok();
    }

    #[test]
    fn test_send_recv() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let server_thread = thread::spawn(move || {
            let (mut stream, _addr) = listener.accept().unwrap();
            let mut buffer = [0u8; 1024];

            if let Ok(n) = stream.read(&mut buffer) {
                stream.write_all(&buffer[..n]).ok();
            }
        });

        thread::sleep(Duration::from_millis(50));

        let shell = create_interactive_shell("127.0.0.1", addr.port()).unwrap();
        shell.sendline("test").unwrap();

        let response = shell.recvline(Duration::from_secs(1));
        assert!(response.is_ok());

        server_thread.join().ok();
    }
}
