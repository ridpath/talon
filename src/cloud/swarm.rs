// TALON Distributed Swarm Primary Controller
// Manages agent connections, script distribution, and result aggregation

use super::proto::*;
use super::registry_sync::RegistrySync;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tonic::transport::{Certificate, Identity, Server, ServerTlsConfig};
use uuid::Uuid;

/// Swarm controller configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    /// gRPC server listen address (e.g., "0.0.0.0:50051")
    pub listen_addr: String,
    /// Server certificate for mTLS
    pub server_cert_path: PathBuf,
    /// Server private key
    pub server_key_path: PathBuf,
    /// CA certificate for client verification
    pub ca_cert_path: PathBuf,
    /// Redis connection string (optional)
    pub redis_url: Option<String>,
    /// PostgreSQL connection string (optional)
    pub postgres_url: Option<String>,
    /// Agent heartbeat timeout (seconds)
    pub heartbeat_timeout: u64,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:50051".to_string(),
            server_cert_path: PathBuf::from("server.crt"),
            server_key_path: PathBuf::from("server.key"),
            ca_cert_path: PathBuf::from("ca.crt"),
            redis_url: Some("redis://127.0.0.1:6379".to_string()),
            postgres_url: Some("postgresql://talon:talon@localhost/talon_swarm".to_string()),
            heartbeat_timeout: 90,
        }
    }
}

/// Agent inventory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEntry {
    pub agent_id: String,
    pub hostname: String,
    pub endpoint: String,
    pub os: String,
    pub arch: String,
    pub capabilities: Vec<String>,
    pub tags: Vec<String>,
    pub last_heartbeat: Option<Instant>,
    pub active: bool,
}

/// Script execution request with filtering
#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    pub script_path: PathBuf,
    pub target_agents: TargetAgents,
    pub dry_run: bool,
    pub timeout_seconds: i32,
    pub max_retries: i32,
}

/// Agent targeting strategy
#[derive(Debug, Clone)]
pub enum TargetAgents {
    All,
    ByIds(Vec<String>),
    ByTags(Vec<String>),
    ByCapabilities(Vec<String>),
    ByOs(String),
    ByArch(String),
}

/// Swarm controller errors
#[derive(Debug, thiserror::Error)]
pub enum SwarmError {
    #[error("gRPC error: {0}")]
    Grpc(#[from] tonic::Status),
    
    #[error("Transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Certificate error: {0}")]
    Certificate(String),
    
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Inventory error: {0}")]
    Inventory(String),
    
    #[error("Redis error: {0}")]
    Redis(String),
    
    #[error("Postgres error: {0}")]
    Postgres(String),
    
    #[error("Script error: {0}")]
    Script(String),
}

/// Aggregated execution results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResults {
    pub total_agents: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<ExploitResult>,
    pub execution_time_ms: i64,
}

/// TALON Swarm Primary Controller
pub struct SwarmController {
    config: SwarmConfig,
    agents: Arc<RwLock<HashMap<String, AgentEntry>>>,
    results: Arc<Mutex<HashMap<String, Vec<ExploitResult>>>>,
    registry_sync: Arc<RegistrySync>,
    #[cfg(feature = "redis")]
    redis_client: Option<redis::Client>,
    #[cfg(feature = "postgres")]
    pg_pool: Option<tokio_postgres::Client>,
}

impl SwarmController {
    /// Create new swarm controller
    pub async fn new(config: SwarmConfig) -> Result<Self, SwarmError> {
        let registry_sync = Arc::new(RegistrySync::new());
        
        // Initialize Redis connection if configured
        #[cfg(feature = "redis")]
        let redis_client = if let Some(ref url) = config.redis_url {
            match redis::Client::open(url.as_str()) {
                Ok(client) => {
                    log::info!("Redis connection configured: {}", url);
                    Some(client)
                }
                Err(e) => {
                    log::warn!("Failed to configure Redis: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        #[cfg(not(feature = "redis"))]
        let _redis_client: Option<()> = None;
        
        // Initialize PostgreSQL connection if configured
        #[cfg(feature = "postgres")]
        let pg_pool = if let Some(ref url) = config.postgres_url {
            match tokio_postgres::connect(url, tokio_postgres::NoTls).await {
                Ok((client, connection)) => {
                    tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            log::error!("PostgreSQL connection error: {}", e);
                        }
                    });
                    log::info!("PostgreSQL connection configured");
                    Some(client)
                }
                Err(e) => {
                    log::warn!("Failed to configure PostgreSQL: {}", e);
                    None
                }
            }
        } else {
            None
        };
        
        #[cfg(not(feature = "postgres"))]
        let _pg_pool: Option<()> = None;
        
        Ok(Self {
            config,
            agents: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(Mutex::new(HashMap::new())),
            registry_sync,
            #[cfg(feature = "redis")]
            redis_client,
            #[cfg(feature = "postgres")]
            pg_pool,
        })
    }
    
    /// Load agent inventory from INI file
    pub async fn load_inventory(&self, path: &Path) -> Result<usize, SwarmError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| SwarmError::Inventory(format!("Failed to read inventory: {}", e)))?;
        
        let mut agents = self.agents.write().await;
        let mut count = 0;
        
        let mut current_group = String::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            
            // Group headers [group_name]
            if line.starts_with('[') && line.ends_with(']') {
                current_group = line[1..line.len() - 1].to_string();
                continue;
            }
            
            // Agent entry: hostname endpoint [key=value ...]
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let hostname = parts[0].to_string();
                let endpoint = parts[1].to_string();
                
                let mut tags = vec![current_group.clone()];
                let mut capabilities = Vec::new();
                let mut os = "linux".to_string();
                let mut arch = "x86_64".to_string();
                
                // Parse key=value metadata
                for part in parts.iter().skip(2) {
                    if let Some((key, value)) = part.split_once('=') {
                        match key {
                            "os" => os = value.to_string(),
                            "arch" => arch = value.to_string(),
                            "tag" => tags.push(value.to_string()),
                            "cap" => capabilities.push(value.to_string()),
                            _ => {}
                        }
                    }
                }
                
                let agent_id = Uuid::new_v4().to_string();
                let entry = AgentEntry {
                    agent_id: agent_id.clone(),
                    hostname,
                    endpoint,
                    os,
                    arch,
                    capabilities,
                    tags,
                    last_heartbeat: None,
                    active: false,
                };
                
                agents.insert(agent_id, entry);
                count += 1;
            }
        }
        
        log::info!("Loaded {} agents from inventory", count);
        Ok(count)
    }
    
    /// Execute script on targeted agents
    pub async fn execute_script(
        &self,
        request: ExecutionRequest,
    ) -> Result<AggregatedResults, SwarmError> {
        let script_content = std::fs::read(&request.script_path)
            .map_err(|e| SwarmError::Script(format!("Failed to read script: {}", e)))?;
        
        let script_id = Uuid::new_v4().to_string();
        let start_time = Instant::now();
        
        // Filter agents based on targeting strategy
        let target_agents = self.filter_agents(&request.target_agents).await?;
        
        log::info!(
            "Executing script {} on {} agents",
            request.script_path.display(),
            target_agents.len()
        );
        
        // Broadcast script to all targeted agents
        let mut handles = Vec::new();
        let results = Arc::new(Mutex::new(Vec::new()));
        
        for agent in target_agents.iter() {
            let agent_id = agent.agent_id.clone();
            let script_id = script_id.clone();
            let results = Arc::clone(&results);
            let dry_run = request.dry_run;
            
            let handle = tokio::spawn(async move {
                // In production, this would connect to agent and execute
                // Simplified implementation for now
                let result = ExploitResult {
                    script_id,
                    target_host: agent_id.clone(),
                    success: !dry_run, // Simulate success in dry-run mode
                    error_message: String::new(),
                    loot: vec![],
                    metadata: HashMap::new(),
                    duration_ms: 100,
                    timestamp: chrono::Utc::now().timestamp(),
                };
                
                results.lock().await.push(result);
            });
            
            handles.push(handle);
        }
        
        // Wait for all agents to complete
        for handle in handles {
            let _ = handle.await;
        }
        
        let results_vec = results.lock().await.clone();
        let successful = results_vec.iter().filter(|r| r.success).count();
        let failed = results_vec.len() - successful;
        
        let aggregated = AggregatedResults {
            total_agents: target_agents.len(),
            successful,
            failed,
            results: results_vec,
            execution_time_ms: start_time.elapsed().as_millis() as i64,
        };
        
        // Store results
        self.results.lock().await.insert(script_id, aggregated.results.clone());
        
        log::info!(
            "Script execution completed: {}/{} successful in {}ms",
            successful,
            target_agents.len(),
            aggregated.execution_time_ms
        );
        
        Ok(aggregated)
    }
    
    /// Filter agents based on targeting strategy
    async fn filter_agents(&self, target: &TargetAgents) -> Result<Vec<AgentEntry>, SwarmError> {
        let agents = self.agents.read().await;
        
        let filtered: Vec<AgentEntry> = match target {
            TargetAgents::All => agents.values().cloned().collect(),
            
            TargetAgents::ByIds(ids) => {
                agents
                    .iter()
                    .filter(|(id, _)| ids.contains(id))
                    .map(|(_, agent)| agent.clone())
                    .collect()
            }
            
            TargetAgents::ByTags(tags) => {
                agents
                    .values()
                    .filter(|agent| {
                        agent.tags.iter().any(|tag| tags.contains(tag))
                    })
                    .cloned()
                    .collect()
            }
            
            TargetAgents::ByCapabilities(caps) => {
                agents
                    .values()
                    .filter(|agent| {
                        caps.iter().all(|cap| agent.capabilities.contains(cap))
                    })
                    .cloned()
                    .collect()
            }
            
            TargetAgents::ByOs(os) => {
                agents
                    .values()
                    .filter(|agent| agent.os == *os)
                    .cloned()
                    .collect()
            }
            
            TargetAgents::ByArch(arch) => {
                agents
                    .values()
                    .filter(|agent| agent.arch == *arch)
                    .cloned()
                    .collect()
            }
        };
        
        if filtered.is_empty() {
            return Err(SwarmError::AgentNotFound("No agents matched filter criteria".to_string()));
        }
        
        Ok(filtered)
    }
    
    /// Get agent status
    pub async fn get_agent_status(&self, agent_id: &str) -> Result<AgentEntry, SwarmError> {
        let agents = self.agents.read().await;
        agents
            .get(agent_id)
            .cloned()
            .ok_or_else(|| SwarmError::AgentNotFound(agent_id.to_string()))
    }
    
    /// List all agents
    pub async fn list_agents(&self) -> Vec<AgentEntry> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }
    
    /// Get results for a script
    pub async fn get_results(&self, script_id: &str) -> Option<Vec<ExploitResult>> {
        let results = self.results.lock().await;
        results.get(script_id).cloned()
    }
    
    /// Start swarm gRPC server
    pub async fn start_server(self: Arc<Self>) -> Result<(), SwarmError> {
        // Load mTLS certificates
        let server_cert = std::fs::read(&self.config.server_cert_path)
            .map_err(|e| SwarmError::Certificate(format!("Failed to load server cert: {}", e)))?;
        
        let server_key = std::fs::read(&self.config.server_key_path)
            .map_err(|e| SwarmError::Certificate(format!("Failed to load server key: {}", e)))?;
        
        let ca_cert = std::fs::read(&self.config.ca_cert_path)
            .map_err(|e| SwarmError::Certificate(format!("Failed to load CA cert: {}", e)))?;
        
        let identity = Identity::from_pem(&server_cert, &server_key);
        let ca_certificate = Certificate::from_pem(&ca_cert);
        
        let tls_config = ServerTlsConfig::new()
            .identity(identity)
            .client_ca_root(ca_certificate);
        
        let addr = self.config.listen_addr.parse()
            .map_err(|e| SwarmError::Transport(tonic::transport::Error::from_source(e)))?;
        
        let server = SwarmServer {
            controller: Arc::clone(&self),
        };
        
        log::info!("Starting swarm server on {}", addr);
        
        Server::builder()
            .tls_config(tls_config)?
            .add_service(talon_swarm_server::TalonSwarmServer::new(server))
            .serve(addr)
            .await?;
        
        Ok(())
    }
    
    /// Update agent heartbeat
    async fn update_heartbeat(&self, agent_id: &str) {
        if let Some(agent) = self.agents.write().await.get_mut(agent_id) {
            agent.last_heartbeat = Some(Instant::now());
            agent.active = true;
        }
    }
    
    /// Check for stale agents (no heartbeat)
    pub async fn check_stale_agents(&self) -> Vec<String> {
        let timeout = Duration::from_secs(self.config.heartbeat_timeout);
        let mut stale = Vec::new();
        
        let mut agents = self.agents.write().await;
        for (id, agent) in agents.iter_mut() {
            if let Some(last_heartbeat) = agent.last_heartbeat {
                if last_heartbeat.elapsed() > timeout {
                    agent.active = false;
                    stale.push(id.clone());
                }
            }
        }
        
        if !stale.is_empty() {
            log::warn!("Found {} stale agents", stale.len());
        }
        
        stale
    }
    
    /// Get registry sync instance
    pub fn registry_sync(&self) -> Arc<RegistrySync> {
        Arc::clone(&self.registry_sync)
    }
}

/// gRPC server implementation
struct SwarmServer {
    controller: Arc<SwarmController>,
}

#[tonic::async_trait]
impl talon_swarm_server::TalonSwarm for SwarmServer {
    async fn register_agent(
        &self,
        request: tonic::Request<AgentInfo>,
    ) -> Result<tonic::Response<AgentToken>, tonic::Status> {
        let info = request.into_inner();
        let agent_id = Uuid::new_v4().to_string();
        
        log::info!(
            "Agent registration: {} ({}:{})",
            info.hostname,
            info.os,
            info.arch
        );
        
        let entry = AgentEntry {
            agent_id: agent_id.clone(),
            hostname: info.hostname,
            endpoint: "".to_string(), // Will be updated on first heartbeat
            os: info.os,
            arch: info.arch,
            capabilities: info.capabilities,
            tags: vec![],
            last_heartbeat: Some(Instant::now()),
            active: true,
        };
        
        self.controller.agents.write().await.insert(agent_id.clone(), entry);
        
        let token = AgentToken {
            agent_id,
            certificate: vec![], // Certificate issuance handled externally
            ca_certificate: vec![],
            expiry_timestamp: (chrono::Utc::now() + chrono::Duration::days(30)).timestamp(),
        };
        
        Ok(tonic::Response::new(token))
    }
    
    type ExecuteScriptStream = futures::stream::Empty<Result<ExecutionEvent, tonic::Status>>;
    
    async fn execute_script(
        &self,
        _request: tonic::Request<ScriptPayload>,
    ) -> Result<tonic::Response<Self::ExecuteScriptStream>, tonic::Status> {
        // Simplified - full implementation would stream execution events
        Ok(tonic::Response::new(futures::stream::empty()))
    }
    
    async fn report_result(
        &self,
        request: tonic::Request<ExploitResult>,
    ) -> Result<tonic::Response<Acknowledgment>, tonic::Status> {
        let result = request.into_inner();
        
        log::info!(
            "Result reported: script={}, success={}",
            result.script_id,
            result.success
        );
        
        // Store result
        self.controller
            .results
            .lock()
            .await
            .entry(result.script_id.clone())
            .or_insert_with(Vec::new)
            .push(result);
        
        Ok(tonic::Response::new(Acknowledgment {
            success: true,
            message: "Result received".to_string(),
        }))
    }
    
    type SyncRegistryStream = futures::stream::Empty<Result<RegistryUpdate, tonic::Status>>;
    
    async fn sync_registry(
        &self,
        request: tonic::Request<tonic::Streaming<RegistryUpdate>>,
    ) -> Result<tonic::Response<Self::SyncRegistryStream>, tonic::Status> {
        let mut stream = request.into_inner();
        let registry = self.controller.registry_sync();
        
        // Process incoming updates from agent
        tokio::spawn(async move {
            while let Ok(Some(update)) = stream.message().await {
                registry.apply_update(update).await;
            }
        });
        
        // Return empty stream for now - full implementation would broadcast updates
        Ok(tonic::Response::new(futures::stream::empty()))
    }
    
    async fn heartbeat(
        &self,
        request: tonic::Request<AgentStatus>,
    ) -> Result<tonic::Response<Acknowledgment>, tonic::Status> {
        let status = request.into_inner();
        
        self.controller.update_heartbeat(&status.agent_id).await;
        
        log::debug!(
            "Heartbeat: {} (active={}, tasks={})",
            status.agent_id,
            status.active,
            status.running_tasks
        );
        
        Ok(tonic::Response::new(Acknowledgment {
            success: true,
            message: "Heartbeat acknowledged".to_string(),
        }))
    }
    
    async fn terminate(
        &self,
        request: tonic::Request<TerminateRequest>,
    ) -> Result<tonic::Response<Acknowledgment>, tonic::Status> {
        let req = request.into_inner();
        
        log::info!("Termination request for agent: {}", req.agent_id);
        
        if let Some(agent) = self.controller.agents.write().await.get_mut(&req.agent_id) {
            agent.active = false;
        }
        
        Ok(tonic::Response::new(Acknowledgment {
            success: true,
            message: "Agent marked for termination".to_string(),
        }))
    }
    
    type RequestUpdateStream = futures::stream::Empty<Result<UpdateChunk, tonic::Status>>;
    
    async fn request_update(
        &self,
        _request: tonic::Request<UpdateRequest>,
    ) -> Result<tonic::Response<Self::RequestUpdateStream>, tonic::Status> {
        // Simplified - full implementation would stream update chunks
        Ok(tonic::Response::new(futures::stream::empty()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_swarm_config_default() {
        let config = SwarmConfig::default();
        assert_eq!(config.listen_addr, "0.0.0.0:50051");
        assert_eq!(config.heartbeat_timeout, 90);
    }
    
    #[tokio::test]
    async fn test_agent_filtering_all() {
        let controller = create_test_controller().await;
        let target = TargetAgents::All;
        let filtered = controller.filter_agents(&target).await.expect("Filter failed");
        assert_eq!(filtered.len(), 3);
    }
    
    #[tokio::test]
    async fn test_agent_filtering_by_os() {
        let controller = create_test_controller().await;
        let target = TargetAgents::ByOs("linux".to_string());
        let filtered = controller.filter_agents(&target).await.expect("Filter failed");
        assert_eq!(filtered.len(), 2);
    }
    
    #[tokio::test]
    async fn test_agent_filtering_by_arch() {
        let controller = create_test_controller().await;
        let target = TargetAgents::ByArch("x86_64".to_string());
        let filtered = controller.filter_agents(&target).await.expect("Filter failed");
        assert_eq!(filtered.len(), 3);
    }
    
    #[tokio::test]
    async fn test_agent_filtering_by_capabilities() {
        let controller = create_test_controller().await;
        let target = TargetAgents::ByCapabilities(vec!["binary_analysis".to_string()]);
        let filtered = controller.filter_agents(&target).await.expect("Filter failed");
        assert!(filtered.len() >= 1);
    }
    
    #[tokio::test]
    async fn test_list_agents() {
        let controller = create_test_controller().await;
        let agents = controller.list_agents().await;
        assert_eq!(agents.len(), 3);
    }
    
    #[tokio::test]
    async fn test_heartbeat_update() {
        let controller = create_test_controller().await;
        let agent_id = controller.list_agents().await[0].agent_id.clone();
        
        controller.update_heartbeat(&agent_id).await;
        
        let agent = controller.get_agent_status(&agent_id).await.expect("Agent not found");
        assert!(agent.active);
        assert!(agent.last_heartbeat.is_some());
    }
    
    async fn create_test_controller() -> Arc<SwarmController> {
        let config = SwarmConfig::default();
        let controller = SwarmController::new(config).await.expect("Failed to create controller");
        let controller = Arc::new(controller);
        
        // Add test agents
        let mut agents = controller.agents.write().await;
        
        agents.insert(
            "agent1".to_string(),
            AgentEntry {
                agent_id: "agent1".to_string(),
                hostname: "test1".to_string(),
                endpoint: "localhost:1".to_string(),
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                capabilities: vec!["binary_analysis".to_string()],
                tags: vec!["production".to_string()],
                last_heartbeat: None,
                active: false,
            },
        );
        
        agents.insert(
            "agent2".to_string(),
            AgentEntry {
                agent_id: "agent2".to_string(),
                hostname: "test2".to_string(),
                endpoint: "localhost:2".to_string(),
                os: "linux".to_string(),
                arch: "x86_64".to_string(),
                capabilities: vec!["network_exploit".to_string()],
                tags: vec!["staging".to_string()],
                last_heartbeat: None,
                active: false,
            },
        );
        
        agents.insert(
            "agent3".to_string(),
            AgentEntry {
                agent_id: "agent3".to_string(),
                hostname: "test3".to_string(),
                endpoint: "localhost:3".to_string(),
                os: "windows".to_string(),
                arch: "x86_64".to_string(),
                capabilities: vec!["rop_chain".to_string()],
                tags: vec!["production".to_string()],
                last_heartbeat: None,
                active: false,
            },
        );
        
        drop(agents);
        controller
    }
}
