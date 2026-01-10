use std::time::Instant;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileEntry {
    pub name: String,
    pub duration_us: u128,
    pub call_count: usize,
    pub memory_delta: Option<i64>,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallGraph {
    pub nodes: HashMap<String, ProfileEntry>,
    pub root: String,
}

pub struct Profiler {
    enabled: bool,
    start_time: Instant,
    entries: Vec<(String, Instant, Option<usize>)>,
    completed: HashMap<String, ProfileEntry>,
    call_stack: Vec<String>,
    memory_baseline: usize,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            enabled: false,
            start_time: Instant::now(),
            entries: Vec::new(),
            completed: HashMap::new(),
            call_stack: Vec::new(),
            memory_baseline: 0,
        }
    }
    
    pub fn enable(&mut self) {
        self.enabled = true;
        self.start_time = Instant::now();
    }
    
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    pub fn start_region(&mut self, name: &str) {
        if !self.enabled {
            return;
        }
        
        self.entries.push((name.to_string(), Instant::now(), None));
        self.call_stack.push(name.to_string());
    }
    
    pub fn end_region(&mut self, name: &str) {
        if !self.enabled {
            return;
        }
        
        if let Some((entry_name, start, _)) = self.entries.iter().rev()
            .find(|(n, _, _)| n == name) {
            
            let duration = start.elapsed();
            let entry_name = entry_name.clone();
            
            self.completed.entry(entry_name.clone())
                .and_modify(|e| {
                    e.duration_us += duration.as_micros();
                    e.call_count += 1;
                })
                .or_insert(ProfileEntry {
                    name: entry_name.clone(),
                    duration_us: duration.as_micros(),
                    call_count: 1,
                    memory_delta: None,
                    children: Vec::new(),
                });
        }
        
        if let Some(idx) = self.call_stack.iter().rposition(|n| n == name) {
            self.call_stack.remove(idx);
        }
    }
    
    pub fn get_results(&self) -> Vec<ProfileEntry> {
        let mut results: Vec<_> = self.completed.values().cloned().collect();
        results.sort_by(|a, b| b.duration_us.cmp(&a.duration_us));
        results
    }
    
    pub fn get_call_graph(&self) -> Option<CallGraph> {
        if self.completed.is_empty() {
            return None;
        }
        
        let root = self.completed.keys().next()?.clone();
        
        Some(CallGraph {
            nodes: self.completed.clone(),
            root,
        })
    }
    
    pub fn print_report(&self) {
        println!("\n╔═══════════════════════════════════════════════════════════════════════════╗");
        println!("║                        PERFORMANCE PROFILE                                ║");
        println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");
        
        let total_duration = self.start_time.elapsed();
        println!("Total execution time: {:.3}ms\n", total_duration.as_secs_f64() * 1000.0);
        
        println!("{:<40} {:>12} {:>10} {:>12}", "Region", "Time (ms)", "Calls", "Avg (μs)");
        println!("{}", "─".repeat(78));
        
        let results = self.get_results();
        for entry in &results {
            let avg_us = entry.duration_us / entry.call_count as u128;
            let time_ms = entry.duration_us as f64 / 1000.0;
            println!("{:<40} {:>12.3} {:>10} {:>12}", 
                     entry.name, 
                     time_ms, 
                     entry.call_count,
                     avg_us);
        }
        
        if !results.is_empty() {
            println!("\nHotspots (Top 5):");
            for (i, entry) in results.iter().take(5).enumerate() {
                let percentage = (entry.duration_us as f64 / total_duration.as_micros() as f64) * 100.0;
                println!("  {}. {} - {:.2}% of total time", i + 1, entry.name, percentage);
            }
        }
        
        println!();
    }
    
    pub fn export_json(&self, path: &str) -> Result<(), String> {
        let results = self.get_results();
        let json = serde_json::to_string_pretty(&results)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        
        fs::write(path, json)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        Ok(())
    }
    
    pub fn export_flamegraph(&self, path: &str) -> Result<(), String> {
        let mut output = String::new();
        
        for entry in &self.get_results() {
            output.push_str(&format!("{} {}\n", entry.name, entry.duration_us));
        }
        
        fs::write(path, output)
            .map_err(|e| format!("Failed to write flamegraph: {}", e))?;
        
        Ok(())
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
        self.completed.clear();
        self.call_stack.clear();
        self.start_time = Instant::now();
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ScopedProfile<'a> {
    profiler: &'a mut Profiler,
    name: String,
}

impl<'a> ScopedProfile<'a> {
    pub fn new(profiler: &'a mut Profiler, name: &str) -> Self {
        profiler.start_region(name);
        Self {
            profiler,
            name: name.to_string(),
        }
    }
}

impl<'a> Drop for ScopedProfile<'a> {
    fn drop(&mut self) {
        self.profiler.end_region(&self.name);
    }
}

#[macro_export]
macro_rules! profile {
    ($profiler:expr, $name:expr, $block:block) => {
        {
            let _scope = $crate::profiler::ScopedProfile::new($profiler, $name);
            $block
        }
    };
}

pub fn print_usage_guide() {
    println!(r#"
╔═══════════════════════════════════════════════════════════════════════════╗
║                        PERFORMANCE PROFILING                              ║
╚═══════════════════════════════════════════════════════════════════════════╝

OVERVIEW
The Talon profiler helps identify performance bottlenecks in your scripts by
measuring execution time of commands and function calls.

USAGE
# Profile a script
talon profile script.tal

# Profile and save results
talon profile script.tal --output report.json

# Profile and generate flamegraph
talon profile script.tal --flamegraph flame.txt

INTERPRETING RESULTS
- Time (ms):     Total time spent in that region
- Calls:         Number of times the region was executed
- Avg (μs):      Average time per call in microseconds
- Hotspots:      Regions consuming the most time

OPTIMIZATION TIPS
1. Focus on hotspots - optimize regions that take the most time
2. Reduce call counts for expensive operations
3. Cache results of repeated computations
4. Use async operations for I/O-bound tasks
5. Batch network requests when possible

EXAMPLE OUTPUT
Region                                        Time (ms)    Calls    Avg (μs)
──────────────────────────────────────────────────────────────────────────
rop_chain_builder                               245.678       12      20473
shellcode_encoder                               128.432       45       2854
memory_scanner                                   89.123        3      29708

Hotspots (Top 5):
  1. rop_chain_builder - 52.31% of total time
  2. shellcode_encoder - 27.35% of total time
  3. memory_scanner - 18.98% of total time

PROGRAMMATIC USAGE
In your Talon script, use profile blocks:

profile "my_expensive_operation" {{
    // Your code here
}}

Or via API:
profiler.start_region("scan")
perform_scan()
profiler.end_region("scan")

For more information: talon man profiler
"#);
}
