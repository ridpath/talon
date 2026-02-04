#![allow(dead_code)]

use crate::ast::Command;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunableParameter {
    pub name: String,
    pub current_value: i64,
    pub min_value: i64,
    pub max_value: i64,
    pub success_count: usize,
    pub failure_count: usize,
    pub learning_rate: f64,
}

pub struct ProbabilisticExecutor {
    tunables: Arc<RwLock<HashMap<String, TunableParameter>>>,
}

impl ProbabilisticExecutor {
    pub fn new() -> Self {
        ProbabilisticExecutor {
            tunables: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn try_all(
        &self,
        strategies: Vec<Vec<Command>>,
        timeout_ms: Option<u64>,
    ) -> Result<(usize, String), String> {
        let duration = timeout_ms
            .map(Duration::from_millis)
            .unwrap_or(Duration::from_secs(300));
        let mut handles = Vec::new();

        for (idx, strategy) in strategies.into_iter().enumerate() {
            let handle = tokio::spawn(async move { (idx, strategy) });
            handles.push(handle);
        }

        let result = match timeout(duration, async {
            for (_idx, handle) in handles.into_iter().enumerate() {
                if let Ok((strategy_idx, _strategy)) = handle.await {
                    return Ok((strategy_idx, "Strategy succeeded".to_string()));
                }
            }
            Err("All strategies failed".to_string())
        })
        .await
        {
            Ok(Ok(r)) => Ok(r),
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Timeout exceeded".to_string()),
        };

        result
    }

    pub async fn race(
        &self,
        threads: Vec<(String, Vec<Command>)>,
        sync_gap_ms: Option<u64>,
    ) -> Result<String, String> {
        let mut handles = Vec::new();

        for (name, commands) in threads.into_iter() {
            let handle = tokio::spawn(async move {
                if let Some(gap) = sync_gap_ms {
                    tokio::time::sleep(Duration::from_millis(gap)).await;
                }
                (name, commands)
            });
            handles.push(handle);
        }

        for handle in handles {
            if let Ok((name, _commands)) = handle.await {
                return Ok(name);
            }
        }

        Err("All threads failed".to_string())
    }

    pub async fn create_tunable(
        &self,
        name: &str,
        initial: i64,
        min: i64,
        max: i64,
    ) -> Result<(), String> {
        let tunable = TunableParameter {
            name: name.to_string(),
            current_value: initial,
            min_value: min,
            max_value: max,
            success_count: 0,
            failure_count: 0,
            learning_rate: 0.1,
        };

        self.tunables
            .write()
            .await
            .insert(name.to_string(), tunable);
        Ok(())
    }

    pub async fn get_tunable_value(&self, name: &str) -> Result<i64, String> {
        let tunables = self.tunables.read().await;
        tunables
            .get(name)
            .map(|t| t.current_value)
            .ok_or_else(|| format!("Tunable not found: {}", name))
    }

    pub async fn optimize_tunable(
        &self,
        name: &str,
        direction: &str,
        success: bool,
    ) -> Result<(), String> {
        let mut tunables = self.tunables.write().await;
        let tunable = tunables
            .get_mut(name)
            .ok_or_else(|| format!("Tunable not found: {}", name))?;

        if success {
            tunable.success_count += 1;
        } else {
            tunable.failure_count += 1;
        }

        let total_attempts = tunable.success_count + tunable.failure_count;
        if total_attempts < 3 {
            return Ok(());
        }

        let success_rate = tunable.success_count as f64 / total_attempts as f64;
        let adjustment =
            ((tunable.max_value - tunable.min_value) as f64 * tunable.learning_rate) as i64;

        match direction {
            "higher" => {
                if success_rate > 0.5 {
                    tunable.current_value =
                        (tunable.current_value + adjustment).min(tunable.max_value);
                } else {
                    tunable.current_value =
                        (tunable.current_value - adjustment).max(tunable.min_value);
                }
            }
            "lower" => {
                if success_rate > 0.5 {
                    tunable.current_value =
                        (tunable.current_value - adjustment).max(tunable.min_value);
                } else {
                    tunable.current_value =
                        (tunable.current_value + adjustment).min(tunable.max_value);
                }
            }
            "auto" => {
                if success_rate > 0.7 {
                    tunable.learning_rate *= 0.9;
                } else if success_rate < 0.3 {
                    tunable.current_value = (tunable.min_value + tunable.max_value) / 2;
                    tunable.learning_rate = 0.1;
                }
            }
            _ => return Err(format!("Unknown optimization direction: {}", direction)),
        }

        Ok(())
    }

    pub async fn get_statistics(&self) -> HashMap<String, TunableParameter> {
        self.tunables.read().await.clone()
    }
}
