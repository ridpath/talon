#![allow(dead_code)]

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
}
