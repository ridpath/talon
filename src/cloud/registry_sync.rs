// TALON Registry Synchronization for Distributed Swarm
// Synchronizes discovered gadgets, libc offsets, and shellcode across team members

use super::proto::{RegistryUpdate, UpdateType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Registry entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub key: String,
    pub value: Vec<u8>,
    pub update_type: String,
    pub discovered_by: Option<String>,
    pub discovered_at: i64,
    pub metadata: HashMap<String, String>,
}

/// Gadget information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GadgetInfo {
    pub binary_name: String,
    pub offset: u64,
    pub instructions: String,
    pub quality_score: f32,
}

/// Libc offset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibcOffsetInfo {
    pub libc_version: String,
    pub symbol_name: String,
    pub offset: u64,
    pub build_id: Option<String>,
}

/// Shellcode variant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellcodeInfo {
    pub name: String,
    pub architecture: String,
    pub variant: Vec<u8>,
    pub size: usize,
    pub constraints: Vec<String>,
}

/// Target information shared across team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    pub hostname: String,
    pub ip: String,
    pub ports: Vec<u16>,
    pub os: String,
    pub vulnerabilities: Vec<String>,
    pub notes: String,
}

/// Registry synchronization engine
pub struct RegistrySync {
    /// Gadget registry (key: "binary:gadget_pattern", value: GadgetInfo)
    gadgets: Arc<RwLock<HashMap<String, GadgetInfo>>>,
    
    /// Libc offset registry (key: "libc_version:symbol", value: offset)
    libc_offsets: Arc<RwLock<HashMap<String, LibcOffsetInfo>>>,
    
    /// Shellcode variants (key: "name:arch:variant_id", value: shellcode bytes)
    shellcode: Arc<RwLock<HashMap<String, ShellcodeInfo>>>,
    
    /// Target intelligence (key: hostname/ip)
    targets: Arc<RwLock<HashMap<String, TargetInfo>>>,
    
    /// Synchronization subscribers (for real-time updates)
    subscribers: Arc<RwLock<Vec<tokio::sync::mpsc::Sender<RegistryUpdate>>>>,
    
    /// Optional Redis client for persistence
    #[cfg(feature = "redis")]
    redis_client: Option<redis::Client>,
}

impl RegistrySync {
    /// Create new registry sync instance
    pub fn new() -> Self {
        #[cfg(feature = "redis")]
        let redis_client = redis::Client::open("redis://127.0.0.1:6379").ok();
        
        Self {
            gadgets: Arc::new(RwLock::new(HashMap::new())),
            libc_offsets: Arc::new(RwLock::new(HashMap::new())),
            shellcode: Arc::new(RwLock::new(HashMap::new())),
            targets: Arc::new(RwLock::new(HashMap::new())),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            #[cfg(feature = "redis")]
            redis_client,
        }
    }
    
    /// Persist update to Redis
    #[cfg(feature = "redis")]
    async fn persist_to_redis(&self, key: &str, value: &[u8]) {
        if let Some(ref redis_client) = self.redis_client {
            if let Ok(mut conn) = redis_client.get_async_connection().await {
                use redis::AsyncCommands;
                let redis_key = format!("swarm:registry:{}", key);
                let _: Result<(), _> = conn.set(&redis_key, value).await;
            }
        }
    }
    
    /// Apply update from remote agent
    pub async fn apply_update(&self, update: RegistryUpdate) {
        let update_type = UpdateType::try_from(update.update_type).ok();
        
        match update_type {
            Some(UpdateType::UpdateGadget) => {
                if let Ok(gadget) = serde_json::from_slice::<GadgetInfo>(&update.value) {
                    log::info!("Registry sync: Received gadget update: {}", update.key);
                    self.gadgets.write().await.insert(update.key.clone(), gadget);
                }
            }
            
            Some(UpdateType::UpdateLibcOffset) => {
                if let Ok(offset_info) = serde_json::from_slice::<LibcOffsetInfo>(&update.value) {
                    log::info!("Registry sync: Received libc offset update: {}", update.key);
                    self.libc_offsets.write().await.insert(update.key.clone(), offset_info);
                }
            }
            
            Some(UpdateType::UpdateShellcode) => {
                if let Ok(shellcode_info) = serde_json::from_slice::<ShellcodeInfo>(&update.value) {
                    log::info!("Registry sync: Received shellcode update: {}", update.key);
                    self.shellcode.write().await.insert(update.key.clone(), shellcode_info);
                }
            }
            
            Some(UpdateType::UpdateTarget) => {
                if let Ok(target_info) = serde_json::from_slice::<TargetInfo>(&update.value) {
                    log::info!("Registry sync: Received target update: {}", update.key);
                    self.targets.write().await.insert(update.key.clone(), target_info);
                }
            }
            
            None => {
                log::warn!("Registry sync: Unknown update type: {}", update.update_type);
            }
        }
        
        // Broadcast to subscribers
        self.broadcast_update(update).await;
    }
    
    /// Broadcast update to all subscribers
    async fn broadcast_update(&self, update: RegistryUpdate) {
        let subscribers = self.subscribers.read().await;
        for tx in subscribers.iter() {
            let _ = tx.send(update.clone()).await;
        }
    }
    
    /// Subscribe to registry updates
    pub async fn subscribe(&self) -> tokio::sync::mpsc::Receiver<RegistryUpdate> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        self.subscribers.write().await.push(tx);
        rx
    }
    
    /// Add gadget to registry
    pub async fn add_gadget(
        &self,
        binary_name: &str,
        offset: u64,
        instructions: &str,
        quality_score: f32,
        discovered_by: Option<String>,
    ) -> String {
        let key = format!("{}:0x{:x}", binary_name, offset);
        
        let gadget = GadgetInfo {
            binary_name: binary_name.to_string(),
            offset,
            instructions: instructions.to_string(),
            quality_score,
        };
        
        self.gadgets.write().await.insert(key.clone(), gadget.clone());
        
        // Create update message
        let update = RegistryUpdate {
            update_type: UpdateType::UpdateGadget as i32,
            key: key.clone(),
            value: serde_json::to_vec(&gadget).unwrap_or_default(),
            metadata: discovered_by
                .map(|agent_id| {
                    let mut m = HashMap::new();
                    m.insert("discovered_by".to_string(), agent_id);
                    m
                })
                .unwrap_or_default(),
        };
        
        self.broadcast_update(update.clone()).await;
        
        // Persist to Redis
        #[cfg(feature = "redis")]
        if let Ok(value) = serde_json::to_vec(&gadget) {
            self.persist_to_redis(&key, &value).await;
        }
        
        log::info!("Added gadget to registry: {} -> {}", key, instructions);
        key
    }
    
    /// Add libc offset to registry
    pub async fn add_libc_offset(
        &self,
        libc_version: &str,
        symbol_name: &str,
        offset: u64,
        build_id: Option<String>,
        discovered_by: Option<String>,
    ) -> String {
        let key = format!("{}:{}", libc_version, symbol_name);
        
        let offset_info = LibcOffsetInfo {
            libc_version: libc_version.to_string(),
            symbol_name: symbol_name.to_string(),
            offset,
            build_id,
        };
        
        self.libc_offsets.write().await.insert(key.clone(), offset_info.clone());
        
        let update = RegistryUpdate {
            update_type: UpdateType::UpdateLibcOffset as i32,
            key: key.clone(),
            value: serde_json::to_vec(&offset_info).unwrap_or_default(),
            metadata: discovered_by
                .map(|agent_id| {
                    let mut m = HashMap::new();
                    m.insert("discovered_by".to_string(), agent_id);
                    m
                })
                .unwrap_or_default(),
        };
        
        self.broadcast_update(update.clone()).await;
        
        // Persist to Redis
        #[cfg(feature = "redis")]
        if let Ok(value) = serde_json::to_vec(&offset_info) {
            self.persist_to_redis(&key, &value).await;
        }
        
        log::info!("Added libc offset to registry: {} -> 0x{:x}", key, offset);
        key
    }
    
    /// Add shellcode variant to registry
    pub async fn add_shellcode(
        &self,
        name: &str,
        architecture: &str,
        variant: Vec<u8>,
        constraints: Vec<String>,
        discovered_by: Option<String>,
    ) -> String {
        let key = format!("{}:{}:{}", name, architecture, uuid::Uuid::new_v4());
        
        let shellcode_info = ShellcodeInfo {
            name: name.to_string(),
            architecture: architecture.to_string(),
            variant: variant.clone(),
            size: variant.len(),
            constraints,
        };
        
        self.shellcode.write().await.insert(key.clone(), shellcode_info.clone());
        
        let update = RegistryUpdate {
            update_type: UpdateType::UpdateShellcode as i32,
            key: key.clone(),
            value: serde_json::to_vec(&shellcode_info).unwrap_or_default(),
            metadata: discovered_by
                .map(|agent_id| {
                    let mut m = HashMap::new();
                    m.insert("discovered_by".to_string(), agent_id);
                    m
                })
                .unwrap_or_default(),
        };
        
        self.broadcast_update(update.clone()).await;
        
        // Persist to Redis
        #[cfg(feature = "redis")]
        if let Ok(value) = serde_json::to_vec(&shellcode_info) {
            self.persist_to_redis(&key, &value).await;
        }
        
        log::info!("Added shellcode to registry: {} ({} bytes)", key, variant.len());
        key
    }
    
    /// Add target intelligence
    pub async fn add_target(
        &self,
        hostname: &str,
        ip: &str,
        ports: Vec<u16>,
        os: &str,
        vulnerabilities: Vec<String>,
        notes: &str,
        discovered_by: Option<String>,
    ) -> String {
        let key = format!("{}:{}", hostname, ip);
        
        let target_info = TargetInfo {
            hostname: hostname.to_string(),
            ip: ip.to_string(),
            ports,
            os: os.to_string(),
            vulnerabilities,
            notes: notes.to_string(),
        };
        
        self.targets.write().await.insert(key.clone(), target_info.clone());
        
        let update = RegistryUpdate {
            update_type: UpdateType::UpdateTarget as i32,
            key: key.clone(),
            value: serde_json::to_vec(&target_info).unwrap_or_default(),
            metadata: discovered_by
                .map(|agent_id| {
                    let mut m = HashMap::new();
                    m.insert("discovered_by".to_string(), agent_id);
                    m
                })
                .unwrap_or_default(),
        };
        
        self.broadcast_update(update.clone()).await;
        
        // Persist to Redis
        #[cfg(feature = "redis")]
        if let Ok(value) = serde_json::to_vec(&target_info) {
            self.persist_to_redis(&key, &value).await;
        }
        
        log::info!("Added target to registry: {}", key);
        key
    }
    
    /// Get gadget by key
    pub async fn get_gadget(&self, key: &str) -> Option<GadgetInfo> {
        self.gadgets.read().await.get(key).cloned()
    }
    
    /// Get all gadgets for binary
    pub async fn get_gadgets_for_binary(&self, binary_name: &str) -> Vec<GadgetInfo> {
        self.gadgets
            .read()
            .await
            .values()
            .filter(|g| g.binary_name == binary_name)
            .cloned()
            .collect()
    }
    
    /// Get libc offset
    pub async fn get_libc_offset(&self, libc_version: &str, symbol: &str) -> Option<u64> {
        let key = format!("{}:{}", libc_version, symbol);
        self.libc_offsets
            .read()
            .await
            .get(&key)
            .map(|info| info.offset)
    }
    
    /// Get all shellcode variants for name and architecture
    pub async fn get_shellcode_variants(
        &self,
        name: &str,
        architecture: &str,
    ) -> Vec<ShellcodeInfo> {
        self.shellcode
            .read()
            .await
            .values()
            .filter(|s| s.name == name && s.architecture == architecture)
            .cloned()
            .collect()
    }
    
    /// Get target information
    pub async fn get_target(&self, hostname_or_ip: &str) -> Option<TargetInfo> {
        let targets = self.targets.read().await;
        
        // Search by exact key match first
        if let Some(target) = targets.get(hostname_or_ip).cloned() {
            return Some(target);
        }
        
        // Search by hostname or IP substring
        targets
            .values()
            .find(|t| t.hostname.contains(hostname_or_ip) || t.ip.contains(hostname_or_ip))
            .cloned()
    }
    
    /// Get all targets
    pub async fn get_all_targets(&self) -> Vec<TargetInfo> {
        self.targets.read().await.values().cloned().collect()
    }
    
    /// Get statistics
    pub async fn get_stats(&self) -> RegistryStats {
        RegistryStats {
            gadget_count: self.gadgets.read().await.len(),
            libc_offset_count: self.libc_offsets.read().await.len(),
            shellcode_count: self.shellcode.read().await.len(),
            target_count: self.targets.read().await.len(),
            subscriber_count: self.subscribers.read().await.len(),
        }
    }
    
    /// Clear all registry data
    pub async fn clear_all(&self) {
        self.gadgets.write().await.clear();
        self.libc_offsets.write().await.clear();
        self.shellcode.write().await.clear();
        self.targets.write().await.clear();
        log::info!("Registry cleared");
    }
}

impl Default for RegistrySync {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub gadget_count: usize,
    pub libc_offset_count: usize,
    pub shellcode_count: usize,
    pub target_count: usize,
    pub subscriber_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_registry_sync_creation() {
        let registry = RegistrySync::new();
        let stats = registry.get_stats().await;
        assert_eq!(stats.gadget_count, 0);
        assert_eq!(stats.libc_offset_count, 0);
    }
    
    #[tokio::test]
    async fn test_add_gadget() {
        let registry = RegistrySync::new();
        let key = registry
            .add_gadget("libc.so.6", 0x1234, "pop rdi; ret", 0.95, None)
            .await;
        
        assert_eq!(key, "libc.so.6:0x1234");
        
        let gadget = registry.get_gadget(&key).await.expect("Gadget not found");
        assert_eq!(gadget.offset, 0x1234);
        assert_eq!(gadget.instructions, "pop rdi; ret");
    }
    
    #[tokio::test]
    async fn test_add_libc_offset() {
        let registry = RegistrySync::new();
        let key = registry
            .add_libc_offset("libc-2.31", "system", 0x50d60, Some("abc123".to_string()), None)
            .await;
        
        assert_eq!(key, "libc-2.31:system");
        
        let offset = registry
            .get_libc_offset("libc-2.31", "system")
            .await
            .expect("Offset not found");
        
        assert_eq!(offset, 0x50d60);
    }
    
    #[tokio::test]
    async fn test_add_shellcode() {
        let registry = RegistrySync::new();
        let shellcode_bytes = vec![0x48, 0x31, 0xc0]; // xor rax, rax
        
        let key = registry
            .add_shellcode(
                "execve",
                "x86_64",
                shellcode_bytes.clone(),
                vec!["no_nulls".to_string()],
                None,
            )
            .await;
        
        assert!(key.starts_with("execve:x86_64:"));
        
        let variants = registry.get_shellcode_variants("execve", "x86_64").await;
        assert_eq!(variants.len(), 1);
        assert_eq!(variants[0].variant, shellcode_bytes);
    }
    
    #[tokio::test]
    async fn test_add_target() {
        let registry = RegistrySync::new();
        
        let key = registry
            .add_target(
                "target.example.com",
                "192.168.1.100",
                vec![22, 80, 443],
                "linux",
                vec!["buffer_overflow".to_string()],
                "Production web server",
                None,
            )
            .await;
        
        assert_eq!(key, "target.example.com:192.168.1.100");
        
        let target = registry
            .get_target("target.example.com")
            .await
            .expect("Target not found");
        
        assert_eq!(target.ip, "192.168.1.100");
        assert_eq!(target.ports, vec![22, 80, 443]);
    }
    
    #[tokio::test]
    async fn test_get_gadgets_for_binary() {
        let registry = RegistrySync::new();
        
        registry
            .add_gadget("test.bin", 0x1000, "pop rax; ret", 0.9, None)
            .await;
        
        registry
            .add_gadget("test.bin", 0x2000, "pop rdi; ret", 0.95, None)
            .await;
        
        registry
            .add_gadget("other.bin", 0x3000, "pop rsi; ret", 0.8, None)
            .await;
        
        let gadgets = registry.get_gadgets_for_binary("test.bin").await;
        assert_eq!(gadgets.len(), 2);
    }
    
    #[tokio::test]
    async fn test_stats() {
        let registry = RegistrySync::new();
        
        registry.add_gadget("test", 0x1000, "pop rax; ret", 0.9, None).await;
        registry.add_libc_offset("libc-2.31", "system", 0x50d60, None, None).await;
        registry
            .add_shellcode("execve", "x86_64", vec![0x48, 0x31, 0xc0], vec![], None)
            .await;
        
        let stats = registry.get_stats().await;
        assert_eq!(stats.gadget_count, 1);
        assert_eq!(stats.libc_offset_count, 1);
        assert_eq!(stats.shellcode_count, 1);
    }
    
    #[tokio::test]
    async fn test_clear_all() {
        let registry = RegistrySync::new();
        
        registry.add_gadget("test", 0x1000, "pop rax; ret", 0.9, None).await;
        registry.add_libc_offset("libc-2.31", "system", 0x50d60, None, None).await;
        
        registry.clear_all().await;
        
        let stats = registry.get_stats().await;
        assert_eq!(stats.gadget_count, 0);
        assert_eq!(stats.libc_offset_count, 0);
    }
}
