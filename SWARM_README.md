# TALON Distributed Swarm Mode

Lightweight distributed exploitation framework with gRPC-based agent orchestration.

## Features

- **mTLS Authentication**: Certificate-based authentication with pinning
- **Auto-Update**: Agents can self-update from primary controller
- **Script Execution Sandbox**: Isolated script execution environment
- **Real-time Progress Reporting**: Streaming execution events
- **Registry Sync**: Share discovered gadgets/offsets across swarm
- **ChaCha20-Poly1305 Encryption**: High-performance authenticated encryption
- **Heartbeat Mechanism**: Automatic agent health monitoring
- **Static Binary**: Lightweight agent (<5MB stripped)

## Prerequisites

### Protocol Buffers Compiler (protoc)

The swarm feature requires `protoc` to compile protocol buffer definitions.

#### Linux/WSL (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y protobuf-compiler
```

#### Linux/WSL (Fedora/CentOS)
```bash
sudo dnf install protobuf-compiler
```

#### macOS
```bash
brew install protobuf
```

#### Windows
1. Download pre-compiled protoc from: https://github.com/protocolbuffers/protobuf/releases
2. Extract and add to PATH
3. Or use Chocolatey: `choco install protoc`

### Verify Installation
```bash
protoc --version  # Should show libprotoc 3.x or higher
```

## Building

### Build TALON with Swarm Support
```bash
cargo build --release --features swarm
```

### Build Agent Binary
```bash
cargo build --release --bin talon-agent --features swarm
```

The agent binary will be at: `target/release/talon-agent` (or `talon-agent.exe` on Windows)

### Build Static Agent (Linux)
For deployment to various Linux systems:
```bash
# On WSL/Linux
cargo build --release --bin talon-agent --features swarm --target x86_64-unknown-linux-musl
strip target/x86_64-unknown-linux-musl/release/talon-agent
```

## Agent Usage

### Start Agent
```bash
./talon-agent start \
  --primary https://controller.example.com:50051 \
  --cert certs/agent.crt \
  --key certs/agent.key \
  --ca-cert certs/ca.crt \
  --heartbeat 30 \
  --max-tasks 4
```

### Register New Agent
```bash
./talon-agent register \
  --primary https://controller.example.com:50051 \
  --output agent-config.json
```

### Agent Info
```bash
./talon-agent info
```

### Request Auto-Update
```bash
./talon-agent update \
  --config agent-config.json \
  --version 0.2.0
```

## Configuration

### Agent Configuration File (agent-config.json)
```json
{
  "primary_endpoint": "https://controller.example.com:50051",
  "agent_id": "agent-001",
  "client_cert_path": "certs/agent.crt",
  "client_key_path": "certs/agent.key",
  "ca_cert_path": "certs/ca.crt",
  "heartbeat_interval": 30,
  "max_concurrent_tasks": 4,
  "capabilities": [
    "binary_analysis",
    "network_exploit",
    "rop_chain",
    "shellcode_generation",
    "format_string",
    "heap_exploitation"
  ]
}
```

### Start with Configuration File
```bash
./talon-agent start --config agent-config.json
```

## Deployment

### SSH Deployment Example
```bash
# Upload agent binary
scp target/release/talon-agent target_host:/tmp/

# Upload certificates
scp certs/agent.crt target_host:/tmp/
scp certs/agent.key target_host:/tmp/
scp certs/ca.crt target_host:/tmp/

# Make executable and start
ssh target_host "chmod +x /tmp/talon-agent && \
  nohup /tmp/talon-agent start \
    --primary https://controller:50051 \
    --cert /tmp/agent.crt \
    --key /tmp/agent.key \
    --ca-cert /tmp/ca.crt \
    > /tmp/agent.log 2>&1 &"
```

### Mass Deployment
See `examples/swarm_agent_deployment.talon` for automated deployment scripts.

## Security

### mTLS Certificate Generation

#### 1. Generate CA Certificate
```bash
openssl genrsa -out ca.key 4096
openssl req -new -x509 -days 365 -key ca.key -out ca.crt \
  -subj "/CN=TALON-CA"
```

#### 2. Generate Agent Certificate
```bash
openssl genrsa -out agent.key 2048
openssl req -new -key agent.key -out agent.csr \
  -subj "/CN=talon-agent-001"
openssl x509 -req -days 365 -in agent.csr -CA ca.crt -CAkey ca.key \
  -set_serial 01 -out agent.crt
```

#### 3. Generate Server Certificate (Primary Controller)
```bash
openssl genrsa -out server.key 2048
openssl req -new -key server.key -out server.csr \
  -subj "/CN=talon.swarm"
openssl x509 -req -days 365 -in server.csr -CA ca.crt -CAkey ca.key \
  -set_serial 02 -out server.crt
```

### Certificate Pinning

Agents only trust certificates signed by the specified CA. This prevents MITM attacks even in adversarial networks.

## Architecture

### gRPC Protocol

The agent communicates with the primary controller using gRPC with the following RPCs:

- `RegisterAgent`: Initial agent registration
- `ExecuteScript`: Stream TALON script execution
- `ReportResult`: Submit exploitation results
- `SyncRegistry`: Bidirectional gadget/offset sync
- `Heartbeat`: Agent health check
- `Terminate`: Graceful agent shutdown
- `RequestUpdate`: Streaming binary auto-update

See `proto/swarm.proto` for full protocol definition.

### Network Topology

```
┌─────────────────────────────────────────────┐
│ TALON Primary (Controller)                  │
│ ┌─────────────┐  ┌──────────────┐          │
│ │ gRPC Server │  │ Redis Client │          │
│ └──────┬──────┘  └──────┬───────┘          │
└────────┼─────────────────┼──────────────────┘
         │ (Control)       │ (State Sync)
         │                 │
    ┌────▼─────────────────▼───────┐
    │    Shared Infrastructure      │
    │  ┌─────────────┐             │
    │  │    Redis    │             │
    │  │  (Hot State)│             │
    │  └─────────────┘             │
    └──────────────────────────────┘
         ▲
         │ gRPC (mTLS)
         │
    ┌────┴────┐       ┌──────────┐
    │ Agent 1 │       │ Agent N  │
    └─────────┘       └──────────┘
```

## Capabilities

Agents support the following capabilities:

- **binary_analysis**: ELF/PE analysis, symbol resolution
- **network_exploit**: Socket-based exploitation
- **rop_chain**: ROP gadget finding and chain generation
- **shellcode_generation**: Architecture-specific shellcode
- **format_string**: Format string exploitation
- **heap_exploitation**: Heap feng shui, tcache, fastbin

## Performance

- **Binary Size**: <5MB (stripped musl static)
- **Startup Time**: <100ms
- **Heartbeat Overhead**: ~50 bytes/30s
- **Concurrent Tasks**: Configurable (default 4)
- **Network Protocol**: gRPC/HTTP2 with compression

## Troubleshooting

### protoc not found during build
```
error: Could not find `protoc`
```
Solution: Install protoc using instructions above and ensure it's in PATH.

### Agent cannot connect to primary
```
error: transport error
```
Solution: Check firewall rules, verify certificates, ensure primary endpoint is correct.

### Certificate verification failed
```
error: invalid peer certificate
```
Solution: Verify certificates are signed by same CA, check certificate expiry dates.

## Examples

- `examples/swarm_agent_config.json`: Example configuration
- `examples/swarm_agent_deployment.talon`: Mass deployment script

## Future Enhancements

- libp2p P2P fallback for mesh networking
- PostgreSQL integration for long-term storage
- Real-time team collaboration
- Web-based swarm management UI
- Agent capability auto-detection

## References

- Protocol Definition: `proto/swarm.proto`
- Agent Implementation: `src/cloud/agent.rs`
- gRPC Service: `src/cloud/proto.rs`
- Research Document: `.talon_cache/research_distributed_systems.md`
