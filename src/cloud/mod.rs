// Distributed Swarm Mode - Agent Infrastructure
// Provides gRPC-based distributed exploitation capabilities with mTLS authentication

#[cfg(feature = "swarm")]
pub mod proto_generated;

#[cfg(feature = "swarm")]
pub mod proto_helpers;

#[cfg(feature = "swarm")]
pub mod proto;

#[cfg(feature = "swarm")]
pub mod agent;

#[cfg(feature = "swarm")]
pub mod swarm;

#[cfg(feature = "swarm")]
pub mod registry_sync;

#[cfg(not(feature = "swarm"))]
pub mod stub {
    //! Stub implementation when swarm feature is disabled
    
    pub fn swarm_not_enabled() -> Result<(), String> {
        Err("Swarm mode not enabled. Build with --features swarm".to_string())
    }
}

#[cfg(feature = "swarm")]
pub use agent::{TalonAgent, AgentConfig, AgentError};

#[cfg(feature = "swarm")]
pub use swarm::{SwarmController, SwarmConfig, SwarmError, ExecutionRequest, TargetAgents, AggregatedResults};

#[cfg(feature = "swarm")]
pub use registry_sync::{RegistrySync, GadgetInfo, LibcOffsetInfo, ShellcodeInfo, TargetInfo, RegistryStats};

#[cfg(feature = "swarm")]
pub use proto::talon_swarm_client::TalonSwarmClient;

#[cfg(feature = "swarm")]
pub use proto::talon_swarm_server::{TalonSwarm, TalonSwarmServer};
