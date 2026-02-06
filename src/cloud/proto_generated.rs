// Pre-generated protobuf code for talon.swarm
// Generated from proto/swarm.proto
// Use this when protoc is not available during build

#![allow(clippy::all)]
#![allow(dead_code)]

use prost::Message;

#[derive(Clone, PartialEq, Message)]
pub struct AgentInfo {
    #[prost(string, tag = "1")]
    pub hostname: String,
    #[prost(string, tag = "2")]
    pub os: String,
    #[prost(string, tag = "3")]
    pub arch: String,
    #[prost(string, tag = "4")]
    pub version: String,
    #[prost(string, repeated, tag = "5")]
    pub capabilities: Vec<String>,
    #[prost(bytes, tag = "6")]
    pub csr: Vec<u8>,
}

#[derive(Clone, PartialEq, Message)]
pub struct AgentToken {
    #[prost(string, tag = "1")]
    pub agent_id: String,
    #[prost(bytes, tag = "2")]
    pub certificate: Vec<u8>,
    #[prost(bytes, tag = "3")]
    pub ca_certificate: Vec<u8>,
    #[prost(int64, tag = "4")]
    pub expiry_timestamp: i64,
}

#[derive(Clone, PartialEq, Message)]
pub struct ScriptPayload {
    #[prost(string, tag = "1")]
    pub script_id: String,
    #[prost(bytes, tag = "2")]
    pub script_content: Vec<u8>,
    #[prost(string, repeated, tag = "3")]
    pub target_hosts: Vec<String>,
    #[prost(map = "string, string", tag = "4")]
    pub variables: std::collections::HashMap<String, String>,
    #[prost(message, optional, tag = "5")]
    pub options: Option<ExecutionOptions>,
}

#[derive(Clone, PartialEq, Message)]
pub struct ExecutionOptions {
    #[prost(int32, tag = "1")]
    pub timeout_seconds: i32,
    #[prost(bool, tag = "2")]
    pub dry_run: bool,
    #[prost(bool, tag = "3")]
    pub verbose: bool,
    #[prost(int32, tag = "4")]
    pub max_retries: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct ExecutionEvent {
    #[prost(string, tag = "1")]
    pub script_id: String,
    #[prost(enumeration = "EventType", tag = "2")]
    pub event_type: i32,
    #[prost(string, tag = "3")]
    pub message: String,
    #[prost(int32, tag = "4")]
    pub progress_percent: i32,
    #[prost(bytes, tag = "5")]
    pub data: Vec<u8>,
    #[prost(int64, tag = "6")]
    pub timestamp: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum EventType {
    EventStarted = 0,
    EventProgress = 1,
    EventOutput = 2,
    EventError = 3,
    EventCompleted = 4,
    EventFailed = 5,
}

#[derive(Clone, PartialEq, Message)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ExploitResult {
    #[prost(string, tag = "1")]
    pub script_id: String,
    #[prost(string, tag = "2")]
    pub target_host: String,
    #[prost(bool, tag = "3")]
    pub success: bool,
    #[prost(string, tag = "4")]
    pub error_message: String,
    #[prost(bytes, tag = "5")]
    #[serde(with = "serde_bytes")]
    pub loot: Vec<u8>,
    #[prost(map = "string, string", tag = "6")]
    pub metadata: std::collections::HashMap<String, String>,
    #[prost(int64, tag = "7")]
    pub duration_ms: i64,
    #[prost(int64, tag = "8")]
    pub timestamp: i64,
}

#[derive(Clone, PartialEq, Message)]
pub struct RegistryUpdate {
    #[prost(enumeration = "UpdateType", tag = "1")]
    pub update_type: i32,
    #[prost(string, tag = "2")]
    pub key: String,
    #[prost(bytes, tag = "3")]
    pub value: Vec<u8>,
    #[prost(map = "string, string", tag = "4")]
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum UpdateType {
    UpdateGadget = 0,
    UpdateLibcOffset = 1,
    UpdateShellcode = 2,
    UpdateTarget = 3,
}

#[derive(Clone, PartialEq, Message)]
pub struct AgentStatus {
    #[prost(string, tag = "1")]
    pub agent_id: String,
    #[prost(bool, tag = "2")]
    pub active: bool,
    #[prost(int32, tag = "3")]
    pub cpu_percent: i32,
    #[prost(int32, tag = "4")]
    pub memory_mb: i32,
    #[prost(int32, tag = "5")]
    pub running_tasks: i32,
    #[prost(int64, tag = "6")]
    pub uptime_seconds: i64,
}

#[derive(Clone, PartialEq, Message)]
pub struct Acknowledgment {
    #[prost(bool, tag = "1")]
    pub success: bool,
    #[prost(string, tag = "2")]
    pub message: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct TerminateRequest {
    #[prost(string, tag = "1")]
    pub agent_id: String,
    #[prost(bool, tag = "2")]
    pub force: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct UpdateRequest {
    #[prost(string, tag = "1")]
    pub agent_id: String,
    #[prost(string, tag = "2")]
    pub target_version: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct UpdateChunk {
    #[prost(bytes, tag = "1")]
    pub data: Vec<u8>,
    #[prost(int32, tag = "2")]
    pub chunk_index: i32,
    #[prost(int32, tag = "3")]
    pub total_chunks: i32,
    #[prost(bytes, tag = "4")]
    pub checksum: Vec<u8>,
}

// gRPC client and server stubs
pub mod talon_swarm_client {
    use tonic::transport::Channel;
    
    #[derive(Clone)]
    pub struct TalonSwarmClient {
        inner: tonic::client::Grpc<Channel>,
    }
    
    impl TalonSwarmClient {
        pub fn new(channel: Channel) -> Self {
            Self {
                inner: tonic::client::Grpc::new(channel),
            }
        }
        
        pub async fn register_agent(
            &mut self,
            request: super::AgentInfo,
        ) -> Result<tonic::Response<super::AgentToken>, tonic::Status> {
            let request = tonic::Request::new(request);
            self.inner
                .unary(request, "/talon.swarm.TalonSwarm/RegisterAgent".parse().expect("invalid path"), tonic::codec::ProstCodec::default())
                .await
        }
        
        pub async fn execute_script(
            &mut self,
            request: super::ScriptPayload,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::ExecutionEvent>>, tonic::Status> {
            let request = tonic::Request::new(request);
            self.inner
                .server_streaming(request, "/talon.swarm.TalonSwarm/ExecuteScript".parse().expect("invalid path"), tonic::codec::ProstCodec::default())
                .await
        }
        
        pub async fn report_result(
            &mut self,
            request: super::ExploitResult,
        ) -> Result<tonic::Response<super::Acknowledgment>, tonic::Status> {
            let request = tonic::Request::new(request);
            self.inner
                .unary(request, "/talon.swarm.TalonSwarm/ReportResult".parse().expect("invalid path"), tonic::codec::ProstCodec::default())
                .await
        }
        
        pub async fn sync_registry(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::RegistryUpdate>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::RegistryUpdate>>, tonic::Status> {
            let request = request.into_streaming_request();
            self.inner
                .streaming(request, "/talon.swarm.TalonSwarm/SyncRegistry".parse().expect("invalid path"), tonic::codec::ProstCodec::default())
                .await
        }
        
        pub async fn heartbeat(
            &mut self,
            request: super::AgentStatus,
        ) -> Result<tonic::Response<super::Acknowledgment>, tonic::Status> {
            let request = tonic::Request::new(request);
            self.inner
                .unary(request, "/talon.swarm.TalonSwarm/Heartbeat".parse().expect("invalid path"), tonic::codec::ProstCodec::default())
                .await
        }
        
        pub async fn terminate(
            &mut self,
            request: super::TerminateRequest,
        ) -> Result<tonic::Response<super::Acknowledgment>, tonic::Status> {
            let request = tonic::Request::new(request);
            self.inner
                .unary(request, "/talon.swarm.TalonSwarm/Terminate".parse().expect("invalid path"), tonic::codec::ProstCodec::default())
                .await
        }
        
        pub async fn request_update(
            &mut self,
            request: super::UpdateRequest,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::UpdateChunk>>, tonic::Status> {
            let request = tonic::Request::new(request);
            self.inner
                .server_streaming(request, "/talon.swarm.TalonSwarm/RequestUpdate".parse().expect("invalid path"), tonic::codec::ProstCodec::default())
                .await
        }
    }
}

pub mod talon_swarm_server {
    #![allow(unused_imports)]
    use super::*;
    
    #[tonic::async_trait]
    pub trait TalonSwarm: Send + Sync + 'static {
        async fn register_agent(
            &self,
            request: tonic::Request<super::AgentInfo>,
        ) -> Result<tonic::Response<super::AgentToken>, tonic::Status>;
        
        type ExecuteScriptStream: futures::Stream<Item = Result<super::ExecutionEvent, tonic::Status>>
            + Send
            + 'static;
        
        async fn execute_script(
            &self,
            request: tonic::Request<super::ScriptPayload>,
        ) -> Result<tonic::Response<Self::ExecuteScriptStream>, tonic::Status>;
        
        async fn report_result(
            &self,
            request: tonic::Request<super::ExploitResult>,
        ) -> Result<tonic::Response<super::Acknowledgment>, tonic::Status>;
        
        type SyncRegistryStream: futures::Stream<Item = Result<super::RegistryUpdate, tonic::Status>>
            + Send
            + 'static;
        
        async fn sync_registry(
            &self,
            request: tonic::Request<tonic::Streaming<super::RegistryUpdate>>,
        ) -> Result<tonic::Response<Self::SyncRegistryStream>, tonic::Status>;
        
        async fn heartbeat(
            &self,
            request: tonic::Request<super::AgentStatus>,
        ) -> Result<tonic::Response<super::Acknowledgment>, tonic::Status>;
        
        async fn terminate(
            &self,
            request: tonic::Request<super::TerminateRequest>,
        ) -> Result<tonic::Response<super::Acknowledgment>, tonic::Status>;
        
        type RequestUpdateStream: futures::Stream<Item = Result<super::UpdateChunk, tonic::Status>>
            + Send
            + 'static;
        
        async fn request_update(
            &self,
            request: tonic::Request<super::UpdateRequest>,
        ) -> Result<tonic::Response<Self::RequestUpdateStream>, tonic::Status>;
    }
    
    #[derive(Clone)]
    pub struct TalonSwarmServer<T: TalonSwarm> {
        inner: std::sync::Arc<T>,
    }
    
    impl<T: TalonSwarm> TalonSwarmServer<T> {
        pub fn new(inner: T) -> Self {
            Self {
                inner: std::sync::Arc::new(inner),
            }
        }
    }
    
    impl<T: TalonSwarm> tonic::server::NamedService for TalonSwarmServer<T> {
        const NAME: &'static str = "talon.swarm.TalonSwarm";
    }
    
    // Note: Full gRPC Service implementation requires tonic-build code generation
    // This is a simplified stub that allows compilation without protoc
    // For production use with live agents:
    // 1. Install protoc from https://github.com/protocolbuffers/protobuf/releases
    // 2. Run: cargo build --features swarm
    // 3. tonic-build will generate the proper Service trait implementation
    // 
    // Current status: Core business logic complete, gRPC runtime requires protoc
}
