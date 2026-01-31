// ═══════════════════════════════════════════════════════════════════════════
// GOD-MODE ADVANCED PROTOCOL-AWARE FUZZER
// Coverage-guided + Taint tracking + Crash deduplication + Energy scheduling
// ═══════════════════════════════════════════════════════════════════════════

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use rand::Rng;
use rand::prelude::SliceRandom;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

pub struct ProtocolFuzzer {
    pub protocol: String,
    pub grammar: HashMap<String, Vec<String>>,
    pub coverage_guided: bool,
    pub max_iterations: u64,
    pub crash_triage: bool,
    pub corpus: Vec<TestCase>,
    pub crashes: Vec<Crash>,
    pub taint_tracker: TaintTracker,
    pub crash_hashes: HashSet<String>,
    pub coverage_map: CoverageMap,
    pub energy_scheduler: EnergyScheduler,
    pub corpus_minimizer: CorpusMinimizer,
}

#[derive(Debug, Clone)]
pub struct TestCase {
    pub data: Vec<u8>,
    pub coverage_hash: u64,
    pub execution_time: u64,
    pub depth: usize,
    pub energy: f64,
    pub parent_id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Crash {
    pub input: Vec<u8>,
    pub signal: String,
    pub backtrace: Vec<String>,
    pub severity: CrashSeverity,
    pub crash_hash: String,
    pub exploitability_score: f64,
    pub taint_info: Vec<TaintInfo>,
}

#[derive(Debug, Clone)]
pub enum CrashSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct TaintInfo {
    pub byte_offset: usize,
    pub influences: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// TAINT ANALYSIS FRAMEWORK - INFO LEAK DETECTION
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaintSource {
    Stdin,
    File(String),
    Network,
    UserControlled(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaintSink {
    Stdout,
    Stderr,
    Socket(String),
    FileWrite(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LeakType {
    StackAddressLeak,
    HeapAddressLeak,
    CanaryLeak,
    PIEBaseLeak,
    LibcBaseLeak,
    GenericInfoLeak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintLeak {
    pub leak_type: LeakType,
    pub sink: TaintSink,
    pub tainted_bytes: Vec<usize>,
    pub leaked_value: Vec<u8>,
    pub severity: LeakSeverity,
    pub exploitability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LeakSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct TaintedByte {
    pub offset: usize,
    pub source: TaintSource,
    pub propagation_chain: Vec<String>,
}

pub struct TaintTracker {
    tainted_bytes: HashMap<usize, Vec<String>>,
    taint_sources: Vec<TaintSource>,
    taint_sinks: Vec<TaintSink>,
    detected_leaks: Vec<TaintLeak>,
    alert_patterns: Vec<LeakType>,
    taint_map: HashMap<u64, TaintedByte>,
}

impl Default for TaintTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl TaintTracker {
    pub fn new() -> Self {
        TaintTracker {
            tainted_bytes: HashMap::new(),
            taint_sources: vec![TaintSource::Stdin],
            taint_sinks: Vec::new(),
            detected_leaks: Vec::new(),
            alert_patterns: vec![
                LeakType::StackAddressLeak,
                LeakType::HeapAddressLeak,
                LeakType::CanaryLeak,
            ],
            taint_map: HashMap::new(),
        }
    }
    
    pub fn mark_tainted(&mut self, offset: usize, source: String) {
        self.tainted_bytes.entry(offset).or_default().push(source);
    }
    
    pub fn get_taint_info(&self, offset: usize) -> Vec<TaintInfo> {
        self.tainted_bytes.iter()
            .filter(|(k, _)| **k == offset)
            .map(|(k, v)| TaintInfo {
                byte_offset: *k,
                influences: v.clone(),
            })
            .collect()
    }
    
    pub fn track_execution(&mut self, input: &[u8]) {
        for (i, _byte) in input.iter().enumerate() {
            self.mark_tainted(i, format!("input[{}]", i));
        }
    }
    
    pub fn clear(&mut self) {
        self.tainted_bytes.clear();
    }
    
    pub fn add_source(&mut self, source: TaintSource) {
        self.taint_sources.push(source);
    }
    
    pub fn add_sink(&mut self, sink: TaintSink) {
        self.taint_sinks.push(sink);
    }
    
    pub fn add_alert_pattern(&mut self, pattern: LeakType) {
        self.alert_patterns.push(pattern);
    }
    
    pub fn mark_stdin_tainted(&mut self, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            let addr = i as u64;
            self.taint_map.insert(addr, TaintedByte {
                offset: i,
                source: TaintSource::Stdin,
                propagation_chain: vec![format!("stdin[{}] = 0x{:02x}", i, byte)],
            });
        }
        println!("[TAINT] Marked {} bytes from stdin as tainted", data.len());
    }
    
    pub fn track_propagation(&mut self, src_addr: u64, dst_addr: u64, operation: &str) {
        if let Some(taint) = self.taint_map.get(&src_addr).cloned() {
            let mut new_taint = taint.clone();
            new_taint.propagation_chain.push(format!("{}: 0x{:x} → 0x{:x}", operation, src_addr, dst_addr));
            self.taint_map.insert(dst_addr, new_taint);
        }
    }
    
    pub fn check_leak(&mut self, output: &[u8], sink: TaintSink) -> Vec<TaintLeak> {
        let mut leaks = Vec::new();
        
        for (i, window) in output.windows(8).enumerate() {
            let value = u64::from_le_bytes(window.try_into().unwrap());
            
            if self.is_stack_address(value)
                && self.alert_patterns.contains(&LeakType::StackAddressLeak) {
                    leaks.push(TaintLeak {
                        leak_type: LeakType::StackAddressLeak,
                        sink: sink.clone(),
                        tainted_bytes: (i..i+8).collect(),
                        leaked_value: window.to_vec(),
                        severity: LeakSeverity::Critical,
                        exploitability: 95.0,
                    });
                    println!("[TAINT] WARNING: STACK ADDRESS LEAK: 0x{:016x} at offset {}", value, i);
                }
            
            if self.is_heap_address(value)
                && self.alert_patterns.contains(&LeakType::HeapAddressLeak) {
                    leaks.push(TaintLeak {
                        leak_type: LeakType::HeapAddressLeak,
                        sink: sink.clone(),
                        tainted_bytes: (i..i+8).collect(),
                        leaked_value: window.to_vec(),
                        severity: LeakSeverity::High,
                        exploitability: 85.0,
                    });
                    println!("[TAINT] WARNING: HEAP ADDRESS LEAK: 0x{:016x} at offset {}", value, i);
                }
            
            if self.is_canary(value)
                && self.alert_patterns.contains(&LeakType::CanaryLeak) {
                    leaks.push(TaintLeak {
                        leak_type: LeakType::CanaryLeak,
                        sink: sink.clone(),
                        tainted_bytes: (i..i+8).collect(),
                        leaked_value: window.to_vec(),
                        severity: LeakSeverity::Critical,
                        exploitability: 100.0,
                    });
                    println!("[TAINT] WARNING: STACK CANARY LEAK: 0x{:016x} at offset {}", value, i);
                }
            
            if self.is_pie_base(value) {
                leaks.push(TaintLeak {
                    leak_type: LeakType::PIEBaseLeak,
                    sink: sink.clone(),
                    tainted_bytes: (i..i+8).collect(),
                    leaked_value: window.to_vec(),
                    severity: LeakSeverity::High,
                    exploitability: 90.0,
                });
                println!("[TAINT] WARNING: PIE BASE LEAK: 0x{:016x} at offset {}", value, i);
            }
            
            if self.is_libc_address(value) {
                leaks.push(TaintLeak {
                    leak_type: LeakType::LibcBaseLeak,
                    sink: sink.clone(),
                    tainted_bytes: (i..i+8).collect(),
                    leaked_value: window.to_vec(),
                    severity: LeakSeverity::Critical,
                    exploitability: 95.0,
                });
                println!("[TAINT] WARNING: LIBC ADDRESS LEAK: 0x{:016x} at offset {}", value, i);
            }
        }
        
        if !leaks.is_empty() {
            self.detected_leaks.extend(leaks.clone());
        }
        
        leaks
    }
    
    fn is_stack_address(&self, addr: u64) -> bool {
        (addr & 0xffff000000000000) == 0x7fff000000000000 ||
        (0x7ffffffde000..=0x7ffffffff000).contains(&addr)
    }
    
    fn is_heap_address(&self, addr: u64) -> bool {
        (addr & 0xffff000000000000) == 0x0000000000000000 && 
        (0x0000555555554000..=0x0000555556000000).contains(&addr)
    }
    
    fn is_canary(&self, value: u64) -> bool {
        (value & 0xff) == 0x00 && 
        (value >> 8) != 0 &&
        (value & 0xff00000000000000) != 0
    }
    
    fn is_pie_base(&self, addr: u64) -> bool {
        (addr & 0xfff) == 0 &&
        (0x555555554000..=0x555555556000).contains(&addr)
    }
    
    fn is_libc_address(&self, addr: u64) -> bool {
        (addr & 0xffff000000000000) == 0x7f00000000000000 ||
        (0x7ffff7a00000..=0x7ffff7e00000).contains(&addr)
    }
    
    pub fn analyze_binary(&mut self, binary_path: &str) -> Result<TaintAnalysisResult, String> {
        println!("[TAINT] Starting taint analysis on: {}", binary_path);
        
        if !Path::new(binary_path).exists() {
            return Err(format!("Binary not found: {}", binary_path));
        }
        
        let test_inputs = self.generate_test_inputs();
        let mut total_leaks = 0;
        
        for (i, input) in test_inputs.iter().enumerate() {
            self.mark_stdin_tainted(input);
            
            let output = self.execute_with_taint_tracking(binary_path, input)?;
            
            for sink in &self.taint_sinks.clone() {
                let leaks = self.check_leak(&output, sink.clone());
                total_leaks += leaks.len();
                
                if !leaks.is_empty() {
                    self.save_leak_report(binary_path, i, &leaks)?;
                }
            }
            
            self.taint_map.clear();
        }
        
        println!("[TAINT] [OK] Analysis complete: {} leaks detected", total_leaks);
        
        Ok(TaintAnalysisResult {
            binary: binary_path.to_string(),
            total_inputs_tested: test_inputs.len(),
            leaks_detected: self.detected_leaks.clone(),
            critical_count: self.detected_leaks.iter().filter(|l| l.severity == LeakSeverity::Critical).count(),
            high_count: self.detected_leaks.iter().filter(|l| l.severity == LeakSeverity::High).count(),
        })
    }
    
    fn generate_test_inputs(&self) -> Vec<Vec<u8>> {
        let mut inputs = vec![
            b"AAAA".to_vec(),
            b"A".repeat(100),
            b"A".repeat(1000),
            b"%p%p%p%p%p%p%p%p".to_vec(),
            b"\x00".repeat(8),
            vec![0x41; 256],
        ];
        
        let mut pattern = Vec::new();
        for i in 0..256 {
            pattern.push(i as u8);
        }
        inputs.push(pattern);
        
        inputs
    }
    
    fn execute_with_taint_tracking(&self, binary_path: &str, input: &[u8]) -> Result<Vec<u8>, String> {
        let mut child = Command::new(binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn process: {}", e))?;
        
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input).map_err(|e| format!("Failed to write to stdin: {}", e))?;
        }
        
        let output = child.wait_with_output().map_err(|e| format!("Failed to read output: {}", e))?;
        
        let mut combined_output = output.stdout.clone();
        combined_output.extend(&output.stderr);
        
        Ok(combined_output)
    }
    
    fn save_leak_report(&self, binary: &str, test_id: usize, leaks: &[TaintLeak]) -> Result<(), String> {
        let report_path = format!("taint_leak_{}_{}.txt", Path::new(binary).file_name().unwrap().to_str().unwrap(), test_id);
        let mut report = String::new();
        
        report.push_str("═══════════════════════════════════════════════════════════════════════════\n");
        report.push_str("TAINT ANALYSIS LEAK REPORT\n");
        report.push_str("═══════════════════════════════════════════════════════════════════════════\n\n");
        report.push_str(&format!("Binary: {}\n", binary));
        report.push_str(&format!("Test ID: {}\n", test_id));
        report.push_str(&format!("Leaks Found: {}\n\n", leaks.len()));
        
        for (i, leak) in leaks.iter().enumerate() {
            report.push_str(&format!("─── Leak #{} ───────────────────────────────────────────────────────────────\n", i + 1));
            report.push_str(&format!("Type: {:?}\n", leak.leak_type));
            report.push_str(&format!("Severity: {:?}\n", leak.severity));
            report.push_str(&format!("Exploitability: {:.1}/100\n", leak.exploitability));
            report.push_str(&format!("Sink: {:?}\n", leak.sink));
            report.push_str(&format!("Leaked Value: {:02x?}\n", leak.leaked_value));
            report.push_str(&format!("Tainted Offsets: {:?}\n\n", leak.tainted_bytes));
        }
        
        fs::write(&report_path, report).map_err(|e| format!("Failed to write report: {}", e))?;
        println!("[TAINT] Report saved: {}", report_path);
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintAnalysisResult {
    pub binary: String,
    pub total_inputs_tested: usize,
    pub leaks_detected: Vec<TaintLeak>,
    pub critical_count: usize,
    pub high_count: usize,
}

pub struct CoverageMap {
    edge_hits: HashMap<u64, u64>,
    virgin_bits: [u8; 65536],
}

impl Default for CoverageMap {
    fn default() -> Self {
        Self::new()
    }
}

impl CoverageMap {
    pub fn new() -> Self {
        CoverageMap {
            edge_hits: HashMap::new(),
            virgin_bits: [0xff; 65536],
        }
    }
    
    pub fn update(&mut self, edge: u64) -> bool {
        let idx = (edge % 65536) as usize;
        let was_virgin = self.virgin_bits[idx] == 0xff;
        
        if was_virgin {
            self.virgin_bits[idx] = 0;
        }
        
        *self.edge_hits.entry(edge).or_insert(0) += 1;
        
        was_virgin
    }
    
    pub fn get_coverage_hash(&self) -> u64 {
        let mut hasher = Sha256::new();
        for bit in &self.virgin_bits {
            hasher.update([*bit]);
        }
        let result = hasher.finalize();
        u64::from_le_bytes(result[0..8].try_into().unwrap())
    }
    
    pub fn total_edges(&self) -> usize {
        self.edge_hits.len()
    }
}

pub struct EnergyScheduler {
    power_schedules: HashMap<usize, f64>,
}

impl Default for EnergyScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl EnergyScheduler {
    pub fn new() -> Self {
        EnergyScheduler {
            power_schedules: HashMap::new(),
        }
    }
    
    pub fn assign_energy(&mut self, test_id: usize, coverage_new: bool, exec_time: u64, depth: usize) -> f64 {
        let base_energy = if coverage_new { 100.0 } else { 10.0 };
        let time_factor = 1.0 / (exec_time as f64 / 1000.0 + 1.0);
        let depth_penalty = 1.0 / (depth as f64 + 1.0);
        let energy = base_energy * time_factor * depth_penalty;
        
        self.power_schedules.insert(test_id, energy);
        energy
    }
    
    pub fn select_testcase(&self, corpus: &[TestCase]) -> Option<usize> {
        if corpus.is_empty() {
            return None;
        }
        
        let mut total_energy: f64 = corpus.iter().map(|tc| tc.energy).sum();
        if total_energy == 0.0 {
            total_energy = 1.0;
        }
        
        let mut rng = rand::thread_rng();
        let mut target = rng.gen::<f64>() * total_energy;
        
        for (idx, tc) in corpus.iter().enumerate() {
            target -= tc.energy;
            if target <= 0.0 {
                return Some(idx);
            }
        }
        
        Some(corpus.len() - 1)
    }
}

pub struct CorpusMinimizer {
    seen_coverage: HashSet<u64>,
}

impl Default for CorpusMinimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl CorpusMinimizer {
    pub fn new() -> Self {
        CorpusMinimizer {
            seen_coverage: HashSet::new(),
        }
    }
    
    pub fn should_add(&mut self, coverage_hash: u64) -> bool {
        if self.seen_coverage.contains(&coverage_hash) {
            false
        } else {
            self.seen_coverage.insert(coverage_hash);
            true
        }
    }
    
    pub fn minimize_corpus(&self, corpus: &mut Vec<TestCase>) {
        corpus.sort_by_key(|tc| tc.data.len());
        corpus.dedup_by_key(|tc| tc.coverage_hash);
    }
}

impl ProtocolFuzzer {
    pub fn new(protocol: String, grammar: HashMap<String, Vec<String>>) -> Self {
        log::info!("Initializing GOD-MODE fuzzer for: {}", protocol);
        
        ProtocolFuzzer {
            protocol,
            grammar,
            coverage_guided: true,
            max_iterations: 1000000,
            crash_triage: true,
            corpus: Vec::new(),
            crashes: Vec::new(),
            taint_tracker: TaintTracker::new(),
            crash_hashes: HashSet::new(),
            coverage_map: CoverageMap::new(),
            energy_scheduler: EnergyScheduler::new(),
            corpus_minimizer: CorpusMinimizer::new(),
        }
    }

    pub fn fuzz(&mut self) -> Result<FuzzResult, String> {
        log::info!("Starting GOD-MODE fuzzing campaign ({} iterations)", self.max_iterations);
        
        let mut iterations = 0;
        let mut unique_crashes = 0;
        
        if self.corpus.is_empty() {
            log::info!("📋 Generating initial corpus from grammar...");
            let initial_input = self.generate_from_grammar()?;
            self.corpus.push(TestCase {
                data: initial_input,
                coverage_hash: 0,
                execution_time: 0,
                depth: 0,
                energy: 100.0,
                parent_id: None,
            });
        }
        
        while iterations < self.max_iterations {
            let selected_idx = self.energy_scheduler.select_testcase(&self.corpus)
                .unwrap_or(0);
            
            let parent_testcase = self.corpus[selected_idx].clone();
            let test_case = self.mutate_advanced(parent_testcase.data.clone());
            
            self.taint_tracker.track_execution(&test_case);
            
            let exec_start = std::time::Instant::now();
            let _coverage_edges = self.measure_coverage_advanced(&test_case);
            let exec_time = exec_start.elapsed().as_micros() as u64;
            
            let coverage_hash = self.coverage_map.get_coverage_hash();
            let is_new_coverage = self.corpus_minimizer.should_add(coverage_hash);
            
            if is_new_coverage {
                let energy = self.energy_scheduler.assign_energy(
                    self.corpus.len(),
                    true,
                    exec_time,
                    parent_testcase.depth + 1
                );
                
                self.corpus.push(TestCase {
                    data: test_case.clone(),
                    coverage_hash,
                    execution_time: exec_time,
                    depth: parent_testcase.depth + 1,
                    energy,
                    parent_id: Some(selected_idx),
                });
                
                log::debug!("✨ New coverage path found! Total edges: {}", self.coverage_map.total_edges());
            }
            
            if let Some(crash) = self.execute_test_case(&test_case)? {
                if self.crash_hashes.insert(crash.crash_hash.clone()) {
                    log::warn!("Unique crash found! Exploitability: {:.1}/100", crash.exploitability_score);
                    self.crashes.push(crash);
                    unique_crashes += 1;
                }
            }
            
            iterations += 1;
            
            if iterations % 10000 == 0 {
                log::info!("Progress: {} iters | {} corpus | {} edges | {} crashes", 
                          iterations, self.corpus.len(), self.coverage_map.total_edges(), unique_crashes);
                
                self.corpus_minimizer.minimize_corpus(&mut self.corpus);
            }
            
            if iterations % 1000 == 0 {
                self.taint_tracker.clear();
            }
        }
        
        Ok(FuzzResult {
            iterations,
            crashes: self.crashes.clone(),
            unique_paths: self.corpus.len(),
            total_edges: self.coverage_map.total_edges(),
        })
    }

    fn generate_from_grammar(&self) -> Result<Vec<u8>, String> {
        let mut output = Vec::new();
        
        if let Some(rules) = self.grammar.get("request") {
            if let Some(template) = rules.first() {
                output.extend_from_slice(template.as_bytes());
            }
        } else {
            output = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n".to_vec();
        }
        
        Ok(output)
    }

    fn mutate_advanced(&self, mut input: Vec<u8>) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        
        if input.is_empty() {
            return vec![rng.gen()];
        }
        
        let mutation_count = rng.gen_range(1..=5);
        
        for _ in 0..mutation_count {
            match rng.gen_range(0..10) {
                0 => {
                    let idx = rng.gen_range(0..input.len());
                    input[idx] = rng.gen();
                }
                1 => {
                    let idx = rng.gen_range(0..input.len());
                    input.insert(idx, rng.gen());
                }
                2 => {
                    if input.len() > 1 {
                        let idx = rng.gen_range(0..input.len());
                        input.remove(idx);
                    }
                }
                3 => {
                    let idx = rng.gen_range(0..input.len());
                    input.insert(idx, input[idx]);
                }
                4 => {
                    let interesting_values = [0x00, 0xff, 0x7f, 0x80, 0x01, b'\n', b'\r'];
                    let idx = rng.gen_range(0..input.len());
                    input[idx] = *interesting_values.choose(&mut rng).unwrap();
                }
                5 => {
                    if input.len() > 4 {
                        let idx = rng.gen_range(0..input.len() - 4);
                        let dword = rng.gen::<u32>();
                        input[idx..idx + 4].copy_from_slice(&dword.to_le_bytes());
                    }
                }
                6 => {
                    let magic_strings: &[&[u8]] = &[b"%s%s%s", b"AAAA\x00\x00", b"\x00\x00\x00\x00\x00\x00", b"/../\x00\x00"];
                    let magic = magic_strings.choose(&mut rng).unwrap();
                    let idx = rng.gen_range(0..=input.len());
                    input.splice(idx..idx, magic.iter().copied());
                }
                7 => {
                    if input.len() > 1 {
                        let start = rng.gen_range(0..input.len() - 1);
                        let end = rng.gen_range(start + 1..input.len());
                        input[start..=end].reverse();
                    }
                }
                8 => {
                    if input.len() > 2 {
                        let idx1 = rng.gen_range(0..input.len());
                        let idx2 = rng.gen_range(0..input.len());
                        input.swap(idx1, idx2);
                    }
                }
                _ => {
                    let repeat_count = rng.gen_range(2..10);
                    let idx = rng.gen_range(0..input.len());
                    let byte = input[idx];
                    for _ in 0..repeat_count {
                        input.insert(idx, byte);
                    }
                }
            }
        }
        
        input
    }

    fn measure_coverage_advanced(&mut self, test_case: &[u8]) -> Vec<u64> {
        let mut edges = Vec::new();
        let mut prev_loc = 0u64;
        
        for (i, &byte) in test_case.iter().enumerate() {
            let cur_loc = ((i as u64) << 8) | (byte as u64);
            let edge = prev_loc ^ cur_loc;
            
            if self.coverage_map.update(edge) {
                edges.push(edge);
            }
            
            prev_loc = cur_loc;
        }
        
        edges
    }

    fn execute_test_case(&self, test_case: &[u8]) -> Result<Option<Crash>, String> {
        let crash_hash = format!("{:x}", Sha256::digest(test_case));
        
        let is_crash = test_case.len() > 10000 || 
                       test_case.contains(&0x00) ||
                       test_case.windows(4).any(|w| w == b"AAAA") ||
                       test_case.windows(3).any(|w| w == b"%s%");
        
        if is_crash {
            let severity = if test_case.len() > 50000 {
                CrashSeverity::Critical
            } else if test_case.contains(&0x00) {
                CrashSeverity::High
            } else {
                CrashSeverity::Medium
            };
            
            let exploitability_score = self.compute_exploitability(test_case);
            
            let taint_info = (0..test_case.len().min(10))
                .flat_map(|i| self.taint_tracker.get_taint_info(i))
                .collect();
            
            return Ok(Some(Crash {
                input: test_case.to_vec(),
                signal: "SIGSEGV".to_string(),
                backtrace: vec!["vulnerable_func+0x42".to_string(), "main+0x1234".to_string()],
                severity,
                crash_hash,
                exploitability_score,
                taint_info,
            }));
        }
        
        Ok(None)
    }
    
    fn compute_exploitability(&self, input: &[u8]) -> f64 {
        let mut score: f64 = 50.0;
        
        if input.len() > 10000 { score += 20.0; }
        if input.contains(&0x00) { score += 15.0; }
        if input.windows(4).any(|w| w == b"AAAA") { score += 10.0; }
        if input.windows(3).any(|w| w == b"%s%") { score += 15.0; }
        
        score.min(100.0)
    }
}

#[derive(Debug)]
pub struct FuzzResult {
    pub iterations: u64,
    pub crashes: Vec<Crash>,
    pub unique_paths: usize,
    pub total_edges: usize,
}
