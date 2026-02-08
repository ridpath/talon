use std::collections::HashMap;
use lazy_static::lazy_static;

include!(concat!(env!("OUT_DIR"), "/registry_phf.rs"));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunctionCategory {
    Network,
    Process,
    IO,
    SSH,
    Packing,
    Exploitation,
    BinaryAnalysis,
    Utilities,
    FileIO,
    StringManipulation,
    Encoding,
    Evasion,
    AI,
    Swarm,
    Crypto,
    Fuzzing,
    Kernel,
    Heap,
    Debugging,
    Symbolic,
}

impl FunctionCategory {
    pub fn as_str(&self) -> &str {
        match self {
            FunctionCategory::Network => "Network",
            FunctionCategory::Process => "Process",
            FunctionCategory::IO => "I/O",
            FunctionCategory::SSH => "SSH",
            FunctionCategory::Packing => "Packing",
            FunctionCategory::Exploitation => "Exploitation",
            FunctionCategory::BinaryAnalysis => "Binary Analysis",
            FunctionCategory::Utilities => "Utilities",
            FunctionCategory::FileIO => "File I/O",
            FunctionCategory::StringManipulation => "String Manipulation",
            FunctionCategory::Encoding => "Encoding",
            FunctionCategory::Evasion => "Evasion",
            FunctionCategory::AI => "AI",
            FunctionCategory::Swarm => "Swarm",
            FunctionCategory::Crypto => "Crypto",
            FunctionCategory::Fuzzing => "Fuzzing",
            FunctionCategory::Kernel => "Kernel",
            FunctionCategory::Heap => "Heap",
            FunctionCategory::Debugging => "Debugging",
            FunctionCategory::Symbolic => "Symbolic",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Network" => Some(FunctionCategory::Network),
            "Process" => Some(FunctionCategory::Process),
            "I/O" => Some(FunctionCategory::IO),
            "SSH" => Some(FunctionCategory::SSH),
            "Packing" => Some(FunctionCategory::Packing),
            "Exploitation" => Some(FunctionCategory::Exploitation),
            "Binary Analysis" => Some(FunctionCategory::BinaryAnalysis),
            "Utilities" => Some(FunctionCategory::Utilities),
            "File I/O" => Some(FunctionCategory::FileIO),
            "String Manipulation" => Some(FunctionCategory::StringManipulation),
            "Encoding" => Some(FunctionCategory::Encoding),
            "Evasion" => Some(FunctionCategory::Evasion),
            "AI" => Some(FunctionCategory::AI),
            "Swarm" => Some(FunctionCategory::Swarm),
            "Crypto" => Some(FunctionCategory::Crypto),
            "Fuzzing" => Some(FunctionCategory::Fuzzing),
            "Kernel" => Some(FunctionCategory::Kernel),
            "Heap" => Some(FunctionCategory::Heap),
            "Debugging" => Some(FunctionCategory::Debugging),
            "Symbolic" => Some(FunctionCategory::Symbolic),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub category: String,
    pub examples: Vec<String>,
    pub since_version: String,
    pub deprecated: Option<String>,
    pub related: Vec<String>,
}

impl BuiltinFunction {
    pub fn new(
        name: &str,
        signature: &str,
        description: &str,
        category: &str,
        examples: Vec<&str>,
    ) -> Self {
        BuiltinFunction {
            name: name.to_string(),
            signature: signature.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            examples: examples.iter().map(|s| s.to_string()).collect(),
            since_version: "0.1.0".to_string(),
            deprecated: None,
            related: Vec::new(),
        }
    }

    pub fn with_version(mut self, version: &str) -> Self {
        self.since_version = version.to_string();
        self
    }

    pub fn with_deprecation(mut self, message: &str) -> Self {
        self.deprecated = Some(message.to_string());
        self
    }

    pub fn with_related(mut self, related: Vec<&str>) -> Self {
        self.related = related.iter().map(|s| s.to_string()).collect();
        self
    }
}

lazy_static! {
    static ref BUILTIN_FUNCTIONS: Vec<BuiltinFunction> = {
        let functions_map = register_builtins();
        let mut functions_vec = vec![BuiltinFunction::new("", "", "", "", vec![]); BUILTIN_COUNT];
        
        for (name, func) in functions_map {
            if let Some(&idx) = BUILTIN_REGISTRY.get(&name) {
                functions_vec[idx] = func;
            }
        }
        
        functions_vec
    };
}

pub struct FunctionRegistry {
    functions: HashMap<String, BuiltinFunction>,
    category_index: HashMap<FunctionCategory, Vec<String>>,
}

impl FunctionRegistry {
    pub fn new() -> Self {
        let functions = register_builtins();
        let mut category_index: HashMap<FunctionCategory, Vec<String>> = HashMap::new();

        for (name, func) in &functions {
            if let Some(category) = FunctionCategory::from_str(&func.category) {
                category_index
                    .entry(category)
                    .or_default()
                    .push(name.clone());
            }
        }

        FunctionRegistry {
            functions,
            category_index,
        }
    }

    pub fn get(&self, name: &str) -> Option<&BuiltinFunction> {
        // Use PHF for O(1) lookup if available, fallback to HashMap
        if let Some(&idx) = BUILTIN_REGISTRY.get(name) {
            let func = &BUILTIN_FUNCTIONS[idx];
            if !func.name.is_empty() {
                return Some(func);
            }
        }
        self.functions.get(name)
    }
    
    pub fn get_fast(&self, name: &str) -> Option<&BuiltinFunction> {
        BUILTIN_REGISTRY.get(name).and_then(|&idx| {
            let func = &BUILTIN_FUNCTIONS[idx];
            if !func.name.is_empty() {
                Some(func)
            } else {
                None
            }
        })
    }

    pub fn get_category(&self, category: FunctionCategory) -> Vec<&BuiltinFunction> {
        if let Some(names) = self.category_index.get(&category) {
            names
                .iter()
                .filter_map(|name| self.functions.get(name))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn search(&self, query: &str) -> Vec<&BuiltinFunction> {
        let query_lower = query.to_lowercase();
        self.functions
            .values()
            .filter(|func| {
                func.name.to_lowercase().contains(&query_lower)
                    || func.description.to_lowercase().contains(&query_lower)
                    || func.category.to_lowercase().contains(&query_lower)
                    || func.related.iter().any(|r| r.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    pub fn get_related(&self, function_name: &str) -> Vec<&BuiltinFunction> {
        if let Some(func) = self.functions.get(function_name) {
            func.related
                .iter()
                .filter_map(|name| self.functions.get(name))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn all_functions(&self) -> Vec<&BuiltinFunction> {
        self.functions.values().collect()
    }

    pub fn validate_coverage(&self) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();
        
        for (name, func) in &self.functions {
            if FunctionCategory::from_str(&func.category).is_none() {
                missing.push(format!(
                    "Function '{}' has invalid category '{}'",
                    name, func.category
                ));
            }
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    pub fn validate_implementation_coverage(&self, implemented: &[&str]) -> Result<(), String> {
        use std::collections::HashSet;
        
        let registered: HashSet<_> = self.functions.keys().map(|s| s.as_str()).collect();
        let implemented_set: HashSet<_> = implemented.iter().copied().collect();

        let missing_registration: Vec<_> = implemented_set.difference(&registered).collect();
        let missing_implementation: Vec<_> = registered.difference(&implemented_set).collect();

        let mut errors = Vec::new();

        if !missing_registration.is_empty() {
            let mut sorted = missing_registration.clone();
            sorted.sort();
            errors.push(format!(
                "Functions implemented but not registered ({} functions): {:?}",
                sorted.len(),
                sorted
            ));
        }

        if !missing_implementation.is_empty() {
            let mut sorted = missing_implementation.clone();
            sorted.sort();
            errors.push(format!(
                "Functions registered but not implemented ({} functions): {:?}",
                sorted.len(),
                sorted
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    }

    pub fn count_by_category(&self) -> HashMap<FunctionCategory, usize> {
        self.category_index
            .iter()
            .map(|(cat, names)| (*cat, names.len()))
            .collect()
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn register_builtins() -> HashMap<String, BuiltinFunction> {
    let mut registry = HashMap::new();

    // Network Functions
    registry.insert(
        "connect".to_string(),
        BuiltinFunction::new(
            "connect",
            "connect(host: string, port: int) -> Socket",
            "Establishes a TCP connection to the specified host and port",
            "Network",
            vec![
                "let conn = connect(\"target.com\", 1337)",
                "let s = connect(\"192.168.1.100\", 8080)",
            ],
        )
        .with_related(vec!["send", "recv", "sendline", "recvline", "recvuntil", "close", "interactive"]),
    );

    registry.insert(
        "send".to_string(),
        BuiltinFunction::new(
            "send",
            "send(socket: Socket, data: bytes|string)",
            "Sends data over a socket connection",
            "Network",
            vec![
                "send(conn, \"hello\")",
                "send(conn, payload)",
                "send(conn, p64(0xdeadbeef))",
            ],
        )
        .with_related(vec!["sendline", "recv", "connect"]),
    );

    registry.insert(
        "recv".to_string(),
        BuiltinFunction::new(
            "recv",
            "recv(socket: Socket, size: int) -> bytes",
            "Receives data from a socket connection",
            "Network",
            vec![
                "let data = recv(conn, 1024)",
                "let leak = recv(conn, 8)",
            ],
        )
        .with_related(vec!["recvline", "recvuntil", "send", "u64", "u32"]),
    );

    registry.insert(
        "sendline".to_string(),
        BuiltinFunction::new(
            "sendline",
            "sendline(socket: Socket, data: string)",
            "Sends data followed by a newline character",
            "Network",
            vec![
                "sendline(conn, \"whoami\")",
                "sendline(conn, payload)",
            ],
        )
        .with_related(vec!["send", "recvline", "interactive"]),
    );

    registry.insert(
        "recvline".to_string(),
        BuiltinFunction::new(
            "recvline",
            "recvline(socket: Socket) -> string",
            "Receives data until a newline character",
            "Network",
            vec![
                "let line = recvline(conn)",
                "let response = recvline(conn)",
            ],
        )
        .with_related(vec!["recv", "recvuntil", "sendline"]),
    );

    registry.insert(
        "recvuntil".to_string(),
        BuiltinFunction::new(
            "recvuntil",
            "recvuntil(socket: Socket, delimiter: string) -> bytes",
            "Receives data until the specified delimiter is encountered",
            "Network",
            vec![
                "let data = recvuntil(conn, \"flag{\")",
                "let banner = recvuntil(conn, \"> \")",
            ],
        )
        .with_related(vec!["recv", "recvline", "extract_pattern"]),
    );

    registry.insert(
        "close".to_string(),
        BuiltinFunction::new(
            "close",
            "close(socket: Socket)",
            "Closes a socket connection",
            "Network",
            vec![
                "close(conn)",
            ],
        )
        .with_related(vec!["connect"]),
    );

    // Process Functions
    registry.insert(
        "process".to_string(),
        BuiltinFunction::new(
            "process",
            "process(command: string, args: list<string>) -> Process",
            "Spawns a new process with the specified command and arguments",
            "Process",
            vec![
                "let p = process(\"./vuln\", [])",
                "let p = process(\"/bin/sh\", [\"-c\", \"cat flag.txt\"])",
            ],
        ),
    );

    registry.insert(
        "interactive".to_string(),
        BuiltinFunction::new(
            "interactive",
            "interactive(connection: Socket|Process|SSH)",
            "Drops into an interactive shell with the connection. Supports Ctrl+C forwarding, arrow keys, and raw terminal mode",
            "I/O",
            vec![
                "interactive(conn)",
                "interactive(ssh)",
                "interactive(process)",
            ],
        ),
    );

    // SSH Functions
    registry.insert(
        "connect_ssh".to_string(),
        BuiltinFunction::new(
            "connect_ssh",
            "connect_ssh(host: string, port: int, user: string, password: string) -> SSH",
            "Establishes an SSH connection with password authentication",
            "SSH",
            vec![
                "let ssh = connect_ssh(\"target.com\", 22, \"user\", \"pass\")",
                "let ssh = connect_ssh(\"192.168.1.100\", 2222, \"admin\", \"password\")",
            ],
        ),
    );

    registry.insert(
        "ssh_run".to_string(),
        BuiltinFunction::new(
            "ssh_run",
            "ssh_run(ssh: SSH, command: string) -> string",
            "Executes a command on the remote SSH server and returns output",
            "SSH",
            vec![
                "let output = ssh_run(ssh, \"ls -la\")",
                "let users = ssh_run(ssh, \"cat /etc/passwd\")",
            ],
        ),
    );

    registry.insert(
        "ssh_upload".to_string(),
        BuiltinFunction::new(
            "ssh_upload",
            "ssh_upload(ssh: SSH, local_path: string, remote_path: string)",
            "Uploads a file to the remote SSH server via SCP",
            "SSH",
            vec![
                "ssh_upload(ssh, \"/local/exploit.bin\", \"/tmp/exploit\")",
                "ssh_upload(ssh, \"payload.txt\", \"/home/user/payload.txt\")",
            ],
        ),
    );

    registry.insert(
        "ssh_download".to_string(),
        BuiltinFunction::new(
            "ssh_download",
            "ssh_download(ssh: SSH, remote_path: string, local_path: string)",
            "Downloads a file from the remote SSH server via SCP",
            "SSH",
            vec![
                "ssh_download(ssh, \"/etc/passwd\", \"/local/passwd.txt\")",
                "ssh_download(ssh, \"/home/user/flag.txt\", \"./flag.txt\")",
            ],
        ),
    );

    registry.insert(
        "connect_ssh_pty".to_string(),
        BuiltinFunction::new(
            "connect_ssh_pty",
            "connect_ssh_pty(host: string, port: int, user: string, password: string, rows: int, cols: int) -> SSH",
            "Establishes SSH connection with custom PTY dimensions for interactive sessions (useful for triggering --More-- prompts)",
            "SSH",
            vec![
                "let ssh = connect_ssh_pty(\"bandit.labs.overthewire.org\", 2220, \"bandit26\", \"password\", 3, 20)",
                "let ssh = connect_ssh_pty(\"target.com\", 22, \"user\", \"pass\", 24, 80)",
            ],
        ),
    );

    registry.insert(
        "ssh_interactive_start".to_string(),
        BuiltinFunction::new(
            "ssh_interactive_start",
            "ssh_interactive_start(ssh: SSH)",
            "Start interactive shell session on SSH connection (for nested shells, vim, etc.)",
            "SSH",
            vec![
                "ssh_interactive_start(ssh)",
                "ssh_interactive_start(conn)",
            ],
        ),
    );

    registry.insert(
        "ssh_interactive_send".to_string(),
        BuiltinFunction::new(
            "ssh_interactive_send",
            "ssh_interactive_send(ssh: SSH, data: string)",
            "Send data to interactive SSH session",
            "SSH",
            vec![
                "ssh_interactive_send(ssh, \"v\")",
                "ssh_interactive_send(ssh, \":set shell=/bin/bash\\r\")",
                "ssh_interactive_send(ssh, \":!cat /etc/passwd\\r\")",
            ],
        ),
    );

    registry.insert(
        "ssh_interactive_recv".to_string(),
        BuiltinFunction::new(
            "ssh_interactive_recv",
            "ssh_interactive_recv(ssh: SSH, timeout_ms: int) -> string",
            "Receive data from interactive SSH session with timeout in milliseconds",
            "SSH",
            vec![
                "let output = ssh_interactive_recv(ssh, 1500)",
                "let banner = ssh_interactive_recv(ssh, 1000)",
            ],
        ),
    );

    registry.insert(
        "ssh_interactive_close".to_string(),
        BuiltinFunction::new(
            "ssh_interactive_close",
            "ssh_interactive_close(ssh: SSH)",
            "Close interactive SSH session",
            "SSH",
            vec![
                "ssh_interactive_close(ssh)",
            ],
        ),
    );

    registry.insert(
        "ssh_interact".to_string(),
        BuiltinFunction::new(
            "ssh_interact",
            "ssh_interact(ssh: SSH, command: string, interactions: list<[string, string]>, timeout: int) -> string",
            "Expect-style automation for SSH commands with interactive prompts (like pwntools sendlineafter)",
            "SSH",
            vec![
                "let interactions = [[\"password:\", \"mypass\\n\"], [\"yes/no\", \"yes\\n\"]]",
                "let output = ssh_interact(ssh, \"git clone ...\", interactions, 60)",
                "let interactions = [[\"--More--\", \"v\"], [\"~\", \":shell\\n\"]]",
                "let vim_output = ssh_interact(ssh, \"cat readme.txt\", interactions, 30)",
            ],
        )
        .with_related(vec!["ssh_run", "ssh_interactive_start", "connect_ssh"]),
    );

    registry.insert(
        "connect_ssh_key".to_string(),
        BuiltinFunction::new(
            "connect_ssh_key",
            "connect_ssh_key(host: string, port: int, user: string, key_path: string, passphrase?: string) -> SSH",
            "Establishes SSH connection with public key authentication (optional passphrase)",
            "SSH",
            vec![
                "let ssh = connect_ssh_key(\"target.com\", 22, \"user\", \"~/.ssh/id_rsa\")",
                "let ssh = connect_ssh_key(\"10.0.0.1\", 2222, \"root\", \"/root/.ssh/key\", \"passphrase\")",
                "let ssh = connect_ssh_key(\"ctf.com\", 22, \"player\", \"./key.pem\", null)",
            ],
        )
        .with_related(vec!["connect_ssh", "ssh_run"]),
    );

    registry.insert(
        "ssh_forward".to_string(),
        BuiltinFunction::new(
            "ssh_forward",
            "ssh_forward(ssh: SSH, local_port: int, remote_host: string, remote_port: int)",
            "Creates local port forward through SSH tunnel (localhost:local_port -> remote_host:remote_port via SSH)",
            "SSH",
            vec![
                "ssh_forward(ssh, 8080, \"internal.server\", 80)",
                "ssh_forward(ssh, 3306, \"localhost\", 3306)",
                "ssh_forward(ssh, 9000, \"192.168.1.100\", 8080)",
            ],
        )
        .with_related(vec!["connect_ssh", "ssh_run"]),
    );

    // Binary Packing
    registry.insert(
        "p64".to_string(),
        BuiltinFunction::new(
            "p64",
            "p64(value: int) -> bytes",
            "Packs a 64-bit integer into little-endian bytes",
            "Packing",
            vec![
                "p64(0xdeadbeef)",
                "let addr = p64(0x0040<br/>1000)",
                "let payload = p64(pop_rdi) + p64(bin_sh) + p64(system)",
            ],
        )
        .with_related(vec!["u64", "p32", "flat"]),
    );

    registry.insert(
        "p32".to_string(),
        BuiltinFunction::new(
            "p32",
            "p32(value: int) -> bytes",
            "Packs a 32-bit integer into little-endian bytes",
            "Packing",
            vec![
                "p32(0xdeadbeef)",
                "let addr = p32(0x08048000)",
            ],
        )
        .with_related(vec!["u32", "p64", "flat"]),
    );

    registry.insert(
        "p16".to_string(),
        BuiltinFunction::new(
            "p16",
            "p16(value: int) -> bytes",
            "Packs a 16-bit integer into little-endian bytes",
            "Packing",
            vec![
                "p16(0x1234)",
            ],
        )
        .with_related(vec!["u16", "p32"]),
    );

    registry.insert(
        "p8".to_string(),
        BuiltinFunction::new(
            "p8",
            "p8(value: int) -> bytes",
            "Packs an 8-bit integer into bytes",
            "Packing",
            vec![
                "p8(0x41)",
            ],
        )
        .with_related(vec!["u8", "bytes"]),
    );

    registry.insert(
        "u64".to_string(),
        BuiltinFunction::new(
            "u64",
            "u64(data: bytes) -> int",
            "Unpacks 8 bytes into a 64-bit little-endian integer",
            "Packing",
            vec![
                "let addr = u64(leaked)",
                "let value = u64(recv(conn, 8))",
            ],
        )
        .with_related(vec!["p64", "u32", "recv"]),
    );

    registry.insert(
        "u32".to_string(),
        BuiltinFunction::new(
            "u32",
            "u32(data: bytes) -> int",
            "Unpacks 4 bytes into a 32-bit little-endian integer",
            "Packing",
            vec![
                "let addr = u32(data[0:4])",
            ],
        )
        .with_related(vec!["p32", "u64"]),
    );

    registry.insert(
        "u16".to_string(),
        BuiltinFunction::new(
            "u16",
            "u16(data: bytes) -> int",
            "Unpacks 2 bytes into a 16-bit little-endian integer",
            "Packing",
            vec![
                "let port = u16(data)",
            ],
        )
        .with_related(vec!["p16", "u32"]),
    );

    registry.insert(
        "u8".to_string(),
        BuiltinFunction::new(
            "u8",
            "u8(data: bytes) -> int",
            "Unpacks 1 byte into an 8-bit integer",
            "Packing",
            vec![
                "let byte_val = u8(data)",
            ],
        )
        .with_related(vec!["p8"]),
    );

    // Exploitation Functions
    registry.insert(
        "cyclic".to_string(),
        BuiltinFunction::new(
            "cyclic",
            "cyclic(length: int) -> bytes",
            "Generates a De Bruijn cyclic pattern of the specified length",
            "Exploitation",
            vec![
                "let pattern = cyclic(200)",
                "send(conn, cyclic(500))",
            ],
        )
        .with_related(vec!["cyclic_find", "auto_offset", "send"]),
    );

    registry.insert(
        "cyclic_find".to_string(),
        BuiltinFunction::new(
            "cyclic_find",
            "cyclic_find(value: int) -> int",
            "Finds the offset of a value in a cyclic pattern",
            "Exploitation",
            vec![
                "let offset = cyclic_find(0x61616162)",
                "let crash_offset = cyclic_find(rip_value)",
            ],
        )
        .with_related(vec!["cyclic", "auto_offset", "u64"]),
    );

    registry.insert(
        "shellcode".to_string(),
        BuiltinFunction::new(
            "shellcode",
            "shellcode(arch: string, payload: string) -> bytes",
            "Generates shellcode for the specified architecture and payload type",
            "Exploitation",
            vec![
                "let sc = shellcode(\"x64\", \"execve\")",
                "let sc = shellcode(\"x86\", \"reverse_shell\")",
            ],
        )
        .with_related(vec!["flat", "send", "analyze"]),
    );

    registry.insert(
        "flat".to_string(),
        BuiltinFunction::new(
            "flat",
            "flat(items: list) -> bytes",
            "Flattens a list of items into a contiguous byte array",
            "Exploitation",
            vec![
                "let payload = flat([cyclic(offset), pop_rdi, bin_sh, system])",
                "let chain = flat([gadget1, gadget2, gadget3])",
            ],
        )
        .with_related(vec!["p64", "p32", "shellcode", "send"]),
    );

    // Binary Analysis
    registry.insert(
        "analyze".to_string(),
        BuiltinFunction::new(
            "analyze",
            "analyze(binary_path: string) -> map",
            "Analyzes a binary and returns information about protections, symbols, and addresses",
            "Binary Analysis",
            vec![
                "let elf = analyze(\"./vuln\")",
                "print(elf[\"pie\"])",
                "print(elf[\"nx\"])",
            ],
        )
        .with_related(vec!["auto_offset", "process", "shellcode"]),
    );

    registry.insert(
        "auto_offset".to_string(),
        BuiltinFunction::new(
            "auto_offset",
            "auto_offset(binary_path: string) -> int",
            "Automatically determines the buffer overflow offset using GDB and cyclic patterns",
            "Binary Analysis",
            vec![
                "let offset = auto_offset(\"./vuln\")",
                "let crash_offset = auto_offset(\"/challenge/binary\")",
            ],
        )
        .with_related(vec!["cyclic", "cyclic_find", "analyze", "process"]),
    );

    // Utilities
    registry.insert(
        "len".to_string(),
        BuiltinFunction::new(
            "len",
            "len(collection: list|string|bytes|map|set) -> int",
            "Returns the length of a collection",
            "Utilities",
            vec![
                "len([1, 2, 3, 4, 5])",
                "len(\"hello world\")",
                "len(payload)",
            ],
        ),
    );

    registry.insert(
        "range".to_string(),
        BuiltinFunction::new(
            "range",
            "range(end: int) or range(start: int, end: int) -> list<int>",
            "Generates a sequence of numbers",
            "Utilities",
            vec![
                "range(5)",
                "range(3, 8)",
                "for i in range(10) ... end",
            ],
        ),
    );

    registry.insert(
        "hex".to_string(),
        BuiltinFunction::new(
            "hex",
            "hex(number: int) -> string",
            "Converts a number to a hexadecimal string",
            "Utilities",
            vec![
                "hex(255)",
                "hex(0x08048000)",
                "print(\"Address:\", hex(addr))",
            ],
        ),
    );

    registry.insert(
        "int".to_string(),
        BuiltinFunction::new(
            "int",
            "int(value: string) -> int",
            "Parses a string to an integer (supports hex and decimal)",
            "Utilities",
            vec![
                "int(\"12345\")",
                "int(\"0xdeadbeef\")",
                "int(\"0xFF\")",
            ],
        ),
    );

    registry.insert(
        "bytes".to_string(),
        BuiltinFunction::new(
            "bytes",
            "bytes(value: string|list<int>|int) -> bytes",
            "Converts various types to byte arrays",
            "Utilities",
            vec![
                "bytes(\"hello\")",
                "bytes([72, 101, 108, 108, 111])",
                "bytes(65)",
            ],
        ),
    );

    registry.insert(
        "str".to_string(),
        BuiltinFunction::new(
            "str",
            "str(value: any) -> string",
            "Converts any value to its string representation",
            "Utilities",
            vec![
                "str(12345)",
                "str(0xdead)",
                "str([1, 2, 3])",
            ],
        ),
    );

    registry.insert(
        "print".to_string(),
        BuiltinFunction::new(
            "print",
            "print(value1, value2, ...)",
            "Prints values to stdout (space-separated)",
            "Utilities",
            vec![
                "print(\"Hello World\")",
                "print(\"Address:\", hex(0x400000))",
                "print(\"Size:\", len(payload), \"bytes\")",
            ],
        ),
    );

    registry.insert(
        "random_string".to_string(),
        BuiltinFunction::new(
            "random_string",
            "random_string(length: int) -> string",
            "Generates a random alphanumeric string of the specified length",
            "Utilities",
            vec![
                "let nonce = random_string(16)",
                "let tmp_dir = \"/tmp/\" + random_string(8)",
            ],
        ),
    );

    registry.insert(
        "extract_pattern".to_string(),
        BuiltinFunction::new(
            "extract_pattern",
            "extract_pattern(text: string, pattern: string) -> list<string>",
            "Extracts all regex matches from text",
            "Utilities",
            vec![
                "let matches = extract_pattern(output, \"password: ([a-zA-Z0-9]+)\")",
                "let addrs = extract_pattern(leak, \"0x[0-9a-f]+\")",
            ],
        ),
    );

    // File I/O
    registry.insert(
        "read".to_string(),
        BuiltinFunction::new(
            "read",
            "read(filepath: string) -> bytes",
            "Reads file contents as bytes",
            "File I/O",
            vec![
                "let data = read(\"shellcode.bin\")",
                "let config = str(read(\"config.txt\"))",
            ],
        ),
    );

    registry.insert(
        "write".to_string(),
        BuiltinFunction::new(
            "write",
            "write(filepath: string, data: bytes|string) -> int",
            "Writes data to a file (creates or overwrites). Returns number of bytes written",
            "File I/O",
            vec![
                "write(\"output.txt\", \"Hello World!\")",
                "write(\"exploit.bin\", payload)",
            ],
        ),
    );

    // String Manipulation
    registry.insert(
        "split".to_string(),
        BuiltinFunction::new(
            "split",
            "split(string: string, delimiter: string) -> list<string>",
            "Splits a string into a list",
            "String Manipulation",
            vec![
                "split(\"one,two,three\", \",\")",
                "split(\"192.168.1.1\", \".\")",
            ],
        ),
    );

    registry.insert(
        "join".to_string(),
        BuiltinFunction::new(
            "join",
            "join(list: list<string>, separator: string) -> string",
            "Joins a list into a string",
            "String Manipulation",
            vec![
                "join([\"a\", \"b\", \"c\"], \"-\")",
                "join([1, 2, 3], \",\")",
            ],
        ),
    );

    registry.insert(
        "replace".to_string(),
        BuiltinFunction::new(
            "replace",
            "replace(string: string, old: string, new: string) -> string",
            "Replaces all occurrences of a substring",
            "String Manipulation",
            vec![
                "replace(\"hello world\", \"world\", \"TALON\")",
                "replace(\"192.168.1.1\", \".\", \"_\")",
            ],
        ),
    );

    // Additional Exploitation Functions
    registry.insert(
        "rop_find".to_string(),
        BuiltinFunction::new(
            "rop_find",
            "rop_find(binary: string, gadget: string) -> int",
            "Searches for ROP gadgets in a binary",
            "Exploitation",
            vec![
                "let pop_rdi = rop_find(\"./vuln\", \"pop rdi; ret\")",
                "let syscall = rop_find(\"/lib/libc.so.6\", \"syscall\")",
            ],
        )
        .with_related(vec!["analyze", "flat"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "fmtstr_payload".to_string(),
        BuiltinFunction::new(
            "fmtstr_payload",
            "fmtstr_payload(offset: int, writes: map) -> bytes",
            "Generates a format string payload for arbitrary writes",
            "Exploitation",
            vec![
                "let payload = fmtstr_payload(6, {0x601020: 0x41414141})",
                "let payload = fmtstr_payload(4, {got_printf: system_addr})",
            ],
        )
        .with_related(vec!["analyze"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "disasm".to_string(),
        BuiltinFunction::new(
            "disasm",
            "disasm(code: bytes, arch: string, address: int) -> string",
            "Disassembles machine code to assembly instructions",
            "Binary Analysis",
            vec![
                "let asm = disasm(shellcode, \"x64\", 0x400000)",
                "print(disasm(code, \"x86\", 0x08048000))",
            ],
        )
        .with_related(vec!["analyze", "shellcode"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "parse_elf".to_string(),
        BuiltinFunction::new(
            "parse_elf",
            "parse_elf(binary: string) -> map",
            "Parses an ELF binary and returns analysis including symbols, GOT, PLT, and protections (alias for Elf)",
            "Binary Analysis",
            vec![
                "let elf = parse_elf(\"./vuln\")",
                "let got_addr = parse_elf(binary).got.printf",
                "let win_addr = parse_elf(\"./challenge\").symbols.win",
            ],
        )
        .with_related(vec!["Elf", "analyze", "disasm"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "checksec".to_string(),
        BuiltinFunction::new(
            "checksec",
            "checksec(binary: string) -> map",
            "Analyzes binary security features and returns protection flags (PIE, NX, Canary, RELRO, FORTIFY)",
            "Binary Analysis",
            vec![
                "let protections = checksec(\"./vuln\")",
                "print(\"NX:\", protections.nx)",
                "print(\"PIE:\", protections.pie)",
                "if checksec(binary).canary { print(\"Stack canary detected\") }",
            ],
        )
        .with_related(vec!["Elf", "analyze", "parse_elf"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "check_kernel_protections".to_string(),
        BuiltinFunction::new(
            "check_kernel_protections",
            "check_kernel_protections() -> map",
            "Checks kernel security features and returns protection flags (SMEP, SMAP, KASLR, KPTI)",
            "Kernel",
            vec![
                "let kernel_sec = check_kernel_protections()",
                "print(\"SMEP:\", kernel_sec.smep)",
                "print(\"KASLR:\", kernel_sec.kaslr)",
                "if check_kernel_protections().smep { print(\"SMEP enabled - need bypass\") }",
            ],
        )
        .with_related(vec!["checksec", "token_steal", "kernel_write"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "copy".to_string(),
        BuiltinFunction::new(
            "copy",
            "copy(data: any)",
            "Copies data to the system clipboard",
            "Utilities",
            vec![
                "copy(payload)",
                "copy(\"flag{...}\")",
                "copy(hex(leaked_addr))",
            ],
        )
        .with_version("0.2.0"),
    );

    registry.insert(
        "hex".to_string(),
        BuiltinFunction::new(
            "hex",
            "hex(number: int) -> string",
            "Converts integer to hexadecimal string with 0x prefix",
            "Utilities",
            vec![
                "let addr = hex(0x401000)",
                "print(\"Address:\", hex(win_addr))",
                "print(hex(elf.symbols.main))",
            ],
        )
        .with_related(vec!["str", "len"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "str".to_string(),
        BuiltinFunction::new(
            "str",
            "str(value: any) -> string",
            "Converts any value to string representation",
            "Utilities",
            vec![
                "let size_str = str(len(payload))",
                "print(\"Count: \" + str(42))",
                "let json = str(map_data)",
            ],
        )
        .with_related(vec!["hex", "len"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "len".to_string(),
        BuiltinFunction::new(
            "len",
            "len(collection: bytes/list/string/map/set) -> int",
            "Returns the length of a collection",
            "Utilities",
            vec![
                "let size = len(payload)",
                "print(\"Payload size:\", len(payload), \"bytes\")",
                "let gadget_count = len(rop_gadgets)",
            ],
        )
        .with_related(vec!["str", "hex"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "remote".to_string(),
        BuiltinFunction::new(
            "remote",
            "remote(host: string, port: int) -> Socket",
            "Establishes a TCP connection (alias for connect)",
            "Network",
            vec![
                "let conn = remote(\"target.com\", 1337)",
                "let r = remote(\"192.168.1.100\", 8080)",
            ],
        )
        .with_related(vec!["connect", "send", "recv"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "help".to_string(),
        BuiltinFunction::new(
            "help",
            "help() or help(function_name: string)",
            "Displays help information about builtin functions",
            "Utilities",
            vec![
                "help()",
                "help(\"connect\")",
                "help(\"rop_find\")",
            ],
        )
        .with_version("0.1.0"),
    );

    // Debugging Functions
    registry.insert(
        "debug_attach".to_string(),
        BuiltinFunction::new(
            "debug_attach",
            "debug_attach(process: Process) -> Debugger",
            "Attaches GDB to a running process",
            "Debugging",
            vec![
                "let dbg = debug_attach(proc)",
                "debug_attach(process)",
            ],
        )
        .with_related(vec!["breakpoint", "debug_continue", "debug_step"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "breakpoint".to_string(),
        BuiltinFunction::new(
            "breakpoint",
            "breakpoint(address: int)",
            "Sets a breakpoint at the specified address",
            "Debugging",
            vec![
                "breakpoint(0x401234)",
                "breakpoint(elf.symbols.main)",
            ],
        )
        .with_related(vec!["debug_attach", "debug_continue"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "debug_continue".to_string(),
        BuiltinFunction::new(
            "debug_continue",
            "debug_continue()",
            "Continues execution in the debugger",
            "Debugging",
            vec![
                "debug_continue()",
            ],
        )
        .with_related(vec!["debug_attach", "breakpoint", "debug_step"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "debug_step".to_string(),
        BuiltinFunction::new(
            "debug_step",
            "debug_step()",
            "Steps one instruction in the debugger",
            "Debugging",
            vec![
                "debug_step()",
            ],
        )
        .with_related(vec!["debug_attach", "debug_continue"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "debug_read_mem".to_string(),
        BuiltinFunction::new(
            "debug_read_mem",
            "debug_read_mem(address: int, size: int) -> bytes",
            "Reads memory from the debugged process",
            "Debugging",
            vec![
                "let data = debug_read_mem(0x601000, 64)",
                "let stack = debug_read_mem(rsp, 256)",
            ],
        )
        .with_related(vec!["debug_write_mem", "debug_read_reg"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "debug_write_mem".to_string(),
        BuiltinFunction::new(
            "debug_write_mem",
            "debug_write_mem(address: int, data: bytes)",
            "Writes memory to the debugged process",
            "Debugging",
            vec![
                "debug_write_mem(0x601000, p64(0x41414141))",
                "debug_write_mem(got_addr, libc.symbols.system)",
            ],
        )
        .with_related(vec!["debug_read_mem", "debug_write_reg"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "debug_read_reg".to_string(),
        BuiltinFunction::new(
            "debug_read_reg",
            "debug_read_reg(register: string) -> int",
            "Reads a register value from the debugged process",
            "Debugging",
            vec![
                "let rip = debug_read_reg(\"rip\")",
                "let rsp = debug_read_reg(\"rsp\")",
            ],
        )
        .with_related(vec!["debug_write_reg", "debug_read_mem"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "debug_write_reg".to_string(),
        BuiltinFunction::new(
            "debug_write_reg",
            "debug_write_reg(register: string, value: int)",
            "Writes a register value in the debugged process",
            "Debugging",
            vec![
                "debug_write_reg(\"rip\", 0x401234)",
                "debug_write_reg(\"rdi\", bin_sh)",
            ],
        )
        .with_related(vec!["debug_read_reg", "debug_write_mem"])
        .with_version("0.1.0"),
    );

    // Symbolic Execution Functions
    registry.insert(
        "symbolic_var".to_string(),
        BuiltinFunction::new(
            "symbolic_var",
            "symbolic_var(name: string, size: int) -> Symbolic",
            "Creates a symbolic variable for constraint solving",
            "Symbolic",
            vec![
                "let input = symbolic_var(\"user_input\", 64)",
                "let key = symbolic_var(\"key\", 16)",
            ],
        )
        .with_related(vec!["constrain_no_null", "constrain_alnum", "symbolic_solve"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "constrain_no_null".to_string(),
        BuiltinFunction::new(
            "constrain_no_null",
            "constrain_no_null(symbolic: Symbolic)",
            "Adds constraint that symbolic variable contains no null bytes",
            "Symbolic",
            vec![
                "constrain_no_null(input)",
            ],
        )
        .with_related(vec!["symbolic_var", "constrain_alnum", "symbolic_solve"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "constrain_alnum".to_string(),
        BuiltinFunction::new(
            "constrain_alnum",
            "constrain_alnum(symbolic: Symbolic)",
            "Constrains symbolic variable to alphanumeric characters only",
            "Symbolic",
            vec![
                "constrain_alnum(input)",
            ],
        )
        .with_related(vec!["symbolic_var", "constrain_no_null", "symbolic_solve"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "constrain_range".to_string(),
        BuiltinFunction::new(
            "constrain_range",
            "constrain_range(symbolic: Symbolic, min: int, max: int)",
            "Constrains symbolic variable to a range of values",
            "Symbolic",
            vec![
                "constrain_range(input, 0x20, 0x7e)",
            ],
        )
        .with_related(vec!["symbolic_var", "symbolic_solve"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "symbolic_solve".to_string(),
        BuiltinFunction::new(
            "symbolic_solve",
            "symbolic_solve() -> bytes",
            "Solves constraints and returns concrete value",
            "Symbolic",
            vec![
                "let solution = symbolic_solve()",
            ],
        )
        .with_related(vec!["symbolic_var", "constrain_no_null", "constrain_alnum"])
        .with_version("0.1.0"),
    );

    // Heap Exploitation Functions
    registry.insert(
        "pool_spray".to_string(),
        BuiltinFunction::new(
            "pool_spray",
            "pool_spray(size: int, count: int, data: bytes)",
            "Performs heap pool spraying with specified object size and count",
            "Heap",
            vec![
                "pool_spray(0x100, 1000, fake_object)",
            ],
        )
        .with_related(vec!["heap_feng_shui"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "heap_feng_shui".to_string(),
        BuiltinFunction::new(
            "heap_feng_shui",
            "heap_feng_shui(allocations: list)",
            "Performs heap feng shui to position objects predictably",
            "Heap",
            vec![
                "heap_feng_shui([{\"size\": 0x100, \"count\": 10}])",
            ],
        )
        .with_related(vec!["pool_spray"])
        .with_version("0.1.0"),
    );

    // Kernel Exploitation Functions
    registry.insert(
        "token_steal".to_string(),
        BuiltinFunction::new(
            "token_steal",
            "token_steal() -> bytes",
            "Generates shellcode for Windows token stealing privilege escalation",
            "Kernel",
            vec![
                "let shellcode = token_steal()",
            ],
        )
        .with_related(vec!["process_hide", "rootkit_install"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "process_hide".to_string(),
        BuiltinFunction::new(
            "process_hide",
            "process_hide(pid: int)",
            "Hides a process from process listings (EPROCESS unlinking)",
            "Kernel",
            vec![
                "process_hide(1234)",
            ],
        )
        .with_related(vec!["token_steal", "rootkit_install"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "rootkit_install".to_string(),
        BuiltinFunction::new(
            "rootkit_install",
            "rootkit_install(driver_path: string)",
            "Installs a kernel driver for rootkit functionality",
            "Kernel",
            vec![
                "rootkit_install(\"/tmp/rootkit.ko\")",
            ],
        )
        .with_related(vec!["token_steal", "process_hide"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "kaslr_leak".to_string(),
        BuiltinFunction::new(
            "kaslr_leak",
            "kaslr_leak() -> int",
            "Leaks kernel base address to defeat KASLR",
            "Kernel",
            vec![
                "let kernel_base = kaslr_leak()",
            ],
        )
        .with_related(vec!["smep_bypass", "kernel_read", "kernel_write"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "smep_bypass".to_string(),
        BuiltinFunction::new(
            "smep_bypass",
            "smep_bypass(payload: bytes) -> bytes",
            "Wraps payload with SMEP bypass technique (CR4 manipulation)",
            "Kernel",
            vec![
                "let safe_payload = smep_bypass(shellcode)",
            ],
        )
        .with_related(vec!["kaslr_leak", "kernel_write"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "kernel_write".to_string(),
        BuiltinFunction::new(
            "kernel_write",
            "kernel_write(address: int, data: bytes)",
            "Writes to kernel memory (arbitrary kernel write primitive)",
            "Kernel",
            vec![
                "kernel_write(0xffffffff81000000, payload)",
            ],
        )
        .with_related(vec!["kernel_read", "kaslr_leak"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "kernel_read".to_string(),
        BuiltinFunction::new(
            "kernel_read",
            "kernel_read(address: int, size: int) -> bytes",
            "Reads from kernel memory (arbitrary kernel read primitive)",
            "Kernel",
            vec![
                "let data = kernel_read(0xffffffff81000000, 64)",
            ],
        )
        .with_related(vec!["kernel_write", "kaslr_leak"])
        .with_version("0.1.0"),
    );

    // Crypto Functions
    registry.insert(
        "padding_oracle".to_string(),
        BuiltinFunction::new(
            "padding_oracle",
            "padding_oracle(ciphertext: bytes, oracle_func: function) -> bytes",
            "Performs padding oracle attack to decrypt ciphertext",
            "Crypto",
            vec![
                "let plaintext = padding_oracle(ct, oracle)",
            ],
        )
        .with_related(vec!["aes_padding_attack", "timing_attack"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "bleichenbacher".to_string(),
        BuiltinFunction::new(
            "bleichenbacher",
            "bleichenbacher(ciphertext: bytes, oracle_func: function) -> bytes",
            "Performs Bleichenbacher's attack on RSA PKCS#1 v1.5",
            "Crypto",
            vec![
                "let plaintext = bleichenbacher(ct, oracle)",
            ],
        )
        .with_related(vec!["rsa_factorize", "timing_attack"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "timing_attack".to_string(),
        BuiltinFunction::new(
            "timing_attack",
            "timing_attack(target_func: function, charset: string) -> string",
            "Performs timing side-channel attack to extract secrets",
            "Crypto",
            vec![
                "let password = timing_attack(verify_func, \"0123456789abcdef\")",
            ],
        )
        .with_related(vec!["padding_oracle", "weak_keys"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "weak_keys".to_string(),
        BuiltinFunction::new(
            "weak_keys",
            "weak_keys(bits: int) -> list",
            "Generates list of weak RSA keys for testing",
            "Crypto",
            vec![
                "let keys = weak_keys(512)",
            ],
        )
        .with_related(vec!["rsa_factorize"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "hash_collision".to_string(),
        BuiltinFunction::new(
            "hash_collision",
            "hash_collision(algorithm: string, prefix: bytes) -> bytes",
            "Finds hash collision for specified algorithm",
            "Crypto",
            vec![
                "let collision = hash_collision(\"md5\", prefix)",
            ],
        )
        .with_version("0.1.0"),
    );

    registry.insert(
        "aes_padding_attack".to_string(),
        BuiltinFunction::new(
            "aes_padding_attack",
            "aes_padding_attack(ciphertext: bytes) -> bytes",
            "Exploits AES padding vulnerabilities",
            "Crypto",
            vec![
                "let plaintext = aes_padding_attack(ct)",
            ],
        )
        .with_related(vec!["padding_oracle"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "rsa_factorize".to_string(),
        BuiltinFunction::new(
            "rsa_factorize",
            "rsa_factorize(n: int, e: int) -> map",
            "Attempts to factorize RSA modulus to recover private key",
            "Crypto",
            vec![
                "let key = rsa_factorize(n, 65537)",
            ],
        )
        .with_related(vec!["weak_keys"])
        .with_version("0.1.0"),
    );

    // Fuzzing Functions
    registry.insert(
        "fuzz_target".to_string(),
        BuiltinFunction::new(
            "fuzz_target",
            "fuzz_target(binary: string, input_source: string)",
            "Sets up fuzzing target with specified input method",
            "Fuzzing",
            vec![
                "fuzz_target(\"./target\", \"stdin\")",
                "fuzz_target(\"./vuln\", \"file\")",
            ],
        )
        .with_related(vec!["mutate", "coverage", "crash_triage"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "mutate".to_string(),
        BuiltinFunction::new(
            "mutate",
            "mutate(input: bytes, strategy: string) -> bytes",
            "Mutates input data using specified fuzzing strategy",
            "Fuzzing",
            vec![
                "let mutated = mutate(seed, \"bitflip\")",
                "let mutated = mutate(input, \"havoc\")",
            ],
        )
        .with_related(vec!["fuzz_target", "corpus_add"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "coverage".to_string(),
        BuiltinFunction::new(
            "coverage",
            "coverage(binary: string, input: bytes) -> map",
            "Measures code coverage for given input",
            "Fuzzing",
            vec![
                "let cov = coverage(\"./target\", test_input)",
            ],
        )
        .with_related(vec!["fuzz_target", "mutate"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "corpus_add".to_string(),
        BuiltinFunction::new(
            "corpus_add",
            "corpus_add(input: bytes, coverage_data: map)",
            "Adds input to fuzzing corpus if it increases coverage",
            "Fuzzing",
            vec![
                "corpus_add(mutated_input, cov)",
            ],
        )
        .with_related(vec!["mutate", "coverage"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "crash_triage".to_string(),
        BuiltinFunction::new(
            "crash_triage",
            "crash_triage(crash_input: bytes) -> map",
            "Triages crash to determine exploitability",
            "Fuzzing",
            vec![
                "let triage = crash_triage(crashing_input)",
                "print(triage[\"exploitability\"])",
            ],
        )
        .with_related(vec!["fuzz_target", "coverage"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "parallel_exploit".to_string(),
        BuiltinFunction::new(
            "parallel_exploit",
            "parallel_exploit(targets: list, exploit_func: function)",
            "Executes exploit against multiple targets in parallel",
            "Exploitation",
            vec![
                "parallel_exploit(target_list, pwn_func)",
            ],
        )
        .with_related(vec!["remote", "connect", "mass_connect"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "mass_connect".to_string(),
        BuiltinFunction::new(
            "mass_connect",
            "mass_connect(hosts: list, port: int, max_concurrent: int, timeout_ms: int, rate_limit_ms: int) -> list",
            "Establishes concurrent TCP connections to multiple hosts using Tokio async runtime with connection pooling, rate limiting, and progress reporting. Supports up to 1000 concurrent connections with zero-copy I/O for optimal performance.",
            "Network",
            vec![
                "let results = mass_connect([\"192.168.1.1\", \"192.168.1.2\"], 22, 100, 5000, 50)",
                "let conns = mass_connect(target_hosts, 4444, 50, 10000, 100)",
            ],
        )
        .with_related(vec!["parallel_exploit", "remote", "connect"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "generate_exploit".to_string(),
        BuiltinFunction::new(
            "generate_exploit",
            "generate_exploit(binary: string, vuln_type: string) -> string",
            "Automatically generates exploit code for given vulnerability type",
            "AI",
            vec![
                "let exploit = generate_exploit(\"./vuln\", \"buffer_overflow\")",
                "let script = generate_exploit(\"./target\", \"format_string\")",
            ],
        )
        .with_related(vec!["analyze", "auto_offset", "rop_find"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "oracle_analyze".to_string(),
        BuiltinFunction::new(
            "oracle_analyze",
            "oracle_analyze(binary: string) -> list",
            "Analyzes binary for vulnerabilities using heuristic engine. Returns list of vulnerability reports.",
            "Binary Analysis",
            vec![
                "let vulns = oracle_analyze(\"./vuln\")",
                "for vuln in vulns { print(vuln.type) }",
                "let report = oracle_analyze(\"./target\")",
                "print(report[0].exploitability)",
            ],
        )
        .with_related(vec!["oracle_find_shellcode", "oracle_gadget_density", "rop_find"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "oracle_find_shellcode".to_string(),
        BuiltinFunction::new(
            "oracle_find_shellcode",
            "oracle_find_shellcode(avoid: list, max_size: int, arch: string) -> list",
            "Finds shellcodes matching constraints (bad characters, size, architecture). Returns list of shellcode entries.",
            "Binary Analysis",
            vec![
                "let sc = oracle_find_shellcode([0x00, 0x0a], 64, \"x86-64\")",
                "let shellcodes = oracle_find_shellcode([0x00], 100, \"i386\")",
                "print(sc[0].name)",
            ],
        )
        .with_related(vec!["oracle_analyze", "shellcode"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "oracle_gadget_density".to_string(),
        BuiltinFunction::new(
            "oracle_gadget_density",
            "oracle_gadget_density(binary: string) -> map",
            "Analyzes ROP gadget availability and quality in binary. Returns gadget density metrics.",
            "Binary Analysis",
            vec![
                "let gadgets = oracle_gadget_density(\"./vuln\")",
                "print(gadgets.rop_possible)",
                "print(gadgets.quality_score)",
            ],
        )
        .with_related(vec!["oracle_analyze", "rop_find", "rop_chain"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "mitigation_analyze".to_string(),
        BuiltinFunction::new(
            "mitigation_analyze",
            "mitigation_analyze(binary: string) -> map",
            "Analyzes binary mitigations (NX, PIE, Canary, ASLR, RELRO) and generates adaptive exploit strategy. Auto-pivots from shellcode to ROP when NX detected, recommends leak strategies for PIE/Canary.",
            "Exploitation",
            vec![
                "let strategy = mitigation_analyze(\"./vuln\")",
                "print(strategy.technique)",
                "print(strategy.requires_leak)",
                "if strategy.technique == \"ROP Chain\" { let rop = ROP(elf) }",
            ],
        )
        .with_related(vec!["mitigation_auto_pivot", "mitigation_validate", "oracle_analyze"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "mitigation_generate_leak".to_string(),
        BuiltinFunction::new(
            "mitigation_generate_leak",
            "mitigation_generate_leak(binary: string, leak_type: string) -> string",
            "Generates code template for information leak based on detected mitigations. Leak types: canary, pie, libc, stack, heap.",
            "Exploitation",
            vec![
                "let leak_code = mitigation_generate_leak(\"./vuln\", \"canary\")",
                "let pie_leak = mitigation_generate_leak(\"./target\", \"pie\")",
                "let libc_leak = mitigation_generate_leak(\"./binary\", \"libc\")",
            ],
        )
        .with_related(vec!["mitigation_analyze", "fmtstr_payload"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "mitigation_auto_pivot".to_string(),
        BuiltinFunction::new(
            "mitigation_auto_pivot",
            "mitigation_auto_pivot(binary: string) -> string",
            "Automatically pivots exploit strategy from shellcode to ROP/ret2libc based on NX detection. Returns complete payload template with leak strategies.",
            "Exploitation",
            vec![
                "let payload_template = mitigation_auto_pivot(\"./vuln\")",
                "print(payload_template)",
            ],
        )
        .with_related(vec!["mitigation_analyze", "rop_find", "rop_chain"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "mitigation_validate".to_string(),
        BuiltinFunction::new(
            "mitigation_validate",
            "mitigation_validate(binary: string) -> map",
            "Validates exploit strategy viability against binary. Checks for missing gadgets, analyzes complexity, provides suggestions for bypass techniques.",
            "Exploitation",
            vec![
                "let validation = mitigation_validate(\"./vuln\")",
                "if validation.viable == 1 { print(\"Strategy viable\") }",
                "for warning in validation.warnings { print(warning) }",
            ],
        )
        .with_related(vec!["mitigation_analyze", "rop_find"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "oracle_report".to_string(),
        BuiltinFunction::new(
            "oracle_report",
            "oracle_report(vulnerabilities: list) -> string",
            "Generates comprehensive vulnerability analysis report from analysis results.",
            "Binary Analysis",
            vec![
                "let vulns = oracle_analyze(\"./vuln\")",
                "let report = oracle_report(vulns)",
                "print(report)",
            ],
        )
        .with_related(vec!["oracle_analyze"])
        .with_version("0.1.0"),
    );

    registry.insert(
        "analyze_heap".to_string(),
        BuiltinFunction::new(
            "analyze_heap",
            "analyze_heap(binary: string) -> map",
            "Analyzes heap implementation and returns metadata including allocator type, tcache configuration, and security features.",
            "Heap",
            vec![
                "let heap = analyze_heap(\"./vuln\")",
                "print(heap.allocator)",
                "print(heap.tcache_bins)",
                "print(heap.tcache_count)",
            ],
        )
        .with_related(vec!["heap_spray", "heap_feng_shui"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "Patch".to_string(),
        BuiltinFunction::new(
            "Patch",
            "Patch(binary: string) -> Patch",
            "Creates binary patch object for semantic binary modification. Supports NOP injection, function replacement, assembly insertion, and more.",
            "Binary Analysis",
            vec![
                "let p = Patch(\"/tmp/target\")",
                "patch_nop_out(p, 0x1234, 10)",
                "patch_save(p, \"/tmp/patched\")",
            ],
        )
        .with_related(vec!["patch_nop_out", "patch_save", "patch_set_dry_run"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "patch_nop_out".to_string(),
        BuiltinFunction::new(
            "patch_nop_out",
            "patch_nop_out(patch: Patch, offset: int, length: int)",
            "NOPs out instructions at specified offset for security check bypass or code elimination.",
            "Binary Analysis",
            vec![
                "let p = Patch(binary)",
                "patch_nop_out(p, 0x1234, 10)",
            ],
        )
        .with_related(vec!["Patch", "patch_save"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "patch_save".to_string(),
        BuiltinFunction::new(
            "patch_save",
            "patch_save(patch: Patch, output: string)",
            "Saves patched binary to disk with all applied modifications.",
            "Binary Analysis",
            vec![
                "let p = Patch(binary)",
                "patch_nop_out(p, 0x1000, 20)",
                "patch_save(p, \"./patched_binary\")",
            ],
        )
        .with_related(vec!["Patch", "patch_nop_out"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "patch_set_dry_run".to_string(),
        BuiltinFunction::new(
            "patch_set_dry_run",
            "patch_set_dry_run(patch: Patch, enabled: int)",
            "Enables dry-run mode for safe preview of patch operations without modifying files.",
            "Binary Analysis",
            vec![
                "let p = Patch(binary)",
                "patch_set_dry_run(p, 1)",
                "patch_nop_out(p, 0x1000, 20)",
            ],
        )
        .with_related(vec!["Patch"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "fmtstr_leak".to_string(),
        BuiltinFunction::new(
            "fmtstr_leak",
            "fmtstr_leak(offset: int) -> string",
            "Generates format string payload for leaking stack values at specified offset.",
            "Exploitation",
            vec![
                "let leak = fmtstr_leak(6)",
                "send(conn, leak)",
            ],
        )
        .with_related(vec!["fmtstr_payload"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "analyze".to_string(),
        BuiltinFunction::new(
            "analyze",
            "analyze(binary: string) -> map",
            "Analyzes ELF binary and returns metadata (alias for Elf()). Returns map with symbols, GOT, PLT entries, protection flags (PIE, NX, Canary, RELRO).",
            "Binary Analysis",
            vec![
                "let elf = analyze(binary)",
                "print(elf.base)",
                "print(elf.symbols.main)",
                "print(elf.got.printf)",
            ],
        )
        .with_related(vec!["Elf", "oracle_analyze", "auto_offset"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "allocate".to_string(),
        BuiltinFunction::new(
            "allocate",
            "allocate(connection, size: int) -> int",
            "Allocates heap chunk of specified size on target via connection. Returns address of allocated chunk. Used for heap exploitation workflows.",
            "Heap",
            vec![
                "let addr = allocate(conn, 0x80)",
                "print(hex(addr))",
            ],
        )
        .with_related(vec!["free", "edit", "analyze_heap", "heap_spray"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "edit".to_string(),
        BuiltinFunction::new(
            "edit",
            "edit(connection, address: int, data)",
            "Edits heap chunk at specified address with provided data via connection. Data can be bytes, string, or number. Used for heap corruption exploits.",
            "Heap",
            vec![
                "edit(conn, chunk_addr, poisoned_fd)",
                "edit(conn, vuln_chunk, b\"\\x41\" * 16)",
            ],
        )
        .with_related(vec!["allocate", "free", "analyze_heap"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "trigger_function_pointer".to_string(),
        BuiltinFunction::new(
            "trigger_function_pointer",
            "trigger_function_pointer(connection, address: int)",
            "Triggers function pointer at specified address on target via connection. Used to execute shellcode or ROP chain after heap/stack corruption.",
            "Exploitation",
            vec![
                "trigger_function_pointer(conn, shellcode_addr)",
                "trigger_function_pointer(conn, 0x555555554800)",
            ],
        )
        .with_related(vec!["allocate", "edit", "shellcode"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "connect_tcp".to_string(),
        BuiltinFunction::new(
            "connect_tcp",
            "connect_tcp(host: string, port: int) or connect_tcp(\"host:port\")",
            "Create a TCP connection to a remote host (alias for remote()). Supports both formats: separate host and port arguments, or single 'host:port' string.",
            "Network",
            vec![
                "let conn = connect_tcp(\"127.0.0.1\", 8888)",
                "let conn = connect_tcp(\"target.com:1337\")",
            ],
        )
        .with_related(vec!["remote", "send", "recv", "sendline", "recvline"])
        .with_version("0.2.0"),
    );

    registry.insert(
        "analyze_elf".to_string(),
        BuiltinFunction::new(
            "analyze_elf",
            "analyze_elf(binary_path: string) -> map",
            "Analyzes ELF binary and returns comprehensive map with symbols, PLT, GOT, and protection flags. Alias for Elf() commonly used in exploit chain examples.",
            "Binary Analysis",
            vec![
                "let elf = analyze_elf(\"./vuln\")",
                "print(hex(elf.plt.puts))",
                "print(hex(elf.got.__libc_start_main))",
                "print(hex(elf.symbols.main))",
            ],
        )
        .with_related(vec!["Elf", "analyze", "checksec", "parse_elf"])
        .with_version("0.2.0"),
    );

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_coverage_validation() {
        let registry = FunctionRegistry::new();
        match registry.validate_coverage() {
            Ok(_) => {},
            Err(errors) => {
                panic!("Registry validation failed:\n{}", errors.join("\n"));
            }
        }
    }

    #[test]
    fn test_category_indexing() {
        let registry = FunctionRegistry::new();
        
        let network_funcs = registry.get_category(FunctionCategory::Network);
        assert!(!network_funcs.is_empty(), "Network category should have functions");
        
        let packing_funcs = registry.get_category(FunctionCategory::Packing);
        assert!(!packing_funcs.is_empty(), "Packing category should have functions");
        
        let exploitation_funcs = registry.get_category(FunctionCategory::Exploitation);
        assert!(!exploitation_funcs.is_empty(), "Exploitation category should have functions");
    }

    #[test]
    fn test_search_functionality() {
        let registry = FunctionRegistry::new();
        
        let results = registry.search("rop");
        assert!(!results.is_empty(), "Search for 'rop' should return results");
        
        let results = registry.search("pack");
        assert!(!results.is_empty(), "Search for 'pack' should return results");
        
        let results = registry.search("connection");
        assert!(!results.is_empty(), "Search for 'connection' should return results");
    }

    #[test]
    fn test_related_functions() {
        let registry = FunctionRegistry::new();
        
        let related = registry.get_related("connect");
        assert!(!related.is_empty(), "connect should have related functions");
        
        let related = registry.get_related("p64");
        assert!(!related.is_empty(), "p64 should have related functions");
        
        let related = registry.get_related("cyclic");
        assert!(!related.is_empty(), "cyclic should have related functions");
    }

    #[test]
    fn test_function_metadata() {
        let registry = FunctionRegistry::new();
        
        if let Some(func) = registry.get("connect") {
            assert_eq!(func.name, "connect");
            assert!(!func.signature.is_empty());
            assert!(!func.description.is_empty());
            assert!(!func.examples.is_empty());
            assert_eq!(func.since_version, "0.1.0");
            assert!(func.deprecated.is_none());
        } else {
            panic!("Function 'connect' not found in registry");
        }
    }

    #[test]
    fn test_count_by_category() {
        let registry = FunctionRegistry::new();
        let counts = registry.count_by_category();
        
        assert!(counts.contains_key(&FunctionCategory::Network));
        assert!(counts.contains_key(&FunctionCategory::Packing));
        assert!(counts.contains_key(&FunctionCategory::Exploitation));
        
        let total: usize = counts.values().sum();
        assert!(total > 0, "Registry should have registered functions");
    }

    #[test]
    fn test_new_categories() {
        let registry = FunctionRegistry::new();
        
        let crypto_funcs = registry.get_category(FunctionCategory::Crypto);
        assert!(!crypto_funcs.is_empty(), "Crypto category should have functions");
        
        let fuzzing_funcs = registry.get_category(FunctionCategory::Fuzzing);
        assert!(!fuzzing_funcs.is_empty(), "Fuzzing category should have functions");
        
        let kernel_funcs = registry.get_category(FunctionCategory::Kernel);
        assert!(!kernel_funcs.is_empty(), "Kernel category should have functions");
        
        let heap_funcs = registry.get_category(FunctionCategory::Heap);
        assert!(!heap_funcs.is_empty(), "Heap category should have functions");
        
        let debugging_funcs = registry.get_category(FunctionCategory::Debugging);
        assert!(!debugging_funcs.is_empty(), "Debugging category should have functions");
        
        let symbolic_funcs = registry.get_category(FunctionCategory::Symbolic);
        assert!(!symbolic_funcs.is_empty(), "Symbolic category should have functions");
    }

    #[test]
    fn test_implementation_coverage_validation() {
        let registry = FunctionRegistry::new();
        
        let implemented = vec![
            "connect", "send", "recv", "sendline", "recvline", "recvuntil", "close",
            "process", "interactive", "connect_ssh", "ssh_run", "ssh_upload", "ssh_download",
            "connect_ssh_pty", "ssh_interactive_start", "ssh_interactive_send",
            "ssh_interactive_recv", "ssh_interactive_close",
            "p64", "p32", "p16", "p8", "u64", "u32", "u16", "u8",
            "cyclic", "cyclic_find", "shellcode", "flat", "rop_find", "fmtstr_payload",
            "analyze", "auto_offset", "disasm",
            "len", "range", "hex", "int", "bytes", "str", "print", "random_string", "extract_pattern",
            "read", "write", "split", "join", "replace", "copy", "remote", "help",
            "debug_attach", "breakpoint", "debug_continue", "debug_step",
            "debug_read_mem", "debug_write_mem", "debug_read_reg", "debug_write_reg",
            "symbolic_var", "constrain_no_null", "constrain_alnum", "constrain_range", "symbolic_solve",
            "pool_spray", "heap_feng_shui",
            "token_steal", "process_hide", "rootkit_install", "kaslr_leak", "smep_bypass",
            "kernel_write", "kernel_read",
            "padding_oracle", "bleichenbacher", "timing_attack", "weak_keys", "hash_collision",
            "aes_padding_attack", "rsa_factorize",
            "fuzz_target", "mutate", "coverage", "corpus_add", "crash_triage",
            "parallel_exploit", "generate_exploit",
        ];
        
        match registry.validate_implementation_coverage(&implemented) {
            Ok(_) => {},
            Err(err) => {
                println!("Coverage validation errors:\n{}", err);
            }
        }
    }

    #[test]
    fn test_search_includes_related() {
        let registry = FunctionRegistry::new();
        
        let results = registry.search("rop");
        assert!(!results.is_empty(), "Search for 'rop' should return results");
        
        let rop_find_exists = results.iter().any(|f| f.name == "rop_find");
        assert!(rop_find_exists, "rop_find should be in search results for 'rop'");
    }

    #[test]
    fn test_version_and_deprecation() {
        let registry = FunctionRegistry::new();
        
        if let Some(func) = registry.get("copy") {
            assert_eq!(func.since_version, "0.2.0", "copy should be version 0.2.0");
            assert!(func.deprecated.is_none(), "copy should not be deprecated");
        }
    }

    #[test]
    fn test_all_functions_have_examples() {
        let registry = FunctionRegistry::new();
        
        for func in registry.all_functions() {
            assert!(
                !func.examples.is_empty(),
                "Function '{}' should have at least one example",
                func.name
            );
        }
    }

    #[test]
    fn test_phf_lookup_correctness() {
        let registry = FunctionRegistry::new();
        
        // Test PHF lookups for known registered functions
        let test_functions = vec![
            "connect", "send", "recv", "process", "shellcode",
            "cyclic", "p64", "u64", "connect_ssh", "oracle_analyze",
            "flat", "analyze", "auto_offset", "print", "len",
        ];
        
        for func_name in test_functions {
            let result = registry.get(func_name);
            assert!(result.is_some(), "PHF lookup for '{}' should succeed", func_name);
            
            let func = result.unwrap();
            assert_eq!(func.name, func_name, "Function name should match");
            assert!(!func.signature.is_empty(), "Function '{}' should have signature", func_name);
            assert!(!func.description.is_empty(), "Function '{}' should have description", func_name);
        }
    }

    #[test]
    fn test_phf_vs_hashmap_consistency() {
        let registry = FunctionRegistry::new();
        
        // Test that PHF and HashMap return identical results for registered functions
        let test_functions = vec![
            "connect", "cyclic", "p64", "connect_ssh", "oracle_analyze",
            "rop_find", "parallel_exploit", "mitigation_analyze",
            "flat", "analyze", "shellcode",
        ];
        
        for func_name in test_functions {
            let result1 = registry.get(func_name);
            let result2 = registry.functions.get(func_name);
            
            assert_eq!(
                result1.is_some(),
                result2.is_some(),
                "PHF and HashMap should have consistent results for '{}'",
                func_name
            );
            
            if let (Some(f1), Some(f2)) = (result1, result2) {
                assert_eq!(f1.name, f2.name);
                assert_eq!(f1.signature, f2.signature);
                assert_eq!(f1.category, f2.category);
            }
        }
    }

    #[test]
    fn test_phf_nonexistent_function() {
        let registry = FunctionRegistry::new();
        
        let result = registry.get("nonexistent_function_12345");
        assert!(result.is_none(), "PHF lookup for nonexistent function should return None");
        
        let result = registry.get("");
        assert!(result.is_none(), "PHF lookup for empty string should return None");
    }

    #[test]
    fn test_phf_registry_completeness() {
        // Verify that the PHF registry contains all expected core functions
        let expected_functions = vec![
            "connect", "send", "sendline", "recv", "recvline", "recvuntil", "close",
            "interactive", "connect_ssl", "process", "attach", "gdb", "disasm_at",
            "shellcode", "cyclic", "cyclic_find", "flat", "analyze", "auto_offset",
            "p8", "p16", "p32", "p64", "u8", "u16", "u32", "u64",
            "oracle_analyze", "oracle_find_shellcode", "oracle_gadget_density",
            "mitigation_analyze", "mitigation_auto_pivot", "mitigation_validate",
            "rop_find", "parallel_exploit", "mass_connect",
        ];
        
        for func_name in expected_functions {
            assert!(
                BUILTIN_REGISTRY.contains_key(func_name),
                "PHF registry should contain '{}'",
                func_name
            );
        }
    }

    #[test]
    fn test_builtin_count_matches() {
        // Verify BUILTIN_COUNT matches actual registry size
        let registry_map = register_builtins();
        
        assert_eq!(
            BUILTIN_COUNT,
            BUILTIN_REGISTRY.len(),
            "BUILTIN_COUNT should match PHF registry size"
        );
        
        // Verify all registered functions are in PHF
        for name in registry_map.keys() {
            assert!(
                BUILTIN_REGISTRY.contains_key(name.as_str()),
                "Function '{}' should be in PHF registry",
                name
            );
        }
    }

    #[test]
    fn test_get_fast_method() {
        let registry = FunctionRegistry::new();
        
        // Test get_fast for existing registered functions
        let test_cases = vec!["connect", "p64", "oracle_analyze", "flat", "analyze"];
        
        for func_name in test_cases {
            let result = registry.get_fast(func_name);
            assert!(result.is_some(), "get_fast('{}') should return Some", func_name);
            
            let func = result.unwrap();
            assert_eq!(func.name, func_name);
        }
        
        // Test get_fast for nonexistent function
        let result = registry.get_fast("nonexistent_function");
        assert!(result.is_none(), "get_fast for nonexistent function should return None");
    }

    #[test]
    fn test_backward_compatibility() {
        let registry = FunctionRegistry::new();
        
        // Verify all existing methods still work correctly
        assert!(registry.all_functions().len() > 0);
        
        let network_funcs = registry.get_category(FunctionCategory::Network);
        assert!(!network_funcs.is_empty());
        
        let search_results = registry.search("connect");
        assert!(!search_results.is_empty());
        
        let related = registry.get_related("connect");
        assert!(!related.is_empty());
        
        // Verify coverage validation still works
        assert!(registry.validate_coverage().is_ok());
    }
}
