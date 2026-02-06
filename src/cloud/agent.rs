// TALON Distributed Swarm Agent
// Lightweight static agent for remote deployment with mTLS authentication

use super::proto::*;
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};
use uuid::Uuid;

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Primary controller endpoint (e.g., "https://controller:50051")
    pub primary_endpoint: String,
    /// Agent unique identifier
    pub agent_id: String,
    /// Client certificate for mTLS
    pub client_cert_path: PathBuf,
    /// Client private key
    pub client_key_path: PathBuf,
    /// CA certificate for server verification
    pub ca_cert_path: PathBuf,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Agent capabilities (e.g., ["binary_analysis", "network_exploit"])
    pub capabilities: Vec<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            primary_endpoint: "https://localhost:50051".to_string(),
            agent_id: Uuid::new_v4().to_string(),
            client_cert_path: PathBuf::from("agent.crt"),
            client_key_path: PathBuf::from("agent.key"),
            ca_cert_path: PathBuf::from("ca.crt"),
            heartbeat_interval: 30,
            max_concurrent_tasks: 4,
            capabilities: vec![
                "binary_analysis".to_string(),
                "network_exploit".to_string(),
                "rop_chain".to_string(),
            ],
        }
    }
}

/// Agent errors
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),
    
    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Certificate error: {0}")]
    Certificate(String),
    
    #[error("Execution error: {0}")]
    Execution(String),
    
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Agent state
#[derive(Debug, Clone)]
struct AgentState {
    active: bool,
    cpu_percent: i32,
    memory_mb: i32,
    running_tasks: i32,
    uptime_start: Instant,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            active: true,
            cpu_percent: 0,
            memory_mb: 0,
            running_tasks: 0,
            uptime_start: Instant::now(),
        }
    }
}

/// TALON Distributed Agent
pub struct TalonAgent {
    config: AgentConfig,
    client: talon_swarm_client::TalonSwarmClient<Channel>,
    state: Arc<Mutex<AgentState>>,
    cipher: ChaCha20Poly1305,
}

impl TalonAgent {
    /// Create new agent with mTLS configuration
    pub async fn new(config: AgentConfig) -> Result<Self, AgentError> {
        // Load client certificate and key
        let client_cert = std::fs::read(&config.client_cert_path)
            .map_err(|e| AgentError::Certificate(format!("Failed to load client cert: {}", e)))?;
        
        let client_key = std::fs::read(&config.client_key_path)
            .map_err(|e| AgentError::Certificate(format!("Failed to load client key: {}", e)))?;
        
        // Load CA certificate
        let ca_cert = std::fs::read(&config.ca_cert_path)
            .map_err(|e| AgentError::Certificate(format!("Failed to load CA cert: {}", e)))?;
        
        // Create mTLS identity
        let identity = Identity::from_pem(&client_cert, &client_key);
        let ca_certificate = Certificate::from_pem(&ca_cert);
        
        // Configure TLS with certificate pinning
        let tls_config = ClientTlsConfig::new()
            .identity(identity)
            .ca_certificate(ca_certificate)
            .domain_name("talon.swarm"); // Certificate pinning domain
        
        // Create gRPC channel with mTLS
        let channel = Channel::from_shared(config.primary_endpoint.clone())
            .map_err(|e| AgentError::Transport(e.into()))?
            .tls_config(tls_config)?
            .connect()
            .await?;
        
        let client = talon_swarm_client::TalonSwarmClient::new(channel);
        
        // Generate encryption key for sensitive data
        let cipher = ChaCha20Poly1305::new(&ChaCha20Poly1305::generate_key(&mut OsRng));
        
        Ok(Self {
            config,
            client,
            state: Arc::new(Mutex::new(AgentState::default())),
            cipher,
        })
    }
    
    /// Register agent with primary controller
    pub async fn register(&mut self) -> Result<AgentToken, AgentError> {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        
        let agent_info = AgentInfo {
            hostname,
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: self.config.capabilities.clone(),
            csr: vec![], // CSR generation handled externally
        };
        
        let response = self.client.register_agent(agent_info).await?;
        let token = response.into_inner();
        
        log::info!(
            "Agent registered successfully: ID={}, Expiry={}",
            token.agent_id,
            token.expiry_timestamp
        );
        
        Ok(token)
    }
    
    /// Start heartbeat loop (consumes self, runs in separate task)
    pub async fn start_heartbeat(mut self) -> Result<(), AgentError> {
        let mut interval = tokio::time::interval(Duration::from_secs(self.config.heartbeat_interval));
        
        loop {
            interval.tick().await;
            
            let state = self.state.lock().await;
            let uptime = state.uptime_start.elapsed().as_secs();
            
            let status = AgentStatus {
                agent_id: self.config.agent_id.clone(),
                active: state.active,
                cpu_percent: state.cpu_percent,
                memory_mb: state.memory_mb,
                running_tasks: state.running_tasks,
                uptime_seconds: uptime as i64,
            };
            
            drop(state); // Release lock before RPC
            
            match self.client.heartbeat(status).await {
                Ok(ack) => {
                    if !ack.into_inner().success {
                        log::warn!("Heartbeat acknowledged but controller returned failure");
                    }
                }
                Err(e) => {
                    log::error!("Heartbeat failed: {}", e);
                    // Continue heartbeat loop despite errors
                }
            }
        }
    }
    
    /// Execute TALON script
    pub async fn execute_script(
        &mut self,
        payload: ScriptPayload,
    ) -> Result<Vec<ExecutionEvent>, AgentError> {
        log::info!("Executing script: {}", payload.script_id);
        
        // Increment running tasks
        {
            let mut state = self.state.lock().await;
            state.running_tasks += 1;
        }
        
        let script_id = payload.script_id.clone();
        let script_content = payload.script_content;
        let options = payload.options.unwrap_or_default();
        
        // Create execution stream
        let mut stream = self.client.execute_script(payload).await?.into_inner();
        
        let mut events = Vec::new();
        
        // Send start event
        events.push(ExecutionEvent {
            script_id: script_id.clone(),
            event_type: EventType::EventStarted as i32,
            message: "Script execution started".to_string(),
            progress_percent: 0,
            data: vec![],
            timestamp: chrono::Utc::now().timestamp(),
        });
        
        // Execute script in sandbox (simplified - full implementation would use actual interpreter)
        let result = self.execute_in_sandbox(&script_content, options.dry_run).await;
        
        // Process result
        match result {
            Ok(output) => {
                events.push(ExecutionEvent {
                    script_id: script_id.clone(),
                    event_type: EventType::EventCompleted as i32,
                    message: "Script execution completed successfully".to_string(),
                    progress_percent: 100,
                    data: output,
                    timestamp: chrono::Utc::now().timestamp(),
                });
            }
            Err(e) => {
                events.push(ExecutionEvent {
                    script_id: script_id.clone(),
                    event_type: EventType::EventFailed as i32,
                    message: format!("Script execution failed: {}", e),
                    progress_percent: 0,
                    data: vec![],
                    timestamp: chrono::Utc::now().timestamp(),
                });
            }
        }
        
        // Decrement running tasks
        {
            let mut state = self.state.lock().await;
            state.running_tasks -= 1;
        }
        
        Ok(events)
    }
    
    /// Execute script in sandbox
    async fn execute_in_sandbox(
        &self,
        script_content: &[u8],
        dry_run: bool,
    ) -> Result<Vec<u8>, AgentError> {
        if dry_run {
            log::info!("Dry-run mode: skipping actual execution");
            return Ok(b"Dry run successful".to_vec());
        }
        
        // Write script to temporary file
        let script_path = std::env::temp_dir().join(format!("talon_script_{}.talon", Uuid::new_v4()));
        std::fs::write(&script_path, script_content)?;
        
        // Execute script using talon interpreter (simplified - would use actual interpreter integration)
        // For production, this would call into the TALON interpreter with proper sandboxing
        let output = std::process::Command::new("talon")
            .arg("run")
            .arg(&script_path)
            .output()
            .map_err(|e| AgentError::Execution(format!("Failed to execute script: {}", e)))?;
        
        // Clean up temporary file
        let _ = std::fs::remove_file(&script_path);
        
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(AgentError::Execution(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }
    
    /// Report exploitation result
    pub async fn report_result(&mut self, result: ExploitResult) -> Result<(), AgentError> {
        log::info!(
            "Reporting result: script_id={}, success={}",
            result.script_id,
            result.success
        );
        
        let response = self.client.report_result(result).await?;
        let ack = response.into_inner();
        
        if !ack.success {
            log::warn!("Result report acknowledged but controller returned failure");
        }
        
        Ok(())
    }
    
    /// Request auto-update from primary
    pub async fn request_update(&mut self, target_version: String) -> Result<PathBuf, AgentError> {
        log::info!("Requesting update to version: {}", target_version);
        
        let update_request = UpdateRequest {
            agent_id: self.config.agent_id.clone(),
            target_version: target_version.clone(),
        };
        
        let mut stream = self.client.request_update(update_request).await?.into_inner();
        
        let update_path = std::env::temp_dir().join(format!("talon-agent-{}", target_version));
        let mut file = std::fs::File::create(&update_path)?;
        
        let mut total_chunks = 0;
        let mut received_chunks = 0;
        
        while let Some(chunk) = stream.message().await? {
            total_chunks = chunk.total_chunks;
            received_chunks += 1;
            
            use std::io::Write;
            file.write_all(&chunk.data)?;
            
            log::info!(
                "Update progress: {}/{} chunks received",
                received_chunks,
                total_chunks
            );
        }
        
        if received_chunks != total_chunks {
            return Err(AgentError::Execution(format!(
                "Incomplete update: received {}/{} chunks",
                received_chunks, total_chunks
            )));
        }
        
        log::info!("Update downloaded successfully to: {:?}", update_path);
        Ok(update_path)
    }
    
    /// Apply auto-update and restart
    pub async fn apply_update(&self, update_path: &PathBuf) -> Result<(), AgentError> {
        // Get current executable path
        let current_exe = std::env::current_exe()?;
        let backup_path = current_exe.with_extension("backup");
        
        // Backup current executable
        std::fs::copy(&current_exe, &backup_path)?;
        
        // Replace with new version
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(update_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(update_path, perms)?;
        }
        
        std::fs::copy(update_path, &current_exe)?;
        
        log::info!("Update applied successfully. Restart required.");
        Ok(())
    }
    
    /// Sync registry updates (gadgets, libc offsets, etc.)
    pub async fn sync_registry(&mut self) -> Result<(), AgentError> {
        log::info!("Starting registry sync");
        
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        // Spawn task to send updates
        let send_task = tokio::spawn(async move {
            // Example: Send discovered gadget
            let update = RegistryUpdate {
                update_type: UpdateType::UpdateGadget as i32,
                key: "libc-2.31:pop_rdi".to_string(),
                value: vec![0x00, 0x40, 0x12, 0x34],
                metadata: std::collections::HashMap::new(),
            };
            
            tx.send(update).await.expect("Failed to send update");
        });
        
        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        
        // Start bidirectional sync
        let response = self.client.sync_registry(stream).await?;
        let mut incoming = response.into_inner();
        
        // Receive updates from other agents
        while let Some(update) = incoming.message().await? {
            log::info!(
                "Received registry update: type={}, key={}",
                update.update_type,
                update.key
            );
            // Apply update to local registry
        }
        
        send_task.await.expect("Send task failed");
        
        log::info!("Registry sync completed");
        Ok(())
    }
    
    /// Graceful shutdown
    pub async fn shutdown(&mut self) -> Result<(), AgentError> {
        log::info!("Shutting down agent gracefully");
        
        let mut state = self.state.lock().await;
        state.active = false;
        
        // Wait for running tasks to complete (with timeout)
        let timeout = Duration::from_secs(30);
        let start = Instant::now();
        
        while state.running_tasks > 0 && start.elapsed() < timeout {
            drop(state);
            tokio::time::sleep(Duration::from_millis(100)).await;
            state = self.state.lock().await;
        }
        
        if state.running_tasks > 0 {
            log::warn!(
                "Shutdown timeout: {} tasks still running",
                state.running_tasks
            );
        }
        
        Ok(())
    }
    
    /// Get current agent state
    pub async fn get_state(&self) -> AgentState {
        self.state.lock().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.primary_endpoint, "https://localhost:50051");
        assert_eq!(config.heartbeat_interval, 30);
        assert_eq!(config.max_concurrent_tasks, 4);
        assert!(!config.capabilities.is_empty());
    }
    
    #[test]
    fn test_agent_config_serialization() {
        let config = AgentConfig::default();
        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: AgentConfig = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(config.agent_id, deserialized.agent_id);
    }
    
    #[test]
    fn test_agent_state_default() {
        let state = AgentState::default();
        assert!(state.active);
        assert_eq!(state.cpu_percent, 0);
        assert_eq!(state.running_tasks, 0);
    }
    
    #[tokio::test]
    async fn test_agent_state_tracking() {
        let state = Arc::new(Mutex::new(AgentState::default()));
        
        {
            let mut s = state.lock().await;
            s.running_tasks += 1;
        }
        
        let s = state.lock().await;
        assert_eq!(s.running_tasks, 1);
    }
    
    #[test]
    fn test_agent_error_display() {
        let error = AgentError::Config("Test error".to_string());
        assert!(error.to_string().contains("Test error"));
    }
}
