use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FunctionDoc {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub parameters: Vec<ParameterDoc>,
    pub returns: String,
    pub examples: Vec<String>,
    pub module: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ParameterDoc {
    pub name: String,
    pub type_hint: String,
    pub description: String,
    pub optional: bool,
    pub default_value: Option<String>,
}

pub struct DocGenerator {
    functions: HashMap<String, FunctionDoc>,
    modules: HashMap<String, Vec<String>>,
}

impl DocGenerator {
    pub fn new() -> Self {
        let mut gen = Self {
            functions: HashMap::new(),
            modules: HashMap::new(),
        };

        gen.populate_stdlib_docs();
        gen
    }

    fn populate_stdlib_docs(&mut self) {
        self.add_function(FunctionDoc {
            name: "pack64".to_string(),
            signature: "pack64(value: int) -> bytes".to_string(),
            description: "Pack a 64-bit integer into little-endian bytes".to_string(),
            parameters: vec![ParameterDoc {
                name: "value".to_string(),
                type_hint: "int".to_string(),
                description: "The integer value to pack".to_string(),
                optional: false,
                default_value: None,
            }],
            returns: "8 bytes in little-endian format".to_string(),
            examples: vec![
                "let addr = pack64(0xdeadbeef)".to_string(),
                "let rop_gadget = pack64(0x401234)".to_string(),
            ],
            module: "packing".to_string(),
            tags: vec!["binary".to_string(), "exploitation".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "cyclic".to_string(),
            signature: "cyclic(length: int) -> bytes".to_string(),
            description: "Generate a De Bruijn sequence for finding buffer overflow offsets"
                .to_string(),
            parameters: vec![ParameterDoc {
                name: "length".to_string(),
                type_hint: "int".to_string(),
                description: "Length of the pattern to generate".to_string(),
                optional: false,
                default_value: None,
            }],
            returns: "Cyclic pattern as bytes".to_string(),
            examples: vec![
                "let pattern = cyclic(1000)".to_string(),
                "let offset = cyclic_find(\"faab\")  # Find offset of pattern".to_string(),
            ],
            module: "exploitation".to_string(),
            tags: vec!["pwn".to_string(), "buffer-overflow".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "connect".to_string(),
            signature: "connect(host: string, port: int) -> socket".to_string(),
            description: "Create a TCP connection to a remote host".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "host".to_string(),
                    type_hint: "string".to_string(),
                    description: "Hostname or IP address".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "port".to_string(),
                    type_hint: "int".to_string(),
                    description: "Port number".to_string(),
                    optional: false,
                    default_value: None,
                },
            ],
            returns: "Socket object for communication".to_string(),
            examples: vec![
                "let sock = connect(\"example.com\", 80)".to_string(),
                "let r = connect(\"192.168.1.100\", 4444)".to_string(),
            ],
            module: "network".to_string(),
            tags: vec!["network".to_string(), "io".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "rop_find".to_string(),
            signature: "rop_find(binary: string, gadget: string) -> list".to_string(),
            description: "Search for ROP gadgets in a binary".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "binary".to_string(),
                    type_hint: "string".to_string(),
                    description: "Path to the binary file".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "gadget".to_string(),
                    type_hint: "string".to_string(),
                    description: "Assembly pattern to search for".to_string(),
                    optional: false,
                    default_value: None,
                },
            ],
            returns: "List of addresses containing the gadget".to_string(),
            examples: vec![
                "let gadgets = rop_find(\"./binary\", \"pop rdi; ret\")".to_string(),
                "let syscall = rop_find(\"./libc.so.6\", \"syscall\")".to_string(),
            ],
            module: "rop".to_string(),
            tags: vec!["pwn".to_string(), "rop".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "shellcode".to_string(),
            signature: "shellcode(arch: string, payload: string, lhost: string, lport: int) -> bytes".to_string(),
            description: "Generate shellcode for various architectures and purposes. Supports x64, x86, ARM, ARM64, and MIPS.".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "arch".to_string(),
                    type_hint: "string".to_string(),
                    description: "Target architecture (x64, x86, arm, arm64, mips)".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "payload".to_string(),
                    type_hint: "string".to_string(),
                    description: "Payload type (execve, reverse_tcp, bind_tcp, read_flag, nop, int3, exit)".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "lhost".to_string(),
                    type_hint: "string".to_string(),
                    description: "Local host for reverse shells".to_string(),
                    optional: true,
                    default_value: Some("127.0.0.1".to_string()),
                },
                ParameterDoc {
                    name: "lport".to_string(),
                    type_hint: "int".to_string(),
                    description: "Local port for reverse shells".to_string(),
                    optional: true,
                    default_value: Some("4444".to_string()),
                }
            ],
            returns: "Raw shellcode bytes".to_string(),
            examples: vec![
                "let sc = shellcode(arch: \"x64\", payload: \"execve\")".to_string(),
                "let rev = shellcode(arch: \"x64\", payload: \"reverse_tcp\", lhost: \"10.0.0.1\", lport: 4444)".to_string(),
            ],
            module: "shellcode".to_string(),
            tags: vec!["shellcode".to_string(), "exploitation".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "cyclic_find".to_string(),
            signature: "cyclic_find(pattern: bytes, search: string|int|bytes) -> int".to_string(),
            description: "Find the offset of a subsequence in a cyclic pattern. Used to determine exact overflow offset.".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "pattern".to_string(),
                    type_hint: "bytes".to_string(),
                    description: "The cyclic pattern generated by cyclic()".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "search".to_string(),
                    type_hint: "string|int|bytes".to_string(),
                    description: "The value to search for (crash address, register value, or string)".to_string(),
                    optional: false,
                    default_value: None,
                }
            ],
            returns: "Offset in the pattern where the search value appears, or null if not found".to_string(),
            examples: vec![
                "let pattern = cyclic(1000)".to_string(),
                "let offset = cyclic_find(pattern, \"faab\")  # Find string offset".to_string(),
                "let offset = cyclic_find(pattern, 0x62616166)  # Find register value offset".to_string(),
            ],
            module: "exploitation".to_string(),
            tags: vec!["pwn".to_string(), "buffer-overflow".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "fmtstr_payload".to_string(),
            signature: "fmtstr_payload(offset: int, writes: map, arch: string) -> bytes".to_string(),
            description: "Generate format string exploit payloads for arbitrary memory writes. Automatically calculates padding and format specifiers.".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "offset".to_string(),
                    type_hint: "int".to_string(),
                    description: "Format string argument offset (where your input appears on the stack)".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "writes".to_string(),
                    type_hint: "map".to_string(),
                    description: "Map of addresses to values to write {\"address\": value}".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "arch".to_string(),
                    type_hint: "string".to_string(),
                    description: "Architecture (x86 or x64)".to_string(),
                    optional: true,
                    default_value: Some("x64".to_string()),
                }
            ],
            returns: "Format string payload bytes".to_string(),
            examples: vec![
                "let payload = fmtstr_payload(offset: 6, writes: {\"0x601048\": 0xdeadbeef})".to_string(),
                "let exploit = fmtstr_payload(offset: 4, writes: {\"0x804a000\": 0x41414141}, arch: \"x86\")".to_string(),
            ],
            module: "exploitation".to_string(),
            tags: vec!["pwn".to_string(), "format-string".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "interactive".to_string(),
            signature: "interactive(host: string, port: int) -> void".to_string(),
            description: "Open an interactive shell session with a remote target. Provides bidirectional communication for live exploitation.".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "host".to_string(),
                    type_hint: "string".to_string(),
                    description: "Target hostname or IP address".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "port".to_string(),
                    type_hint: "int".to_string(),
                    description: "Target port number".to_string(),
                    optional: false,
                    default_value: None,
                }
            ],
            returns: "void (enters interactive mode)".to_string(),
            examples: vec![
                "interactive(host: \"127.0.0.1\", port: 4444)".to_string(),
                "interactive(host: \"challenges.ctf.com\", port: 1337)".to_string(),
            ],
            module: "network".to_string(),
            tags: vec!["pwn".to_string(), "network".to_string(), "shell".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "disasm".to_string(),
            signature: "disasm(bytes: bytes|string, addr: int, offset: int, length: int) -> void".to_string(),
            description: "Disassemble binary code with enhanced visualization. Supports multiple architectures and control flow analysis.".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "bytes".to_string(),
                    type_hint: "bytes|string".to_string(),
                    description: "Raw bytes to disassemble or path to binary file".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "addr".to_string(),
                    type_hint: "int".to_string(),
                    description: "Base address for disassembly".to_string(),
                    optional: true,
                    default_value: Some("0x400000".to_string()),
                },
                ParameterDoc {
                    name: "offset".to_string(),
                    type_hint: "int".to_string(),
                    description: "Offset in file to start disassembly (file mode only)".to_string(),
                    optional: true,
                    default_value: Some("0".to_string()),
                },
                ParameterDoc {
                    name: "length".to_string(),
                    type_hint: "int".to_string(),
                    description: "Number of bytes to disassemble (file mode only)".to_string(),
                    optional: true,
                    default_value: Some("256".to_string()),
                }
            ],
            returns: "void (prints disassembly)".to_string(),
            examples: vec![
                "disasm(shellcode_bytes, addr: 0x400000)".to_string(),
                "disasm(\"./binary\", offset: 0x1000, length: 512)".to_string(),
            ],
            module: "reversing".to_string(),
            tags: vec!["disassembly".to_string(), "reversing".to_string(), "analysis".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "parallel_exploit".to_string(),
            signature: "parallel_exploit(targets: list[string], payload: bytes) -> list".to_string(),
            description: "Execute exploitation payloads against multiple targets concurrently using Tokio async runtime. Supports up to 10 concurrent connections with automatic timeout handling.".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "targets".to_string(),
                    type_hint: "list[string]".to_string(),
                    description: "List of target addresses in \"host:port\" format".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "payload".to_string(),
                    type_hint: "bytes".to_string(),
                    description: "Exploit payload to send to each target".to_string(),
                    optional: false,
                    default_value: None,
                }
            ],
            returns: "list of exploit results with success status for each target".to_string(),
            examples: vec![
                "parallel_exploit([\"192.168.1.1:1337\", \"192.168.1.2:1337\"], payload)".to_string(),
                "let targets = [\"10.0.0.1:4444\", \"10.0.0.2:4444\", \"10.0.0.3:4444\"]\nparallel_exploit(targets, cyclic(200) + pack64(0xdeadbeef))".to_string(),
            ],
            module: "parallel".to_string(),
            tags: vec!["concurrent".to_string(), "exploitation".to_string(), "network".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "generate_exploit".to_string(),
            signature: "generate_exploit(binary: string, vuln_type: string, arch: string) -> string".to_string(),
            description: "Generate exploit code using AI models (local or cloud). Supports buffer overflows, format strings, ROP chains, and more. Defaults to local model with fallback to template-based generation.".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "binary".to_string(),
                    type_hint: "string".to_string(),
                    description: "Path to target binary or challenge name".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "vuln_type".to_string(),
                    type_hint: "string".to_string(),
                    description: "Type of vulnerability (buffer_overflow, format_string, rop, heap, etc)".to_string(),
                    optional: true,
                    default_value: Some("buffer_overflow".to_string()),
                },
                ParameterDoc {
                    name: "arch".to_string(),
                    type_hint: "string".to_string(),
                    description: "Target architecture (x64, x86, arm, arm64)".to_string(),
                    optional: true,
                    default_value: Some("x64".to_string()),
                }
            ],
            returns: "string containing generated exploit code in TALON DSL".to_string(),
            examples: vec![
                "generate_exploit(\"./vuln\", vuln_type: \"buffer_overflow\", arch: \"x64\")".to_string(),
                "let exploit = generate_exploit(\"challenge.bin\")\nprint(exploit)".to_string(),
            ],
            module: "ai".to_string(),
            tags: vec!["ai".to_string(), "codegen".to_string(), "automation".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "p64".to_string(),
            signature: "p64(value: int) -> bytes".to_string(),
            description: "Pack 64-bit integer to little-endian bytes. Essential for building ROP chains and overflow payloads.".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "value".to_string(),
                    type_hint: "int".to_string(),
                    description: "Integer value to pack (0 to 2^64-1)".to_string(),
                    optional: false,
                    default_value: None,
                }
            ],
            returns: "8-byte little-endian representation".to_string(),
            examples: vec![
                "let addr = p64(0x400080)".to_string(),
                "let rop_chain = p64(pop_rdi) + p64(bin_sh) + p64(system)".to_string(),
                "let payload = cyclic(40) + p64(0xdeadbeef)".to_string(),
            ],
            module: "packing".to_string(),
            tags: vec!["packing".to_string(), "bytes".to_string(), "exploitation".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "p32".to_string(),
            signature: "p32(value: int) -> bytes".to_string(),
            description: "Pack 32-bit integer to little-endian bytes. Used for x86 exploitation and 32-bit addresses.".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "value".to_string(),
                    type_hint: "int".to_string(),
                    description: "Integer value to pack (0 to 2^32-1)".to_string(),
                    optional: false,
                    default_value: None,
                }
            ],
            returns: "4-byte little-endian representation".to_string(),
            examples: vec![
                "let addr = p32(0x08048080)".to_string(),
                "let payload = b\"A\" * 40 + p32(ret_addr)".to_string(),
            ],
            module: "packing".to_string(),
            tags: vec!["packing".to_string(), "bytes".to_string(), "x86".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "u64".to_string(),
            signature: "u64(data: bytes) -> int".to_string(),
            description:
                "Unpack 8 bytes to 64-bit little-endian integer. Used to parse leaked addresses."
                    .to_string(),
            parameters: vec![ParameterDoc {
                name: "data".to_string(),
                type_hint: "bytes".to_string(),
                description: "Byte data to unpack (at least 8 bytes)".to_string(),
                optional: false,
                default_value: None,
            }],
            returns: "64-bit integer value".to_string(),
            examples: vec![
                "let leaked = u64(response[0:8])".to_string(),
                "let libc_base = u64(leak) - libc_offset".to_string(),
            ],
            module: "packing".to_string(),
            tags: vec![
                "packing".to_string(),
                "bytes".to_string(),
                "leak".to_string(),
            ],
        });

        self.add_function(FunctionDoc {
            name: "u32".to_string(),
            signature: "u32(data: bytes) -> int".to_string(),
            description: "Unpack 4 bytes to 32-bit little-endian integer.".to_string(),
            parameters: vec![ParameterDoc {
                name: "data".to_string(),
                type_hint: "bytes".to_string(),
                description: "Byte data to unpack (at least 4 bytes)".to_string(),
                optional: false,
                default_value: None,
            }],
            returns: "32-bit integer value".to_string(),
            examples: vec!["let leaked_addr = u32(response[0:4])".to_string()],
            module: "packing".to_string(),
            tags: vec![
                "packing".to_string(),
                "bytes".to_string(),
                "x86".to_string(),
            ],
        });

        self.add_function(FunctionDoc {
            name: "parse_elf".to_string(),
            signature: "parse_elf(path: string) -> map".to_string(),
            description: "Parse ELF binary and extract symbols, PLT, GOT, sections, and security features. Returns map with symbol addresses prefixed by type (sym_, plt_, got_).".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "path".to_string(),
                    type_hint: "string".to_string(),
                    description: "Path to ELF binary file".to_string(),
                    optional: false,
                    default_value: None,
                }
            ],
            returns: "map containing binary metadata and all symbols".to_string(),
            examples: vec![
                "let elf = parse_elf(\"./vuln\")\nlet main = elf[\"sym_main\"]\nlet puts_plt = elf[\"plt_puts\"]\nlet puts_got = elf[\"got_puts\"]".to_string(),
                "let elf = parse_elf(\"/lib/x86_64-linux-gnu/libc.so.6\")\nif elf[\"pie\"] { print(\"PIE enabled\") }".to_string(),
            ],
            module: "binary".to_string(),
            tags: vec!["elf".to_string(), "binary".to_string(), "symbols".to_string(), "got".to_string(), "plt".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "remote".to_string(),
            signature: "remote(host: string, port: int) -> connection".to_string(),
            description: "Create TCP connection to remote host. Returns connection object for send/recv operations.".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "host".to_string(),
                    type_hint: "string".to_string(),
                    description: "Target hostname or IP address".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "port".to_string(),
                    type_hint: "int".to_string(),
                    description: "Target port number".to_string(),
                    optional: false,
                    default_value: None,
                }
            ],
            returns: "connection object with id, host, port, and type".to_string(),
            examples: vec![
                "let conn = remote(\"192.168.1.100\", 1337)\nsendline(conn, \"GET / HTTP/1.0\")\nlet response = recvline(conn)".to_string(),
                "let r = remote(host: \"challenges.ctf.com\", port: 9001)".to_string(),
            ],
            module: "network".to_string(),
            tags: vec!["network".to_string(), "tcp".to_string(), "remote".to_string(), "connection".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "process".to_string(),
            signature: "process(binary: string, args: list?) -> connection".to_string(),
            description: "Spawn local process for exploitation. Returns connection object for I/O operations.".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "binary".to_string(),
                    type_hint: "string".to_string(),
                    description: "Path to executable binary".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "args".to_string(),
                    type_hint: "list".to_string(),
                    description: "Optional command-line arguments".to_string(),
                    optional: true,
                    default_value: Some("[]".to_string()),
                }
            ],
            returns: "connection object with id, binary, and type".to_string(),
            examples: vec![
                "let p = process(\"./vulnerable\")\nsendline(p, cyclic(100))\nlet leak = recvline(p)".to_string(),
                "let p = process(binary: \"./binary\", args: [\"-v\", \"--debug\"])".to_string(),
            ],
            module: "process".to_string(),
            tags: vec!["process".to_string(), "local".to_string(), "exploitation".to_string(), "connection".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "send".to_string(),
            signature: "send(conn: connection, data: bytes|string) -> int".to_string(),
            description: "Send data to connection (remote or process). Returns number of bytes sent.".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "conn".to_string(),
                    type_hint: "connection".to_string(),
                    description: "Connection object from remote() or process()".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "data".to_string(),
                    type_hint: "bytes|string".to_string(),
                    description: "Data to send".to_string(),
                    optional: false,
                    default_value: None,
                }
            ],
            returns: "number of bytes sent".to_string(),
            examples: vec![
                "let conn = remote(\"10.0.0.1\", 9001)\nsend(conn, \"HELLO\")\nsend(conn, p64(0xdeadbeef))".to_string(),
            ],
            module: "network".to_string(),
            tags: vec!["network".to_string(), "io".to_string(), "send".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "sendline".to_string(),
            signature: "sendline(conn: connection, data: bytes|string) -> int".to_string(),
            description:
                "Send data with newline appended. Returns number of bytes sent including newline."
                    .to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "conn".to_string(),
                    type_hint: "connection".to_string(),
                    description: "Connection object from remote() or process()".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "data".to_string(),
                    type_hint: "bytes|string".to_string(),
                    description: "Data to send (newline will be appended)".to_string(),
                    optional: false,
                    default_value: None,
                },
            ],
            returns: "number of bytes sent including newline".to_string(),
            examples: vec![
                "let p = process(\"./vuln\")\nsendline(p, cyclic(100))\nsendline(p, \"admin\")"
                    .to_string(),
            ],
            module: "network".to_string(),
            tags: vec!["network".to_string(), "io".to_string(), "send".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "recv".to_string(),
            signature: "recv(conn: connection, n: int) -> bytes".to_string(),
            description: "Receive exactly n bytes from connection. Blocks until all bytes received.".to_string(),
            parameters: vec![
                ParameterDoc {
                    name: "conn".to_string(),
                    type_hint: "connection".to_string(),
                    description: "Connection object from remote() or process()".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDoc {
                    name: "n".to_string(),
                    type_hint: "int".to_string(),
                    description: "Number of bytes to receive".to_string(),
                    optional: false,
                    default_value: None,
                }
            ],
            returns: "received bytes".to_string(),
            examples: vec![
                "let conn = remote(\"ctf.com\", 1337)\nlet leak = recv(conn, 8)\nlet addr = u64(leak)".to_string(),
            ],
            module: "network".to_string(),
            tags: vec!["network".to_string(), "io".to_string(), "recv".to_string()],
        });

        self.add_function(FunctionDoc {
            name: "recvline".to_string(),
            signature: "recvline(conn: connection) -> bytes".to_string(),
            description: "Receive data until newline. Returns data including the newline."
                .to_string(),
            parameters: vec![ParameterDoc {
                name: "conn".to_string(),
                type_hint: "connection".to_string(),
                description: "Connection object from remote() or process()".to_string(),
                optional: false,
                default_value: None,
            }],
            returns: "received line with newline".to_string(),
            examples: vec![
                "let p = process(\"./binary\")\nlet banner = recvline(p)\nprint(banner)"
                    .to_string(),
            ],
            module: "network".to_string(),
            tags: vec!["network".to_string(), "io".to_string(), "recv".to_string()],
        });
    }

    fn add_function(&mut self, doc: FunctionDoc) {
        let module = doc.module.clone();
        self.modules
            .entry(module.clone())
            .or_default()
            .push(doc.name.clone());

        self.functions.insert(doc.name.clone(), doc);
    }

    pub fn get_function_doc(&self, name: &str) -> Option<&FunctionDoc> {
        self.functions.get(name)
    }

    pub fn search(&self, query: &str) -> Vec<&FunctionDoc> {
        let query_lower = query.to_lowercase();

        self.functions
            .values()
            .filter(|doc| {
                doc.name.to_lowercase().contains(&query_lower)
                    || doc.description.to_lowercase().contains(&query_lower)
                    || doc
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    pub fn list_modules(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }

    pub fn get_module_functions(&self, module: &str) -> Vec<&FunctionDoc> {
        self.modules
            .get(module)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| self.functions.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn display_function(&self, name: &str) {
        if let Some(doc) = self.get_function_doc(name) {
            self.print_function_doc(doc);
        } else {
            println!(
                "{} {}",
                "[ERROR]".red(),
                format!("Function '{}' not found in documentation.", name).red()
            );

            let similar: Vec<_> = self
                .functions
                .keys()
                .filter(|k| k.contains(name) || name.contains(k.as_str()))
                .take(5)
                .collect();

            if !similar.is_empty() {
                println!("\n{}", "TIP: Did you mean one of these?".yellow());
                for suggestion in similar {
                    println!("  {} {}", "•".cyan(), suggestion.green());
                }
            } else {
                println!(
                    "\n{} Use help() to see all available functions",
                    "TIP:".yellow()
                );
            }
        }
    }

    pub fn display_search_results(&self, query: &str) {
        let results = self.search(query);

        if results.is_empty() {
            println!(
                "{} {}",
                "[SEARCH]".yellow(),
                format!("No functions found matching '{}'", query).bright_black()
            );
            return;
        }

        println!(
            "\n{} {} {}",
            "[SEARCH]".yellow(),
            "Search Results for".cyan().bold(),
            query.green().bold()
        );
        println!("{}\n", "─".repeat(60).bright_black());

        for doc in results {
            println!("{} {}", "[*]".cyan(), doc.name.green().bold());
            println!("   {}", doc.description.white());
            println!(
                "   {} {} | {} {}",
                "Module:".bright_black(),
                doc.module.yellow(),
                "Tags:".bright_black(),
                doc.tags.join(", ").magenta()
            );
            println!();
        }
    }

    fn print_function_doc(&self, doc: &FunctionDoc) {
        println!("\n{}", "═".repeat(80).cyan());
        println!(
            "  {} {}",
            "[DOC]".blue(),
            doc.name.to_uppercase().cyan().bold()
        );
        println!("{}\n", "═".repeat(80).cyan());

        println!("{}", "SIGNATURE".yellow().bold());
        println!("  {}\n", doc.signature.green());

        println!("{}", "DESCRIPTION".yellow().bold());
        println!("  {}\n", doc.description.white());

        if !doc.parameters.is_empty() {
            println!("{}", "PARAMETERS".yellow().bold());
            for param in &doc.parameters {
                let optional_marker = if param.optional {
                    format!(" {}", "(optional)".bright_black())
                } else {
                    String::new()
                };
                println!(
                    "  {} {} {}{}",
                    "•".cyan(),
                    param.name.green().bold(),
                    format!(": {}", param.type_hint).magenta(),
                    optional_marker
                );
                println!("      {}", param.description.white());
                if let Some(default) = &param.default_value {
                    println!("      {} {}", "Default:".bright_black(), default.yellow());
                }
                println!();
            }
        }

        println!("{}", "RETURNS".yellow().bold());
        println!("  {}\n", doc.returns.white());

        if !doc.examples.is_empty() {
            println!("{}", "EXAMPLES".yellow().bold());
            for (idx, example) in doc.examples.iter().enumerate() {
                println!(
                    "  {} {}",
                    format!("{}.", idx + 1).bright_black(),
                    example.cyan()
                );
            }
            println!();
        }

        if !doc.tags.is_empty() {
            println!("{}", "TAGS".yellow().bold());
            print!("  ");
            for tag in &doc.tags {
                print!("{} ", format!("#{}", tag).magenta());
            }
            println!("\n");
        }

        println!(
            "{} {}",
            "MODULE:".bright_black(),
            doc.module.yellow().bold()
        );
        println!();
    }

    pub fn list_all(&self) {
        println!("\n╔═══════════════════════════════════════════════════════════════════════════╗");
        println!("║                    TALON STANDARD LIBRARY                                 ║");
        println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");

        let mut modules: Vec<_> = self.modules.keys().collect();
        modules.sort();

        for module in modules {
            println!("{}:", module.to_uppercase());
            if let Some(functions) = self.modules.get(module.as_str()) {
                for func_name in functions {
                    if let Some(doc) = self.functions.get(func_name) {
                        println!(
                            "  {:20} - {}",
                            doc.name,
                            doc.description.chars().take(50).collect::<String>()
                        );
                    }
                }
            }
            println!();
        }

        println!("Use 'talon doc <function>' to see detailed documentation");
        println!("Use 'talon doc search <query>' to search documentation\n");
    }

    pub fn export_markdown(&self, output_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(output_dir).map_err(|e| format!("Failed to create directory: {}", e))?;

        for module_name in self.modules.keys() {
            let module_path = output_dir.join(format!("{}.md", module_name));
            let mut content = String::new();

            content.push_str(&format!("# {} Module\n\n", module_name.to_uppercase()));

            let funcs = self.get_module_functions(module_name);
            for doc in funcs {
                content.push_str(&format!("## {}\n\n", doc.name));
                content.push_str(&format!("**Signature:** `{}`\n\n", doc.signature));
                content.push_str(&format!("{}\n\n", doc.description));

                if !doc.parameters.is_empty() {
                    content.push_str("### Parameters\n\n");
                    for param in &doc.parameters {
                        content.push_str(&format!(
                            "- **{}** (`{}`): {}\n",
                            param.name, param.type_hint, param.description
                        ));
                    }
                    content.push('\n');
                }

                content.push_str(&format!("### Returns\n\n{}\n\n", doc.returns));

                if !doc.examples.is_empty() {
                    content.push_str("### Examples\n\n```talon\n");
                    for example in &doc.examples {
                        content.push_str(&format!("{}\n", example));
                    }
                    content.push_str("```\n\n");
                }

                content.push_str("---\n\n");
            }

            fs::write(&module_path, content)
                .map_err(|e| format!("Failed to write module doc: {}", e))?;
        }

        Ok(())
    }
}

impl Default for DocGenerator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn print_doc_help() {
    println!(
        r#"
╔═══════════════════════════════════════════════════════════════════════════╗
║                    DOCUMENTATION SYSTEM                                   ║
╚═══════════════════════════════════════════════════════════════════════════╝

USAGE
# Show documentation for a function
talon doc <function_name>

# Search documentation
talon doc search <query>

# List all documented functions
talon doc list

# List functions by module
talon doc module <module_name>

# Export documentation to Markdown
talon doc export ./docs

EXAMPLES
talon doc pack64
talon doc search "shellcode"
talon doc module rop
talon doc list

INTERACTIVE REPL MODE
Within the REPL, use the help() function:

> help(pack64)
> help("cyclic")
> search("overflow")

For comprehensive language reference: talon man
"#
    );
}
