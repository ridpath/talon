// ═══════════════════════════════════════════════════════════════════════════
// HEAP GROOMING & FENG SHUI - ADVANCED HEAP LAYOUT MANIPULATION
// ═══════════════════════════════════════════════════════════════════════════
// World-class heap grooming primitives for controlling heap layout,
// cache alignment, and exploit reliability

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Heap grooming strategy
#[derive(Debug, Clone, PartialEq)]
pub enum GroomingStrategy {
    /// Spray identical chunks to fill tcache/fastbin
    Spray { size: usize, count: usize },
    /// Create holes for chunk consolidation
    Holes { size: usize, pattern: Vec<bool> },
    /// Align chunks to cache lines
    CacheAlign { size: usize, alignment: usize },
    /// Create predictable heap layout
    FengShui { layout: Vec<HeapBlock> },
    /// Fill bins in specific order
    BinFilling { bins: Vec<usize> },
}

/// Heap block in feng shui layout
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeapBlock {
    pub size: usize,
    pub keep: bool, // Keep allocated or free
    pub data: Vec<u8>,
}

impl HeapBlock {
    pub fn new(size: usize, keep: bool) -> Self {
        HeapBlock {
            size,
            keep,
            data: vec![0x41; size],
        }
    }

    pub fn with_data(size: usize, keep: bool, data: Vec<u8>) -> Self {
        HeapBlock { size, keep, data }
    }
}

/// Heap grooming plan
pub struct HeapGroom {
    pub binary: String,
    pub strategy: GroomingStrategy,
    pub target_layout: Vec<HeapBlock>,
    pub allocations: Vec<AllocationStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationStep {
    pub step: usize,
    pub action: String,
    pub size: usize,
    pub data: Option<Vec<u8>>,
    pub free: bool,
}

impl HeapGroom {
    /// Create new heap grooming plan
    pub fn new(binary: &str, strategy: GroomingStrategy) -> Self {
        HeapGroom {
            binary: binary.to_string(),
            strategy,
            target_layout: Vec::new(),
            allocations: Vec::new(),
        }
    }

    /// Generate spray pattern
    pub fn spray(size: usize, count: usize) -> Vec<Vec<u8>> {
        log::info!(
            "Generating spray pattern: {} chunks of size 0x{:x}",
            count,
            size
        );

        let mut payloads = Vec::new();
        for i in 0..count {
            let mut data = vec![b'A' + (i % 26) as u8; size];
            // Add marker
            let marker = format!("SPRAY{:04}", i);
            data[..marker.len()].copy_from_slice(marker.as_bytes());
            payloads.push(data);
        }

        payloads
    }

    /// Generate hole pattern for consolidation
    pub fn create_holes(size: usize, pattern: Vec<bool>) -> Vec<AllocationStep> {
        log::info!(
            "Creating hole pattern: size=0x{:x}, holes={}",
            size,
            pattern.len()
        );

        let mut steps = Vec::new();

        // Step 1: Allocate all chunks
        for (i, &_keep) in pattern.iter().enumerate() {
            steps.push(AllocationStep {
                step: i,
                action: format!("malloc(0x{:x})", size),
                size,
                data: Some(vec![b'A'; size]),
                free: false,
            });
        }

        // Step 2: Free chunks based on pattern
        for (i, &_keep) in pattern.iter().enumerate() {
            if !_keep {
                steps.push(AllocationStep {
                    step: pattern.len() + i,
                    action: format!("free(chunk_{})", i),
                    size,
                    data: None,
                    free: true,
                });
            }
        }

        steps
    }

    /// Align allocations to cache lines (64 bytes typical)
    pub fn cache_align(size: usize, alignment: usize) -> Vec<AllocationStep> {
        log::info!(
            "Cache-aligning chunks: size=0x{:x}, align={}",
            size,
            alignment
        );

        let mut steps = Vec::new();
        let padding = alignment - (size % alignment);

        if padding == alignment {
            // Already aligned
            steps.push(AllocationStep {
                step: 0,
                action: format!("malloc(0x{:x})", size),
                size,
                data: Some(vec![b'A'; size]),
                free: false,
            });
        } else {
            // Add padding chunk
            steps.push(AllocationStep {
                step: 0,
                action: format!("malloc(0x{:x}) // padding", padding),
                size: padding,
                data: Some(vec![b'P'; padding]),
                free: true, // Will be freed
            });

            // Add aligned chunk
            steps.push(AllocationStep {
                step: 1,
                action: format!("malloc(0x{:x}) // aligned", size),
                size,
                data: Some(vec![b'A'; size]),
                free: false,
            });
        }

        steps
    }

    /// Generate feng shui layout plan
    pub fn feng_shui(layout: Vec<HeapBlock>) -> Vec<AllocationStep> {
        log::info!("Generating feng shui layout: {} blocks", layout.len());

        let mut steps = Vec::new();
        let mut chunk_index = HashMap::new();

        // Allocate all blocks
        for (i, block) in layout.iter().enumerate() {
            let step_num = steps.len();
            steps.push(AllocationStep {
                step: step_num,
                action: format!("malloc(0x{:x})", block.size),
                size: block.size,
                data: Some(block.data.clone()),
                free: false,
            });
            chunk_index.insert(i, step_num);
        }

        // Free blocks that shouldn't be kept
        for (i, block) in layout.iter().enumerate() {
            if !block.keep {
                let step_num = steps.len();
                steps.push(AllocationStep {
                    step: step_num,
                    action: format!("free(chunk_{})", i),
                    size: block.size,
                    data: None,
                    free: true,
                });
            }
        }

        steps
    }

    /// Fill tcache bins in order
    pub fn fill_tcache(sizes: Vec<usize>, bins_per_size: usize) -> Vec<AllocationStep> {
        log::info!(
            "Filling tcache: {} sizes, {} bins each",
            sizes.len(),
            bins_per_size
        );

        let mut steps = Vec::new();
        let mut chunk_counter = 0;

        for size in sizes {
            // Allocate 7 chunks (tcache max)
            for _ in 0..bins_per_size.min(7) {
                steps.push(AllocationStep {
                    step: chunk_counter,
                    action: format!("malloc(0x{:x})", size),
                    size,
                    data: Some(vec![b'T'; size]),
                    free: false,
                });
                chunk_counter += 1;
            }

            // Free all to populate tcache
            for i in 0..bins_per_size.min(7) {
                steps.push(AllocationStep {
                    step: chunk_counter,
                    action: format!("free(chunk_0x{:x}_{})", size, i),
                    size,
                    data: None,
                    free: true,
                });
                chunk_counter += 1;
            }
        }

        steps
    }

    /// Generate exploit script for grooming
    pub fn generate_script(&self) -> String {
        let mut script = String::new();

        script.push_str(&format!("# Heap Grooming Script for {}\n", self.binary));
        script.push_str(&format!("# Strategy: {:?}\n\n", self.strategy));

        let steps = match &self.strategy {
            GroomingStrategy::Spray { size, count } => {
                let payloads = Self::spray(*size, *count);
                let mut s = Vec::new();
                for (i, payload) in payloads.iter().enumerate() {
                    s.push(AllocationStep {
                        step: i,
                        action: format!("spray[{}] = malloc(0x{:x})", i, size),
                        size: *size,
                        data: Some(payload.clone()),
                        free: false,
                    });
                }
                s
            }
            GroomingStrategy::Holes { size, pattern } => Self::create_holes(*size, pattern.clone()),
            GroomingStrategy::CacheAlign { size, alignment } => {
                Self::cache_align(*size, *alignment)
            }
            GroomingStrategy::FengShui { layout } => Self::feng_shui(layout.clone()),
            GroomingStrategy::BinFilling { bins } => Self::fill_tcache(bins.clone(), 7),
        };

        for step in steps {
            script.push_str(&format!("# Step {}: {}\n", step.step, step.action));
            if step.free {
                script.push_str(&format!("free({})\n\n", step.step));
            } else {
                script.push_str(&format!(
                    "chunk_{} = malloc(0x{:x})\n",
                    step.step, step.size
                ));
                if let Some(data) = &step.data {
                    script.push_str(&format!(
                        "write(chunk_{}, {:?})\n\n",
                        step.step,
                        &data[..data.len().min(16)]
                    ));
                }
            }
        }

        script
    }

    /// Visualize heap layout
    pub fn visualize(&self) -> String {
        let mut vis = String::new();
        vis.push_str("═══════════════════════════════════════════\n");
        vis.push_str("         HEAP LAYOUT VISUALIZATION        \n");
        vis.push_str("═══════════════════════════════════════════\n\n");

        match &self.strategy {
            GroomingStrategy::FengShui { layout } => {
                for (i, block) in layout.iter().enumerate() {
                    let status = if block.keep { "[KEEP]" } else { "[FREE]" };
                    vis.push_str(&format!(
                        "Chunk {}: 0x{:04x} bytes {}\n",
                        i, block.size, status
                    ));
                    vis.push_str("  ┌────────────────────────────────────────────────────┐\n");

                    let preview = if block.data.len() > 8 {
                        format!("{:02x?}...", &block.data[..8])
                    } else {
                        format!("{:02x?}", block.data)
                    };
                    vis.push_str(&format!("  │ Data: {:<43} │\n", preview));
                    vis.push_str("  └────────────────────────────────────────────────────┘\n");
                }
                vis
            }
            _ => format!("Visualization not available for {:?}", self.strategy),
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ADVANCED GROOMING TECHNIQUES
// ────────────────────────────────────────────────────────────────────────────

/// Calculate optimal spray count for target reliability
pub fn calculate_spray_count(target_prob: f64, chunk_size: usize) -> usize {
    // Heuristic: Higher probability requires more chunks
    let base_count = 100;
    let size_factor = 1.0 + (chunk_size as f64 / 1024.0);
    let prob_factor = target_prob * 2.0;

    (base_count as f64 * size_factor * prob_factor) as usize
}

/// Generate anti-consolidation guards
pub fn anti_consolidation_guards(target_size: usize) -> Vec<HeapBlock> {
    let guard_size = 0x20;
    vec![
        HeapBlock::new(guard_size, true),
        HeapBlock::new(target_size, false),
        HeapBlock::new(guard_size, true),
    ]
}

/// Create tcache dup pattern for double-free
pub fn tcache_dup_pattern(size: usize) -> Vec<AllocationStep> {
    vec![
        AllocationStep {
            step: 0,
            action: format!("A = malloc(0x{:x})", size),
            size,
            data: Some(vec![b'A'; size]),
            free: false,
        },
        AllocationStep {
            step: 1,
            action: "free(A)".to_string(),
            size,
            data: None,
            free: true,
        },
        AllocationStep {
            step: 2,
            action: "free(A) // double-free".to_string(),
            size,
            data: None,
            free: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spray_generation() {
        let payloads = HeapGroom::spray(0x80, 10);
        assert_eq!(payloads.len(), 10);
        assert_eq!(payloads[0].len(), 0x80);
    }

    #[test]
    fn test_hole_pattern() {
        let pattern = vec![true, false, true, false, true];
        let steps = HeapGroom::create_holes(0x80, pattern.clone());

        // Should have allocation steps + free steps
        assert!(steps.len() >= pattern.len());
    }

    #[test]
    fn test_cache_align() {
        let steps = HeapGroom::cache_align(0x50, 64);
        assert!(!steps.is_empty());
    }

    #[test]
    fn test_feng_shui() {
        let layout = vec![
            HeapBlock::new(0x80, true),
            HeapBlock::new(0x90, false),
            HeapBlock::new(0x80, true),
        ];

        let steps = HeapGroom::feng_shui(layout);
        assert_eq!(steps.len(), 4); // 3 allocations + 1 free
    }

    #[test]
    fn test_tcache_filling() {
        let steps = HeapGroom::fill_tcache(vec![0x20, 0x40], 7);
        assert!(!steps.is_empty());
    }

    #[test]
    fn test_spray_count_calculation() {
        let count = calculate_spray_count(0.95, 0x80);
        assert!(count > 100);
    }

    #[test]
    fn test_anti_consolidation_guards() {
        let guards = anti_consolidation_guards(0x80);
        assert_eq!(guards.len(), 3);
        assert!(guards[0].keep);
        assert!(!guards[1].keep);
        assert!(guards[2].keep);
    }

    #[test]
    fn test_tcache_dup_pattern() {
        let pattern = tcache_dup_pattern(0x80);
        assert_eq!(pattern.len(), 3);
        assert!(pattern[1].free);
        assert!(pattern[2].free);
    }

    #[test]
    fn test_heap_block_creation() {
        let block = HeapBlock::new(0x80, true);
        assert_eq!(block.size, 0x80);
        assert!(block.keep);
        assert_eq!(block.data.len(), 0x80);
    }

    #[test]
    fn test_heap_groom_creation() {
        let groom = HeapGroom::new(
            "./vuln",
            GroomingStrategy::Spray {
                size: 0x80,
                count: 100,
            },
        );
        assert_eq!(groom.binary, "./vuln");
    }
}
