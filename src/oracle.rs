use crate::binary_analyzer::{BinaryAnalysis, BinaryAnalyzer};
use crate::elf_tools::ElfContext;
use crate::rop_tools::RopChain;
use crate::shellcode_db::{ShellcodeDatabase, ShellcodeEntry};
use serde::{Deserialize, Serialize};
use std::fs;

#[cfg(test)]
use crate::binary_analyzer::BinaryProtections;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VulnerabilityType {
    StackOverflow,
    FormatString,
    IntegerOverflow,
    UseAfterFree,
    HeapOverflow,
    DoubleFree,
    NullPointerDereference,
    RaceCondition,
    CommandInjection,
    PathTraversal,
    UncontrolledRecursion,
}

impl std::fmt::Display for VulnerabilityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VulnerabilityType::StackOverflow => write!(f, "Stack Buffer Overflow"),
            VulnerabilityType::FormatString => write!(f, "Format String Vulnerability"),
            VulnerabilityType::IntegerOverflow => write!(f, "Integer Overflow"),
            VulnerabilityType::UseAfterFree => write!(f, "Use-After-Free"),
            VulnerabilityType::HeapOverflow => write!(f, "Heap Buffer Overflow"),
            VulnerabilityType::DoubleFree => write!(f, "Double Free"),
            VulnerabilityType::NullPointerDereference => write!(f, "Null Pointer Dereference"),
            VulnerabilityType::RaceCondition => write!(f, "Race Condition"),
            VulnerabilityType::CommandInjection => write!(f, "Command Injection"),
            VulnerabilityType::PathTraversal => write!(f, "Path Traversal"),
            VulnerabilityType::UncontrolledRecursion => write!(f, "Uncontrolled Recursion"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityReport {
    pub vuln_type: VulnerabilityType,
    pub location: String,
    pub confidence: f32,
    pub exploitability: Exploitability,
    pub details: String,
    pub suggested_exploit: Option<String>,
    pub gadget_availability: GadgetAvailability,
    pub recommended_shellcode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Exploitability {
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Exploitability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Exploitability::Critical => write!(f, "Critical"),
            Exploitability::High => write!(f, "High"),
            Exploitability::Medium => write!(f, "Medium"),
            Exploitability::Low => write!(f, "Low"),
            Exploitability::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GadgetAvailability {
    pub total_gadgets: usize,
    pub useful_gadgets: usize,
    pub pop_gadgets: usize,
    pub syscall_gadgets: usize,
    pub quality_score: f32,
    pub rop_possible: bool,
}

pub struct VulnerabilityOracle {
    binary_path: String,
    analysis: Option<BinaryAnalysis>,
    elf_context: Option<ElfContext>,
    rop_chain: Option<RopChain>,
    shellcode_db: ShellcodeDatabase,
}

impl std::fmt::Debug for VulnerabilityOracle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulnerabilityOracle")
            .field("binary_path", &self.binary_path)
            .field("analysis", &self.analysis)
            .field("has_elf_context", &self.elf_context.is_some())
            .field("has_rop_chain", &self.rop_chain.is_some())
            .field("shellcode_db_loaded", &true)
            .finish()
    }
}

impl VulnerabilityOracle {
    pub fn new(binary_path: &str) -> Result<Self, String> {
        log::info!("Initializing Vulnerability Oracle for {}", binary_path);

        if !std::path::Path::new(binary_path).exists() {
            return Err(format!("Binary not found: {}", binary_path));
        }

        Ok(VulnerabilityOracle {
            binary_path: binary_path.to_string(),
            analysis: None,
            elf_context: None,
            rop_chain: None,
            shellcode_db: ShellcodeDatabase::new(),
        })
    }

    pub fn analyze_flow(&mut self) -> Result<Vec<VulnerabilityReport>, String> {
        log::info!("Starting vulnerability flow analysis");

        let analysis = BinaryAnalyzer::analyze(&self.binary_path)?;
        self.analysis = Some(analysis.clone());

        let elf_ctx = ElfContext::load(&self.binary_path).ok();
        self.elf_context = elf_ctx;

        let rop_chain = RopChain::new(&self.binary_path).ok();
        self.rop_chain = rop_chain;

        let mut vulnerabilities = Vec::new();

        vulnerabilities.extend(self.detect_stack_overflow()?);
        vulnerabilities.extend(self.detect_format_string()?);
        vulnerabilities.extend(self.detect_integer_overflow()?);
        vulnerabilities.extend(self.detect_use_after_free()?);
        vulnerabilities.extend(self.detect_heap_overflow()?);

        log::info!("Found {} potential vulnerabilities", vulnerabilities.len());

        Ok(vulnerabilities)
    }

    fn detect_stack_overflow(&self) -> Result<Vec<VulnerabilityReport>, String> {
        let mut reports = Vec::new();
        let binary_data = fs::read(&self.binary_path)
            .map_err(|e| format!("Failed to read binary: {}", e))?;

        let dangerous_functions = [
            (b"strcpy" as &[u8], "strcpy() - unbounded copy"),
            (b"strcat", "strcat() - unbounded concatenation"),
            (b"gets", "gets() - no bounds checking"),
            (b"sprintf", "sprintf() - unbounded format"),
            (b"vsprintf", "vsprintf() - unbounded format"),
            (b"scanf", "scanf() - potential overflow"),
            (b"sscanf", "sscanf() - potential overflow"),
        ];

        for (func_name, description) in &dangerous_functions {
            if binary_data
                .windows(func_name.len())
                .any(|window| window == *func_name)
            {
                let gadget_info = self.analyze_gadget_density();
                let exploitability = self.calculate_exploitability(
                    &VulnerabilityType::StackOverflow,
                    &gadget_info,
                );

                let suggested_exploit = if exploitability >= Exploitability::Medium {
                    Some(self.generate_exploit_suggestion(
                        &VulnerabilityType::StackOverflow,
                        &gadget_info,
                    ))
                } else {
                    None
                };

                let recommended_shellcode =
                    self.recommend_shellcode(&VulnerabilityType::StackOverflow, &gadget_info);

                reports.push(VulnerabilityReport {
                    vuln_type: VulnerabilityType::StackOverflow,
                    location: format!("Uses {}", description),
                    confidence: 0.85,
                    exploitability,
                    details: format!(
                        "Binary uses {} which is vulnerable to buffer overflow. \
                         Detected via static analysis of function imports.",
                        description
                    ),
                    suggested_exploit,
                    gadget_availability: gadget_info.clone(),
                    recommended_shellcode,
                });
            }
        }

        Ok(reports)
    }

    fn detect_format_string(&self) -> Result<Vec<VulnerabilityReport>, String> {
        let mut reports = Vec::new();
        let binary_data = fs::read(&self.binary_path)
            .map_err(|e| format!("Failed to read binary: {}", e))?;

        let format_functions = [
            (b"printf" as &[u8], "printf()"),
            (b"fprintf", "fprintf()"),
            (b"sprintf", "sprintf()"),
            (b"snprintf", "snprintf()"),
            (b"vprintf", "vprintf()"),
            (b"vfprintf", "vfprintf()"),
        ];

        for (func_name, description) in &format_functions {
            if binary_data
                .windows(func_name.len())
                .any(|window| window == *func_name)
            {
                let gadget_info = self.analyze_gadget_density();
                let exploitability = self.calculate_exploitability(
                    &VulnerabilityType::FormatString,
                    &gadget_info,
                );

                let suggested_exploit = if exploitability >= Exploitability::Medium {
                    Some(self.generate_exploit_suggestion(
                        &VulnerabilityType::FormatString,
                        &gadget_info,
                    ))
                } else {
                    None
                };

                reports.push(VulnerabilityReport {
                    vuln_type: VulnerabilityType::FormatString,
                    location: format!("Uses {}", description),
                    confidence: 0.70,
                    exploitability,
                    details: format!(
                        "Binary uses {} with potentially user-controlled format string. \
                         This can lead to arbitrary read/write primitives.",
                        description
                    ),
                    suggested_exploit,
                    gadget_availability: gadget_info.clone(),
                    recommended_shellcode: None,
                });
            }
        }

        Ok(reports)
    }

    fn detect_integer_overflow(&self) -> Result<Vec<VulnerabilityReport>, String> {
        let mut reports = Vec::new();
        let binary_data = fs::read(&self.binary_path)
            .map_err(|e| format!("Failed to read binary: {}", e))?;

        let alloc_functions = [
            (b"malloc" as &[u8], "malloc()"),
            (b"calloc", "calloc()"),
            (b"realloc", "realloc()"),
            (b"new", "operator new"),
        ];

        for (func_name, description) in &alloc_functions {
            if binary_data
                .windows(func_name.len())
                .any(|window| window == *func_name)
            {
                let gadget_info = self.analyze_gadget_density();

                reports.push(VulnerabilityReport {
                    vuln_type: VulnerabilityType::IntegerOverflow,
                    location: format!("Uses {}", description),
                    confidence: 0.40,
                    exploitability: Exploitability::Medium,
                    details: format!(
                        "Binary uses {} with potentially unchecked size calculations. \
                         Integer overflow in allocation size can lead to heap corruption.",
                        description
                    ),
                    suggested_exploit: None,
                    gadget_availability: gadget_info,
                    recommended_shellcode: None,
                });
            }
        }

        Ok(reports)
    }

    fn detect_use_after_free(&self) -> Result<Vec<VulnerabilityReport>, String> {
        let mut reports = Vec::new();
        let binary_data = fs::read(&self.binary_path)
            .map_err(|e| format!("Failed to read binary: {}", e))?;

        let has_free = binary_data
            .windows(4)
            .any(|window| window == b"free");
        let has_malloc = binary_data
            .windows(6)
            .any(|window| window == b"malloc");

        if has_free && has_malloc {
            let gadget_info = self.analyze_gadget_density();

            reports.push(VulnerabilityReport {
                vuln_type: VulnerabilityType::UseAfterFree,
                location: "Dynamic memory management".to_string(),
                confidence: 0.35,
                exploitability: Exploitability::Medium,
                details: "Binary uses dynamic memory allocation and deallocation. \
                          Manual analysis required to confirm UAF vulnerability."
                    .to_string(),
                suggested_exploit: None,
                gadget_availability: gadget_info,
                recommended_shellcode: None,
            });
        }

        Ok(reports)
    }

    fn detect_heap_overflow(&self) -> Result<Vec<VulnerabilityReport>, String> {
        let mut reports = Vec::new();
        let binary_data = fs::read(&self.binary_path)
            .map_err(|e| format!("Failed to read binary: {}", e))?;

        let heap_functions = [
            (b"memcpy" as &[u8], "memcpy()"),
            (b"memmove", "memmove()"),
            (b"strncpy", "strncpy()"),
        ];

        let has_malloc = binary_data
            .windows(6)
            .any(|window| window == b"malloc");

        if has_malloc {
            for (func_name, description) in &heap_functions {
                if binary_data
                    .windows(func_name.len())
                    .any(|window| window == *func_name)
                {
                    let gadget_info = self.analyze_gadget_density();

                    reports.push(VulnerabilityReport {
                        vuln_type: VulnerabilityType::HeapOverflow,
                        location: format!("Uses {} with heap allocation", description),
                        confidence: 0.50,
                        exploitability: Exploitability::Medium,
                        details: format!(
                            "Binary uses {} on heap-allocated memory. \
                             Improper bounds checking can lead to heap overflow.",
                            description
                        ),
                        suggested_exploit: None,
                        gadget_availability: gadget_info,
                        recommended_shellcode: None,
                    });
                    break;
                }
            }
        }

        Ok(reports)
    }

    fn analyze_gadget_density(&self) -> GadgetAvailability {
        if let Some(ref rop_chain) = self.rop_chain {
            let total_gadgets = rop_chain.gadgets.len();
            let useful_gadgets = rop_chain
                .gadgets
                .iter()
                .filter(|g| g.quality_score > 50)
                .count();

            let pop_gadgets = rop_chain
                .gadgets
                .iter()
                .filter(|g| {
                    g.instructions
                        .iter()
                        .any(|inst| inst.to_lowercase().starts_with("pop"))
                })
                .count();

            let syscall_gadgets = rop_chain
                .gadgets
                .iter()
                .filter(|g| {
                    g.instructions
                        .iter()
                        .any(|inst| inst.contains("syscall") || inst.contains("int 0x80"))
                })
                .count();

            let quality_score = if total_gadgets > 0 {
                (useful_gadgets as f32 / total_gadgets as f32) * 100.0
            } else {
                0.0
            };

            let rop_possible = pop_gadgets >= 3 && syscall_gadgets >= 1;

            GadgetAvailability {
                total_gadgets,
                useful_gadgets,
                pop_gadgets,
                syscall_gadgets,
                quality_score,
                rop_possible,
            }
        } else {
            GadgetAvailability {
                total_gadgets: 0,
                useful_gadgets: 0,
                pop_gadgets: 0,
                syscall_gadgets: 0,
                quality_score: 0.0,
                rop_possible: false,
            }
        }
    }

    fn calculate_exploitability(
        &self,
        vuln_type: &VulnerabilityType,
        gadget_info: &GadgetAvailability,
    ) -> Exploitability {
        let analysis = self.analysis.as_ref();
        if analysis.is_none() {
            return Exploitability::Low;
        }

        let protections = &analysis.unwrap().protections;

        let mut score = 100.0;

        if protections.nx {
            score -= 20.0;
        }
        if protections.pie {
            score -= 15.0;
        }
        if protections.canary {
            score -= 15.0;
        }
        if protections.aslr {
            score -= 10.0;
        }
        if protections.fortify {
            score -= 10.0;
        }

        match vuln_type {
            VulnerabilityType::StackOverflow => {
                if protections.nx && !gadget_info.rop_possible {
                    score -= 30.0;
                }
            }
            VulnerabilityType::FormatString => {
                score += 10.0;
            }
            _ => {}
        }

        if gadget_info.rop_possible {
            score += 20.0;
        }

        score += gadget_info.quality_score * 0.1;

        match score as i32 {
            90..=100 => Exploitability::Critical,
            70..=89 => Exploitability::High,
            40..=69 => Exploitability::Medium,
            10..=39 => Exploitability::Low,
            _ => Exploitability::None,
        }
    }

    fn generate_exploit_suggestion(
        &self,
        vuln_type: &VulnerabilityType,
        gadget_info: &GadgetAvailability,
    ) -> String {
        let analysis = self.analysis.as_ref();
        if analysis.is_none() {
            return "Insufficient information for exploit suggestion".to_string();
        }

        let protections = &analysis.unwrap().protections;

        match vuln_type {
            VulnerabilityType::StackOverflow => {
                if protections.nx {
                    if gadget_info.rop_possible {
                        format!(
                            "ROP chain exploitation recommended. {} useful gadgets available. \
                             Build chain using pop_rdi, pop_rsi, syscall gadgets.",
                            gadget_info.useful_gadgets
                        )
                    } else {
                        "NX enabled but insufficient gadgets for ROP. Consider ret2libc or \
                         information leak to bypass ASLR."
                            .to_string()
                    }
                } else {
                    "Direct shellcode injection possible (NX disabled). Use stack-based payload."
                        .to_string()
                }
            }
            VulnerabilityType::FormatString => {
                "Format string exploitation: Use %n to write arbitrary values. \
                 Leak stack/libc addresses with %p. Consider GOT overwrite."
                    .to_string()
            }
            _ => format!("Manual analysis required for {} exploitation", vuln_type),
        }
    }

    fn recommend_shellcode(
        &self,
        vuln_type: &VulnerabilityType,
        gadget_info: &GadgetAvailability,
    ) -> Option<String> {
        if *vuln_type != VulnerabilityType::StackOverflow {
            return None;
        }

        let analysis = self.analysis.as_ref()?;

        if analysis.protections.nx || !gadget_info.rop_possible {
            return None;
        }

        let arch = &analysis.architecture;
        let shellcode_name = match arch.as_str() {
            "x86_64" => "x64_execve_sh",
            "i386" => "x86_execve_sh",
            _ => return None,
        };

        Some(shellcode_name.to_string())
    }

    pub fn find_shellcode(
        &self,
        avoid_bytes: &[u8],
        max_size: Option<usize>,
        arch: Option<&str>,
    ) -> Result<Vec<ShellcodeEntry>, String> {
        let analysis = self.analysis.as_ref();
        let target_arch = arch.or_else(|| analysis.map(|a| a.architecture.as_str()));

        let candidates: Vec<&ShellcodeEntry> = if let Some(arch_str) = target_arch {
            self.shellcode_db.list_by_arch(arch_str)
        } else {
            self.shellcode_db.list()
        };

        let mut results = Vec::new();

        for shellcode in candidates {
            let has_badchars = avoid_bytes
                .iter()
                .any(|&bad| shellcode.bytes.contains(&bad));

            if has_badchars {
                continue;
            }

            if let Some(max) = max_size {
                if shellcode.size > max {
                    continue;
                }
            }

            results.push(shellcode.clone());
        }

        results.sort_by_key(|sc| sc.size);

        log::info!(
            "Found {} shellcodes matching constraints (avoid: {:?}, max_size: {:?})",
            results.len(),
            avoid_bytes,
            max_size
        );

        Ok(results)
    }

    pub fn calculate_reliability_score(&self, report: &VulnerabilityReport) -> f32 {
        let mut score = report.confidence;

        match report.exploitability {
            Exploitability::Critical => score *= 1.2,
            Exploitability::High => score *= 1.1,
            Exploitability::Medium => score *= 1.0,
            Exploitability::Low => score *= 0.8,
            Exploitability::None => score *= 0.5,
        }

        if report.gadget_availability.rop_possible {
            score *= 1.1;
        }

        if report.suggested_exploit.is_some() {
            score *= 1.05;
        }

        if report.recommended_shellcode.is_some() {
            score *= 1.1;
        }

        score.min(1.0)
    }

    pub fn generate_report_summary(&self, reports: &[VulnerabilityReport]) -> String {
        let mut summary = String::new();
        summary.push_str("Vulnerability Analysis Report\n");
        summary.push_str(&"=".repeat(60));
        summary.push('\n');

        if let Some(analysis) = &self.analysis {
            summary.push_str(&format!("Binary: {}\n", self.binary_path));
            summary.push_str(&format!("Architecture: {}\n", analysis.architecture));
            summary.push_str(&format!("Bit Width: {}-bit\n", analysis.bitness));
            summary.push_str("\nProtections:\n");
            summary.push_str(&format!("  NX: {}\n", analysis.protections.nx));
            summary.push_str(&format!("  PIE: {}\n", analysis.protections.pie));
            summary.push_str(&format!("  Canary: {}\n", analysis.protections.canary));
            summary.push_str(&format!("  ASLR: {}\n", analysis.protections.aslr));
            summary.push_str(&format!("  FORTIFY: {}\n", analysis.protections.fortify));
            summary.push('\n');
        }

        summary.push_str(&format!("Total Vulnerabilities Found: {}\n\n", reports.len()));

        for (idx, report) in reports.iter().enumerate() {
            let reliability = self.calculate_reliability_score(report);
            summary.push_str(&format!("{}. {}\n", idx + 1, report.vuln_type));
            summary.push_str(&format!("   Location: {}\n", report.location));
            summary.push_str(&format!(
                "   Confidence: {:.0}%\n",
                report.confidence * 100.0
            ));
            summary.push_str(&format!("   Exploitability: {}\n", report.exploitability));
            summary.push_str(&format!("   Reliability: {:.0}%\n", reliability * 100.0));
            summary.push_str(&format!("   Details: {}\n", report.details));

            if let Some(ref exploit) = report.suggested_exploit {
                summary.push_str(&format!("   Suggested Exploit: {}\n", exploit));
            }

            summary.push_str(&format!(
                "   Gadgets: {} total, {} useful (ROP possible: {})\n",
                report.gadget_availability.total_gadgets,
                report.gadget_availability.useful_gadgets,
                report.gadget_availability.rop_possible
            ));

            if let Some(ref shellcode) = report.recommended_shellcode {
                summary.push_str(&format!("   Recommended Shellcode: {}\n", shellcode));
            }

            summary.push('\n');
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerability_type_display() {
        assert_eq!(
            VulnerabilityType::StackOverflow.to_string(),
            "Stack Buffer Overflow"
        );
        assert_eq!(
            VulnerabilityType::FormatString.to_string(),
            "Format String Vulnerability"
        );
    }

    #[test]
    fn test_exploitability_ordering() {
        assert!(Exploitability::Critical > Exploitability::High);
        assert!(Exploitability::High > Exploitability::Medium);
        assert!(Exploitability::Medium > Exploitability::Low);
        assert!(Exploitability::Low > Exploitability::None);
    }

    #[test]
    fn test_gadget_availability_score() {
        let availability = GadgetAvailability {
            total_gadgets: 100,
            useful_gadgets: 75,
            pop_gadgets: 10,
            syscall_gadgets: 2,
            quality_score: 75.0,
            rop_possible: true,
        };

        assert_eq!(availability.quality_score, 75.0);
        assert!(availability.rop_possible);
    }

    #[test]
    fn test_find_shellcode_with_constraints() {
        let oracle = VulnerabilityOracle {
            binary_path: "test".to_string(),
            analysis: None,
            elf_context: None,
            rop_chain: None,
            shellcode_db: ShellcodeDatabase::new(),
        };

        let avoid = vec![0x00, 0x0a];
        let result = oracle.find_shellcode(&avoid, Some(100), Some("x86-64"));

        assert!(result.is_ok());
    }

    #[test]
    fn test_reliability_score_calculation() {
        let oracle = VulnerabilityOracle {
            binary_path: "test".to_string(),
            analysis: None,
            elf_context: None,
            rop_chain: None,
            shellcode_db: ShellcodeDatabase::new(),
        };

        let report = VulnerabilityReport {
            vuln_type: VulnerabilityType::StackOverflow,
            location: "test".to_string(),
            confidence: 0.85,
            exploitability: Exploitability::High,
            details: "test".to_string(),
            suggested_exploit: Some("test".to_string()),
            gadget_availability: GadgetAvailability {
                total_gadgets: 100,
                useful_gadgets: 75,
                pop_gadgets: 10,
                syscall_gadgets: 2,
                quality_score: 75.0,
                rop_possible: true,
            },
            recommended_shellcode: Some("x64_execve_sh".to_string()),
        };

        let score = oracle.calculate_reliability_score(&report);
        assert!(score > 0.8);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_oracle_creation_with_invalid_path() {
        let result = VulnerabilityOracle::new("/nonexistent/binary");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Binary not found"));
    }

    #[test]
    fn test_exploitability_calculation_with_protections() {
        let oracle = VulnerabilityOracle {
            binary_path: "test".to_string(),
            analysis: Some(BinaryAnalysis {
                architecture: "x86_64".to_string(),
                os: "Linux".to_string(),
                bitness: 64,
                endianness: "little".to_string(),
                protections: BinaryProtections {
                    nx: true,
                    pie: true,
                    relro: crate::binary_analyzer::RelroLevel::Full,
                    canary: true,
                    aslr: true,
                    fortify: true,
                },
                sections: vec![],
                symbols: vec![],
                entry_point: 0,
                base_address: 0,
            }),
            elf_context: None,
            rop_chain: None,
            shellcode_db: ShellcodeDatabase::new(),
        };

        let gadget_info = GadgetAvailability {
            total_gadgets: 50,
            useful_gadgets: 10,
            pop_gadgets: 3,
            syscall_gadgets: 1,
            quality_score: 20.0,
            rop_possible: true,
        };

        let exploitability = oracle.calculate_exploitability(
            &VulnerabilityType::StackOverflow,
            &gadget_info,
        );

        assert!(
            exploitability == Exploitability::Low || exploitability == Exploitability::Medium
        );
    }
}
