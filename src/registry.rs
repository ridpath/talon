use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub category: String,
    pub examples: Vec<String>,
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
        }
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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
        ),
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

    registry
}
