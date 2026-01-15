// ═══════════════════════════════════════════════════════════════════════════
// GDB INTEGRATION - DYNAMIC HEAP ANALYSIS & DEBUGGING
// ═══════════════════════════════════════════════════════════════════════════
// World-class GDB integration for live heap inspection, leak extraction,
// and dynamic exploit development

use std::process::{Command, Stdio, Child, ChildStdin, ChildStdout};
use std::io::{Write, BufRead, BufReader};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// GDB session for exploit development
pub struct GdbSession {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    breakpoints: Vec<u64>,
    _registers: HashMap<String, u64>,
}

impl GdbSession {
    /// Attach to a running process
    pub fn attach(pid: u32) -> Result<Self, String> {
        Self::start(&format!("--pid={}", pid))
    }
    
    /// Start GDB with a binary
    pub fn start(args: &str) -> Result<Self, String> {
        let mut cmd = Command::new("gdb");
        cmd.arg("-q") // Quiet mode
           .arg("-batch-silent")
           .arg("-ex").arg("set pagination off")
           .arg("-ex").arg("set confirm off");
        
        if !args.is_empty() {
            for arg in args.split_whitespace() {
                cmd.arg(arg);
            }
        }
        
        cmd.stdin(Stdio::piped())
           .stdout(Stdio::piped())
           .stderr(Stdio::null());
        
        let mut process = cmd.spawn()
            .map_err(|e| format!("Failed to start GDB: {}", e))?;
        
        let stdin = process.stdin.take()
            .ok_or("Failed to open GDB stdin")?;
        let stdout = BufReader::new(process.stdout.take()
            .ok_or("Failed to open GDB stdout")?);
        
        log::info!("Started GDB session");
        
        Ok(GdbSession {
            process,
            stdin,
            stdout,
            breakpoints: Vec::new(),
            _registers: HashMap::new(),
        })
    }
    
    /// Execute GDB command and get output
    pub fn execute(&mut self, cmd: &str) -> Result<String, String> {
        writeln!(self.stdin, "{}", cmd)
            .map_err(|e| format!("Failed to write command: {}", e))?;
        
        let mut output = String::new();
        let mut line = String::new();
        
        loop {
            line.clear();
            match self.stdout.read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => {
                    output.push_str(&line);
                    if line.starts_with("(gdb)") {
                        break;
                    }
                }
                Err(e) => return Err(format!("Failed to read output: {}", e)),
            }
        }
        
        Ok(output.trim().to_string())
    }
    
    /// Read memory at address
    pub fn read_memory(&mut self, addr: u64, size: usize) -> Result<Vec<u8>, String> {
        let cmd = format!("x/{}xb 0x{:x}", size, addr);
        let output = self.execute(&cmd)?;
        
        let mut bytes = Vec::new();
        for line in output.lines() {
            if let Some(hex_part) = line.split(':').nth(1) {
                for byte_str in hex_part.split_whitespace() {
                    if let Ok(byte) = u8::from_str_radix(byte_str.trim_start_matches("0x"), 16) {
                        bytes.push(byte);
                    }
                }
            }
        }
        
        Ok(bytes)
    }
    
    /// Write memory at address
    pub fn write_memory(&mut self, addr: u64, data: &[u8]) -> Result<(), String> {
        for (i, &byte) in data.iter().enumerate() {
            let cmd = format!("set {{unsigned char}}0x{:x}=0x{:02x}", addr + i as u64, byte);
            self.execute(&cmd)?;
        }
        Ok(())
    }
    
    /// Get register value
    pub fn get_register(&mut self, reg: &str) -> Result<u64, String> {
        let cmd = format!("info register {}", reg);
        let output = self.execute(&cmd)?;
        
        for line in output.lines() {
            if let Some(hex_str) = line.split_whitespace().nth(1) {
                let value = u64::from_str_radix(hex_str.trim_start_matches("0x"), 16)
                    .map_err(|_| format!("Failed to parse register value: {}", hex_str))?;
                return Ok(value);
            }
        }
        
        Err(format!("Failed to get register {}", reg))
    }
    
    /// Set breakpoint
    pub fn breakpoint(&mut self, addr: u64) -> Result<(), String> {
        let cmd = format!("break *0x{:x}", addr);
        self.execute(&cmd)?;
        self.breakpoints.push(addr);
        log::info!("Set breakpoint at 0x{:x}", addr);
        Ok(())
    }
    
    /// Continue execution
    pub fn continue_exec(&mut self) -> Result<String, String> {
        self.execute("continue")
    }
    
    /// Single step
    pub fn step(&mut self) -> Result<String, String> {
        self.execute("stepi")
    }
    
    /// Get heap info (glibc-specific)
    pub fn heap_info(&mut self) -> Result<HeapInfo, String> {
        let arena_output = self.execute("p main_arena")?;
        let chunks_output = self.execute("heap chunks")?;
        
        Ok(HeapInfo {
            arena_address: Self::extract_address(&arena_output).unwrap_or(0),
            chunks: Self::parse_heap_chunks(&chunks_output),
            tcache: None,
        })
    }
    
    /// Get tcache bins
    pub fn tcache_bins(&mut self) -> Result<Vec<TcacheBin>, String> {
        let output = self.execute("tcache")?;
        Ok(Self::parse_tcache_bins(&output))
    }
    
    /// Leak libc base address
    pub fn leak_libc_base(&mut self) -> Result<u64, String> {
        // Try common methods
        let methods = [
            "info proc mappings | grep libc",
            "p &system",
            "x/gx &__libc_start_main",
        ];
        
        for method in &methods {
            if let Ok(output) = self.execute(method) {
                if let Some(addr) = Self::extract_address(&output) {
                    // Align to page boundary
                    let base = addr & !0xfff;
                    log::info!("Leaked libc base: 0x{:x}", base);
                    return Ok(base);
                }
            }
        }
        
        Err("Failed to leak libc base".to_string())
    }
    
    /// Leak heap base address
    pub fn leak_heap_base(&mut self) -> Result<u64, String> {
        let output = self.execute("info proc mappings | grep heap")?;
        
        if let Some(addr) = Self::extract_address(&output) {
            log::info!("Leaked heap base: 0x{:x}", addr);
            return Ok(addr);
        }
        
        Err("Failed to leak heap base".to_string())
    }
    
    /// Find ROP gadgets in memory range
    pub fn find_gadgets(&mut self, start: u64, end: u64, pattern: &str) -> Result<Vec<u64>, String> {
        let cmd = format!("find /b 0x{:x}, 0x{:x}, {}", start, end, pattern);
        let output = self.execute(&cmd)?;
        
        let mut gadgets = Vec::new();
        for line in output.lines() {
            if let Some(addr) = Self::extract_address(line) {
                gadgets.push(addr);
            }
        }
        
        Ok(gadgets)
    }
    
    // Helper: Extract hex address from string
    fn extract_address(s: &str) -> Option<u64> {
        for word in s.split_whitespace() {
            if let Some(hex) = word.strip_prefix("0x") {
                if let Ok(addr) = u64::from_str_radix(hex, 16) {
                    if addr > 0x1000 { // Sanity check
                        return Some(addr);
                    }
                }
            }
        }
        None
    }
    
    // Helper: Parse heap chunks output
    fn parse_heap_chunks(output: &str) -> Vec<HeapChunkInfo> {
        let mut chunks = Vec::new();
        
        for line in output.lines() {
            if line.contains("Chunk") {
                if let Some(addr) = Self::extract_address(line) {
                    chunks.push(HeapChunkInfo {
                        address: addr,
                        size: 0,
                        in_use: !line.contains("free"),
                    });
                }
            }
        }
        
        chunks
    }
    
    // Helper: Parse tcache bins output
    fn parse_tcache_bins(output: &str) -> Vec<TcacheBin> {
        let mut bins = Vec::new();
        let mut current_size = 0;
        
        for line in output.lines() {
            if line.contains("tcache_entry[") {
                if let Some(size_str) = line.split('[').nth(1).and_then(|s| s.split(']').next()) {
                    current_size = size_str.parse().unwrap_or(0);
                }
            } else if let Some(addr) = Self::extract_address(line) {
                bins.push(TcacheBin {
                    size: current_size,
                    chunk_address: addr,
                });
            }
        }
        
        bins
    }
}

impl Drop for GdbSession {
    fn drop(&mut self) {
        let _ = self.execute("quit");
        let _ = self.process.kill();
    }
}

// ────────────────────────────────────────────────────────────────────────────
// DATA STRUCTURES
// ────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapInfo {
    pub arena_address: u64,
    pub chunks: Vec<HeapChunkInfo>,
    pub tcache: Option<Vec<TcacheBin>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeapChunkInfo {
    pub address: u64,
    pub size: usize,
    pub in_use: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcacheBin {
    pub size: usize,
    pub chunk_address: u64,
}

// ────────────────────────────────────────────────────────────────────────────
// HELPER FUNCTIONS
// ────────────────────────────────────────────────────────────────────────────

/// Quick attach to process and leak addresses
pub fn quick_leak(pid: u32) -> Result<(u64, u64), String> {
    let mut gdb = GdbSession::attach(pid)?;
    
    let libc_base = gdb.leak_libc_base()?;
    let heap_base = gdb.leak_heap_base()?;
    
    Ok((libc_base, heap_base))
}

/// Dump heap state for analysis
pub fn dump_heap(pid: u32) -> Result<HeapInfo, String> {
    let mut gdb = GdbSession::attach(pid)?;
    let mut info = gdb.heap_info()?;
    info.tcache = Some(gdb.tcache_bins()?);
    Ok(info)
}

/// Find "pop rdi; ret" gadget
pub fn find_pop_rdi(pid: u32) -> Result<u64, String> {
    let mut gdb = GdbSession::attach(pid)?;
    let libc_base = gdb.leak_libc_base()?;
    
    // Search for pop rdi (0x5f) + ret (0xc3)
    let gadgets = gdb.find_gadgets(libc_base, libc_base + 0x200000, "0x5f, 0xc3")?;
    
    gadgets.first().copied()
        .ok_or("pop rdi gadget not found".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_address() {
        assert_eq!(
            GdbSession::extract_address("0x7ffff7a00000 some text"),
            Some(0x7ffff7a00000)
        );
        assert_eq!(
            GdbSession::extract_address("rax: 0x555555554000"),
            Some(0x555555554000)
        );
        assert_eq!(GdbSession::extract_address("no address here"), None);
    }
    
    #[test]
    fn test_heap_chunk_info_creation() {
        let chunk = HeapChunkInfo {
            address: 0x555555554290,
            size: 0x80,
            in_use: true,
        };
        assert_eq!(chunk.address, 0x555555554290);
        assert!(chunk.in_use);
    }
    
    #[test]
    fn test_tcache_bin_creation() {
        let bin = TcacheBin {
            size: 0x20,
            chunk_address: 0x555555554290,
        };
        assert_eq!(bin.size, 0x20);
    }
}
