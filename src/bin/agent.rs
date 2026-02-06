// TALON Distributed Swarm Agent Binary
// Lightweight static agent for remote deployment (<5MB stripped)

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use talon::cloud::{AgentConfig, TalonAgent};

#[derive(Parser)]
#[command(name = "talon-agent")]
#[command(about = "TALON Distributed Swarm Agent", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start agent and connect to primary controller
    Start {
        /// Primary controller endpoint (e.g., https://controller:50051)
        #[arg(short, long, default_value = "https://localhost:50051")]
        primary: String,
        
        /// Client certificate path for mTLS
        #[arg(short, long, default_value = "agent.crt")]
        cert: PathBuf,
        
        /// Client private key path
        #[arg(short, long, default_value = "agent.key")]
        key: PathBuf,
        
        /// CA certificate path for server verification
        #[arg(long, default_value = "ca.crt")]
        ca_cert: PathBuf,
        
        /// Heartbeat interval in seconds
        #[arg(long, default_value = "30")]
        heartbeat: u64,
        
        /// Maximum concurrent tasks
        #[arg(long, default_value = "4")]
        max_tasks: usize,
        
        /// Configuration file (overrides CLI args)
        #[arg(short = 'f', long)]
        config: Option<PathBuf>,
    },
    
    /// Register agent with primary controller
    Register {
        /// Primary controller endpoint
        #[arg(short, long)]
        primary: String,
        
        /// Output path for agent configuration
        #[arg(short, long, default_value = "agent-config.json")]
        output: PathBuf,
    },
    
    /// Request auto-update from primary
    Update {
        /// Configuration file
        #[arg(short, long, default_value = "agent-config.json")]
        config: PathBuf,
        
        /// Target version to update to
        #[arg(short, long)]
        version: String,
    },
    
    /// Generate agent capabilities report
    Info,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Start {
            primary,
            cert,
            key,
            ca_cert,
            heartbeat,
            max_tasks,
            config,
        } => {
            let agent_config = if let Some(config_path) = config {
                // Load config from file
                let config_data = std::fs::read_to_string(&config_path)?;
                let mut config: AgentConfig = serde_json::from_str(&config_data)?;
                
                // CLI args override config file
                config.primary_endpoint = primary;
                config.client_cert_path = cert;
                config.client_key_path = key;
                config.ca_cert_path = ca_cert;
                config.heartbeat_interval = heartbeat;
                config.max_concurrent_tasks = max_tasks;
                
                config
            } else {
                // Build config from CLI args
                AgentConfig {
                    primary_endpoint: primary,
                    agent_id: uuid::Uuid::new_v4().to_string(),
                    client_cert_path: cert,
                    client_key_path: key,
                    ca_cert_path: ca_cert,
                    heartbeat_interval: heartbeat,
                    max_concurrent_tasks: max_tasks,
                    capabilities: vec![
                        "binary_analysis".to_string(),
                        "network_exploit".to_string(),
                        "rop_chain".to_string(),
                        "shellcode_generation".to_string(),
                    ],
                }
            };
            
            log::info!("Starting TALON agent...");
            log::info!("Agent ID: {}", agent_config.agent_id);
            log::info!("Primary endpoint: {}", agent_config.primary_endpoint);
            log::info!("Capabilities: {:?}", agent_config.capabilities);
            
            // Create and initialize agent
            let mut agent = TalonAgent::new(agent_config.clone()).await?;
            
            // Register with primary
            match agent.register().await {
                Ok(token) => {
                    log::info!("Registration successful!");
                    log::info!("Token expiry: {}", token.expiry_timestamp);
                }
                Err(e) => {
                    log::error!("Registration failed: {}", e);
                    return Err(e.into());
                }
            }
            
            // Get agent state reference for shutdown
            let agent_state = agent.get_state().await;
            
            // Start heartbeat loop in background (consumes agent)
            let heartbeat_handle = tokio::spawn(async move {
                if let Err(e) = agent.start_heartbeat().await {
                    log::error!("Heartbeat loop failed: {}", e);
                }
            });
            
            // Main event loop
            log::info!("Agent running. Press Ctrl+C to shutdown.");
            
            // Wait for shutdown signal
            tokio::signal::ctrl_c().await?;
            
            log::info!("Shutdown signal received");
            
            // Abort heartbeat task
            heartbeat_handle.abort();
            
            log::info!("Agent stopped gracefully");
        }
        
        Commands::Register { primary, output } => {
            log::info!("Registering new agent with primary: {}", primary);
            
            // Generate temporary config
            let config = AgentConfig {
                primary_endpoint: primary,
                agent_id: uuid::Uuid::new_v4().to_string(),
                ..Default::default()
            };
            
            // Try to connect and register
            let mut agent = TalonAgent::new(config.clone()).await?;
            
            match agent.register().await {
                Ok(token) => {
                    log::info!("Registration successful!");
                    
                    // Save config with token
                    let config_json = serde_json::to_string_pretty(&config)?;
                    std::fs::write(&output, config_json)?;
                    
                    log::info!("Configuration saved to: {:?}", output);
                    log::info!("Agent ID: {}", token.agent_id);
                }
                Err(e) => {
                    log::error!("Registration failed: {}", e);
                    return Err(e.into());
                }
            }
        }
        
        Commands::Update { config, version } => {
            log::info!("Requesting update to version: {}", version);
            
            // Load config
            let config_data = std::fs::read_to_string(&config)?;
            let agent_config: AgentConfig = serde_json::from_str(&config_data)?;
            
            // Create agent
            let mut agent = TalonAgent::new(agent_config).await?;
            
            // Request update
            let update_path = agent.request_update(version).await?;
            
            log::info!("Update downloaded to: {:?}", update_path);
            
            // Apply update
            agent.apply_update(&update_path).await?;
            
            log::info!("Update applied successfully. Please restart the agent.");
        }
        
        Commands::Info => {
            println!("TALON Agent v{}", env!("CARGO_PKG_VERSION"));
            println!("OS: {}", std::env::consts::OS);
            println!("Architecture: {}", std::env::consts::ARCH);
            println!();
            println!("Capabilities:");
            println!("  - Binary Analysis");
            println!("  - Network Exploitation");
            println!("  - ROP Chain Generation");
            println!("  - Shellcode Generation");
            println!("  - Format String Exploitation");
            println!("  - Heap Exploitation");
            println!();
            println!("Features:");
            println!("  - mTLS Authentication");
            println!("  - Auto-Update");
            println!("  - Script Execution Sandbox");
            println!("  - Real-time Progress Reporting");
            println!("  - Registry Sync (Gadgets/Libc Offsets)");
        }
    }
    
    Ok(())
}
