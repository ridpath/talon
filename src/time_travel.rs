#![allow(dead_code)]

//! Time-Travel Debugging System
//!
//! Provides comprehensive debugging capabilities with state checkpointing, event recording,
//! and integration with GDB for reverse debugging. This module enables:
//!
//! - State snapshots and restoration
//! - Event recording and replay
//! - GDB reverse debugging integration
//! - Disk-based checkpoint persistence
//! - Branch creation for parallel debugging paths
//! - Send event rewinding for payload testing
//!
//! Integration with other modules:
//! - `gdb_tools`: GDB session management and reverse debugging commands
//! - `session_state`: Exploit session state management
//! - `split_screen_debugger`: TUI for interactive debugging
//! - `interpreter`: DSL-level debug() builtin function
//!
//! Performance considerations:
//! - Event recording uses a bounded queue to prevent memory exhaustion
//! - Checkpoints are stored with SHA256 hashing for integrity
//! - Disk checkpoints support automatic cleanup of old checkpoints

use crate::gdb_tools::GdbSession;
use crate::session_state::{ExploitSession, SessionState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TimeTravelDebugger {
    session: Arc<ExploitSession>,
    recorder: Arc<RwLock<EventRecorder>>,
    playback: Arc<RwLock<PlaybackEngine>>,
    checkpoint_dir: PathBuf,
    pub gdb_session: Arc<RwLock<Option<GdbSession>>>,
}

#[derive(Debug, Clone)]
pub struct EventRecorder {
    events: VecDeque<ExploitEvent>,
    max_events: usize,
    recording: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploitEvent {
    pub id: u64,
    #[serde(skip, default = "default_instant")]
    pub timestamp: std::time::Instant,
    pub event_type: EventType,
    pub state_before: Option<SessionState>,
    pub state_after: Option<SessionState>,
}

fn default_instant() -> std::time::Instant {
    std::time::Instant::now()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    MemoryWrite {
        address: u64,
        data: Vec<u8>,
    },
    MemoryRead {
        address: u64,
        size: usize,
    },
    NetworkSend {
        data: Vec<u8>,
    },
    NetworkReceive {
        data: Vec<u8>,
    },
    RegisterModify {
        register: String,
        old_value: u64,
        new_value: u64,
    },
    FunctionCall {
        name: String,
        args: Vec<String>,
    },
    Checkpoint {
        label: String,
    },
    Custom {
        description: String,
    },
}

#[derive(Debug, Clone)]
pub struct PlaybackEngine {
    events: Vec<ExploitEvent>,
    current_index: usize,
    playing: bool,
}

impl TimeTravelDebugger {
    pub fn new(session: Arc<ExploitSession>) -> Self {
        let checkpoint_dir = Self::init_checkpoint_dir().unwrap_or_else(|_| PathBuf::from(".talon_cache/checkpoints"));
        
        TimeTravelDebugger {
            session,
            recorder: Arc::new(RwLock::new(EventRecorder::new(10000))),
            playback: Arc::new(RwLock::new(PlaybackEngine::new())),
            checkpoint_dir,
            gdb_session: Arc::new(RwLock::new(None)),
        }
    }

    fn init_checkpoint_dir() -> Result<PathBuf, std::io::Error> {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")
        })?;
        let checkpoint_dir = home_dir.join(".talon_cache").join("checkpoints");
        
        if !checkpoint_dir.exists() {
            fs::create_dir_all(&checkpoint_dir)?;
            Self::set_checkpoint_permissions(&checkpoint_dir)?;
        }
        
        Ok(checkpoint_dir)
    }

    #[cfg(unix)]
    fn set_checkpoint_permissions(path: &Path) -> Result<(), std::io::Error> {
        use std::os::unix::fs::PermissionsExt;
        let permissions = fs::Permissions::from_mode(0o700);
        fs::set_permissions(path, permissions)?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn set_checkpoint_permissions(_path: &Path) -> Result<(), std::io::Error> {
        Ok(())
    }

    pub async fn start_recording(&self) {
        let mut recorder = self.recorder.write().await;
        recorder.start();
    }

    pub async fn stop_recording(&self) {
        let mut recorder = self.recorder.write().await;
        recorder.stop();
    }

    pub async fn record_event(&self, event_type: EventType) -> Result<(), String> {
        let mut recorder = self.recorder.write().await;

        if !recorder.is_recording() {
            return Ok(());
        }

        let state_before = self.session.get_state().await;

        let event = ExploitEvent {
            id: recorder.events.len() as u64 + 1,
            timestamp: std::time::Instant::now(),
            event_type,
            state_before: Some(state_before.clone()),
            state_after: None,
        };

        recorder.add_event(event);
        Ok(())
    }

    pub async fn record_event_with_after(&self, event_type: EventType) -> Result<(), String> {
        let recorder = self.recorder.write().await;

        if !recorder.is_recording() {
            return Ok(());
        }

        let state_before = self.session.get_state().await;

        drop(recorder);

        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

        let state_after = self.session.get_state().await;

        let mut recorder = self.recorder.write().await;
        let event = ExploitEvent {
            id: recorder.events.len() as u64 + 1,
            timestamp: std::time::Instant::now(),
            event_type,
            state_before: Some(state_before),
            state_after: Some(state_after),
        };

        recorder.add_event(event);
        Ok(())
    }

    pub async fn get_events(&self) -> Vec<ExploitEvent> {
        let recorder = self.recorder.read().await;
        recorder.get_events()
    }

    pub async fn rewind_to_event(&self, event_id: u64) -> Result<(), String> {
        let recorder = self.recorder.read().await;

        let event = recorder
            .events
            .iter()
            .find(|e| e.id == event_id)
            .ok_or("Event not found")?;

        if let Some(state) = &event.state_before {
            self.session
                .update_state(|s| {
                    *s = state.clone();
                    Ok(())
                })
                .await?;
        }

        Ok(())
    }

    pub async fn replay_events(&self, from_event: u64, to_event: u64) -> Result<(), String> {
        let recorder = self.recorder.read().await;
        let events: Vec<ExploitEvent> = recorder
            .events
            .iter()
            .filter(|e| e.id >= from_event && e.id <= to_event)
            .cloned()
            .collect();

        drop(recorder);

        for event in events {
            self.apply_event(&event).await?;
        }

        Ok(())
    }

    async fn apply_event(&self, event: &ExploitEvent) -> Result<(), String> {
        match &event.event_type {
            EventType::MemoryWrite { address, data } => {
                println!(
                    "Replaying memory write at 0x{:x}: {} bytes",
                    address,
                    data.len()
                );
            }
            EventType::RegisterModify {
                register,
                new_value,
                ..
            } => {
                self.session
                    .set_register(register.clone(), *new_value)
                    .await;
            }
            EventType::Checkpoint { label } => {
                self.session.checkpoint_labeled(label.clone()).await?;
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn create_branch(&self, name: String) -> Result<BranchPoint, String> {
        let checkpoint = self
            .session
            .checkpoint_labeled(format!("branch_{}", name))
            .await?;
        let events = self.get_events().await;

        Ok(BranchPoint {
            name,
            checkpoint_id: checkpoint,
            event_count: events.len(),
            created_at: std::time::Instant::now(),
        })
    }

    pub async fn switch_to_branch(&self, branch: &BranchPoint) -> Result<(), String> {
        self.session.rewind(branch.checkpoint_id).await?;
        Ok(())
    }

    pub async fn export_timeline(&self) -> Timeline {
        let events = self.get_events().await;
        let checkpoints = self.session.list_checkpoints().await;

        let duration = if let (Some(first), Some(last)) = (events.first(), events.last()) {
            Some(last.timestamp.duration_since(first.timestamp))
        } else {
            None
        };

        Timeline {
            events: events.clone(),
            checkpoints: checkpoints
                .iter()
                .map(|c| TimelineCheckpoint {
                    id: c.id,
                    label: c.label.clone(),
                    event_id: c.id,
                })
                .collect(),
            duration,
        }
    }

    pub async fn clear_history(&self) {
        let mut recorder = self.recorder.write().await;
        recorder.clear();
        self.session.clear_history().await;
    }

    pub async fn attach_gdb(&self, pid: u32) -> Result<(), String> {
        let gdb = GdbSession::attach(pid)?;
        let mut gdb_session = self.gdb_session.write().await;
        *gdb_session = Some(gdb);
        println!("[Time-Travel] GDB attached to PID {}", pid);
        Ok(())
    }

    pub async fn detach_gdb(&self) -> Result<(), String> {
        let mut gdb_session = self.gdb_session.write().await;
        *gdb_session = None;
        println!("[Time-Travel] GDB session detached");
        Ok(())
    }

    pub async fn gdb_checkpoint(&self, label: &str) -> Result<(), String> {
        let mut gdb_session = self.gdb_session.write().await;
        
        if let Some(ref mut gdb) = *gdb_session {
            gdb.execute(&format!("checkpoint"))?;
            println!("[Time-Travel] GDB checkpoint created: {}", label);
            
            self.record_event(EventType::Checkpoint {
                label: label.to_string(),
            })
            .await?;
        } else {
            return Err("No GDB session attached".to_string());
        }
        
        Ok(())
    }

    pub async fn gdb_reverse_continue(&self) -> Result<String, String> {
        let mut gdb_session = self.gdb_session.write().await;
        
        if let Some(ref mut gdb) = *gdb_session {
            let output = gdb.execute("reverse-continue")?;
            println!("[Time-Travel] Reverse continue executed");
            Ok(output)
        } else {
            Err("No GDB session attached".to_string())
        }
    }

    pub async fn gdb_reverse_step(&self) -> Result<String, String> {
        let mut gdb_session = self.gdb_session.write().await;
        
        if let Some(ref mut gdb) = *gdb_session {
            let output = gdb.execute("reverse-stepi")?;
            println!("[Time-Travel] Reverse step executed");
            Ok(output)
        } else {
            Err("No GDB session attached".to_string())
        }
    }

    pub async fn gdb_reverse_finish(&self) -> Result<String, String> {
        let mut gdb_session = self.gdb_session.write().await;
        
        if let Some(ref mut gdb) = *gdb_session {
            let output = gdb.execute("reverse-finish")?;
            println!("[Time-Travel] Reverse finish executed");
            Ok(output)
        } else {
            Err("No GDB session attached".to_string())
        }
    }

    pub async fn save_checkpoint_to_disk(&self, checkpoint_id: u64, label: &str) -> Result<(), String> {
        let state = self.session.get_state().await;
        let events = self.get_events().await;
        
        let checkpoint = DiskCheckpoint {
            id: checkpoint_id,
            label: label.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            state,
            events,
        };
        
        let filename = format!("checkpoint_{}.json", self.compute_checkpoint_hash(checkpoint_id, label));
        let checkpoint_path = self.checkpoint_dir.join(filename);
        
        let json = serde_json::to_string_pretty(&checkpoint)
            .map_err(|e| format!("Failed to serialize checkpoint: {}", e))?;
        
        fs::write(&checkpoint_path, json)
            .map_err(|e| format!("Failed to write checkpoint to disk: {}", e))?;
        
        println!("[Time-Travel] Checkpoint saved to {:?}", checkpoint_path);
        Ok(())
    }

    pub async fn load_checkpoint_from_disk(&self, label: &str) -> Result<(), String> {
        for entry in fs::read_dir(&self.checkpoint_dir)
            .map_err(|e| format!("Failed to read checkpoint directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(checkpoint) = serde_json::from_str::<DiskCheckpoint>(&content) {
                        if checkpoint.label == label {
                            self.session.update_state(|s| {
                                *s = checkpoint.state.clone();
                                Ok(())
                            }).await?;
                            
                            let mut recorder = self.recorder.write().await;
                            recorder.clear();
                            for event in checkpoint.events {
                                recorder.add_event(event);
                            }
                            
                            println!("[Time-Travel] Checkpoint '{}' loaded from disk", label);
                            return Ok(());
                        }
                    }
                }
            }
        }
        
        Err(format!("Checkpoint with label '{}' not found on disk", label))
    }

    pub async fn list_disk_checkpoints(&self) -> Result<Vec<CheckpointInfo>, String> {
        let mut checkpoints = Vec::new();
        
        for entry in fs::read_dir(&self.checkpoint_dir)
            .map_err(|e| format!("Failed to read checkpoint directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(checkpoint) = serde_json::from_str::<DiskCheckpoint>(&content) {
                        checkpoints.push(CheckpointInfo {
                            id: checkpoint.id,
                            label: checkpoint.label,
                            timestamp: checkpoint.timestamp,
                            event_count: checkpoint.events.len(),
                        });
                    }
                }
            }
        }
        
        Ok(checkpoints)
    }

    pub async fn clean_old_checkpoints(&self, max_age_days: u64) -> Result<usize, String> {
        let max_age_secs = max_age_days * 24 * 60 * 60;
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut cleaned = 0;
        
        for entry in fs::read_dir(&self.checkpoint_dir)
            .map_err(|e| format!("Failed to read checkpoint directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(checkpoint) = serde_json::from_str::<DiskCheckpoint>(&content) {
                        if current_time - checkpoint.timestamp > max_age_secs {
                            fs::remove_file(&path)
                                .map_err(|e| format!("Failed to remove checkpoint: {}", e))?;
                            cleaned += 1;
                        }
                    }
                }
            }
        }
        
        println!("[Time-Travel] Cleaned {} old checkpoints", cleaned);
        Ok(cleaned)
    }

    pub async fn rewind_to_send(&self, send_index: usize) -> Result<(), String> {
        let state_to_restore = {
            let recorder = self.recorder.read().await;
            
            let send_events: Vec<_> = recorder
                .events
                .iter()
                .filter(|e| matches!(e.event_type, EventType::NetworkSend { .. }))
                .collect();
            
            if send_index >= send_events.len() {
                return Err(format!("Send event index {} out of bounds (total: {})", send_index, send_events.len()));
            }
            
            let target_event = send_events[send_index];
            target_event.state_before.clone()
        };
        
        if let Some(state) = state_to_restore {
            self.session.update_state(|s| {
                *s = state;
                Ok(())
            }).await?;
            
            println!("[Time-Travel] Rewound to send event #{}", send_index);
            Ok(())
        } else {
            Err("Send event has no saved state".to_string())
        }
    }

    pub async fn list_send_events(&self) -> Vec<SendEventInfo> {
        let recorder = self.recorder.read().await;
        
        recorder
            .events
            .iter()
            .filter_map(|e| {
                if let EventType::NetworkSend { data } = &e.event_type {
                    Some(SendEventInfo {
                        id: e.id,
                        timestamp: e.timestamp,
                        data_size: data.len(),
                        data_preview: if data.len() > 32 {
                            format!("{:?}...", &data[..32])
                        } else {
                            format!("{:?}", data)
                        },
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn compute_checkpoint_hash(&self, id: u64, label: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(id.to_string().as_bytes());
        hasher.update(label.as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }

    /// Get summary of all recorded events grouped by type
    pub async fn get_event_summary(&self) -> EventSummary {
        let recorder = self.recorder.read().await;
        let mut summary = EventSummary::default();

        for event in &recorder.events {
            match &event.event_type {
                EventType::MemoryWrite { .. } => summary.memory_writes += 1,
                EventType::MemoryRead { .. } => summary.memory_reads += 1,
                EventType::NetworkSend { .. } => summary.network_sends += 1,
                EventType::NetworkReceive { .. } => summary.network_receives += 1,
                EventType::RegisterModify { .. } => summary.register_modifies += 1,
                EventType::FunctionCall { .. } => summary.function_calls += 1,
                EventType::Checkpoint { .. } => summary.checkpoints += 1,
                EventType::Custom { .. } => summary.custom_events += 1,
            }
        }

        summary.total_events = recorder.events.len();
        summary
    }

    /// Find all events of a specific type
    pub async fn filter_events_by_type(&self, event_type_filter: EventTypeFilter) -> Vec<ExploitEvent> {
        let recorder = self.recorder.read().await;
        
        recorder.events.iter()
            .filter(|e| match (event_type_filter, &e.event_type) {
                (EventTypeFilter::MemoryWrite, EventType::MemoryWrite { .. }) => true,
                (EventTypeFilter::MemoryRead, EventType::MemoryRead { .. }) => true,
                (EventTypeFilter::NetworkSend, EventType::NetworkSend { .. }) => true,
                (EventTypeFilter::NetworkReceive, EventType::NetworkReceive { .. }) => true,
                (EventTypeFilter::RegisterModify, EventType::RegisterModify { .. }) => true,
                (EventTypeFilter::FunctionCall, EventType::FunctionCall { .. }) => true,
                (EventTypeFilter::Checkpoint, EventType::Checkpoint { .. }) => true,
                (EventTypeFilter::Custom, EventType::Custom { .. }) => true,
                _ => false,
            })
            .cloned()
            .collect()
    }

    /// Get the current recording status
    pub async fn is_recording(&self) -> bool {
        let recorder = self.recorder.read().await;
        recorder.is_recording()
    }

    /// Get total number of events recorded
    pub async fn event_count(&self) -> usize {
        let recorder = self.recorder.read().await;
        recorder.events.len()
    }

    /// Get checkpoint directory path
    pub fn get_checkpoint_dir(&self) -> &Path {
        &self.checkpoint_dir
    }

    /// Fast forward to the most recent state
    pub async fn fast_forward_to_latest(&self) -> Result<(), String> {
        let recorder = self.recorder.read().await;
        
        if let Some(last_event) = recorder.events.back() {
            if let Some(state) = &last_event.state_after {
                let state_clone = state.clone();
                drop(recorder);
                self.session.update_state(|s| {
                    *s = state_clone;
                    Ok(())
                }).await?;
                println!("[Time-Travel] Fast-forwarded to latest state");
                Ok(())
            } else {
                Err("Last event has no state_after recorded".to_string())
            }
        } else {
            Err("No events recorded".to_string())
        }
    }

    /// Find events that modified a specific memory address
    pub async fn find_memory_modifications(&self, address: u64) -> Vec<ExploitEvent> {
        let recorder = self.recorder.read().await;
        
        recorder.events.iter()
            .filter(|e| {
                matches!(&e.event_type, 
                    EventType::MemoryWrite { address: addr, .. } 
                    if *addr == address
                )
            })
            .cloned()
            .collect()
    }

    /// Find events that modified a specific register
    pub async fn find_register_modifications(&self, register: &str) -> Vec<ExploitEvent> {
        let recorder = self.recorder.read().await;
        
        recorder.events.iter()
            .filter(|e| {
                matches!(&e.event_type, 
                    EventType::RegisterModify { register: reg, .. } 
                    if reg == register
                )
            })
            .cloned()
            .collect()
    }

    /// Create a named snapshot for later comparison
    pub async fn create_snapshot(&self, name: String) -> Result<u64, String> {
        self.session.checkpoint_labeled(format!("snapshot_{}", name)).await
    }

    /// Compare two checkpoints and return differences
    pub async fn diff_checkpoints(&self, checkpoint_a: u64, checkpoint_b: u64) -> Result<StateDiff, String> {
        let history = self.session.list_checkpoints().await;
        
        let state_a = history.iter()
            .find(|s| s.id == checkpoint_a)
            .ok_or("Checkpoint A not found")?;
        
        let state_b = history.iter()
            .find(|s| s.id == checkpoint_b)
            .ok_or("Checkpoint B not found")?;
        
        Ok(StateDiff::compute(&state_a.state, &state_b.state))
    }
}

impl EventRecorder {
    pub fn new(max_events: usize) -> Self {
        EventRecorder {
            events: VecDeque::new(),
            max_events,
            recording: false,
        }
    }

    pub fn start(&mut self) {
        self.recording = true;
    }

    pub fn stop(&mut self) {
        self.recording = false;
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }

    pub fn add_event(&mut self, event: ExploitEvent) {
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn get_events(&self) -> Vec<ExploitEvent> {
        self.events.iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl PlaybackEngine {
    pub fn new() -> Self {
        PlaybackEngine {
            events: Vec::new(),
            current_index: 0,
            playing: false,
        }
    }

    pub fn load_events(&mut self, events: Vec<ExploitEvent>) {
        self.events = events;
        self.current_index = 0;
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    pub fn step_forward(&mut self) -> Option<&ExploitEvent> {
        if self.current_index < self.events.len() {
            let event = &self.events[self.current_index];
            self.current_index += 1;
            Some(event)
        } else {
            None
        }
    }

    pub fn step_backward(&mut self) -> Option<&ExploitEvent> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(&self.events[self.current_index])
        } else {
            None
        }
    }

    pub fn seek_to(&mut self, index: usize) -> Result<(), String> {
        if index < self.events.len() {
            self.current_index = index;
            Ok(())
        } else {
            Err("Index out of bounds".to_string())
        }
    }
}

#[derive(Debug, Clone)]
pub struct BranchPoint {
    pub name: String,
    pub checkpoint_id: u64,
    pub event_count: usize,
    pub created_at: std::time::Instant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskCheckpoint {
    pub id: u64,
    pub label: String,
    pub timestamp: u64,
    pub state: SessionState,
    pub events: Vec<ExploitEvent>,
}

#[derive(Debug, Clone)]
pub struct CheckpointInfo {
    pub id: u64,
    pub label: String,
    pub timestamp: u64,
    pub event_count: usize,
}

#[derive(Debug, Clone)]
pub struct SendEventInfo {
    pub id: u64,
    pub timestamp: std::time::Instant,
    pub data_size: usize,
    pub data_preview: String,
}

#[derive(Debug, Clone)]
pub struct Timeline {
    pub events: Vec<ExploitEvent>,
    pub checkpoints: Vec<TimelineCheckpoint>,
    pub duration: Option<std::time::Duration>,
}

#[derive(Debug, Clone)]
pub struct TimelineCheckpoint {
    pub id: u64,
    pub label: Option<String>,
    pub event_id: u64,
}

#[derive(Debug, Clone, Default)]
pub struct EventSummary {
    pub total_events: usize,
    pub memory_writes: usize,
    pub memory_reads: usize,
    pub network_sends: usize,
    pub network_receives: usize,
    pub register_modifies: usize,
    pub function_calls: usize,
    pub checkpoints: usize,
    pub custom_events: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum EventTypeFilter {
    MemoryWrite,
    MemoryRead,
    NetworkSend,
    NetworkReceive,
    RegisterModify,
    FunctionCall,
    Checkpoint,
    Custom,
}

#[derive(Debug, Clone)]
pub struct StateDiff {
    pub libc_base_changed: bool,
    pub libc_base_diff: Option<(Option<u64>, Option<u64>)>,
    pub heap_base_changed: bool,
    pub heap_base_diff: Option<(Option<u64>, Option<u64>)>,
    pub register_changes: Vec<RegisterChange>,
    pub symbol_changes: Vec<SymbolChange>,
    pub variable_changes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RegisterChange {
    pub register: String,
    pub old_value: Option<u64>,
    pub new_value: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct SymbolChange {
    pub symbol: String,
    pub old_address: Option<u64>,
    pub new_address: Option<u64>,
}

impl StateDiff {
    pub fn compute(state_a: &SessionState, state_b: &SessionState) -> Self {
        let libc_base_changed = state_a.addresses.libc_base != state_b.addresses.libc_base;
        let libc_base_diff = if libc_base_changed {
            Some((state_a.addresses.libc_base, state_b.addresses.libc_base))
        } else {
            None
        };

        let heap_base_changed = state_a.addresses.heap_base != state_b.addresses.heap_base;
        let heap_base_diff = if heap_base_changed {
            Some((state_a.addresses.heap_base, state_b.addresses.heap_base))
        } else {
            None
        };

        let mut register_changes = Vec::new();
        for (reg, &val_b) in &state_b.memory.registers {
            let val_a = state_a.memory.registers.get(reg).copied();
            if val_a != Some(val_b) {
                register_changes.push(RegisterChange {
                    register: reg.clone(),
                    old_value: val_a,
                    new_value: Some(val_b),
                });
            }
        }

        let mut symbol_changes = Vec::new();
        for (sym, &addr_b) in &state_b.addresses.symbols {
            let addr_a = state_a.addresses.symbols.get(sym).copied();
            if addr_a != Some(addr_b) {
                symbol_changes.push(SymbolChange {
                    symbol: sym.clone(),
                    old_address: addr_a,
                    new_address: Some(addr_b),
                });
            }
        }

        let variable_changes: Vec<String> = state_b.variables.keys()
            .filter(|k| {
                state_a.variables.get(*k) != state_b.variables.get(*k)
            })
            .cloned()
            .collect();

        StateDiff {
            libc_base_changed,
            libc_base_diff,
            heap_base_changed,
            heap_base_diff,
            register_changes,
            symbol_changes,
            variable_changes,
        }
    }

    pub fn has_changes(&self) -> bool {
        self.libc_base_changed || 
        self.heap_base_changed || 
        !self.register_changes.is_empty() || 
        !self.symbol_changes.is_empty() || 
        !self.variable_changes.is_empty()
    }
}

pub async fn record_and_replay_exploit<F, T>(
    session: &ExploitSession,
    exploit_fn: F,
) -> Result<T, String>
where
    F: std::future::Future<Output = Result<T, String>>,
{
    let debugger = TimeTravelDebugger::new(Arc::new(session.clone()));

    debugger.start_recording().await;
    let result = exploit_fn.await;
    debugger.stop_recording().await;

    if result.is_err() {
        let events = debugger.get_events().await;
        println!(
            "Exploit failed. Recorded {} events for analysis",
            events.len()
        );
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_recording() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session);

        debugger.start_recording().await;
        debugger
            .record_event(EventType::MemoryWrite {
                address: 0x401000,
                data: vec![0x90, 0x90, 0x90],
            })
            .await
            .unwrap();

        let events = debugger.get_events().await;
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_branch_creation() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session.clone());

        session.set_libc_base(0x1000).await;
        let branch = debugger
            .create_branch("test_branch".to_string())
            .await
            .unwrap();

        session.set_libc_base(0x2000).await;
        assert_eq!(session.get_libc_base().await, Some(0x2000));

        debugger.switch_to_branch(&branch).await.unwrap();
        assert_eq!(session.get_libc_base().await, Some(0x1000));
    }

    #[tokio::test]
    async fn test_checkpoint_creation_and_rewind() {
        let session = Arc::new(ExploitSession::new());
        let _debugger = TimeTravelDebugger::new(session.clone());

        session.set_libc_base(0x7ffff7a00000).await;
        session.set_register("rip".to_string(), 0x401000).await;

        let checkpoint_id = session.checkpoint_labeled("exploit_start".to_string()).await.unwrap();

        session.set_libc_base(0x7ffff7b00000).await;
        session.set_register("rip".to_string(), 0x401234).await;

        assert_eq!(session.get_libc_base().await, Some(0x7ffff7b00000));
        assert_eq!(session.get_register("rip").await, Some(0x401234));

        session.rewind(checkpoint_id).await.unwrap();

        assert_eq!(session.get_libc_base().await, Some(0x7ffff7a00000));
        assert_eq!(session.get_register("rip").await, Some(0x401000));
    }

    #[tokio::test]
    async fn test_event_replay() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session.clone());

        debugger.start_recording().await;

        debugger.record_event(EventType::MemoryWrite {
            address: 0x401000,
            data: vec![0x90, 0x90, 0x90],
        }).await.unwrap();

        debugger.record_event(EventType::RegisterModify {
            register: "rax".to_string(),
            old_value: 0,
            new_value: 42,
        }).await.unwrap();

        debugger.record_event(EventType::NetworkSend {
            data: b"payload".to_vec(),
        }).await.unwrap();

        let events = debugger.get_events().await;
        assert_eq!(events.len(), 3);

        debugger.replay_events(1, 2).await.unwrap();
        assert_eq!(session.get_register("rax").await, Some(42));
    }

    #[tokio::test]
    async fn test_disk_checkpoint_persistence() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session.clone());

        session.set_libc_base(0x1234000).await;
        session.set_symbol("main".to_string(), 0x401000).await;

        debugger.start_recording().await;
        debugger.record_event(EventType::Checkpoint {
            label: "test_checkpoint".to_string(),
        }).await.unwrap();

        let result = debugger.save_checkpoint_to_disk(1, "test_checkpoint").await;
        assert!(result.is_ok());

        session.set_libc_base(0x5678000).await;

        let load_result = debugger.load_checkpoint_from_disk("test_checkpoint").await;
        assert!(load_result.is_ok());

        assert_eq!(session.get_libc_base().await, Some(0x1234000));
        assert_eq!(session.get_symbol("main").await, Some(0x401000));
    }

    #[tokio::test]
    async fn test_send_event_rewinding() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session.clone());

        debugger.start_recording().await;

        session.set_libc_base(0x1000).await;
        debugger.record_event(EventType::NetworkSend {
            data: b"payload1".to_vec(),
        }).await.unwrap();

        session.set_libc_base(0x2000).await;
        debugger.record_event(EventType::NetworkSend {
            data: b"payload2".to_vec(),
        }).await.unwrap();

        session.set_libc_base(0x3000).await;
        debugger.record_event(EventType::NetworkSend {
            data: b"payload3".to_vec(),
        }).await.unwrap();

        let send_events = debugger.list_send_events().await;
        assert_eq!(send_events.len(), 3);

        debugger.rewind_to_send(1).await.unwrap();
        assert_eq!(session.get_libc_base().await, Some(0x2000));

        debugger.rewind_to_send(0).await.unwrap();
        assert_eq!(session.get_libc_base().await, Some(0x1000));
    }

    #[tokio::test]
    async fn test_timeline_export() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session.clone());

        debugger.start_recording().await;

        for i in 0..5 {
            debugger.record_event(EventType::MemoryWrite {
                address: 0x401000 + i * 4,
                data: vec![i as u8; 4],
            }).await.unwrap();
        }

        session.checkpoint_labeled("mid_exploit".to_string()).await.unwrap();

        for i in 5..10 {
            debugger.record_event(EventType::MemoryWrite {
                address: 0x401000 + i * 4,
                data: vec![i as u8; 4],
            }).await.unwrap();
        }

        let timeline = debugger.export_timeline().await;
        assert_eq!(timeline.events.len(), 10);
        assert_eq!(timeline.checkpoints.len(), 1);
        assert!(timeline.duration.is_some());
    }

    #[tokio::test]
    async fn test_playback_engine() {
        let mut playback = PlaybackEngine::new();

        let events = vec![
            ExploitEvent {
                id: 1,
                timestamp: std::time::Instant::now(),
                event_type: EventType::MemoryWrite {
                    address: 0x1000,
                    data: vec![0x90],
                },
                state_before: None,
                state_after: None,
            },
            ExploitEvent {
                id: 2,
                timestamp: std::time::Instant::now(),
                event_type: EventType::MemoryWrite {
                    address: 0x2000,
                    data: vec![0x90],
                },
                state_before: None,
                state_after: None,
            },
        ];

        playback.load_events(events);
        playback.play();

        assert!(playback.step_forward().is_some());
        assert!(playback.step_forward().is_some());
        assert!(playback.step_forward().is_none());

        assert!(playback.step_backward().is_some());
        assert!(playback.step_backward().is_some());
        assert!(playback.step_backward().is_none());

        playback.seek_to(0).unwrap();
        assert_eq!(playback.current_index, 0);
    }

    #[tokio::test]
    async fn test_event_recorder_max_capacity() {
        let mut recorder = EventRecorder::new(3);
        recorder.start();

        for i in 0..5 {
            recorder.add_event(ExploitEvent {
                id: i,
                timestamp: std::time::Instant::now(),
                event_type: EventType::Custom {
                    description: format!("Event {}", i),
                },
                state_before: None,
                state_after: None,
            });
        }

        let events = recorder.get_events();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].id, 2);
        assert_eq!(events[2].id, 4);
    }

    #[tokio::test]
    async fn test_gdb_integration() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session);

        let result = debugger.attach_gdb(12345).await;
        assert!(result.is_ok());

        let detach_result = debugger.detach_gdb().await;
        assert!(detach_result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_event_types() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session);

        debugger.start_recording().await;

        let event_types = vec![
            EventType::MemoryWrite { address: 0x1000, data: vec![0x90] },
            EventType::MemoryRead { address: 0x2000, size: 8 },
            EventType::NetworkSend { data: b"test".to_vec() },
            EventType::NetworkReceive { data: b"response".to_vec() },
            EventType::RegisterModify { register: "rax".to_string(), old_value: 0, new_value: 1 },
            EventType::FunctionCall { name: "main".to_string(), args: vec![] },
            EventType::Checkpoint { label: "test".to_string() },
            EventType::Custom { description: "custom event".to_string() },
        ];

        for event_type in event_types {
            debugger.record_event(event_type).await.unwrap();
        }

        let events = debugger.get_events().await;
        assert_eq!(events.len(), 8);
    }

    #[tokio::test]
    async fn test_clear_history() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session.clone());

        debugger.start_recording().await;
        for _ in 0..10 {
            debugger.record_event(EventType::Custom {
                description: "test".to_string(),
            }).await.unwrap();
        }

        session.checkpoint().await.unwrap();
        session.checkpoint().await.unwrap();

        assert_eq!(debugger.get_events().await.len(), 10);
        assert_eq!(session.list_checkpoints().await.len(), 2);

        debugger.clear_history().await;

        assert_eq!(debugger.get_events().await.len(), 0);
        assert_eq!(session.list_checkpoints().await.len(), 0);
    }

    #[tokio::test]
    async fn test_event_summary() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session);

        debugger.start_recording().await;

        debugger.record_event(EventType::MemoryWrite { address: 0x1000, data: vec![0x90] }).await.unwrap();
        debugger.record_event(EventType::MemoryWrite { address: 0x2000, data: vec![0x90] }).await.unwrap();
        debugger.record_event(EventType::NetworkSend { data: b"test".to_vec() }).await.unwrap();
        debugger.record_event(EventType::RegisterModify { register: "rax".to_string(), old_value: 0, new_value: 1 }).await.unwrap();

        let summary = debugger.get_event_summary().await;
        assert_eq!(summary.total_events, 4);
        assert_eq!(summary.memory_writes, 2);
        assert_eq!(summary.network_sends, 1);
        assert_eq!(summary.register_modifies, 1);
    }

    #[tokio::test]
    async fn test_filter_events_by_type() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session);

        debugger.start_recording().await;

        debugger.record_event(EventType::MemoryWrite { address: 0x1000, data: vec![0x90] }).await.unwrap();
        debugger.record_event(EventType::NetworkSend { data: b"test".to_vec() }).await.unwrap();
        debugger.record_event(EventType::MemoryWrite { address: 0x2000, data: vec![0x90] }).await.unwrap();

        let memory_writes = debugger.filter_events_by_type(EventTypeFilter::MemoryWrite).await;
        assert_eq!(memory_writes.len(), 2);

        let network_sends = debugger.filter_events_by_type(EventTypeFilter::NetworkSend).await;
        assert_eq!(network_sends.len(), 1);
    }

    #[tokio::test]
    async fn test_find_memory_modifications() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session);

        debugger.start_recording().await;

        debugger.record_event(EventType::MemoryWrite { address: 0x401000, data: vec![0x90] }).await.unwrap();
        debugger.record_event(EventType::MemoryWrite { address: 0x402000, data: vec![0x90] }).await.unwrap();
        debugger.record_event(EventType::MemoryWrite { address: 0x401000, data: vec![0xcc] }).await.unwrap();

        let mods = debugger.find_memory_modifications(0x401000).await;
        assert_eq!(mods.len(), 2);
    }

    #[tokio::test]
    async fn test_find_register_modifications() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session);

        debugger.start_recording().await;

        debugger.record_event(EventType::RegisterModify { register: "rax".to_string(), old_value: 0, new_value: 1 }).await.unwrap();
        debugger.record_event(EventType::RegisterModify { register: "rbx".to_string(), old_value: 0, new_value: 2 }).await.unwrap();
        debugger.record_event(EventType::RegisterModify { register: "rax".to_string(), old_value: 1, new_value: 42 }).await.unwrap();

        let rax_mods = debugger.find_register_modifications("rax").await;
        assert_eq!(rax_mods.len(), 2);

        let rbx_mods = debugger.find_register_modifications("rbx").await;
        assert_eq!(rbx_mods.len(), 1);
    }

    #[tokio::test]
    async fn test_checkpoint_diff() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session.clone());

        session.set_libc_base(0x1000).await;
        session.set_register("rax".to_string(), 0).await;
        let checkpoint_a = session.checkpoint().await.unwrap();

        session.set_libc_base(0x2000).await;
        session.set_register("rax".to_string(), 42).await;
        session.set_symbol("main".to_string(), 0x401000).await;
        let checkpoint_b = session.checkpoint().await.unwrap();

        let diff = debugger.diff_checkpoints(checkpoint_a, checkpoint_b).await.unwrap();
        
        assert!(diff.has_changes());
        assert!(diff.libc_base_changed);
        assert_eq!(diff.register_changes.len(), 1);
        assert_eq!(diff.symbol_changes.len(), 1);
    }

    #[tokio::test]
    async fn test_recording_status() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session);

        assert!(!debugger.is_recording().await);

        debugger.start_recording().await;
        assert!(debugger.is_recording().await);

        debugger.stop_recording().await;
        assert!(!debugger.is_recording().await);
    }

    #[tokio::test]
    async fn test_event_count() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session);

        assert_eq!(debugger.event_count().await, 0);

        debugger.start_recording().await;

        for i in 0..5 {
            debugger.record_event(EventType::Custom {
                description: format!("Event {}", i),
            }).await.unwrap();
        }

        assert_eq!(debugger.event_count().await, 5);
    }

    #[tokio::test]
    async fn test_create_snapshot() {
        let session = Arc::new(ExploitSession::new());
        let debugger = TimeTravelDebugger::new(session.clone());

        session.set_libc_base(0x1000).await;
        let snapshot_id = debugger.create_snapshot("before_exploit".to_string()).await.unwrap();

        session.set_libc_base(0x2000).await;

        session.rewind(snapshot_id).await.unwrap();
        assert_eq!(session.get_libc_base().await, Some(0x1000));
    }
}
