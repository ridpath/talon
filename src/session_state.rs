#![allow(clippy::upper_case_acronyms)]

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ExploitSession {
    state: Arc<RwLock<SessionState>>,
    history: Arc<RwLock<Vec<StateSnapshot>>>,
}

#[derive(Debug, Clone)]
pub struct SessionState {
    pub target: TargetInfo,
    pub memory: MemoryState,
    pub addresses: AddressTable,
    pub connections: HashMap<String, ConnectionInfo>,
    pub variables: HashMap<String, SessionValue>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub pid: Option<u32>,
    pub address: Option<SocketAddr>,
    pub binary_path: Option<String>,
    pub architecture: Architecture,
    pub protections: ProtectionFlags,
}

#[derive(Debug, Clone)]
pub enum Architecture {
    X86,
    X86_64,
    ARM,
    ARM64,
    MIPS,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ProtectionFlags {
    pub nx: bool,
    pub pie: bool,
    pub canary: bool,
    pub aslr: bool,
    pub relro: RelroLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelroLevel {
    None,
    Partial,
    Full,
}

#[derive(Debug, Clone)]
pub struct MemoryState {
    pub stack_pointer: Option<u64>,
    pub base_pointer: Option<u64>,
    pub instruction_pointer: Option<u64>,
    pub registers: HashMap<String, u64>,
    pub allocations: Vec<MemoryAllocation>,
}

#[derive(Debug, Clone)]
pub struct MemoryAllocation {
    pub address: u64,
    pub size: usize,
    pub permissions: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AddressTable {
    pub libc_base: Option<u64>,
    pub heap_base: Option<u64>,
    pub stack_base: Option<u64>,
    pub binary_base: Option<u64>,
    pub symbols: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub connection_type: ConnectionType,
    pub status: ConnectionStatus,
    pub bytes_sent: usize,
    pub bytes_received: usize,
    pub established_at: std::time::Instant,
}

#[derive(Debug, Clone)]
pub enum ConnectionType {
    Socket(SocketAddr),
    Process(u32),
    Serial(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error,
}

#[derive(Debug, Clone)]
pub enum SessionValue {
    Integer(i64),
    String(String),
    Bytes(Vec<u8>),
    Address(u64),
    List(Vec<SessionValue>),
    Map(HashMap<String, SessionValue>),
}

#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub id: u64,
    pub timestamp: std::time::Instant,
    pub state: SessionState,
    pub label: Option<String>,
}

impl ExploitSession {
    pub fn new() -> Self {
        ExploitSession {
            state: Arc::new(RwLock::new(SessionState::default())),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn connect(target: &str, port: u16) -> Result<Self, String> {
        let session = Self::new();

        let addr = format!("{}:{}", target, port)
            .parse::<SocketAddr>()
            .map_err(|e| format!("Invalid address: {}", e))?;

        {
            let mut state = session.state.write().await;
            state.target.address = Some(addr);
        }

        Ok(session)
    }

    pub async fn attach(pid: u32) -> Result<Self, String> {
        let session = Self::new();

        {
            let mut state = session.state.write().await;
            state.target.pid = Some(pid);
        }

        Ok(session)
    }

    pub async fn get_state(&self) -> SessionState {
        let state = self.state.read().await;
        state.clone()
    }

    pub async fn update_state<F>(&self, updater: F) -> Result<(), String>
    where
        F: FnOnce(&mut SessionState) -> Result<(), String>,
    {
        let mut state = self.state.write().await;
        updater(&mut state)
    }

    pub async fn set_libc_base(&self, address: u64) {
        let mut state = self.state.write().await;
        state.addresses.libc_base = Some(address);
    }

    pub async fn get_libc_base(&self) -> Option<u64> {
        let state = self.state.read().await;
        state.addresses.libc_base
    }

    pub async fn set_symbol(&self, name: String, address: u64) {
        let mut state = self.state.write().await;
        state.addresses.symbols.insert(name, address);
    }

    pub async fn get_symbol(&self, name: &str) -> Option<u64> {
        let state = self.state.read().await;
        state.addresses.symbols.get(name).copied()
    }

    pub async fn set_register(&self, name: String, value: u64) {
        let mut state = self.state.write().await;
        state.memory.registers.insert(name, value);
    }

    pub async fn get_register(&self, name: &str) -> Option<u64> {
        let state = self.state.read().await;
        state.memory.registers.get(name).copied()
    }

    pub async fn set_variable(&self, name: String, value: SessionValue) {
        let mut state = self.state.write().await;
        state.variables.insert(name, value);
    }

    pub async fn get_variable(&self, name: &str) -> Option<SessionValue> {
        let state = self.state.read().await;
        state.variables.get(name).cloned()
    }

    pub async fn add_connection(&self, name: String, info: ConnectionInfo) {
        let mut state = self.state.write().await;
        state.connections.insert(name, info);
    }

    pub async fn get_connection(&self, name: &str) -> Option<ConnectionInfo> {
        let state = self.state.read().await;
        state.connections.get(name).cloned()
    }

    pub async fn set_metadata(&self, key: String, value: String) {
        let mut state = self.state.write().await;
        state.metadata.insert(key, value);
    }

    pub async fn get_metadata(&self, key: &str) -> Option<String> {
        let state = self.state.read().await;
        state.metadata.get(key).cloned()
    }

    pub async fn checkpoint(&self) -> Result<u64, String> {
        let state = self.state.read().await;
        let mut history = self.history.write().await;

        let snapshot_id = history.len() as u64 + 1;
        let snapshot = StateSnapshot {
            id: snapshot_id,
            timestamp: std::time::Instant::now(),
            state: state.clone(),
            label: None,
        };

        history.push(snapshot);
        Ok(snapshot_id)
    }

    pub async fn checkpoint_labeled(&self, label: String) -> Result<u64, String> {
        let state = self.state.read().await;
        let mut history = self.history.write().await;

        let snapshot_id = history.len() as u64 + 1;
        let snapshot = StateSnapshot {
            id: snapshot_id,
            timestamp: std::time::Instant::now(),
            state: state.clone(),
            label: Some(label),
        };

        history.push(snapshot);
        Ok(snapshot_id)
    }

    pub async fn rewind(&self, snapshot_id: u64) -> Result<(), String> {
        let history = self.history.read().await;

        let snapshot = history
            .iter()
            .find(|s| s.id == snapshot_id)
            .ok_or_else(|| format!("Checkpoint {} not found", snapshot_id))?;

        let mut state = self.state.write().await;
        *state = snapshot.state.clone();

        Ok(())
    }

    pub async fn rewind_to_label(&self, label: &str) -> Result<(), String> {
        let history = self.history.read().await;

        let snapshot = history
            .iter()
            .rev()
            .find(|s| s.label.as_deref() == Some(label))
            .ok_or_else(|| format!("Checkpoint with label '{}' not found", label))?;

        let mut state = self.state.write().await;
        *state = snapshot.state.clone();

        Ok(())
    }

    pub async fn list_checkpoints(&self) -> Vec<StateSnapshot> {
        let history = self.history.read().await;
        history.clone()
    }

    pub async fn clear_history(&self) {
        let mut history = self.history.write().await;
        history.clear();
    }
}

impl Default for SessionState {
    fn default() -> Self {
        SessionState {
            target: TargetInfo {
                pid: None,
                address: None,
                binary_path: None,
                architecture: Architecture::Unknown,
                protections: ProtectionFlags {
                    nx: false,
                    pie: false,
                    canary: false,
                    aslr: false,
                    relro: RelroLevel::None,
                },
            },
            memory: MemoryState {
                stack_pointer: None,
                base_pointer: None,
                instruction_pointer: None,
                registers: HashMap::new(),
                allocations: Vec::new(),
            },
            addresses: AddressTable {
                libc_base: None,
                heap_base: None,
                stack_base: None,
                binary_base: None,
                symbols: HashMap::new(),
            },
            connections: HashMap::new(),
            variables: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_creation() {
        let session = ExploitSession::new();
        let state = session.get_state().await;
        assert!(state.target.pid.is_none());
    }

    #[tokio::test]
    async fn test_libc_base() {
        let session = ExploitSession::new();
        session.set_libc_base(0x7ffff7a00000).await;
        let base = session.get_libc_base().await;
        assert_eq!(base, Some(0x7ffff7a00000));
    }

    #[tokio::test]
    async fn test_checkpoint_rewind() {
        let session = ExploitSession::new();
        session.set_libc_base(0x1000).await;

        let checkpoint = session.checkpoint().await.unwrap();

        session.set_libc_base(0x2000).await;
        assert_eq!(session.get_libc_base().await, Some(0x2000));

        session.rewind(checkpoint).await.unwrap();
        assert_eq!(session.get_libc_base().await, Some(0x1000));
    }
}
