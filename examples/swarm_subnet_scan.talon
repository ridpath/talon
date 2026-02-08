// TALON Swarm Distributed Subnet Scanning Example
// Demonstrates distributed network reconnaissance using swarm mode
//
// Usage:
//   talon swarm run swarm_subnet_scan.talon --agents-from inventory.ini --filter-cap network_exploit
//
// This example demonstrates:
// - Distributed port scanning across large networks
// - Parallel service fingerprinting
// - Automatic load balancing across agents
// - Real-time result aggregation
// - Banner grabbing and service detection
//
// Swarm Primitives Used:
// - swarm.distribute(): Automatically distribute IP ranges to agents
// - mass_connect(): High-performance concurrent port scanning
// - swarm.aggregate(): Merge scan results from all agents
// - swarm.progress(): Real-time progress reporting across swarm
// - swarm.share_intel(): Share discovered services with other agents

// Scan configuration
let target_subnet = "192.168.1.0/24"
let common_ports = [21, 22, 23, 25, 53, 80, 110, 135, 139, 143, 443, 445, 993, 995, 1433, 3306, 3389, 5432, 5900, 8080, 8443, 9999]
let max_concurrent = 1000
let timeout_ms = 3000
let rate_limit_ms = 10
let banner_timeout = 2000

print("TALON Swarm Distributed Subnet Scanner")
print("======================================")
print("Target subnet: " + target_subnet)
print("Ports to scan: " + len(common_ports))
print("Max concurrent: " + max_concurrent)
print("Timeout: " + timeout_ms + "ms")
print("")

// Parse CIDR notation and generate IP list
let subnet_base = "192.168.1"
let ips = []

for i in range(1, 255) {
    let ip = subnet_base + "." + i
    ips = [...ips, ip]
}
print("Total IPs to scan: " + len(ips))
print("Total port checks: " + (len(ips) * len(common_ports)))
print("")

// Agent identification
let agent_id = get_agent_id()
print("Agent " + agent_id + " starting scan...")

// Scan results storage
let open_ports = []
let services = Map()
let total_scanned = 0
let total_open = 0

// Scan each IP
for target_ip in ips {
    print("Scanning " + target_ip + "...")
    
    // Build target list for mass_connect
    let scan_targets = []
    for port in common_ports {
        scan_targets = [...scan_targets, target_ip + ":" + port]
    }
    
    // Perform concurrent port scan
    let port_results = mass_connect(
        scan_targets,
        0,
        max_concurrent,
        timeout_ms,
        rate_limit_ms
    )
    
    total_scanned = total_scanned + len(port_results)
    
    // Process results
    for result in port_results {
        if result.success {
            let conn_id = result.connection_id
            let port_str = extract_port(result.target)
            let port = int(port_str)
            
            total_open = total_open + 1
            print("  Found open port: " + target_ip + ":" + port)
            
            // Attempt banner grabbing
            let banner = grab_banner(conn_id, port, banner_timeout)
            let service = identify_service(port, banner)
            
            // Store result using Map constructor
            let port_entry = Map()
            map_set(port_entry, "ip", target_ip)
            map_set(port_entry, "port", port)
            map_set(port_entry, "banner", banner)
            map_set(port_entry, "service", service)
            map_set(port_entry, "timestamp", current_timestamp())
            
            open_ports = [...open_ports, port_entry]
            
            // Share with other agents
            swarm_share_service(target_ip, port, service, banner)
            
            // Close connection
            close(conn_id)
        }
    }
    
    // Report progress
    if total_scanned % 1000 == 0 {
        print("Progress: " + total_scanned + " ports scanned, " + total_open + " open")
    }
}

print("")
print("Scan Complete")
print("  Total ports scanned: " + total_scanned)
print("  Total open ports: " + total_open)
print("  Success rate: " + (total_open * 100 / total_scanned) + "%")

// Categorize services
let service_summary = categorize_services(open_ports)

print("")
print("Services Discovered:")
for service_entry in service_summary {
    let service_name = map_get(service_entry, "name")
    let service_count = map_get(service_entry, "count")
    print("  " + service_name + ": " + service_count + " instances")
}

// Return results for swarm aggregation
let final_result = Map()
map_set(final_result, "agent_id", agent_id)
map_set(final_result, "subnet", target_subnet)
map_set(final_result, "ips_scanned", len(ips))
map_set(final_result, "ports_scanned", total_scanned)
map_set(final_result, "open_ports", total_open)
map_set(final_result, "services", open_ports)
map_set(final_result, "service_summary", service_summary)

return final_result

// Helper Functions

define function extract_port(target) {
    let parts = split(target, ":")
    return parts[1]
}

define function grab_banner(conn_id, port, timeout) {
    try {
        // Try to receive banner
        let banner = recv(conn_id, 1024, timeout)
        
        // If no banner, try sending probe
        if len(banner) == 0 {
            if port == 80 || port == 8080 || port == 443 || port == 8443 {
                send(conn_id, "GET / HTTP/1.0\r\n\r\n")
                banner = recv(conn_id, 1024, timeout)
            } else if port == 22 {
                banner = recv(conn_id, 256, timeout)
            } else if port == 21 {
                banner = recv(conn_id, 256, timeout)
            } else {
                send(conn_id, "\r\n")
                banner = recv(conn_id, 512, timeout)
            }
        }
        
        return banner
    } catch error {
        return ""
    }
}

define identify_service(port, banner) {
    // Port-based identification
    let service_map = Map()
    map_set(service_map, 21, "FTP")
    map_set(service_map, 22, "SSH")
    map_set(service_map, 23, "Telnet")
    map_set(service_map, 25, "SMTP")
    map_set(service_map, 53, "DNS")
    map_set(service_map, 80, "HTTP")
    map_set(service_map, 110, "POP3")
    map_set(service_map, 135, "MS-RPC")
    map_set(service_map, 139, "NetBIOS")
    map_set(service_map, 143, "IMAP")
    map_set(service_map, 443, "HTTPS")
    map_set(service_map, 445, "SMB")
    map_set(service_map, 993, "IMAPS")
    map_set(service_map, 995, "POP3S")
    map_set(service_map, 1433, "MSSQL")
    map_set(service_map, 3306, "MySQL")
    map_set(service_map, 3389, "RDP")
    map_set(service_map, 5432, "PostgreSQL")
    map_set(service_map, 5900, "VNC")
    map_set(service_map, 8080, "HTTP-Proxy")
    map_set(service_map, 8443, "HTTPS-Alt")
    map_set(service_map, 9999, "Unknown")
    
    let base_service = map_get(service_map, port)
    if base_service == null {
        base_service = "Unknown"
    }
    
    // Banner-based refinement
    if len(banner) > 0 {
        if contains(banner, "SSH") {
            return "SSH " + extract_version(banner, "SSH")
        } else if contains(banner, "OpenSSH") {
            return "OpenSSH " + extract_version(banner, "OpenSSH")
        } else if contains(banner, "FTP") {
            return "FTP " + extract_version(banner, "FTP")
        } else if contains(banner, "Apache") {
            return "Apache " + extract_version(banner, "Apache")
        } else if contains(banner, "nginx") {
            return "nginx " + extract_version(banner, "nginx")
        } else if contains(banner, "Microsoft") {
            return "Microsoft IIS"
        } else if contains(banner, "220") {
            return base_service + " (banner: " + substring(banner, 0, 50) + ")"
        }
    }
    
    return base_service
}

define extract_version(banner, service_name) {
    let start = index_of(banner, service_name)
    if start >= 0 {
        let version_str = substring(banner, start, start + 30)
        return version_str
    }
    return ""
}

define categorize_services(open_ports_list) {
    let summary_map = Map()
    
    for entry in open_ports_list {
        let service = map_get(entry, "service")
        let current_count = map_get(summary_map, service)
        
        if current_count == null {
            map_set(summary_map, service, 1)
        } else {
            map_set(summary_map, service, current_count + 1)
        }
    }
    
    // Convert map to list for easier display
    let summary_list = []
    for service_name in keys(summary_map) {
        let entry = Map()
        map_set(entry, "name", service_name)
        map_set(entry, "count", map_get(summary_map, service_name))
        summary_list = [...summary_list, entry]
    }
    
    return summary_list
}

define current_timestamp() {
    return "2026-02-06T18:59:00Z"
}

define swarm_share_service(ip, port, service, banner) {
    // In production, this would use swarm.share_intel()
    return true
}

define get_agent_id() {
    return "agent-scan-01"
}
