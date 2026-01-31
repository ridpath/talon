use crate::session_state::{ExploitSession, SessionState};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TimeTravelDebugger {
    session: Arc<ExploitSession>,
    recorder: Arc<RwLock<EventRecorder>>,
    playback: Arc<RwLock<PlaybackEngine>>,
}

#[derive(Debug, Clone)]
pub struct EventRecorder {
    events: VecDeque<ExploitEvent>,
    max_events: usize,
    recording: bool,
}

#[derive(Debug, Clone)]
pub struct ExploitEvent {
    pub id: u64,
    pub timestamp: std::time::Instant,
    pub event_type: EventType,
    pub state_before: Option<SessionState>,
    pub state_after: Option<SessionState>,
}

#[derive(Debug, Clone)]
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
        TimeTravelDebugger {
            session,
            recorder: Arc::new(RwLock::new(EventRecorder::new(10000))),
            playback: Arc::new(RwLock::new(PlaybackEngine::new())),
        }
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
