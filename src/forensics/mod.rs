// Forensics module for live response and artifact-less execution
//
// This module provides:
// - Artifact-less execution (memfd_create, reflective DLL, process hollowing)
// - Live response forensics (eBPF monitoring, syscall tracing)
// - Anti-sandbox detection (VM/container detection)
// - Parent PID spoofing for stealth operations
//
// Platform Support:
// - Linux: memfd_create, eBPF monitoring, syscall tracing
// - Windows: Reflective DLL injection, process hollowing
// - Cross-platform: VM/container detection

pub mod artifact_less;
pub mod live_response;

// Re-export main types from artifact_less
pub use artifact_less::{
    ExecutionError, MemfdExecutor, ReflectiveDllInjector, ProcessHollower, ParentPidSpoofer,
};

// Re-export main types from live_response
pub use live_response::{
    ForensicsError, SyscallEvent, SyscallTrace, SyscallTracer, EnvironmentType,
    EnvironmentDetection, VmContainerDetector, EbpfMonitor,
};
