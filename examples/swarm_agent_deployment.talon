# TALON Distributed Swarm Agent Deployment Example
# Demonstrates agent deployment and distributed exploitation

# Example 1: Deploy agent to remote system via SSH
let agent_binary = read_file("target/release/talon-agent")
let target_host = "10.0.0.50"

# Connect to target via SSH
let conn = connect_ssh(target_host, 22, "operator", "password")

# Upload agent binary
ssh_upload(conn, agent_binary, "/tmp/talon-agent")

# Upload mTLS certificates
ssh_upload(conn, "certs/agent.crt", "/tmp/agent.crt")
ssh_upload(conn, "certs/agent.key", "/tmp/agent.key")
ssh_upload(conn, "certs/ca.crt", "/tmp/ca.crt")

# Make agent executable
ssh_run(conn, "chmod +x /tmp/talon-agent")

# Start agent in background
ssh_run(conn, "nohup /tmp/talon-agent start --primary https://controller:50051 --cert /tmp/agent.crt --key /tmp/agent.key --ca-cert /tmp/ca.crt > /tmp/agent.log 2>&1 &")

print("Agent deployed successfully to: " + target_host)

# Example 2: Deploy to multiple agents via mass connection
let targets = ["10.0.0.50", "10.0.0.51", "10.0.0.52"]
let results = mass_connect(targets, 22, 10, 5000, 50)

for result in results {
    if result.success {
        let conn = result.connection_id
        print("Deploying agent to: " + result.target)
        
        # Upload and start agent (same as Example 1)
        ssh_upload(conn, agent_binary, "/tmp/talon-agent")
        ssh_upload(conn, "certs/agent.crt", "/tmp/agent.crt")
        ssh_upload(conn, "certs/agent.key", "/tmp/agent.key")
        ssh_upload(conn, "certs/ca.crt", "/tmp/ca.crt")
        ssh_run(conn, "chmod +x /tmp/talon-agent")
        ssh_run(conn, "nohup /tmp/talon-agent start --primary https://controller:50051 --cert /tmp/agent.crt --key /tmp/agent.key --ca-cert /tmp/ca.crt > /tmp/agent.log 2>&1 &")
    }
}

print("Swarm deployment complete: " + str(len(results)) + " agents deployed")

# Example 3: Agent auto-update workflow
# This would typically be triggered by the primary controller
# Shown here for completeness

# Agent configuration
let agent_config = {
    "primary_endpoint": "https://controller:50051",
    "agent_id": "agent-001",
    "client_cert_path": "certs/agent.crt",
    "client_key_path": "certs/agent.key",
    "ca_cert_path": "certs/ca.crt",
    "heartbeat_interval": 30,
    "max_concurrent_tasks": 4
}

# Save configuration
write_file("agent-config.json", json(agent_config))

print("Agent configuration saved")
print("")
print("Agent Deployment Commands:")
print("  Deploy: scp talon-agent target:/tmp/")
print("  Start:  ./talon-agent start --config agent-config.json")
print("  Info:   ./talon-agent info")
print("  Update: ./talon-agent update --config agent-config.json --version 0.2.0")
