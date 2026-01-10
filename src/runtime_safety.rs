// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║   Runtime Safety & Resource Management System                             ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicU64, AtomicBool, Ordering};
use std::time::Instant;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub max_execution_time_ms: u64,
    pub max_memory_bytes: usize,
    pub max_recursion_depth: usize,
    pub strict_mode: bool,
    pub bounds_checking: bool,
    pub type_checking: bool,
    pub overflow_checking: bool,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        SafetyConfig {
            max_execution_time_ms: 300000,
            max_memory_bytes: 1024 * 1024 * 512,
            max_recursion_depth: 1000,
            strict_mode: false,
            bounds_checking: true,
            type_checking: true,
            overflow_checking: true,
        }
    }
}

impl SafetyConfig {
    pub fn permissive() -> Self {
        SafetyConfig {
            max_execution_time_ms: u64::MAX,
            max_memory_bytes: usize::MAX,
            max_recursion_depth: usize::MAX,
            strict_mode: false,
            bounds_checking: true,
            type_checking: false,
            overflow_checking: false,
        }
    }

    pub fn strict() -> Self {
        SafetyConfig {
            max_execution_time_ms: 60000,
            max_memory_bytes: 1024 * 1024 * 256,
            max_recursion_depth: 500,
            strict_mode: true,
            bounds_checking: true,
            type_checking: true,
            overflow_checking: true,
        }
    }
}

#[derive(Clone)]
pub struct RuntimeSafety {
    config: Arc<SafetyConfig>,
    recursion_depth: Arc<AtomicUsize>,
    memory_used: Arc<AtomicU64>,
    start_time: Arc<Instant>,
    timeout_occurred: Arc<AtomicBool>,
}

impl RuntimeSafety {
    pub fn new(config: SafetyConfig) -> Self {
        RuntimeSafety {
            config: Arc::new(config),
            recursion_depth: Arc::new(AtomicUsize::new(0)),
            memory_used: Arc::new(AtomicU64::new(0)),
            start_time: Arc::new(Instant::now()),
            timeout_occurred: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn enter_function(&self) -> Result<RecursionGuard, String> {
        let depth = self.recursion_depth.fetch_add(1, Ordering::SeqCst) + 1;
        
        if depth > self.config.max_recursion_depth {
            self.recursion_depth.fetch_sub(1, Ordering::SeqCst);
            return Err(format!(
                "[ERROR] RECURSION LIMIT EXCEEDED\n\
                 Maximum recursion depth: {}\n\
                 Current depth: {}\n\n\
                 TIP: This prevents stack overflow. Increase limit with:\n\
                 set_recursion_limit {}",
                self.config.max_recursion_depth,
                depth,
                depth + 1000
            ));
        }

        self.check_timeout()?;

        Ok(RecursionGuard {
            counter: self.recursion_depth.clone(),
        })
    }

    pub fn check_timeout(&self) -> Result<(), String> {
        if self.timeout_occurred.load(Ordering::SeqCst) {
            return Err("[ERROR] EXECUTION TIMEOUT - Script terminated".to_string());
        }

        let elapsed = self.start_time.elapsed();
        if elapsed.as_millis() as u64 > self.config.max_execution_time_ms {
            self.timeout_occurred.store(true, Ordering::SeqCst);
            return Err(format!(
                "[ERROR] EXECUTION TIMEOUT\n\
                 Maximum execution time: {}ms\n\
                 Elapsed time: {}ms\n\n\
                 TIP: Increase timeout with:\n\
                 set_timeout {}",
                self.config.max_execution_time_ms,
                elapsed.as_millis(),
                elapsed.as_millis() + 60000
            ));
        }

        Ok(())
    }

    pub fn allocate_memory(&self, bytes: usize) -> Result<MemoryGuard, String> {
        let current = self.memory_used.fetch_add(bytes as u64, Ordering::SeqCst);
        let new_total = current + bytes as u64;

        if new_total > self.config.max_memory_bytes as u64 {
            self.memory_used.fetch_sub(bytes as u64, Ordering::SeqCst);
            return Err(format!(
                "[ERROR] MEMORY LIMIT EXCEEDED\n\
                 Maximum memory: {} MB\n\
                 Current usage: {} MB\n\
                 Requested: {} bytes\n\n\
                 TIP: Increase memory limit with:\n\
                 set_memory_limit {}",
                self.config.max_memory_bytes / (1024 * 1024),
                new_total / (1024 * 1024),
                bytes,
                (new_total / (1024 * 1024)) + 256
            ));
        }

        Ok(MemoryGuard {
            counter: self.memory_used.clone(),
            bytes: bytes as u64,
        })
    }

    pub fn check_bounds(&self, index: usize, len: usize) -> Result<(), String> {
        if !self.config.bounds_checking {
            return Ok(());
        }

        if index >= len {
            return Err(format!(
                "[ERROR] INDEX OUT OF BOUNDS\n\
                 Index: {}\n\
                 Length: {}\n\n\
                 TIP: Valid indices are 0 to {}",
                index,
                len,
                len.saturating_sub(1)
            ));
        }

        Ok(())
    }

    pub fn check_overflow_add(&self, a: i64, b: i64) -> Result<i64, String> {
        if !self.config.overflow_checking {
            return Ok(a.wrapping_add(b));
        }

        a.checked_add(b).ok_or_else(|| {
            format!(
                "[ERROR] INTEGER OVERFLOW\n\
                 Operation: {} + {}\n\
                 Result would exceed i64::MAX ({})\n\n\
                 TIP: Use wrapping arithmetic or larger types",
                a, b, i64::MAX
            )
        })
    }

    pub fn check_overflow_mul(&self, a: i64, b: i64) -> Result<i64, String> {
        if !self.config.overflow_checking {
            return Ok(a.wrapping_mul(b));
        }

        a.checked_mul(b).ok_or_else(|| {
            format!(
                "[ERROR] INTEGER OVERFLOW\n\
                 Operation: {} * {}\n\
                 Result would exceed i64 range\n\n\
                 TIP: Use wrapping arithmetic or larger types",
                a, b
            )
        })
    }

    pub fn check_overflow_sub(&self, a: i64, b: i64) -> Result<i64, String> {
        if !self.config.overflow_checking {
            return Ok(a.wrapping_sub(b));
        }

        a.checked_sub(b).ok_or_else(|| {
            format!(
                "[ERROR] INTEGER UNDERFLOW\n\
                 Operation: {} - {}\n\
                 Result would be less than i64::MIN ({})\n\n\
                 TIP: Use wrapping arithmetic or unsigned types",
                a, b, i64::MIN
            )
        })
    }

    pub fn check_divide_by_zero(&self, divisor: i64) -> Result<(), String> {
        if divisor == 0 {
            return Err(
                "[ERROR] DIVISION BY ZERO\n\n\
                 TIP: Check your divisor before performing division".to_string()
            );
        }
        Ok(())
    }

    pub fn get_stats(&self) -> SafetyStats {
        SafetyStats {
            recursion_depth: self.recursion_depth.load(Ordering::SeqCst),
            memory_used: self.memory_used.load(Ordering::SeqCst),
            elapsed_ms: self.start_time.elapsed().as_millis() as u64,
            timeout_occurred: self.timeout_occurred.load(Ordering::SeqCst),
            config: (*self.config).clone(),
        }
    }

    pub fn is_strict(&self) -> bool {
        self.config.strict_mode
    }

    pub fn update_config(&mut self, new_config: SafetyConfig) {
        self.config = Arc::new(new_config);
    }
}

pub struct RecursionGuard {
    counter: Arc<AtomicUsize>,
}

impl Drop for RecursionGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }
}

pub struct MemoryGuard {
    counter: Arc<AtomicU64>,
    bytes: u64,
}

impl Drop for MemoryGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(self.bytes, Ordering::SeqCst);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SafetyStats {
    pub recursion_depth: usize,
    pub memory_used: u64,
    pub elapsed_ms: u64,
    pub timeout_occurred: bool,
    pub config: SafetyConfig,
}

impl std::fmt::Display for SafetyStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "╔═══════════════════════════════════════════════════════════════╗")?;
        writeln!(f, "║   RUNTIME SAFETY STATISTICS                                   ║")?;
        writeln!(f, "╚═══════════════════════════════════════════════════════════════╝")?;
        writeln!(f)?;
        writeln!(f, "Resource Usage:")?;
        writeln!(f, "   Recursion depth:  {} / {}", 
                 self.recursion_depth, self.config.max_recursion_depth)?;
        writeln!(f, "   Memory used:      {} MB / {} MB", 
                 self.memory_used / (1024 * 1024), 
                 self.config.max_memory_bytes / (1024 * 1024))?;
        writeln!(f, "   Elapsed time:     {}ms / {}ms", 
                 self.elapsed_ms, self.config.max_execution_time_ms)?;
        writeln!(f)?;
        writeln!(f, "Safety Features:")?;
        writeln!(f, "   Strict mode:      {}", if self.config.strict_mode { "[OK] Enabled" } else { "[ERROR] Disabled" })?;
        writeln!(f, "   Bounds checking:  {}", if self.config.bounds_checking { "[OK] Enabled" } else { "[ERROR] Disabled" })?;
        writeln!(f, "   Type checking:    {}", if self.config.type_checking { "[OK] Enabled" } else { "[ERROR] Disabled" })?;
        writeln!(f, "   Overflow checks:  {}", if self.config.overflow_checking { "[OK] Enabled" } else { "[ERROR] Disabled" })?;
        writeln!(f)?;
        writeln!(f, "Timeout status:    {}", 
                 if self.timeout_occurred { "[ERROR] TIMED OUT" } else { "[OK] Running" })?;
        
        Ok(())
    }
}
