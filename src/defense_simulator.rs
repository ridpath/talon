use crate::ast::Command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseProfile {
    pub name: String,
    pub mitigations: Vec<Mitigation>,
    pub detection_rules: Vec<DetectionRule>,
    pub response_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mitigation {
    pub name: String,
    pub enabled: bool,
    pub effectiveness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRule {
    pub pattern: String,
    pub severity: Severity,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResult {
    pub total_attempts: usize,
    pub successful_attempts: usize,
    pub detected_attempts: usize,
    pub blocked_attempts: usize,
    pub success_rate: f64,
    pub detection_rate: f64,
    pub recommendations: Vec<String>,
}

pub struct DefenseSimulator {
    profiles: HashMap<String, DefenseProfile>,
}

impl DefenseSimulator {
    pub fn new() -> Self {
        let mut simulator = DefenseSimulator {
            profiles: HashMap::new(),
        };
        simulator.initialize_profiles();
        simulator
    }

    fn initialize_profiles(&mut self) {
        let windows_hvci = DefenseProfile {
            name: "Windows_11_HVCI".to_string(),
            mitigations: vec![
                Mitigation {
                    name: "Hypervisor-Protected Code Integrity".to_string(),
                    enabled: true,
                    effectiveness: 0.95,
                },
                Mitigation {
                    name: "Kernel Control Flow Guard".to_string(),
                    enabled: true,
                    effectiveness: 0.90,
                },
                Mitigation {
                    name: "Kernel Data Execution Prevention".to_string(),
                    enabled: true,
                    effectiveness: 0.85,
                },
            ],
            detection_rules: vec![
                DetectionRule {
                    pattern: "kernel_memory_write".to_string(),
                    severity: Severity::Critical,
                    confidence: 0.95,
                },
                DetectionRule {
                    pattern: "unsigned_driver_load".to_string(),
                    severity: Severity::Critical,
                    confidence: 0.99,
                },
            ],
            response_actions: vec![
                "terminate_process".to_string(),
                "alert_security_center".to_string(),
            ],
        };

        let selinux = DefenseProfile {
            name: "SELinux_Enforcing".to_string(),
            mitigations: vec![
                Mitigation {
                    name: "Mandatory Access Control".to_string(),
                    enabled: true,
                    effectiveness: 0.92,
                },
                Mitigation {
                    name: "Type Enforcement".to_string(),
                    enabled: true,
                    effectiveness: 0.88,
                },
            ],
            detection_rules: vec![
                DetectionRule {
                    pattern: "unauthorized_file_access".to_string(),
                    severity: Severity::High,
                    confidence: 0.90,
                },
                DetectionRule {
                    pattern: "privilege_escalation".to_string(),
                    severity: Severity::Critical,
                    confidence: 0.93,
                },
            ],
            response_actions: vec!["deny_access".to_string(), "log_violation".to_string()],
        };

        let anticheat = DefenseProfile {
            name: "GameGuard_AntiCheat".to_string(),
            mitigations: vec![
                Mitigation {
                    name: "Memory Integrity Checks".to_string(),
                    enabled: true,
                    effectiveness: 0.80,
                },
                Mitigation {
                    name: "Anti-Debug Protection".to_string(),
                    enabled: true,
                    effectiveness: 0.75,
                },
                Mitigation {
                    name: "Code Injection Detection".to_string(),
                    enabled: true,
                    effectiveness: 0.85,
                },
            ],
            detection_rules: vec![
                DetectionRule {
                    pattern: "debugger_present".to_string(),
                    severity: Severity::High,
                    confidence: 0.88,
                },
                DetectionRule {
                    pattern: "memory_scanner".to_string(),
                    severity: Severity::Medium,
                    confidence: 0.70,
                },
            ],
            response_actions: vec!["terminate_game".to_string(), "ban_account".to_string()],
        };

        self.profiles
            .insert("Windows_11_HVCI".to_string(), windows_hvci);
        self.profiles
            .insert("SELinux_Enforcing".to_string(), selinux);
        self.profiles
            .insert("GameGuard_AntiCheat".to_string(), anticheat);
    }

    pub fn stress_test(
        &self,
        profile_name: &str,
        exploit_commands: &[Command],
        iterations: usize,
    ) -> Result<StressTestResult, String> {
        let profile = self
            .profiles
            .get(profile_name)
            .ok_or_else(|| format!("Defense profile not found: {}", profile_name))?;

        let mut successful = 0;
        let mut detected = 0;
        let mut blocked = 0;

        for _ in 0..iterations {
            let (success, was_detected, was_blocked) =
                self.simulate_attack(profile, exploit_commands);

            if success {
                successful += 1;
            }
            if was_detected {
                detected += 1;
            }
            if was_blocked {
                blocked += 1;
            }
        }

        let success_rate = successful as f64 / iterations as f64;
        let detection_rate = detected as f64 / iterations as f64;

        let recommendations = self.generate_recommendations(
            profile,
            detection_rate,
            blocked as f64 / iterations as f64,
        );

        Ok(StressTestResult {
            total_attempts: iterations,
            successful_attempts: successful,
            detected_attempts: detected,
            blocked_attempts: blocked,
            success_rate,
            detection_rate,
            recommendations,
        })
    }

    fn simulate_attack(
        &self,
        profile: &DefenseProfile,
        commands: &[Command],
    ) -> (bool, bool, bool) {
        let mut detected = false;
        let mut blocked = false;
        let mut suspicious_syscalls = 0;
        let mut dangerous_operations = 0;

        for cmd in commands {
            for mitigation in &profile.mitigations {
                if !mitigation.enabled {
                    continue;
                }

                if self.triggers_mitigation(cmd, &mitigation.name) {
                    dangerous_operations += 1;
                    let detection_threshold = (dangerous_operations as f64 * 0.15).min(1.0);
                    if detection_threshold >= (1.0 - mitigation.effectiveness) {
                        blocked = true;
                        log::debug!(
                            "Mitigation '{}' blocked operation (threshold: {:.2})",
                            mitigation.name,
                            detection_threshold
                        );
                    }
                }
            }

            for rule in &profile.detection_rules {
                if self.matches_detection_rule(cmd, &rule.pattern) {
                    suspicious_syscalls += 1;
                    let detection_score = (suspicious_syscalls as f64 * 0.1).min(1.0);
                    if detection_score >= (1.0 - rule.confidence) {
                        detected = true;
                        log::debug!(
                            "Detection rule '{}' triggered (score: {:.2}, severity: {:?})",
                            rule.pattern,
                            detection_score,
                            rule.severity
                        );
                    }
                }
            }
        }

        let success = !blocked;
        log::info!("Attack simulation: {} suspicious syscalls, {} dangerous operations, blocked={}, detected={}",
                  suspicious_syscalls, dangerous_operations, blocked, detected);
        (success, detected, blocked)
    }

    fn triggers_mitigation(&self, cmd: &Command, mitigation_name: &str) -> bool {
        match mitigation_name {
            "Hypervisor-Protected Code Integrity" => {
                matches!(
                    cmd,
                    Command::WriteFile { .. }
                        | Command::DumpMemory { .. }
                        | Command::RunCommand { .. }
                )
            }
            "Kernel Control Flow Guard" => {
                matches!(cmd, Command::RunCommand { .. } | Command::Connect { .. })
            }
            "Kernel Data Execution Prevention" => {
                matches!(cmd, Command::RunCommand { .. } | Command::ExecuteShellcode)
            }
            "Mandatory Access Control" => {
                matches!(cmd, Command::WriteFile { .. } | Command::ReadFile { .. })
            }
            "Type Enforcement" => {
                matches!(cmd, Command::RunCommand { .. })
            }
            "Memory Integrity Checks" => {
                matches!(cmd, Command::DumpMemory { .. })
            }
            "Anti-Debug Protection" => {
                matches!(cmd, Command::AntiDebugCheck)
            }
            "Code Injection Detection" => {
                matches!(cmd, Command::RunCommand { .. } | Command::ExecuteShellcode)
            }
            _ => false,
        }
    }

    fn matches_detection_rule(&self, cmd: &Command, pattern: &str) -> bool {
        match pattern {
            "kernel_memory_write" => {
                matches!(cmd, Command::DumpMemory { .. })
            }
            "unsigned_driver_load" => {
                matches!(cmd, Command::RunCommand { .. })
            }
            "unauthorized_file_access" => {
                matches!(cmd, Command::ReadFile { .. } | Command::WriteFile { .. })
            }
            "privilege_escalation" => {
                matches!(cmd, Command::RunCommand { .. })
            }
            "debugger_present" => {
                matches!(cmd, Command::AntiDebugCheck)
            }
            "memory_scanner" => {
                matches!(cmd, Command::DumpMemory { .. })
            }
            _ => {
                pattern.contains("memory") && matches!(cmd, Command::DumpMemory { .. })
                    || pattern.contains("debugger") && matches!(cmd, Command::AntiDebugCheck)
                    || pattern.contains("file_access") && matches!(cmd, Command::WriteFile { .. })
            }
        }
    }

    fn generate_recommendations(
        &self,
        profile: &DefenseProfile,
        detection_rate: f64,
        block_rate: f64,
    ) -> Vec<String> {
        let mut recs = Vec::new();

        if block_rate > 0.8 {
            recs.push(format!(
                "High block rate ({:.0}%). Profile '{}' effectively prevents execution:",
                block_rate * 100.0,
                profile.name
            ));

            for mitigation in &profile.mitigations {
                if mitigation.enabled && mitigation.effectiveness > 0.8 {
                    recs.push(format!(
                        "  - {} (effectiveness: {:.0}%) is blocking operations",
                        mitigation.name,
                        mitigation.effectiveness * 100.0
                    ));
                }
            }

            recs.push("\nRecommended mitigations:".to_string());
            recs.push("1. Implement timing delays (100-500ms) between operations".to_string());
            recs.push(
                "2. Use return-oriented programming (ROP) instead of direct shellcode".to_string(),
            );
            recs.push("3. Chain legitimate system calls to avoid detection patterns".to_string());
        } else if block_rate > 0.5 {
            recs.push(format!(
                "Moderate block rate ({:.0}%). Some operations are being blocked:",
                block_rate * 100.0
            ));
            recs.push("1. Analyze which specific commands trigger blocks".to_string());
            recs.push("2. Modify payload encoding to bypass signature detection".to_string());
        } else if block_rate > 0.1 {
            recs.push(format!(
                "Low block rate ({:.0}%). Most operations succeed.",
                block_rate * 100.0
            ));
        } else {
            recs.push("No operations blocked. Exploit bypasses all mitigations.".to_string());
        }

        if detection_rate > 0.8 {
            recs.push(format!(
                "\nHigh detection rate ({:.0}%) - Exploit will be noticed:",
                detection_rate * 100.0
            ));
            for rule in &profile.detection_rules {
                if rule.confidence > 0.8 {
                    recs.push(format!(
                        "  - '{}' rule (confidence: {:.0}%, severity: {:?})",
                        rule.pattern,
                        rule.confidence * 100.0,
                        rule.severity
                    ));
                }
            }
            recs.push("\nEvasion techniques:".to_string());
            recs.push("- Obfuscate memory access patterns".to_string());
            recs.push("- Mimic legitimate process behavior".to_string());
        } else if detection_rate > 0.3 {
            recs.push(format!(
                "\nModerate detection rate ({:.0}%). Stealth could be improved.",
                detection_rate * 100.0
            ));
        } else {
            recs.push(
                "\nLow detection rate. Exploit operates below detection thresholds.".to_string(),
            );
        }

        recs
    }

    pub fn list_profiles(&self) -> Vec<String> {
        self.profiles.keys().cloned().collect()
    }

    pub fn add_custom_profile(&mut self, profile: DefenseProfile) {
        self.profiles.insert(profile.name.clone(), profile);
    }
}
