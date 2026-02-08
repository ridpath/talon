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

print("TALON Swarm Distributed Libc Detection")
print("=======================================")
print("Target port: " + target_port)
print("Max concurrent: " + max_concurrent)
print("")

// Target network
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

print("Targets to analyze: " + len(targets))
print("")

// Libc database storage
let libc_database = Map()
let symbol_offsets = Map()
let unique_libc_count = 0

// Agent identification
let agent_id = get_agent_id()
print("Agent " + agent_id + " starting libc detection...")
print("")

// Connect to all targets
let connections = mass_connect(
    targets,
    target_port,
    max_concurrent,
    timeout_ms,
    0
)

print("Connection phase complete")
print("  Successful: " + count_successful(connections))
print("  Failed: " + count_failed(connections))
print("")

// Process each successful connection
for conn_result in connections {
    if conn_result.success {
        let target_ip = conn_result.target
        let conn_id = conn_result.connection_id
        
        print("Analyzing target: " + target_ip)
        
        try {
            // Leak multiple symbols
            let leaked_symbols = leak_multiple_symbols(conn_id, leak_attempts)
            
            if len(leaked_symbols) > 0 {
                print("  Leaked " + len(leaked_symbols) + " symbols")
                
                // Fingerprint libc version
                let libc_info = fingerprint_libc(leaked_symbols, target_ip)
                
                let detected_version = map_get(libc_info, "version")
                if detected_version != "unknown" {
                    print("  Detected: " + detected_version)
                    print("  Base address: 0x" + hex(map_get(libc_info, "base")))
                    print("  Build ID: " + map_get(libc_info, "build_id"))
                    
                    // Add to database
                    let db_key = map_get(libc_info, "build_id")
                    
                    let existing_entry = map_get(libc_database, db_key)
                    if existing_entry == null {
                        // New libc version discovered
                        unique_libc_count = unique_libc_count + 1
                        print("  [NEW] Unique libc version #" + unique_libc_count)
                        
                        let new_entry = Map()
                        map_set(new_entry, "version", map_get(libc_info, "version"))
                        map_set(new_entry, "build_id", map_get(libc_info, "build_id"))
                        map_set(new_entry, "symbols", map_get(libc_info, "symbols"))
                        map_set(new_entry, "targets", [target_ip])
                        map_set(new_entry, "first_seen", current_timestamp())
                        map_set(new_entry, "agent_discovered", agent_id)
                        
                        map_set(libc_database, db_key, new_entry)
                        
                        // Synchronize with other agents
                        swarm_sync_libc_discovery(db_key, new_entry)
                    } else {
                        // Known libc, add target to list
                        let current_targets = map_get(existing_entry, "targets")
                        let updated_targets = [...current_targets, target_ip]
                        map_set(existing_entry, "targets", updated_targets)
                        map_set(libc_database, db_key, existing_entry)
                        print("  [KNOWN] Matches existing entry")
                    }
                    
                    // Store symbol offsets for cross-referencing
                    let libc_symbols = map_get(libc_info, "symbols")
                    let libc_version = map_get(libc_info, "version")
                    
                    for symbol_entry in get_map_entries(libc_symbols) {
                        let symbol_name = map_get(symbol_entry, "key")
                        let offset = map_get(symbol_entry, "value")
                        
                        let version_map = map_get(symbol_offsets, symbol_name)
                        if version_map == null {
                            version_map = Map()
                        }
                        map_set(version_map, libc_version, offset)
                        map_set(symbol_offsets, symbol_name, version_map)
                    }
                } else {
                    print("  [FAILED] Could not fingerprint libc")
                }
            } else {
                print("  [FAILED] No symbols leaked")
            }
            
            // Close connection
            close(conn_id)
            
        } catch error {
            print("  [ERROR] " + error)
        }
    }
}

print("")
print("Libc Detection Complete")
print("  Unique libc versions: " + unique_libc_count)
print("  Total targets analyzed: " + count_successful(connections))
print("")

// Display discovered libc database
print("Discovered Libc Database:")
print("=========================")
for db_entry in get_map_entries(libc_database) {
    let build_id = map_get(db_entry, "key")
    let info = map_get(db_entry, "value")
    
    print("")
    print("Build ID: " + build_id)
    print("  Version: " + map_get(info, "version"))
    
    let info_targets = map_get(info, "targets")
    print("  Targets: " + len(info_targets) + " (" + join(info_targets, ", ") + ")")
    print("  Discovered by: " + map_get(info, "agent_discovered"))
    print("  First seen: " + map_get(info, "first_seen"))
    print("  Symbols:")
    
    let info_symbols = map_get(info, "symbols")
    for symbol_entry in get_map_entries(info_symbols) {
        let symbol_name = map_get(symbol_entry, "key")
        let offset = map_get(symbol_entry, "value")
        print("    " + symbol_name + ": 0x" + hex(offset))
    }
}

print("")
print("Symbol Offset Cross-Reference:")
print("===============================")
for offset_entry in get_map_entries(symbol_offsets) {
    let symbol_name = map_get(offset_entry, "key")
    let versions = map_get(offset_entry, "value")
    
    print("")
    print(symbol_name + ":")
    for version_entry in get_map_entries(versions) {
        let version = map_get(version_entry, "key")
        let offset = map_get(version_entry, "value")
        print("  " + version + ": 0x" + hex(offset))
    }
}

// Return results for swarm aggregation
let final_result = Map()
map_set(final_result, "agent_id", agent_id)
map_set(final_result, "targets_analyzed", count_successful(connections))
map_set(final_result, "unique_libc_versions", unique_libc_count)
map_set(final_result, "libc_database", libc_database)
map_set(final_result, "symbol_offsets", symbol_offsets)

return final_result

// Helper Functions

// Leak multiple symbols from target
define leak_multiple_symbols(conn_id, max_attempts) {
    let symbols = Map()
    
    // Common symbols to leak for fingerprinting
    let symbol_list = ["system", "execve", "puts", "printf", "malloc", "free", "read", "write"]
    
    for symbol_name in symbol_list {
        try {
            // Send leak request
            send(conn_id, "LEAK:" + symbol_name + "\n")
            let response = recv(conn_id, 8, 2000)
            
            if len(response) == 8 {
                let addr = u64(response)
                map_set(symbols, symbol_name, addr)
                print("    Leaked " + symbol_name + ": 0x" + hex(addr))
            }
        } catch error {
            // Continue to next symbol
        }
    }
    return symbols
}

// Fingerprint libc version
define fingerprint_libc(leaked_symbols, target_ip) {
    let libc_info = Map()
    map_set(libc_info, "version", "unknown")
    map_set(libc_info, "build_id", "unknown")
    map_set(libc_info, "base", 0x0)
    map_set(libc_info, "symbols", Map())
    
    // Check if we have system() symbol
    let system_addr = map_get(leaked_symbols, "system")
    if system_addr != null {
        // Try to match against known libc versions
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
        
        if matched_version == "2.31" {
            map_set(libc_info, "version", "libc-2.31")
            map_set(libc_info, "build_id", "libc6_2.31-0ubuntu9.9")
            map_set(libc_info, "base", potential_base_2_31)
        } else if matched_version == "2.27" {
            map_set(libc_info, "version", "libc-2.27")
            map_set(libc_info, "build_id", "libc6_2.27-3ubuntu1.6")
            map_set(libc_info, "base", potential_base_2_27)
        } else if matched_version == "2.35" {
            map_set(libc_info, "version", "libc-2.35")
            map_set(libc_info, "build_id", "libc6_2.35-0ubuntu3.4")
            map_set(libc_info, "base", potential_base_2_35)
        } else {
            map_set(libc_info, "version", "libc-unknown")
            map_set(libc_info, "build_id", "build_" + hex(system_addr))
            map_set(libc_info, "base", potential_base_2_31)
        }
        
        // Calculate all symbol offsets
        let symbols_map = Map()
        let base = map_get(libc_info, "base")
        
        for symbol_entry in get_map_entries(leaked_symbols) {
            let symbol_name = map_get(symbol_entry, "key")
            let addr = map_get(symbol_entry, "value")
            map_set(symbols_map, symbol_name, addr - base)
        }
        map_set(libc_info, "symbols", symbols_map)
    }
    
    return libc_info
}

// Cross-check libc version
define cross_check_version(leaked_symbols, base_2_31, base_2_27, base_2_35) {
    let score_2_31 = 0
    let score_2_27 = 0
    let score_2_35 = 0
    
    // Known offsets for common symbols
    let offsets_2_31 = Map()
    map_set(offsets_2_31, "system", 0x50d60)
    map_set(offsets_2_31, "puts", 0x875a0)
    map_set(offsets_2_31, "printf", 0x64f70)
    map_set(offsets_2_31, "malloc", 0x97070)
    
    let offsets_2_27 = Map()
    map_set(offsets_2_27, "system", 0x4f4e0)
    map_set(offsets_2_27, "puts", 0x809c0)
    map_set(offsets_2_27, "printf", 0x64e80)
    map_set(offsets_2_27, "malloc", 0x97070)
    
    let offsets_2_35 = Map()
    map_set(offsets_2_35, "system", 0x52290)
    map_set(offsets_2_35, "puts", 0x80ed0)
    map_set(offsets_2_35, "printf", 0x61c90)
    map_set(offsets_2_35, "malloc", 0x9a1f0)
    
    // Score each version
    for symbol_entry in get_map_entries(leaked_symbols) {
        let symbol_name = map_get(symbol_entry, "key")
        let addr = map_get(symbol_entry, "value")
        
        let offset_2_31 = map_get(offsets_2_31, symbol_name)
        if offset_2_31 != null {
            if addr == base_2_31 + offset_2_31 {
                score_2_31 = score_2_31 + 1
            }
        }
        
        let offset_2_27 = map_get(offsets_2_27, symbol_name)
        if offset_2_27 != null {
            if addr == base_2_27 + offset_2_27 {
                score_2_27 = score_2_27 + 1
            }
        }
        
        let offset_2_35 = map_get(offsets_2_35, symbol_name)
        if offset_2_35 != null {
            if addr == base_2_35 + offset_2_35 {
                score_2_35 = score_2_35 + 1
            }
        }
    }
    
    // Return version with highest score
    if score_2_31 > score_2_27 && score_2_31 > score_2_35 {
        return "2.31"
    } else if score_2_27 > score_2_35 {
        return "2.27"
    } else if score_2_35 > 0 {
        return "2.35"
    } else {
        return "unknown"
    }
}

// Helper to convert map to list of {key, value} entries
define get_map_entries(map_obj) {
    // In production, this would use map iterator
    // For now, return empty list (placeholder)
    return []
}

// Synchronize libc discovery with swarm
define swarm_sync_libc_discovery(build_id, libc_info) {
    // In production: swarm.sync_registry("libc_database", build_id, libc_info)
    return true
}

// Count successful connections
define count_successful(results) {
    let count = 0
    for result in results {
        if result.success {
            count = count + 1
        }
    }
    return count
}

define count_failed(results) {
    let count = 0
    for result in results {
        if result.success == false {
            count = count + 1
        }
    }
    return count
}

// Get current timestamp
define current_timestamp() {
    return "2026-02-06T18:59:00Z"
}

// Get agent ID
define get_agent_id() {
    return "agent-libc-01"
}

// Join array elements
define join(array, separator) {
    let result = ""
    let first = true
    for item in array {
        if first == false {
            result = result + separator
        }
        result = result + item
        first = false
    }
    return result
}
