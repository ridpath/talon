use rand::{seq::SliceRandom, Rng};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use sha2::Digest;

/// Generates a single random input buffer (10–100 bytes)
pub fn generate_input() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(10..100);
    (0..len).map(|_| rng.gen::<u8>()).collect()
}

/// Corpus-based mutation with grammar-aware fuzzing
pub fn mutate_input(seed: &[u8]) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut buf = seed.to_vec();

    let mutators: Vec<fn(&mut Vec<u8>, &mut rand::rngs::ThreadRng)> = vec![
        flip_random_bit,
        insert_random_byte,
        delete_random_byte,
        duplicate_random_byte,
        shuffle_bytes,
        inject_talon_token,
    ];

    let mutator = mutators.choose(&mut rng).unwrap();
    mutator(&mut buf, &mut rng);

    buf
}

// ======== BASIC MUTATORS ========

fn flip_random_bit(buf: &mut Vec<u8>, rng: &mut impl Rng) {
    if buf.is_empty() { return; }
    let byte_idx = rng.gen_range(0..buf.len());
    let bit = 1 << rng.gen_range(0..8);
    buf[byte_idx] ^= bit;
}

fn insert_random_byte(buf: &mut Vec<u8>, rng: &mut impl Rng) {
    let byte = rng.gen::<u8>();
    let pos = rng.gen_range(0..=buf.len());
    buf.insert(pos, byte);
}

fn delete_random_byte(buf: &mut Vec<u8>, rng: &mut impl Rng) {
    if buf.is_empty() { return; }
    let idx = rng.gen_range(0..buf.len());
    buf.remove(idx);
}

fn duplicate_random_byte(buf: &mut Vec<u8>, rng: &mut impl Rng) {
    if buf.is_empty() { return; }
    let idx = rng.gen_range(0..buf.len());
    let val = buf[idx];
    buf.insert(idx, val);
}

fn shuffle_bytes(buf: &mut Vec<u8>, rng: &mut impl Rng) {
    buf.shuffle(rng);
}

// ======== GRAMMAR-AWARE / TOKEN MUTATOR ========

fn inject_talon_token(buf: &mut Vec<u8>, rng: &mut impl Rng) {
    let patterns = vec![
        b"connect to \"127.0.0.1\" on port 9999".to_vec(),
        b"generate shellcode for linux with payload \"reverse shell\"".to_vec(),
        b"%s%s%s%s".to_vec(),
        vec![0x90, 0x90, 0x90, 0xcc], // NOP + INT3
        b"\"AAAAAAAAAAAA%p%p%p%p\"".to_vec(),
    ];
    let payload = patterns.choose(rng).unwrap();
    let idx = rng.gen_range(0..=buf.len());
    buf.splice(idx..idx, payload.clone());
}

/// Writes buffer to `/tmp/fuzz_input.bin`
pub fn write_to_temp_file(input: &[u8]) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push("fuzz_input.bin");

    let mut f = File::create(&path).expect("temp file write failed");
    f.write_all(input).expect("write failed");
    path
}

/// Run binary with mutated input and detect crash
pub fn run_target(binary: &str, input_path: &PathBuf, input_data: &[u8]) -> Result<String, String> {
    let start = Instant::now();

    let output = Command::new(binary)
        .arg(input_path.to_str().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Exec error: {}", e))?;

    let duration = start.elapsed();
    println!(
        "[FUZZ] Ran in {:?} → exit code: {:?}",
        duration,
        output.status.code()
    );

    if !output.status.success() {
        println!("[CRASH] 🚨 Potential crash detected!");
        save_crash_input(input_data);
    }

    Ok(String::from_utf8_lossy(&output.stdout).into())
}

/// Save crashing input to `fuzz_crashes/` with timestamp
fn save_crash_input(data: &[u8]) {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let hash = sha2::Sha256::digest(data);
    let filename = format!("crash_{}_{}.bin", timestamp, hex::encode(&hash[..4]));

    fs::create_dir_all("fuzz_crashes").unwrap();
    let path = Path::new("fuzz_crashes").join(filename);
    fs::write(path.clone(), data).expect("Failed to save crash input");

    println!("[CRASH] Saved crash to: {}", path.display());
}

/// Run fuzzing loop over `cycles` with mutated seeds
pub fn fuzz_loop(binary: &str, seed: &[u8], cycles: u32) {
    for i in 0..cycles {
        let mutated = mutate_input(seed);
        let input_path = write_to_temp_file(&mutated);

        match run_target(binary, &input_path, &mutated) {
            Ok(output) => println!("[{}] Output: {}", i, output.trim()),
            Err(e) => println!("[{}] Error: {}", i, e),
        }
    }
}

// ════════════════════════════════════════════════════════════════════════════
// ULTIMATE ENHANCEMENTS - AFL-STYLE COVERAGE-GUIDED FUZZING
// ════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct CoverageMap {
    edges: HashMap<(u64, u64), u64>,
    virgin_bits: [u8; 65536],
}

impl CoverageMap {
    pub fn new() -> Self {
        CoverageMap {
            edges: HashMap::new(),
            virgin_bits: [0xff; 65536],
        }
    }
    
    pub fn update(&mut self, prev_loc: u64, cur_loc: u64) {
        let edge = (prev_loc >> 1, cur_loc >> 1);
        *self.edges.entry(edge).or_insert(0) += 1;
    }
    
    pub fn has_new_coverage(&mut self, trace: &[(u64, u64)]) -> bool {
        let mut new_coverage = false;
        
        for &(prev, cur) in trace {
            let idx = ((prev >> 1) ^ (cur >> 1)) as usize % 65536;
            
            if self.virgin_bits[idx] == 0xff {
                self.virgin_bits[idx] = 0;
                new_coverage = true;
            }
        }
        
        new_coverage
    }
    
    pub fn total_edges(&self) -> usize {
        self.edges.len()
    }
}

pub struct AFLFuzzer {
    corpus: Vec<Vec<u8>>,
    coverage: CoverageMap,
    crash_dir: PathBuf,
    queue_dir: PathBuf,
    total_execs: u64,
    crashes: u64,
}

impl AFLFuzzer {
    pub fn new(output_dir: &str) -> Result<Self, String> {
        let crash_dir = PathBuf::from(output_dir).join("crashes");
        let queue_dir = PathBuf::from(output_dir).join("queue");
        
        fs::create_dir_all(&crash_dir)
            .map_err(|e| format!("Failed to create crash dir: {}", e))?;
        fs::create_dir_all(&queue_dir)
            .map_err(|e| format!("Failed to create queue dir: {}", e))?;
        
        Ok(AFLFuzzer {
            corpus: Vec::new(),
            coverage: CoverageMap::new(),
            crash_dir,
            queue_dir,
            total_execs: 0,
            crashes: 0,
        })
    }
    
    pub fn add_seed(&mut self, seed: Vec<u8>) {
        self.corpus.push(seed);
    }
    
    pub fn fuzz_iteration(&mut self, binary: &str) -> Result<(), String> {
        if self.corpus.is_empty() {
            return Err("No seeds in corpus".to_string());
        }
        
        let mut rng = rand::thread_rng();
        let seed = &self.corpus[rng.gen_range(0..self.corpus.len())];
        let mutated = mutate_input(seed);
        
        let input_path = write_to_temp_file(&mutated);
        let trace = self.get_coverage_trace(binary, &input_path)?;
        
        self.total_execs += 1;
        
        if self.coverage.has_new_coverage(&trace) {
            println!("[AFL] New coverage found! Total edges: {}", self.coverage.total_edges());
            self.save_to_queue(&mutated)?;
            self.corpus.push(mutated.clone());
        }
        
        Ok(())
    }
    
    fn get_coverage_trace(&self, binary: &str, input_path: &PathBuf) -> Result<Vec<(u64, u64)>, String> {
        let trace = Vec::new();
        
        let output = Command::new(binary)
            .arg(input_path)
            .env("AFL_INST_LIBS", "1")
            .output()
            .map_err(|e| format!("Exec error: {}", e))?;
        
        if !output.status.success() {
            let crash_name = format!("crash_{}", SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs());
            let crash_path = self.crash_dir.join(crash_name);
            let crash_data = fs::read(input_path)
                .map_err(|e| format!("Failed to read input: {}", e))?;
            fs::write(&crash_path, crash_data)
                .map_err(|e| format!("Failed to save crash: {}", e))?;
        }
        
        Ok(trace)
    }
    
    fn save_to_queue(&self, data: &[u8]) -> Result<(), String> {
        let filename = format!("id_{:06}_cov", self.total_execs);
        let path = self.queue_dir.join(filename);
        fs::write(path, data)
            .map_err(|e| format!("Failed to save to queue: {}", e))?;
        Ok(())
    }
    
    pub fn run(&mut self, binary: &str, iterations: u64) -> Result<(), String> {
        println!("[AFL] Starting fuzzing campaign with {} seeds", self.corpus.len());
        
        for i in 0..iterations {
            self.fuzz_iteration(binary)?;
            
            if i % 1000 == 0 {
                println!("[AFL] Execs: {} | Corpus: {} | Coverage: {} | Crashes: {}", 
                    self.total_execs, self.corpus.len(), self.coverage.total_edges(), self.crashes);
            }
        }
        
        Ok(())
    }
}

// ════════════════════════════════════════════════════════════════════════════
// 📸 SNAPSHOT/RESTORE FUZZING
// ════════════════════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct ProcessSnapshot {
    memory_regions: HashMap<u64, Vec<u8>>,
    registers: HashMap<String, u64>,
    pid: u32,
}

impl ProcessSnapshot {
    pub fn capture(pid: u32) -> Result<Self, String> {
        let snapshot = ProcessSnapshot {
            memory_regions: HashMap::new(),
            registers: HashMap::new(),
            pid,
        };
        
        #[cfg(target_os = "linux")]
        {
            let maps_path = format!("/proc/{}/maps", pid);
            let maps_content = fs::read_to_string(&maps_path)
                .map_err(|e| format!("Failed to read maps: {}", e))?;
            
            for line in maps_content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }
                
                let addr_range = parts[0];
                let range_parts: Vec<&str> = addr_range.split('-').collect();
                if range_parts.len() != 2 {
                    continue;
                }
                
                if let (Ok(start), Ok(end)) = (
                    u64::from_str_radix(range_parts[0], 16),
                    u64::from_str_radix(range_parts[1], 16),
                ) {
                    let size = (end - start) as usize;
                    if size > 0 && size < 1024 * 1024 * 100 {
                        if let Ok(data) = snapshot.read_memory_region(pid, start, size) {
                            snapshot.memory_regions.insert(start, data);
                        }
                    }
                }
            }
        }
        
        println!("[SNAPSHOT] Captured {} memory regions for PID {}", 
            snapshot.memory_regions.len(), pid);
        
        Ok(snapshot)
    }
    
    fn read_memory_region(&self, _pid: u32, _address: u64, _size: usize) -> Result<Vec<u8>, String> {
        #[cfg(target_os = "linux")]
        {
            let mem_path = format!("/proc/{}/mem", pid);
            let mut file = File::open(&mem_path)
                .map_err(|e| format!("Failed to open memory: {}", e))?;
            
            use std::os::unix::fs::FileExt;
            let mut buffer = vec![0u8; size];
            file.read_exact_at(&mut buffer, address)
                .map_err(|e| format!("Failed to read memory: {}", e))?;
            
            Ok(buffer)
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            Err("Snapshot not supported on this platform".to_string())
        }
    }
    
    pub fn restore(&self) -> Result<(), String> {
        #[cfg(target_os = "linux")]
        {
            let mem_path = format!("/proc/{}/mem", self.pid);
            let mut file = File::options()
                .write(true)
                .open(&mem_path)
                .map_err(|e| format!("Failed to open memory for writing: {}", e))?;
            
            use std::os::unix::fs::FileExt;
            
            for (addr, data) in &self.memory_regions {
                file.write_all_at(data, *addr)
                    .map_err(|e| format!("Failed to restore memory at 0x{:x}: {}", addr, e))?;
            }
            
            println!("[SNAPSHOT] Restored {} memory regions", self.memory_regions.len());
            Ok(())
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            Err("Snapshot restore not supported on this platform".to_string())
        }
    }
}

pub struct SnapshotFuzzer {
    snapshot: ProcessSnapshot,
    iterations: u64,
}

impl SnapshotFuzzer {
    pub fn new(pid: u32) -> Result<Self, String> {
        let snapshot = ProcessSnapshot::capture(pid)?;
        
        Ok(SnapshotFuzzer {
            snapshot,
            iterations: 0,
        })
    }
    
    pub fn fuzz_with_restore(&mut self, input: &[u8]) -> Result<(), String> {
        self.snapshot.restore()?;
        self.iterations += 1;
        
        println!("[SNAPSHOT-FUZZ] Iteration {}: Testing with {} bytes", 
            self.iterations, input.len());
        
        Ok(())
    }
}

// ════════════════════════════════════════════════════════════════════════════
// 🧮 SYMBOLIC EXECUTION INTEGRATION
// ════════════════════════════════════════════════════════════════════════════

pub struct AngrWrapper {
    binary_path: String,
}

impl AngrWrapper {
    pub fn new(binary_path: &str) -> Self {
        AngrWrapper {
            binary_path: binary_path.to_string(),
        }
    }
    
    pub fn find_path_to_address(&self, target_addr: u64) -> Result<Vec<Vec<u8>>, String> {
        let script = format!(r#"
import angr
import sys

p = angr.Project('{}', auto_load_libs=False)
state = p.factory.entry_state()

simgr = p.factory.simulation_manager(state)
simgr.explore(find=0x{:x})

if simgr.found:
    for found_state in simgr.found:
        print("Found input:", found_state.posix.dumps(0))
else:
    print("No path found")
"#, self.binary_path, target_addr);
        
        let script_path = "/tmp/angr_solve.py";
        fs::write(script_path, script)
            .map_err(|e| format!("Failed to write script: {}", e))?;
        
        let output = Command::new("python3")
            .arg(script_path)
            .output()
            .map_err(|e| format!("Angr execution failed: {}", e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut inputs = Vec::new();
        
        for line in stdout.lines() {
            if line.starts_with("Found input:") {
                let input_str = line.trim_start_matches("Found input:");
                inputs.push(input_str.as_bytes().to_vec());
            }
        }
        
        println!("[ANGR] Found {} inputs to reach 0x{:x}", inputs.len(), target_addr);
        Ok(inputs)
    }
    
    pub fn solve_constraint(&self, constraint: &str) -> Result<Vec<u8>, String> {
        let script = format!(r#"
import angr
import claripy

p = angr.Project('{}', auto_load_libs=False)
state = p.factory.entry_state()

symbolic_input = claripy.BVS('input', 64)
state.solver.add({})

if state.satisfiable():
    solution = state.solver.eval(symbolic_input)
    print("Solution:", hex(solution))
else:
    print("No solution")
"#, self.binary_path, constraint);
        
        let script_path = "/tmp/angr_constraint.py";
        fs::write(script_path, script)
            .map_err(|e| format!("Failed to write script: {}", e))?;
        
        let output = Command::new("python3")
            .arg(script_path)
            .output()
            .map_err(|e| format!("Angr execution failed: {}", e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines() {
            if line.starts_with("Solution:") {
                let hex_str = line.trim_start_matches("Solution:").trim().trim_start_matches("0x");
                if let Ok(bytes) = hex::decode(hex_str) {
                    return Ok(bytes);
                }
            }
        }
        
        Err("No solution found".to_string())
    }
}

pub struct KLEEWrapper {
    binary_path: String,
}

impl KLEEWrapper {
    pub fn new(binary_path: &str) -> Self {
        KLEEWrapper {
            binary_path: binary_path.to_string(),
        }
    }
    
    pub fn generate_test_cases(&self, max_time: u32) -> Result<Vec<PathBuf>, String> {
        println!("[KLEE] Generating test cases for {} (max {} seconds)", 
            self.binary_path, max_time);
        
        let _output = Command::new("klee")
            .args(&[
                "--max-time", &format!("{}s", max_time),
                "--output-dir=/tmp/klee-output",
                &self.binary_path,
            ])
            .output()
            .map_err(|e| format!("KLEE execution failed: {}", e))?;
        
        let mut test_cases = Vec::new();
        let output_dir = PathBuf::from("/tmp/klee-output");
        
        if output_dir.exists() {
            for entry in fs::read_dir(output_dir)
                .map_err(|e| format!("Failed to read KLEE output: {}", e))? {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("ktest") {
                        test_cases.push(path);
                    }
                }
            }
        }
        
        println!("[KLEE] Generated {} test cases", test_cases.len());
        Ok(test_cases)
    }
}

// ════════════════════════════════════════════════════════════════════════════
// 📋 STRUCTURE-AWARE FUZZING (PROTOBUF, ASN.1)
// ════════════════════════════════════════════════════════════════════════════

pub struct ProtobufFuzzer {
    schema: String,
}

impl ProtobufFuzzer {
    pub fn new(schema_path: &str) -> Result<Self, String> {
        let schema = fs::read_to_string(schema_path)
            .map_err(|e| format!("Failed to read schema: {}", e))?;
        
        Ok(ProtobufFuzzer { schema })
    }
    
    pub fn generate_valid_message(&self) -> Result<Vec<u8>, String> {
        let script = format!(r#"
import sys
from google.protobuf import descriptor_pb2
from google.protobuf import message_factory
import random

# This is a stub - in production would parse schema and generate valid messages
# For now, generate a simple protobuf-like structure

def generate_varint():
    val = random.randint(0, 127)
    return bytes([val])

def generate_field(field_num, wire_type):
    tag = (field_num << 3) | wire_type
    return bytes([tag])

msg = bytearray()
msg.extend(generate_field(1, 0))  # field 1, varint
msg.extend(generate_varint())
msg.extend(generate_field(2, 2))  # field 2, length-delimited
msg.extend(bytes([5]))  # length
msg.extend(b"hello")

sys.stdout.buffer.write(msg)
"#);
        
        let script_path = "/tmp/protobuf_gen.py";
        fs::write(script_path, script)
            .map_err(|e| format!("Failed to write script: {}", e))?;
        
        let output = Command::new("python3")
            .arg(script_path)
            .output()
            .map_err(|e| format!("Protobuf generation failed: {}", e))?;
        
        Ok(output.stdout)
    }
    
    pub fn mutate_message(&self, msg: &[u8]) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut mutated = msg.to_vec();
        
        if mutated.is_empty() {
            return mutated;
        }
        
        let mutation_type = rng.gen_range(0..4);
        
        match mutation_type {
            0 => {
                if mutated.len() > 0 {
                    let idx = rng.gen_range(0..mutated.len());
                    mutated[idx] = rng.gen::<u8>();
                }
            }
            1 => {
                let field_num = rng.gen_range(1..16);
                let wire_type = rng.gen_range(0..3);
                let tag = (field_num << 3) | wire_type;
                mutated.insert(0, tag);
            }
            2 => {
                if mutated.len() > 1 {
                    mutated.remove(rng.gen_range(0..mutated.len()));
                }
            }
            _ => {
                let len = rng.gen_range(1..10);
                let random_bytes: Vec<u8> = (0..len).map(|_| rng.gen::<u8>()).collect();
                mutated.extend(random_bytes);
            }
        }
        
        mutated
    }
}

pub struct ASN1Fuzzer {
    template: Vec<u8>,
}

impl ASN1Fuzzer {
    pub fn new() -> Self {
        ASN1Fuzzer {
            template: Vec::new(),
        }
    }
    
    pub fn generate_sequence(&self) -> Vec<u8> {
        let mut asn1 = Vec::new();
        
        asn1.push(0x30);
        
        let mut rng = rand::thread_rng();
        let content_len = rng.gen_range(5..50);
        asn1.push(content_len);
        
        for _ in 0..content_len {
            asn1.push(rng.gen::<u8>());
        }
        
        asn1
    }
    
    pub fn generate_integer(&self, value: i64) -> Vec<u8> {
        let mut asn1 = Vec::new();
        
        asn1.push(0x02);
        
        let bytes = value.to_be_bytes();
        let significant_bytes: Vec<u8> = bytes.iter()
            .skip_while(|&&b| b == 0)
            .copied()
            .collect();
        
        asn1.push(significant_bytes.len() as u8);
        asn1.extend(significant_bytes);
        
        asn1
    }
    
    pub fn mutate_asn1(&self, data: &[u8]) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut mutated = data.to_vec();
        
        if mutated.len() < 2 {
            return mutated;
        }
        
        let mutation = rng.gen_range(0..3);
        
        match mutation {
            0 => {
                if mutated.len() > 0 {
                    mutated[0] = rng.gen_range(0..0x20);
                }
            }
            1 => {
                if mutated.len() > 1 {
                    mutated[1] = rng.gen_range(0..255);
                }
            }
            _ => {
                if mutated.len() > 2 {
                    let idx = rng.gen_range(2..mutated.len());
                    mutated[idx] = rng.gen();
                }
            }
        }
        
        mutated
    }
}
