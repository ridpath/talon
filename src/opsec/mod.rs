// OpSec (Operational Security) modules for EDR evasion and anti-forensics

#[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
pub mod syscalls;

// Re-export main types for convenience
#[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
pub use syscalls::{SyscallResolver, SyscallError, HookDetector, HookType};
