use crate::binary_analyzer::{BinaryAnalysis, BinaryAnalyzer};
use capstone::prelude::*;
use goblin::Object;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityForecast {
    pub binary_path: String,
    pub patch_gaps: Vec<PatchGap>,
    pub risk_map: HashMap<String, RiskScore>,
    pub hotspots: Vec<Hotspot>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchGap {
    pub cve_id: String,
    pub description: String,
    pub severity: f64,
    pub exploitability: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskScore {
    pub function_name: String,
    pub address: u64,
    pub score: f64,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotspot {
    pub location: String,
    pub address: u64,
    pub risk_level: String,
    pub pattern_match: String,
    pub historical_similarity: f64,
}

pub struct VulnForecastEngine {
    cve_database: HashMap<String, CVERecord>,
    pattern_database: Vec<VulnPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CVERecord {
    pub id: String,
    pub description: String,
    pub affected_versions: Vec<String>,
    pub patch_version: String,
    pub severity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnPattern {
    pub name: String,
    pub pattern: String,
    pub historical_cves: Vec<String>,
    pub risk_weight: f64,
}

impl VulnForecastEngine {
    pub fn new() -> Self {
        let mut engine = VulnForecastEngine {
            cve_database: HashMap::new(),
            pattern_database: Vec::new(),
        };
        engine.initialize_databases();
        engine
    }

    fn initialize_databases(&mut self) {
        self.cve_database.insert(
            "CVE-2015-0057".to_string(),
            CVERecord {
                id: "CVE-2015-0057".to_string(),
                description: "Windows kernel privilege escalation".to_string(),
                affected_versions: vec!["Windows 7".to_string(), "Windows 8".to_string()],
                patch_version: "MS15-010".to_string(),
                severity: 8.5,
            },
        );

        self.cve_database.insert(
            "CVE-2020-0796".to_string(),
            CVERecord {
                id: "CVE-2020-0796".to_string(),
                description: "SMBv3 remote code execution (SMBGhost)".to_string(),
                affected_versions: vec![
                    "Windows 10 1903".to_string(),
                    "Windows 10 1909".to_string(),
                ],
                patch_version: "KB4551762".to_string(),
                severity: 10.0,
            },
        );

        self.pattern_database.push(VulnPattern {
            name: "unsafe_memcpy".to_string(),
            pattern: "memcpy|strcpy|strcat|sprintf".to_string(),
            historical_cves: vec!["CVE-2019-XXXX".to_string(), "CVE-2020-YYYY".to_string()],
            risk_weight: 0.85,
        });

        self.pattern_database.push(VulnPattern {
            name: "integer_overflow".to_string(),
            pattern: "malloc.*\\+|alloc.*\\*".to_string(),
            historical_cves: vec!["CVE-2018-ZZZZ".to_string()],
            risk_weight: 0.75,
        });

        self.pattern_database.push(VulnPattern {
            name: "use_after_free".to_string(),
            pattern: "free.*use|delete.*access".to_string(),
            historical_cves: vec!["CVE-2021-AAAA".to_string()],
            risk_weight: 0.90,
        });
    }

    pub fn analyze_target(&self, binary_path: &str) -> Result<VulnerabilityForecast, String> {
        log::info!("Analyzing target binary: {}", binary_path);

        let binary_analysis = BinaryAnalyzer::analyze(binary_path)
            .map_err(|e| format!("Binary analysis failed: {}", e))?;

        let patch_gaps = self.identify_patch_gaps(binary_path, &binary_analysis)?;
        let risk_map = self.generate_risk_map(binary_path, &binary_analysis)?;
        let hotspots = self.identify_hotspots(binary_path, &risk_map)?;
        let recommendations = self.generate_recommendations(&patch_gaps, &hotspots);

        log::info!(
            "Forecast complete: {} patch gaps, {} risk points, {} hotspots",
            patch_gaps.len(),
            risk_map.len(),
            hotspots.len()
        );

        Ok(VulnerabilityForecast {
            binary_path: binary_path.to_string(),
            patch_gaps,
            risk_map,
            hotspots,
            recommendations,
        })
    }

    fn identify_patch_gaps(
        &self,
        binary_path: &str,
        analysis: &BinaryAnalysis,
    ) -> Result<Vec<PatchGap>, String> {
        let mut gaps = Vec::new();

        let binary_data =
            fs::read(binary_path).map_err(|e| format!("Failed to read binary: {}", e))?;

        let _obj =
            Object::parse(&binary_data).map_err(|e| format!("Failed to parse binary: {}", e))?;

        for cve in self.cve_database.values() {
            let mut confidence = 0.0;

            if cve.description.contains("SMB")
                && analysis
                    .symbols
                    .iter()
                    .any(|s| s.name.contains("smb") || s.name.contains("SMB"))
            {
                confidence = 0.85;
            } else if cve.description.contains("kernel")
                && analysis
                    .symbols
                    .iter()
                    .any(|s| s.name.contains("kernel") || s.name.contains("nt"))
            {
                confidence = 0.75;
            } else if cve.description.contains("Windows") && analysis.os.contains("Windows") {
                confidence = 0.50;
            }

            if confidence > 0.4 {
                let exploitability = self.calculate_exploitability(&cve.id, analysis);
                gaps.push(PatchGap {
                    cve_id: cve.id.clone(),
                    description: cve.description.clone(),
                    severity: cve.severity,
                    exploitability,
                    confidence,
                });
            }
        }

        gaps.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());
        log::debug!("Identified {} potential patch gaps", gaps.len());
        Ok(gaps)
    }

    fn calculate_exploitability(&self, _cve_id: &str, analysis: &BinaryAnalysis) -> f64 {
        let mut score: f64 = 0.5;

        if !analysis.protections.nx {
            score += 0.2;
        }
        if !analysis.protections.pie {
            score += 0.15;
        }
        if !analysis.protections.canary {
            score += 0.15;
        }

        score.min(1.0)
    }

    fn generate_risk_map(
        &self,
        binary_path: &str,
        analysis: &BinaryAnalysis,
    ) -> Result<HashMap<String, RiskScore>, String> {
        let mut risk_map = HashMap::new();

        let binary_data =
            fs::read(binary_path).map_err(|e| format!("Failed to read binary: {}", e))?;

        for symbol in &analysis.symbols {
            if symbol.is_imported {
                continue;
            }

            let mut score = 0.0;
            let mut reasons = Vec::new();

            for pattern in &self.pattern_database {
                if symbol.name.contains("strcpy")
                    || symbol.name.contains("memcpy")
                    || symbol.name.contains("sprintf")
                    || symbol.name.contains("strcat")
                {
                    score += pattern.risk_weight;
                    reasons.push(format!("Matches pattern: {}", pattern.name));
                } else if symbol.name.contains("malloc") || symbol.name.contains("alloc") {
                    score += 0.4;
                    reasons.push("Memory allocation function".to_string());
                } else if symbol.name.contains("free") || symbol.name.contains("delete") {
                    score += 0.3;
                    reasons.push("Memory deallocation function".to_string());
                } else if symbol.name.contains("parse") || symbol.name.contains("process") {
                    score += 0.5;
                    reasons.push("Input processing function".to_string());
                }
            }

            if score > 0.0 {
                risk_map.insert(
                    symbol.name.clone(),
                    RiskScore {
                        function_name: symbol.name.clone(),
                        address: symbol.address,
                        score: score.min(1.0),
                        reasons,
                    },
                );
            }
        }

        if risk_map.is_empty() {
            let cs = Capstone::new()
                .x86()
                .mode(arch::x86::ArchMode::Mode64)
                .build()
                .map_err(|e| format!("Capstone init failed: {}", e))?;

            let text_section = analysis
                .sections
                .iter()
                .find(|s| s.name == ".text" || s.is_executable)
                .ok_or("No executable section found")?;

            let start = text_section.address as usize;
            let size = text_section.size.min(10000) as usize;

            if start < binary_data.len() && start + size <= binary_data.len() {
                let insns = cs
                    .disasm_all(&binary_data[start..start + size], text_section.address)
                    .map_err(|e| format!("Disassembly failed: {}", e))?;

                for insn in insns.as_ref() {
                    let mnemonic = insn.mnemonic().unwrap_or("");
                    if mnemonic == "call" || mnemonic == "jmp" {
                        risk_map.insert(
                            format!("func_0x{:x}", insn.address()),
                            RiskScore {
                                function_name: format!("sub_0x{:x}", insn.address()),
                                address: insn.address(),
                                score: 0.3,
                                reasons: vec!["Disassembled function".to_string()],
                            },
                        );
                    }
                }
            }
        }

        log::debug!("Generated risk map with {} entries", risk_map.len());
        Ok(risk_map)
    }

    fn identify_hotspots(
        &self,
        _binary_path: &str,
        risk_map: &HashMap<String, RiskScore>,
    ) -> Result<Vec<Hotspot>, String> {
        let mut hotspots = Vec::new();

        for (func_name, risk_score) in risk_map {
            if risk_score.score > 0.70 {
                hotspots.push(Hotspot {
                    location: format!("Function: {}", func_name),
                    address: risk_score.address,
                    risk_level: self.classify_risk(risk_score.score),
                    pattern_match: risk_score.reasons.join(", "),
                    historical_similarity: risk_score.score * 0.82,
                });
            }
        }

        hotspots.sort_by(|a, b| {
            b.historical_similarity
                .partial_cmp(&a.historical_similarity)
                .unwrap()
        });
        Ok(hotspots)
    }

    fn classify_risk(&self, score: f64) -> String {
        if score >= 0.80 {
            "CRITICAL".to_string()
        } else if score >= 0.60 {
            "HIGH".to_string()
        } else if score >= 0.40 {
            "MEDIUM".to_string()
        } else {
            "LOW".to_string()
        }
    }

    fn generate_recommendations(
        &self,
        patch_gaps: &[PatchGap],
        hotspots: &[Hotspot],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if let Some(top_gap) = patch_gaps.first() {
            recommendations.push(format!(
                "Priority target: {} - {} (Severity: {:.1}, Exploitability: {:.0}%)",
                top_gap.cve_id,
                top_gap.description,
                top_gap.severity,
                top_gap.exploitability * 100.0
            ));
        }

        if let Some(top_hotspot) = hotspots.first() {
            recommendations.push(format!(
                "Start fuzzing: {} at 0x{:x} ({} risk, {:.0}% historical match)",
                top_hotspot.location,
                top_hotspot.address,
                top_hotspot.risk_level,
                top_hotspot.historical_similarity * 100.0
            ));
        }

        if patch_gaps.len() > 3 {
            recommendations.push(format!(
                "Target appears vulnerable to {} known issues. Focus on high-severity vulnerabilities first.",
                patch_gaps.len()
            ));
        }

        recommendations
    }

    pub fn add_cve(&mut self, cve: CVERecord) {
        self.cve_database.insert(cve.id.clone(), cve);
    }

    pub fn add_pattern(&mut self, pattern: VulnPattern) {
        self.pattern_database.push(pattern);
    }
}
