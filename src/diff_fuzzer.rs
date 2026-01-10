// ═══════════════════════════════════════════════════════════════════════════
// DIFFERENTIAL FUZZING - 1-DAY/0-DAY DISCOVERY
// Advanced patch analysis and behavioral divergence detection
// ═══════════════════════════════════════════════════════════════════════════

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentialFuzzer {
    pub target_old: String,
    pub target_new: String,
    pub corpus: Vec<Vec<u8>>,
    pub crashes: Vec<DivergenceCase>,
    pub iterations: u64,
    pub timeout_ms: u64,
    pub detect_modes: Vec<DetectionMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionMode {
    CrashesOnlyInOld,
    CrashesOnlyInNew,
    BehaviorChange,
    OutputDivergence,
    TimingDivergence,
    SanitizerViolations,
    ReturnCodeChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceCase {
    pub input: Vec<u8>,
    pub old_result: ExecutionResult,
    pub new_result: ExecutionResult,
    pub divergence_type: DivergenceType,
    pub severity: Severity,
    pub poc_generated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub crashed: bool,
    pub timeout: bool,
    pub execution_time_ms: u64,
    pub asan_violation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DivergenceType {
    CrashIntroduced,
    CrashFixed,
    OutputChanged,
    BehaviorChanged,
    TimingChanged,
    MemorySafetyViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl DifferentialFuzzer {
    pub fn new(target_old: String, target_new: String) -> Self {
        println!("[DIFF-FUZZ] Initializing differential fuzzer");
        println!("[DIFF-FUZZ]   Old: {}", target_old);
        println!("[DIFF-FUZZ]   New: {}", target_new);
        
        DifferentialFuzzer {
            target_old,
            target_new,
            corpus: Vec::new(),
            crashes: Vec::new(),
            iterations: 100000,
            timeout_ms: 1000,
            detect_modes: vec![
                DetectionMode::CrashesOnlyInOld,
                DetectionMode::BehaviorChange,
                DetectionMode::SanitizerViolations,
            ],
        }
    }
    
    pub fn load_corpus(&mut self, corpus_path: &str) -> Result<(), String> {
        println!("[DIFF-FUZZ] Loading corpus from: {}", corpus_path);
        
        let path = Path::new(corpus_path);
        
        if corpus_path.contains('*') {
            let parent = path.parent().unwrap_or(Path::new("."));
            let pattern = path.file_name().unwrap().to_str().unwrap();
            
            if let Ok(entries) = fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    if let Some(name_str) = file_name.to_str() {
                        if name_str.ends_with(&pattern.replace("*", "")) {
                            if let Ok(data) = fs::read(entry.path()) {
                                self.corpus.push(data);
                            }
                        }
                    }
                }
            }
        } else if path.is_file() {
            let data = fs::read(path).map_err(|e| e.to_string())?;
            self.corpus.push(data);
        } else if path.is_dir() {
            for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                if entry.path().is_file() {
                    let data = fs::read(entry.path()).map_err(|e| e.to_string())?;
                    self.corpus.push(data);
                }
            }
        }
        
        println!("[DIFF-FUZZ] [OK] Loaded {} seeds", self.corpus.len());
        Ok(())
    }
    
    pub fn set_iterations(&mut self, iterations: u64) {
        self.iterations = iterations;
    }
    
    pub fn set_timeout(&mut self, timeout_ms: u64) {
        self.timeout_ms = timeout_ms;
    }
    
    pub fn add_detection_mode(&mut self, mode: DetectionMode) {
        self.detect_modes.push(mode);
    }
    
    pub fn fuzz(&mut self) -> Result<(), String> {
        println!("[DIFF-FUZZ] Starting differential fuzzing");
        println!("[DIFF-FUZZ]   Iterations: {}", self.iterations);
        println!("[DIFF-FUZZ]   Timeout: {}ms", self.timeout_ms);
        println!("[DIFF-FUZZ]   Detection modes: {}", self.detect_modes.len());
        
        if self.corpus.is_empty() {
            return Err("Corpus is empty - load seeds first".to_string());
        }
        
        let mut divergences_found = 0;
        let mut total_executions = 0;
        
        for i in 0..self.iterations {
            let input = self.generate_input();
            
            let old_result = self.execute_target(&self.target_old.clone(), &input)?;
            let new_result = self.execute_target(&self.target_new.clone(), &input)?;
            
            total_executions += 2;
            
            if let Some(divergence) = self.analyze_divergence(&input, &old_result, &new_result) {
                divergences_found += 1;
                self.crashes.push(divergence.clone());
                
                println!("\n[DIFF-FUZZ] DIVERGENCE FOUND (#{})!", divergences_found);
                println!("[DIFF-FUZZ]   Type: {:?}", divergence.divergence_type);
                println!("[DIFF-FUZZ]   Severity: {:?}", divergence.severity);
                
                self.save_divergence(&divergence, divergences_found)?;
                
                if divergence.severity >= Severity::High {
                    self.generate_poc(&divergence, divergences_found)?;
                }
            }
            
            if (i + 1) % 1000 == 0 {
                println!("[DIFF-FUZZ] Progress: {}/{} iterations, {} divergences found", 
                         i + 1, self.iterations, divergences_found);
            }
        }
        
        println!("\n[DIFF-FUZZ] [OK] Fuzzing complete!");
        println!("[DIFF-FUZZ]   Total executions: {}", total_executions);
        println!("[DIFF-FUZZ]   Divergences found: {}", divergences_found);
        
        self.print_summary();
        
        Ok(())
    }
    
    fn generate_input(&self) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        
        let base_input = &self.corpus[rng.gen_range(0..self.corpus.len())];
        let mut input = base_input.clone();
        
        let mutation_count = rng.gen_range(1..=5);
        for _ in 0..mutation_count {
            let mutation_type = rng.gen_range(0..6);
            
            match mutation_type {
                0 => {
                    if !input.is_empty() {
                        let idx = rng.gen_range(0..input.len());
                        input[idx] = rng.gen();
                    }
                }
                1 => {
                    if input.len() >= 2 {
                        let idx = rng.gen_range(0..input.len() - 1);
                        input.swap(idx, idx + 1);
                    }
                }
                2 => {
                    if !input.is_empty() {
                        let idx = rng.gen_range(0..input.len());
                        input.remove(idx);
                    }
                }
                3 => {
                    let idx = rng.gen_range(0..=input.len());
                    input.insert(idx, rng.gen());
                }
                4 => {
                    if !input.is_empty() {
                        let idx = rng.gen_range(0..input.len());
                        let val = input[idx];
                        input[idx] = val.wrapping_add(rng.gen_range(1..=16));
                    }
                }
                5 => {
                    let interesting = [0x00, 0xFF, 0x7F, 0x80, 0x41, 0x61];
                    if !input.is_empty() {
                        let idx = rng.gen_range(0..input.len());
                        input[idx] = interesting[rng.gen_range(0..interesting.len())];
                    }
                }
                _ => {}
            }
        }
        
        input
    }
    
    fn execute_target(&self, target: &str, input: &Vec<u8>) -> Result<ExecutionResult, String> {
        let start = Instant::now();
        
        let mut child = Command::new(target)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", target, e))?;
        
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(input);
        }
        
        let timeout = Duration::from_millis(self.timeout_ms);
        let mut timeout_occurred = false;
        
        std::thread::sleep(Duration::from_millis(10));
        
        let output = match child.try_wait() {
            Ok(Some(_)) => child.wait_with_output().map_err(|e| e.to_string())?,
            Ok(None) => {
                std::thread::sleep(timeout);
                match child.try_wait() {
                    Ok(Some(_)) => child.wait_with_output().map_err(|e| e.to_string())?,
                    Ok(None) => {
                        let _ = child.kill();
                        timeout_occurred = true;
                        child.wait_with_output().map_err(|e| e.to_string())?
                    }
                    Err(e) => return Err(e.to_string()),
                }
            }
            Err(e) => return Err(e.to_string()),
        };
        
        let execution_time = start.elapsed().as_millis() as u64;
        
        let exit_code = output.status.code();
        let crashed = exit_code.is_some() && (exit_code.unwrap() < 0 || exit_code.unwrap() > 128);
        
        let asan_violation = String::from_utf8_lossy(&output.stderr)
            .contains("AddressSanitizer") || 
            String::from_utf8_lossy(&output.stderr).contains("ASAN");
        
        Ok(ExecutionResult {
            exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            crashed,
            timeout: timeout_occurred,
            execution_time_ms: execution_time,
            asan_violation,
        })
    }
    
    fn analyze_divergence(
        &self,
        input: &Vec<u8>,
        old_result: &ExecutionResult,
        new_result: &ExecutionResult,
    ) -> Option<DivergenceCase> {
        for mode in &self.detect_modes {
            match mode {
                DetectionMode::CrashesOnlyInOld => {
                    if old_result.crashed && !new_result.crashed {
                        return Some(DivergenceCase {
                            input: input.clone(),
                            old_result: old_result.clone(),
                            new_result: new_result.clone(),
                            divergence_type: DivergenceType::CrashFixed,
                            severity: Severity::High,
                            poc_generated: false,
                        });
                    }
                }
                DetectionMode::CrashesOnlyInNew => {
                    if !old_result.crashed && new_result.crashed {
                        return Some(DivergenceCase {
                            input: input.clone(),
                            old_result: old_result.clone(),
                            new_result: new_result.clone(),
                            divergence_type: DivergenceType::CrashIntroduced,
                            severity: Severity::Critical,
                            poc_generated: false,
                        });
                    }
                }
                DetectionMode::BehaviorChange => {
                    if old_result.exit_code != new_result.exit_code {
                        return Some(DivergenceCase {
                            input: input.clone(),
                            old_result: old_result.clone(),
                            new_result: new_result.clone(),
                            divergence_type: DivergenceType::BehaviorChanged,
                            severity: Severity::Medium,
                            poc_generated: false,
                        });
                    }
                }
                DetectionMode::OutputDivergence => {
                    if old_result.stdout != new_result.stdout {
                        return Some(DivergenceCase {
                            input: input.clone(),
                            old_result: old_result.clone(),
                            new_result: new_result.clone(),
                            divergence_type: DivergenceType::OutputChanged,
                            severity: Severity::Low,
                            poc_generated: false,
                        });
                    }
                }
                DetectionMode::SanitizerViolations => {
                    if new_result.asan_violation && !old_result.asan_violation {
                        return Some(DivergenceCase {
                            input: input.clone(),
                            old_result: old_result.clone(),
                            new_result: new_result.clone(),
                            divergence_type: DivergenceType::MemorySafetyViolation,
                            severity: Severity::Critical,
                            poc_generated: false,
                        });
                    }
                }
                DetectionMode::TimingDivergence => {
                    let time_diff = (old_result.execution_time_ms as i64 
                                    - new_result.execution_time_ms as i64).abs();
                    
                    if time_diff > 100 {
                        return Some(DivergenceCase {
                            input: input.clone(),
                            old_result: old_result.clone(),
                            new_result: new_result.clone(),
                            divergence_type: DivergenceType::TimingChanged,
                            severity: Severity::Info,
                            poc_generated: false,
                        });
                    }
                }
                DetectionMode::ReturnCodeChange => {
                    if old_result.exit_code != new_result.exit_code {
                        return Some(DivergenceCase {
                            input: input.clone(),
                            old_result: old_result.clone(),
                            new_result: new_result.clone(),
                            divergence_type: DivergenceType::BehaviorChanged,
                            severity: Severity::Medium,
                            poc_generated: false,
                        });
                    }
                }
            }
        }
        
        None
    }
    
    fn save_divergence(&self, divergence: &DivergenceCase, id: usize) -> Result<(), String> {
        let filename = format!("divergence_{:04}.bin", id);
        fs::write(&filename, &divergence.input)
            .map_err(|e| format!("Failed to save divergence: {}", e))?;
        
        let report = format!(
            "DIVERGENCE REPORT #{}\n\
             ═══════════════════════════════════════════════════════════════\n\
             Type: {:?}\n\
             Severity: {:?}\n\
             Input size: {} bytes\n\
             Input file: {}\n\
             \n\
             OLD VERSION RESULT:\n\
             Exit code: {:?}\n\
             Crashed: {}\n\
             Timeout: {}\n\
             Execution time: {}ms\n\
             ASAN violation: {}\n\
             Stderr: {}\n\
             \n\
             NEW VERSION RESULT:\n\
             Exit code: {:?}\n\
             Crashed: {}\n\
             Timeout: {}\n\
             Execution time: {}ms\n\
             ASAN violation: {}\n\
             Stderr: {}\n\
             ═══════════════════════════════════════════════════════════════\n",
            id,
            divergence.divergence_type,
            divergence.severity,
            divergence.input.len(),
            filename,
            divergence.old_result.exit_code,
            divergence.old_result.crashed,
            divergence.old_result.timeout,
            divergence.old_result.execution_time_ms,
            divergence.old_result.asan_violation,
            String::from_utf8_lossy(&divergence.old_result.stderr),
            divergence.new_result.exit_code,
            divergence.new_result.crashed,
            divergence.new_result.timeout,
            divergence.new_result.execution_time_ms,
            divergence.new_result.asan_violation,
            String::from_utf8_lossy(&divergence.new_result.stderr)
        );
        
        let report_filename = format!("divergence_{:04}_report.txt", id);
        fs::write(&report_filename, report)
            .map_err(|e| format!("Failed to save report: {}", e))?;
        
        Ok(())
    }
    
    fn generate_poc(&self, divergence: &DivergenceCase, id: usize) -> Result<(), String> {
        let poc_code = format!(
            r#"#!/usr/bin/env python3
# Auto-generated PoC for Divergence #{}
# Type: {:?}
# Severity: {:?}

import subprocess
import sys

def exploit():
    # Input that triggers divergence
    payload = {}
    
    print("[*] Running exploit...")
    print(f"[*] Payload size: {{len(payload)}} bytes")
    
    # Test on old version
    print("\n[*] Testing OLD version: {}")
    try:
        p = subprocess.Popen(
            ["./{}"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        stdout, stderr = p.communicate(input=payload, timeout=5)
        print(f"[*] Old version exit code: {{p.returncode}}")
        if stderr:
            print(f"[!] Old version stderr:\n{{stderr.decode()}}")
    except subprocess.TimeoutExpired:
        print("[!] Old version TIMEOUT")
    except Exception as e:
        print(f"[!] Old version error: {{e}}")
    
    # Test on new version
    print("\n[*] Testing NEW version: {}")
    try:
        p = subprocess.Popen(
            ["./{}"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        stdout, stderr = p.communicate(input=payload, timeout=5)
        print(f"[*] New version exit code: {{p.returncode}}")
        if stderr:
            print(f"[!] New version stderr:\n{{stderr.decode()}}")
    except subprocess.TimeoutExpired:
        print("[!] New version TIMEOUT")
    except Exception as e:
        print(f"[!] New version error: {{e}}")

if __name__ == "__main__":
    exploit()
"#,
            id,
            divergence.divergence_type,
            divergence.severity,
            format!("bytes({:?})", divergence.input),
            self.target_old,
            self.target_old,
            self.target_new,
            self.target_new
        );
        
        let poc_filename = format!("poc_{:04}.py", id);
        fs::write(&poc_filename, poc_code)
            .map_err(|e| format!("Failed to generate PoC: {}", e))?;
        
        println!("[DIFF-FUZZ] PoC generated: {}", poc_filename);
        
        Ok(())
    }
    
    fn print_summary(&self) {
        println!("\n╔═══════════════════════════════════════════════════════════════╗");
        println!("║               DIFFERENTIAL FUZZING SUMMARY                    ║");
        println!("╠═══════════════════════════════════════════════════════════════╣");
        println!("║ Total divergences found: {:>36} ║", self.crashes.len());
        
        let mut by_type: HashMap<String, usize> = HashMap::new();
        let mut by_severity: HashMap<String, usize> = HashMap::new();
        
        for crash in &self.crashes {
            let type_key = format!("{:?}", crash.divergence_type);
            *by_type.entry(type_key).or_insert(0) += 1;
            
            let severity_key = format!("{:?}", crash.severity);
            *by_severity.entry(severity_key).or_insert(0) += 1;
        }
        
        println!("║                                                               ║");
        println!("║ By Type:                                                      ║");
        for (dtype, count) in by_type.iter() {
            println!("║   {:32} {:>26} ║", dtype, count);
        }
        
        println!("║                                                               ║");
        println!("║ By Severity:                                                  ║");
        for (sev, count) in by_severity.iter() {
            println!("║   {:32} {:>26} ║", sev, count);
        }
        
        println!("╚═══════════════════════════════════════════════════════════════╝\n");
    }
    
    pub fn save_report(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        
        fs::write(path, json)
            .map_err(|e| format!("Failed to write report: {}", e))?;
        
        println!("[DIFF-FUZZ] Full report saved to: {}", path);
        Ok(())
    }
}

pub fn create_differential_fuzzer(target_old: String, target_new: String) -> DifferentialFuzzer {
    DifferentialFuzzer::new(target_old, target_new)
}

pub fn load_corpus_glob(fuzzer: &mut DifferentialFuzzer, pattern: &str) -> Result<(), String> {
    fuzzer.load_corpus(pattern)
}
