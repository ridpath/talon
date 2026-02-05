// OpSec (Operational Security) modules for EDR evasion and anti-forensics

#[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
pub mod syscalls;

pub mod memory_scrub;

// Re-export main types for convenience
#[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
pub use syscalls::{SyscallResolver, SyscallError, HookDetector, HookType};

pub use memory_scrub::{
    SecureString, MemoryScrubber, ScrubError, AntiDebugger,
};

#[cfg(all(target_os = "windows", feature = "game-hacking-windows"))]
pub use memory_scrub::DpapiProtector;
