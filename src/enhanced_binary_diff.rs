use capstone::prelude::*;
use goblin::Object;
use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Debug, Clone)]
pub struct EnhancedDiffResult {
    pub function_changes: Vec<FunctionDiff>,
    pub security_regressions: Vec<SecurityRegression>,
    pub new_exploits: Vec<ExploitOpportunity>,
    pub patch_analysis: PatchAnalysis,
    pub control_flow_changes: Vec<CFGChange>,
    pub vulnerability_score: f64,
    pub attack_surface_delta: AttackSurfaceDelta,
}

#[derive(Debug, Clone)]
pub struct FunctionDiff {
    pub name: String,
    pub offset_old: u64,
    pub offset_new: u64,
    pub size_old: usize,
    pub size_new: usize,
    pub changes: Vec<CodeChange>,
    pub cfg_old: Option<ControlFlowGraph>,
    pub cfg_new: Option<ControlFlowGraph>,
    pub complexity_change: i32,
}

#[derive(Debug, Clone)]
pub struct CodeChange {
    pub offset: u64,
    pub old_bytes: Vec<u8>,
    pub new_bytes: Vec<u8>,
    pub old_disasm: Vec<String>,
    pub new_disasm: Vec<String>,
    pub description: String,
    pub severity: Severity,
    pub exploitability_rating: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone)]
pub struct SecurityRegression {
    pub description: String,
    pub function: String,
    pub severity: Severity,
    pub exploit_complexity: String,
    pub recommendation: String,
    pub cwe_id: Option<String>,
    pub cvss_score: f64,
}

#[derive(Debug, Clone)]
pub struct ExploitOpportunity {
    pub vulnerability_type: String,
    pub function: String,
    pub description: String,
    pub exploitation_steps: Vec<String>,
    pub poc_code: Option<String>,
    pub prerequisites: Vec<String>,
    pub reliability: String,
}

#[derive(Debug, Clone)]
pub struct PatchAnalysis {
    pub is_security_patch: bool,
    pub cve_candidates: Vec<String>,
    pub patch_quality: String,
    pub bypass_opportunities: Vec<String>,
    pub completeness_score: f64,
    pub variant_analysis: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    pub basic_blocks: Vec<BasicBlock>,
    pub edges: Vec<(usize, usize)>,
    pub entry_point: usize,
    pub cyclomatic_complexity: usize,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: usize,
    pub start_addr: u64,
    pub end_addr: u64,
    pub instructions: Vec<String>,
    pub is_security_check: bool,
}

#[derive(Debug, Clone)]
pub struct CFGChange {
    pub function: String,
    pub change_type: String,
    pub description: String,
    pub security_impact: String,
}

#[derive(Debug, Clone)]
pub struct AttackSurfaceDelta {
    pub new_entry_points: Vec<String>,
    pub removed_entry_points: Vec<String>,
    pub new_dangerous_functions: Vec<String>,
    pub removed_mitigations: Vec<String>,
    pub added_mitigations: Vec<String>,
}

pub struct EnhancedBinaryDiffer;

impl EnhancedBinaryDiffer {
    pub fn diff(file1: &str, file2: &str) -> Result<EnhancedDiffResult, String> {
        log::info!("GOD-MODE BINARY DIFF: {} vs {}", file1, file2);

        let data1 = fs::read(file1).map_err(|e| format!("Failed to read {}: {}", file1, e))?;
        let data2 = fs::read(file2).map_err(|e| format!("Failed to read {}: {}", file2, e))?;

        let obj1 =
            Object::parse(&data1).map_err(|e| format!("Failed to parse {}: {}", file1, e))?;
        let obj2 =
            Object::parse(&data2).map_err(|e| format!("Failed to parse {}: {}", file2, e))?;

        log::info!("Analyzing function changes with disassembly...");
        let function_changes = Self::analyze_function_changes(&obj1, &obj2, &data1, &data2)?;

        log::info!("Detecting security regressions...");
        let security_regressions = Self::detect_security_regressions(&function_changes);

        log::info!("Finding exploit opportunities...");
        let new_exploits =
            Self::find_exploit_opportunities(&function_changes, &security_regressions);

        log::info!("Analyzing patch quality...");
        let patch_analysis = Self::analyze_patch(&function_changes, &security_regressions);

        log::info!("Analyzing control flow changes...");
        let control_flow_changes = Self::analyze_control_flow_changes(&function_changes);

        log::info!("Computing vulnerability score...");
        let vulnerability_score =
            Self::compute_vulnerability_score(&security_regressions, &function_changes);

        log::info!("Analyzing attack surface delta...");
        let attack_surface_delta = Self::analyze_attack_surface(&obj1, &obj2, &function_changes);

        Ok(EnhancedDiffResult {
            function_changes,
            security_regressions,
            new_exploits,
            patch_analysis,
            control_flow_changes,
            vulnerability_score,
            attack_surface_delta,
        })
    }

    fn analyze_function_changes(
        obj1: &Object,
        obj2: &Object,
        data1: &[u8],
        data2: &[u8],
    ) -> Result<Vec<FunctionDiff>, String> {
        let mut diffs = Vec::new();

        match (obj1, obj2) {
            (Object::Elf(elf1), Object::Elf(elf2)) => {
                let syms1: HashMap<String, goblin::elf::Sym> = elf1
                    .syms
                    .iter()
                    .filter_map(|sym| {
                        elf1.strtab
                            .get_at(sym.st_name)
                            .map(|name| (name.to_string(), sym))
                    })
                    .collect();

                let syms2: HashMap<String, goblin::elf::Sym> = elf2
                    .syms
                    .iter()
                    .filter_map(|sym| {
                        elf2.strtab
                            .get_at(sym.st_name)
                            .map(|name| (name.to_string(), sym))
                    })
                    .collect();

                for (name, sym1) in syms1.iter() {
                    if let Some(sym2) = syms2.get(name) {
                        if sym1.st_value != sym2.st_value || sym1.st_size != sym2.st_size {
                            let changes = Self::analyze_code_changes_with_disasm(
                                data1,
                                sym1.st_value,
                                sym1.st_size as usize,
                                data2,
                                sym2.st_value,
                                sym2.st_size as usize,
                            );

                            let cfg_old =
                                Self::build_cfg(data1, sym1.st_value, sym1.st_size as usize);
                            let cfg_new =
                                Self::build_cfg(data2, sym2.st_value, sym2.st_size as usize);

                            let complexity_change = match (&cfg_old, &cfg_new) {
                                (Some(c1), Some(c2)) => {
                                    c2.cyclomatic_complexity as i32
                                        - c1.cyclomatic_complexity as i32
                                }
                                _ => 0,
                            };

                            diffs.push(FunctionDiff {
                                name: name.to_string(),
                                offset_old: sym1.st_value,
                                offset_new: sym2.st_value,
                                size_old: sym1.st_size as usize,
                                size_new: sym2.st_size as usize,
                                changes,
                                cfg_old,
                                cfg_new,
                                complexity_change,
                            });
                        }
                    }
                }
            }
            _ => return Err("Only ELF binaries supported for now".to_string()),
        }

        Ok(diffs)
    }

    fn analyze_code_changes_with_disasm(
        data1: &[u8],
        addr1: u64,
        size1: usize,
        data2: &[u8],
        addr2: u64,
        size2: usize,
    ) -> Vec<CodeChange> {
        let mut changes = Vec::new();

        let bytes1 = Self::extract_bytes(data1, addr1, size1);
        let bytes2 = Self::extract_bytes(data2, addr2, size2);

        let disasm1 = Self::disassemble(&bytes1, addr1);
        let disasm2 = Self::disassemble(&bytes2, addr2);

        if bytes1 != bytes2 {
            let description = Self::classify_code_change(&disasm1, &disasm2);
            let severity = Self::assess_change_severity(&disasm1, &disasm2);
            let exploitability = Self::compute_exploitability(&disasm1, &disasm2);

            changes.push(CodeChange {
                offset: addr1,
                old_bytes: bytes1.clone(),
                new_bytes: bytes2.clone(),
                old_disasm: disasm1,
                new_disasm: disasm2,
                description,
                severity,
                exploitability_rating: exploitability,
            });
        }

        changes
    }

    fn extract_bytes(data: &[u8], addr: u64, size: usize) -> Vec<u8> {
        let start = addr as usize;
        let end = start + size;
        if end <= data.len() {
            data[start..end].to_vec()
        } else {
            vec![]
        }
    }

    fn disassemble(bytes: &[u8], base_addr: u64) -> Vec<String> {
        let cs = Capstone::new()
            .x86()
            .mode(arch::x86::ArchMode::Mode64)
            .syntax(arch::x86::ArchSyntax::Intel)
            .detail(true)
            .build();

        match cs {
            Ok(cs) => cs
                .disasm_all(bytes, base_addr)
                .ok()
                .map(|insns| {
                    insns
                        .iter()
                        .map(|i| {
                            format!(
                                "{:#x}: {} {}",
                                i.address(),
                                i.mnemonic().unwrap_or(""),
                                i.op_str().unwrap_or("")
                            )
                        })
                        .collect()
                })
                .unwrap_or_default(),
            Err(_) => vec![],
        }
    }

    fn classify_code_change(old_disasm: &[String], new_disasm: &[String]) -> String {
        if old_disasm.len() > new_disasm.len() {
            "Code removed - potential optimization or security check removal".to_string()
        } else if new_disasm.len() > old_disasm.len() {
            "Code added - new functionality or mitigation".to_string()
        } else {
            "Code modified - logic change".to_string()
        }
    }

    fn assess_change_severity(old_disasm: &[String], new_disasm: &[String]) -> Severity {
        let old_str = old_disasm.join(" ");
        let new_str = new_disasm.join(" ");

        if old_str.contains("cmp") && !new_str.contains("cmp") {
            return Severity::Critical;
        }
        if old_str.contains("test") && !new_str.contains("test") {
            return Severity::High;
        }
        if old_disasm.len() > new_disasm.len() + 5 {
            return Severity::High;
        }

        Severity::Medium
    }

    fn compute_exploitability(old_disasm: &[String], new_disasm: &[String]) -> f64 {
        let mut score: f64 = 0.0;

        let old_str = old_disasm.join(" ");
        let new_str = new_disasm.join(" ");

        if old_str.contains("cmp") && !new_str.contains("cmp") {
            score += 30.0;
        }
        if old_str.contains("test") && !new_str.contains("test") {
            score += 25.0;
        }
        if old_str.contains("mov") && old_str.contains("fs:0x28") && !new_str.contains("fs:0x28") {
            score += 40.0;
        }
        if old_str.contains("call") && new_str.contains("jmp") {
            score += 20.0;
        }
        if old_disasm.len() > new_disasm.len() + 3 {
            score += 15.0;
        }

        score.min(100.0)
    }

    fn build_cfg(data: &[u8], addr: u64, size: usize) -> Option<ControlFlowGraph> {
        let bytes = Self::extract_bytes(data, addr, size);
        if bytes.is_empty() {
            return None;
        }

        let mut basic_blocks = Vec::new();
        let mut edges = Vec::new();
        let mut current_block_id = 0;
        let mut current_block_start = addr;
        let mut instructions = Vec::new();

        let disasm = Self::disassemble(&bytes, addr);

        for (idx, insn) in disasm.iter().enumerate() {
            instructions.push(insn.clone());

            let is_branch = insn.contains("jmp")
                || insn.contains("je")
                || insn.contains("jne")
                || insn.contains("call")
                || insn.contains("ret");

            if is_branch || idx == disasm.len() - 1 {
                let is_security_check = instructions
                    .iter()
                    .any(|i| i.contains("cmp") || i.contains("test") || i.contains("fs:0x28"));

                basic_blocks.push(BasicBlock {
                    id: current_block_id,
                    start_addr: current_block_start,
                    end_addr: addr + (idx as u64 + 1) * 4,
                    instructions: instructions.clone(),
                    is_security_check,
                });

                if current_block_id > 0 {
                    edges.push((current_block_id - 1, current_block_id));
                }

                current_block_id += 1;
                current_block_start = addr + (idx as u64 + 1) * 4;
                instructions.clear();
            }
        }

        let cyclomatic_complexity = edges.len() - basic_blocks.len() + 2;

        Some(ControlFlowGraph {
            basic_blocks,
            edges,
            entry_point: 0,
            cyclomatic_complexity,
        })
    }

    fn detect_security_regressions(changes: &[FunctionDiff]) -> Vec<SecurityRegression> {
        let mut regressions = Vec::new();

        for func_diff in changes {
            for change in &func_diff.changes {
                if Self::is_bounds_check_removed(&change.old_disasm, &change.new_disasm) {
                    regressions.push(SecurityRegression {
                        description: "Bounds check removed - potential buffer overflow".to_string(),
                        function: func_diff.name.clone(),
                        severity: Severity::Critical,
                        exploit_complexity: "Low - trivial buffer overflow".to_string(),
                        recommendation: "Re-add bounds checking or use safe functions".to_string(),
                        cwe_id: Some("CWE-119".to_string()),
                        cvss_score: 9.8,
                    });
                }

                if Self::is_null_check_removed(&change.old_disasm, &change.new_disasm) {
                    regressions.push(SecurityRegression {
                        description: "NULL pointer check removed".to_string(),
                        function: func_diff.name.clone(),
                        severity: Severity::High,
                        exploit_complexity: "Medium - NULL dereference exploit".to_string(),
                        recommendation: "Re-add NULL pointer validation".to_string(),
                        cwe_id: Some("CWE-476".to_string()),
                        cvss_score: 7.5,
                    });
                }

                if Self::is_canary_removed(&change.old_disasm, &change.new_disasm) {
                    regressions.push(SecurityRegression {
                        description: "Stack canary check removed".to_string(),
                        function: func_diff.name.clone(),
                        severity: Severity::High,
                        exploit_complexity: "Low - canary bypass not needed".to_string(),
                        recommendation: "Re-enable stack protection".to_string(),
                        cwe_id: Some("CWE-693".to_string()),
                        cvss_score: 8.1,
                    });
                }

                if Self::is_integer_overflow_check_removed(&change.old_disasm, &change.new_disasm) {
                    regressions.push(SecurityRegression {
                        description: "Integer overflow check removed".to_string(),
                        function: func_diff.name.clone(),
                        severity: Severity::High,
                        exploit_complexity: "Medium - integer overflow to memory corruption"
                            .to_string(),
                        recommendation: "Re-add integer overflow validation".to_string(),
                        cwe_id: Some("CWE-190".to_string()),
                        cvss_score: 7.8,
                    });
                }

                if Self::is_authentication_check_removed(&change.old_disasm, &change.new_disasm) {
                    regressions.push(SecurityRegression {
                        description: "Authentication or authorization check removed".to_string(),
                        function: func_diff.name.clone(),
                        severity: Severity::Critical,
                        exploit_complexity: "Low - trivial auth bypass".to_string(),
                        recommendation: "Re-add authentication validation".to_string(),
                        cwe_id: Some("CWE-306".to_string()),
                        cvss_score: 9.8,
                    });
                }
            }

            if let (Some(cfg_old), Some(cfg_new)) = (&func_diff.cfg_old, &func_diff.cfg_new) {
                let security_blocks_removed = cfg_old
                    .basic_blocks
                    .iter()
                    .filter(|b| b.is_security_check)
                    .count()
                    - cfg_new
                        .basic_blocks
                        .iter()
                        .filter(|b| b.is_security_check)
                        .count();

                if security_blocks_removed > 0 {
                    regressions.push(SecurityRegression {
                        description: format!(
                            "{} security check basic blocks removed from control flow",
                            security_blocks_removed
                        ),
                        function: func_diff.name.clone(),
                        severity: Severity::Critical,
                        exploit_complexity: "Low - multiple security checks bypassed".to_string(),
                        recommendation: "Restore removed security validations".to_string(),
                        cwe_id: Some("CWE-20".to_string()),
                        cvss_score: 9.1,
                    });
                }
            }
        }

        regressions
    }

    fn find_exploit_opportunities(
        changes: &[FunctionDiff],
        regressions: &[SecurityRegression],
    ) -> Vec<ExploitOpportunity> {
        let mut exploits = Vec::new();

        for regression in regressions {
            match regression.severity {
                Severity::Critical | Severity::High => {
                    let poc = Self::generate_advanced_poc(regression);
                    let vuln_type = Self::classify_vulnerability(&regression.description);
                    let steps = Self::generate_exploitation_steps(&vuln_type, regression);
                    let prereqs = Self::identify_prerequisites(&vuln_type);
                    let reliability = Self::assess_reliability(&regression.exploit_complexity);

                    exploits.push(ExploitOpportunity {
                        vulnerability_type: vuln_type,
                        function: regression.function.clone(),
                        description: regression.description.clone(),
                        exploitation_steps: steps,
                        poc_code: Some(poc),
                        prerequisites: prereqs,
                        reliability,
                    });
                }
                _ => {}
            }
        }

        for func_diff in changes {
            if func_diff.size_new > func_diff.size_old + 100 {
                exploits.push(ExploitOpportunity {
                    vulnerability_type: "Logic Change".to_string(),
                    function: func_diff.name.clone(),
                    description: "Significant code added - investigate for vulnerabilities"
                        .to_string(),
                    exploitation_steps: vec![
                        "1. Reverse engineer new code paths".to_string(),
                        "2. Identify input validation weaknesses".to_string(),
                        "3. Test edge cases and boundary conditions".to_string(),
                        "4. Fuzz new functionality with malformed inputs".to_string(),
                    ],
                    poc_code: None,
                    prerequisites: vec!["Source code or detailed reversing".to_string()],
                    reliability: "Unknown - requires analysis".to_string(),
                });
            }
        }

        exploits
    }

    fn classify_vulnerability(description: &str) -> String {
        if description.contains("buffer overflow") || description.contains("bounds check") {
            "Buffer Overflow (Stack)".to_string()
        } else if description.contains("NULL") {
            "NULL Pointer Dereference".to_string()
        } else if description.contains("canary") {
            "Stack Canary Bypass".to_string()
        } else if description.contains("integer overflow") {
            "Integer Overflow".to_string()
        } else if description.contains("authentication") || description.contains("authorization") {
            "Authentication Bypass".to_string()
        } else {
            "Unknown Vulnerability".to_string()
        }
    }

    fn generate_exploitation_steps(
        vuln_type: &str,
        regression: &SecurityRegression,
    ) -> Vec<String> {
        match vuln_type {
            "Buffer Overflow (Stack)" => vec![
                "1. Generate cyclic pattern to find offset to return address".to_string(),
                "2. Identify bad characters (null bytes, newlines, etc.)".to_string(),
                "3. Leak libc base address if ASLR/PIE enabled".to_string(),
                "4. Build ROP chain or inject shellcode".to_string(),
                "5. Trigger overflow and achieve code execution".to_string(),
                format!(
                    "6. {} (CWE: {}, CVSS: {})",
                    regression.recommendation,
                    regression.cwe_id.as_ref().unwrap_or(&"Unknown".to_string()),
                    regression.cvss_score
                ),
            ],
            "NULL Pointer Dereference" => vec![
                "1. Map page 0 using mmap (requires CAP_SYS_ADMIN or kernel <2.6.23)".to_string(),
                "2. Place controlled data at dereferenced offset".to_string(),
                "3. Trigger NULL dereference to jump to controlled data".to_string(),
                "4. Achieve arbitrary code execution".to_string(),
            ],
            "Authentication Bypass" => vec![
                "1. Identify authentication check location".to_string(),
                "2. Craft input that bypasses removed validation".to_string(),
                "3. Escalate privileges or access restricted functionality".to_string(),
                "4. Maintain persistence if applicable".to_string(),
            ],
            _ => vec![
                "1. Reverse engineer vulnerability in detail".to_string(),
                "2. Develop proof-of-concept exploit".to_string(),
                "3. Test reliability across different environments".to_string(),
            ],
        }
    }

    fn identify_prerequisites(vuln_type: &str) -> Vec<String> {
        match vuln_type {
            "Buffer Overflow (Stack)" => vec![
                "Control over input buffer".to_string(),
                "Known offset to return address".to_string(),
                "ROP gadgets or executable stack".to_string(),
            ],
            "NULL Pointer Dereference" => vec![
                "Ability to map page 0 or kernel exploit".to_string(),
                "Control over dereferenced data".to_string(),
            ],
            "Authentication Bypass" => vec![
                "Network access or local execution".to_string(),
                "Understanding of authentication mechanism".to_string(),
            ],
            _ => vec!["Varies by vulnerability".to_string()],
        }
    }

    fn assess_reliability(complexity: &str) -> String {
        if complexity.contains("Low") {
            "High - reliable exploitation".to_string()
        } else if complexity.contains("Medium") {
            "Medium - may require multiple attempts".to_string()
        } else {
            "Low - requires significant effort".to_string()
        }
    }

    fn analyze_patch(
        changes: &[FunctionDiff],
        regressions: &[SecurityRegression],
    ) -> PatchAnalysis {
        let is_security_patch = changes.iter().any(|c| {
            c.name.contains("check")
                || c.name.contains("validate")
                || c.name.contains("sanitize")
                || c.name.contains("auth")
                || c.name.contains("verify")
        }) || !regressions.is_empty();

        let cve_candidates = if !regressions.is_empty() {
            regressions
                .iter()
                .filter(|r| r.severity == Severity::Critical)
                .map(|r| {
                    format!(
                        "CVE-202X-XXXXX (candidate for {} in {})",
                        r.description, r.function
                    )
                })
                .collect()
        } else {
            vec![]
        };

        let mut bypass_opportunities = vec![
            "Check if patch is complete or only fixes specific case".to_string(),
            "Look for similar unpatched functions (variant analysis)".to_string(),
            "Test boundary conditions of the fix".to_string(),
            "Verify fix works across all code paths".to_string(),
        ];

        let completeness_score = if regressions.is_empty() {
            95.0
        } else {
            let critical_count = regressions
                .iter()
                .filter(|r| r.severity == Severity::Critical)
                .count();
            let high_count = regressions
                .iter()
                .filter(|r| r.severity == Severity::High)
                .count();
            100.0 - (critical_count as f64 * 15.0 + high_count as f64 * 8.0)
        };

        let variant_analysis: Vec<String> = changes
            .iter()
            .map(|c| format!("Check {} for similar patterns", c.name))
            .collect();

        if completeness_score < 70.0 {
            bypass_opportunities
                .push("CRITICAL: Multiple high-severity issues - patch incomplete".to_string());
        }

        PatchAnalysis {
            is_security_patch,
            cve_candidates,
            patch_quality: if completeness_score > 90.0 {
                "Excellent".to_string()
            } else if completeness_score > 70.0 {
                "Good".to_string()
            } else {
                "Poor - requires revision".to_string()
            },
            bypass_opportunities,
            completeness_score,
            variant_analysis,
        }
    }

    fn is_bounds_check_removed(old: &[String], new: &[String]) -> bool {
        let old_str = old.join(" ");
        let new_str = new.join(" ");
        (old_str.contains("cmp") || old_str.contains("sub"))
            && !(new_str.contains("cmp") || new_str.contains("sub"))
    }

    fn is_null_check_removed(old: &[String], new: &[String]) -> bool {
        let old_str = old.join(" ");
        let new_str = new.join(" ");
        (old_str.contains("test") && old_str.contains("je"))
            && !(new_str.contains("test") && new_str.contains("je"))
    }

    fn is_canary_removed(old: &[String], new: &[String]) -> bool {
        let old_str = old.join(" ");
        let new_str = new.join(" ");
        old_str.contains("fs:0x28") && !new_str.contains("fs:0x28")
    }

    fn is_integer_overflow_check_removed(old: &[String], new: &[String]) -> bool {
        let old_str = old.join(" ");
        let new_str = new.join(" ");
        (old_str.contains("jo ") || old_str.contains("jc "))
            && !(new_str.contains("jo ") || new_str.contains("jc "))
    }

    fn is_authentication_check_removed(old: &[String], new: &[String]) -> bool {
        old.len() > new.len() + 3 && old.iter().any(|i| i.contains("cmp") || i.contains("test"))
    }

    fn generate_advanced_poc(regression: &SecurityRegression) -> String {
        format!(
            r#"// ═══════════════════════════════════════════════════════════
// POC for {} in {}
// Severity: {:?} | CWE: {} | CVSS: {}
// Complexity: {}
// ═══════════════════════════════════════════════════════════

// Step 1: Analyze binary and detect protections
analyze binary "vulnerable_binary"
find vulnerability type=buffer_overflow in function={}

// Step 2: Generate De Bruijn pattern for offset discovery
let pattern = cyclic(1000)
send pattern
let crash_offset = cyclic_find(pattern, crash_eip)

// Step 3: Leak addresses if ASLR/PIE enabled
let puts_plt = elf_symbol("puts@plt")
let puts_got = elf_symbol("puts@got")
let rop1 = [crash_offset padding, puts_plt, main_addr, puts_got]
send rop1
let libc_base = recv_leak() - libc_offset("puts")

// Step 4: Build final ROP chain
let system = libc_base + libc_offset("system")
let bin_sh = libc_base + libc_offset("/bin/sh")
let pop_rdi = find_gadget("pop rdi; ret")

let final_payload = [
    "A" * crash_offset,
    pop_rdi,
    bin_sh,
    system
]

// Step 5: Exploit
connect to "target.com" on port 1337
send final_payload
interactive

// Recommendation: {}
"#,
            regression.description,
            regression.function,
            regression.severity,
            regression.cwe_id.as_ref().unwrap_or(&"Unknown".to_string()),
            regression.cvss_score,
            regression.exploit_complexity,
            regression.function,
            regression.recommendation
        )
    }

    fn analyze_control_flow_changes(changes: &[FunctionDiff]) -> Vec<CFGChange> {
        let mut cfg_changes = Vec::new();

        for func_diff in changes {
            if let (Some(cfg_old), Some(cfg_new)) = (&func_diff.cfg_old, &func_diff.cfg_new) {
                if cfg_old.cyclomatic_complexity != cfg_new.cyclomatic_complexity {
                    let change_type =
                        if cfg_new.cyclomatic_complexity > cfg_old.cyclomatic_complexity {
                            "Complexity Increased"
                        } else {
                            "Complexity Decreased"
                        };

                    let security_impact =
                        if cfg_new.cyclomatic_complexity < cfg_old.cyclomatic_complexity {
                            "Potential security check removal - reduced validation paths"
                        } else {
                            "Added logic - may introduce new vulnerabilities"
                        };

                    cfg_changes.push(CFGChange {
                        function: func_diff.name.clone(),
                        change_type: change_type.to_string(),
                        description: format!(
                            "Cyclomatic complexity: {} → {}",
                            cfg_old.cyclomatic_complexity, cfg_new.cyclomatic_complexity
                        ),
                        security_impact: security_impact.to_string(),
                    });
                }

                let security_blocks_old = cfg_old
                    .basic_blocks
                    .iter()
                    .filter(|b| b.is_security_check)
                    .count();
                let security_blocks_new = cfg_new
                    .basic_blocks
                    .iter()
                    .filter(|b| b.is_security_check)
                    .count();

                if security_blocks_old != security_blocks_new {
                    cfg_changes.push(CFGChange {
                        function: func_diff.name.clone(),
                        change_type: "Security Block Count Changed".to_string(),
                        description: format!(
                            "Security check blocks: {} → {}",
                            security_blocks_old, security_blocks_new
                        ),
                        security_impact: if security_blocks_new < security_blocks_old {
                            "CRITICAL: Security validations removed"
                        } else {
                            "Positive: Additional security checks added"
                        }
                        .to_string(),
                    });
                }
            }
        }

        cfg_changes
    }

    fn compute_vulnerability_score(
        regressions: &[SecurityRegression],
        changes: &[FunctionDiff],
    ) -> f64 {
        let mut score = 0.0;

        for regression in regressions {
            score += regression.cvss_score;

            match regression.severity {
                Severity::Critical => score += 20.0,
                Severity::High => score += 10.0,
                Severity::Medium => score += 5.0,
                Severity::Low => score += 2.0,
                _ => {}
            }
        }

        for func_diff in changes {
            for change in &func_diff.changes {
                score += change.exploitability_rating;
            }

            if func_diff.complexity_change < -5 {
                score += 5.0;
            }
        }

        score.min(100.0)
    }

    fn analyze_attack_surface(
        obj1: &Object,
        obj2: &Object,
        changes: &[FunctionDiff],
    ) -> AttackSurfaceDelta {
        let mut new_entry_points = Vec::new();
        let mut removed_entry_points = Vec::new();
        let mut new_dangerous_functions = Vec::new();
        let mut removed_mitigations = Vec::new();
        let mut added_mitigations = Vec::new();

        if let (Object::Elf(elf1), Object::Elf(elf2)) = (obj1, obj2) {
            let syms1: HashSet<&str> = elf1
                .syms
                .iter()
                .filter_map(|sym| elf1.strtab.get_at(sym.st_name))
                .collect();

            let syms2: HashSet<&str> = elf2
                .syms
                .iter()
                .filter_map(|sym| elf2.strtab.get_at(sym.st_name))
                .collect();

            for sym in syms2.difference(&syms1) {
                if sym.contains("main") || sym.contains("init") || sym.contains("handler") {
                    new_entry_points.push(sym.to_string());
                }

                if sym.contains("strcpy") || sym.contains("sprintf") || sym.contains("gets") {
                    new_dangerous_functions.push(sym.to_string());
                }
            }

            for sym in syms1.difference(&syms2) {
                if sym.contains("main") || sym.contains("init") || sym.contains("handler") {
                    removed_entry_points.push(sym.to_string());
                }
            }
        }

        for func_diff in changes {
            for change in &func_diff.changes {
                if change.old_disasm.iter().any(|i| i.contains("fs:0x28"))
                    && !change.new_disasm.iter().any(|i| i.contains("fs:0x28"))
                {
                    removed_mitigations.push(format!("Stack canary in {}", func_diff.name));
                }

                if !change.old_disasm.iter().any(|i| i.contains("fs:0x28"))
                    && change.new_disasm.iter().any(|i| i.contains("fs:0x28"))
                {
                    added_mitigations.push(format!("Stack canary in {}", func_diff.name));
                }
            }
        }

        AttackSurfaceDelta {
            new_entry_points,
            removed_entry_points,
            new_dangerous_functions,
            removed_mitigations,
            added_mitigations,
        }
    }

    pub fn print_analysis(result: &EnhancedDiffResult) {
        use colored::Colorize;

        println!(
            "\n{}",
            "═══════════════ GOD-MODE BINARY DIFF ANALYSIS ═══════════════"
                .bold()
                .cyan()
        );

        println!(
            "\n{} {}/100",
            "Vulnerability Score:".bold(),
            result.vulnerability_score.to_string().red().bold()
        );

        println!(
            "\n{} {}",
            " Function Changes:".bold().yellow(),
            result.function_changes.len()
        );
        for func in &result.function_changes {
            println!(
                "  • {} ({} → {} bytes, complexity: {})",
                func.name.green(),
                func.size_old,
                func.size_new,
                if func.complexity_change > 0 {
                    format!("+{}", func.complexity_change).red()
                } else if func.complexity_change < 0 {
                    format!("{}", func.complexity_change).green()
                } else {
                    "0".normal()
                }
            );

            for change in &func.changes {
                println!(
                    "    [{:?}] {} (exploitability: {:.1}/100)",
                    change.severity, change.description, change.exploitability_rating
                );
                if !change.old_disasm.is_empty() && !change.new_disasm.is_empty() {
                    println!(
                        "      Old: {}",
                        change
                            .old_disasm
                            .first()
                            .unwrap_or(&String::new())
                            .bright_black()
                    );
                    println!(
                        "      New: {}",
                        change
                            .new_disasm
                            .first()
                            .unwrap_or(&String::new())
                            .bright_black()
                    );
                }
            }
        }

        println!(
            "\n{} {}",
            "Security Regressions:".bold().red(),
            result.security_regressions.len()
        );
        for reg in &result.security_regressions {
            let severity_str = format!("{:?}", reg.severity);
            println!(
                "  • [{}] {} in {}",
                severity_str.red().bold(),
                reg.description.red(),
                reg.function.yellow()
            );
            println!(
                "    CWE: {} | CVSS: {:.1} | Complexity: {}",
                reg.cwe_id.as_ref().unwrap_or(&"N/A".to_string()).cyan(),
                reg.cvss_score,
                reg.exploit_complexity.bright_black()
            );
            println!("    Recommendation: {}", reg.recommendation.green());
        }

        println!(
            "\n{} {}",
            "Exploit Opportunities:".bold().red(),
            result.new_exploits.len()
        );
        for exploit in &result.new_exploits {
            println!(
                "  • {} in {}",
                exploit.vulnerability_type.red().bold(),
                exploit.function.yellow()
            );
            println!("    {}", exploit.description.bright_black());
            println!("    Reliability: {}", exploit.reliability.cyan());
            println!("    Prerequisites:");
            for prereq in &exploit.prerequisites {
                println!("      - {}", prereq);
            }
            println!("    Exploitation Steps:");
            for step in &exploit.exploitation_steps {
                println!("      {}", step.bright_black());
            }
        }

        println!("\n{}", "Patch Analysis:".bold().cyan());
        println!(
            "  Security Patch: {}",
            if result.patch_analysis.is_security_patch {
                "Yes".green()
            } else {
                "No".bright_black()
            }
        );
        println!(
            "  Quality: {} ({:.1}%)",
            result.patch_analysis.patch_quality.cyan().bold(),
            result.patch_analysis.completeness_score
        );

        if !result.patch_analysis.cve_candidates.is_empty() {
            println!("  CVE Candidates:");
            for cve in &result.patch_analysis.cve_candidates {
                println!("    - {}", cve.red());
            }
        }

        if !result.patch_analysis.bypass_opportunities.is_empty() {
            println!("  Bypass Opportunities:");
            for opp in &result.patch_analysis.bypass_opportunities {
                println!("    - {}", opp.yellow());
            }
        }

        if !result.patch_analysis.variant_analysis.is_empty() {
            println!("  Variant Analysis Recommendations:");
            for variant in &result.patch_analysis.variant_analysis {
                println!("    - {}", variant.bright_black());
            }
        }

        println!("\n{}", "Control Flow Changes:".bold().magenta());
        for cfg_change in &result.control_flow_changes {
            println!(
                "  • {} in {}",
                cfg_change.change_type.cyan(),
                cfg_change.function.yellow()
            );
            println!("    {}", cfg_change.description);
            println!(
                "    Impact: {}",
                if cfg_change.security_impact.contains("CRITICAL") {
                    cfg_change.security_impact.red().bold()
                } else if cfg_change.security_impact.contains("Positive") {
                    cfg_change.security_impact.green()
                } else {
                    cfg_change.security_impact.yellow()
                }
            );
        }

        println!("\n{}", "Attack Surface Delta:".bold().magenta());
        if !result.attack_surface_delta.new_entry_points.is_empty() {
            println!(
                "  New Entry Points: {}",
                result
                    .attack_surface_delta
                    .new_entry_points
                    .join(", ")
                    .green()
            );
        }
        if !result.attack_surface_delta.removed_entry_points.is_empty() {
            println!(
                "  Removed Entry Points: {}",
                result
                    .attack_surface_delta
                    .removed_entry_points
                    .join(", ")
                    .red()
            );
        }
        if !result
            .attack_surface_delta
            .new_dangerous_functions
            .is_empty()
        {
            println!(
                "  New Dangerous Functions: {}",
                result
                    .attack_surface_delta
                    .new_dangerous_functions
                    .join(", ")
                    .red()
                    .bold()
            );
        }
        if !result.attack_surface_delta.removed_mitigations.is_empty() {
            println!(
                "  Removed Mitigations: {}",
                result
                    .attack_surface_delta
                    .removed_mitigations
                    .join(", ")
                    .red()
                    .bold()
            );
        }
        if !result.attack_surface_delta.added_mitigations.is_empty() {
            println!(
                "  Added Mitigations: {}",
                result
                    .attack_surface_delta
                    .added_mitigations
                    .join(", ")
                    .green()
            );
        }

        println!(
            "\n{}\n",
            "═══════════════════════════════════════════════════════════".cyan()
        );
    }
}
