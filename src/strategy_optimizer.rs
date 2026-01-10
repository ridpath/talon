#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::ast::Command;

#[derive(Debug, Clone)]
pub struct Strategy {
    pub name: String,
    pub parameters: HashMap<String, TunableParam>,
    pub implementation: Vec<Command>,
    pub success_count: usize,
    pub failure_count: usize,
    pub avg_execution_time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunableParam {
    pub name: String,
    pub current_value: i64,
    pub min_value: i64,
    pub max_value: i64,
    pub learning_rate: f64,
    pub optimization_direction: OptimizationDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationDirection {
    Maximize,
    Minimize,
    Auto,
}

pub struct StrategyOptimizer {
    strategies: Arc<RwLock<HashMap<String, Strategy>>>,
    execution_history: Arc<RwLock<Vec<ExecutionRecord>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub strategy_name: String,
    pub parameters: HashMap<String, i64>,
    pub success: bool,
    pub execution_time_ms: u64,
    pub timestamp: u64,
}

impl StrategyOptimizer {
    pub fn new() -> Self {
        StrategyOptimizer {
            strategies: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn define_strategy(
        &self,
        name: &str,
        parameters: HashMap<String, TunableParam>,
        implementation: Vec<Command>,
    ) -> Result<(), String> {
        let strategy = Strategy {
            name: name.to_string(),
            parameters,
            implementation,
            success_count: 0,
            failure_count: 0,
            avg_execution_time: 0.0,
        };

        self.strategies.write().await.insert(name.to_string(), strategy);
        Ok(())
    }

    pub async fn execute_strategy(
        &self,
        name: &str,
    ) -> Result<Vec<Command>, String> {
        let strategies = self.strategies.read().await;
        let strategy = strategies.get(name)
            .ok_or_else(|| format!("Strategy not found: {}", name))?;

        Ok(strategy.implementation.clone())
    }

    pub async fn record_execution(
        &self,
        strategy_name: &str,
        success: bool,
        execution_time_ms: u64,
    ) -> Result<(), String> {
        let mut strategies = self.strategies.write().await;
        let strategy = strategies.get_mut(strategy_name)
            .ok_or_else(|| format!("Strategy not found: {}", strategy_name))?;

        if success {
            strategy.success_count += 1;
        } else {
            strategy.failure_count += 1;
        }

        let total_executions = (strategy.success_count + strategy.failure_count) as f64;
        strategy.avg_execution_time = 
            (strategy.avg_execution_time * (total_executions - 1.0) + execution_time_ms as f64) / total_executions;

        let params: HashMap<String, i64> = strategy.parameters
            .iter()
            .map(|(k, v)| (k.clone(), v.current_value))
            .collect();

        let record = ExecutionRecord {
            strategy_name: strategy_name.to_string(),
            parameters: params,
            success,
            execution_time_ms,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        drop(strategies);
        self.execution_history.write().await.push(record);

        self.auto_tune_parameters(strategy_name, success).await?;

        Ok(())
    }

    async fn auto_tune_parameters(&self, strategy_name: &str, success: bool) -> Result<(), String> {
        let mut strategies = self.strategies.write().await;
        let strategy = strategies.get_mut(strategy_name)
            .ok_or_else(|| format!("Strategy not found: {}", strategy_name))?;

        let total_attempts = strategy.success_count + strategy.failure_count;
        if total_attempts < 5 {
            return Ok(());
        }

        let success_rate = strategy.success_count as f64 / total_attempts as f64;
        let recent_history: Vec<bool> = self.execution_history.read().await
            .iter()
            .filter(|r| r.strategy_name == strategy_name)
            .rev()
            .take(10)
            .map(|r| r.success)
            .collect();

        let recent_success_rate = if !recent_history.is_empty() {
            recent_history.iter().filter(|&&s| s).count() as f64 / recent_history.len() as f64
        } else {
            success_rate
        };

        for (param_name, param) in strategy.parameters.iter_mut() {
            let range = (param.max_value - param.min_value) as f64;
            let normalized_position = (param.current_value - param.min_value) as f64 / range;
            
            let gradient = if success {
                1.0 - recent_success_rate
            } else {
                -(1.0 - recent_success_rate)
            };

            let adjustment = (range * param.learning_rate * gradient) as i64;

            match param.optimization_direction {
                OptimizationDirection::Maximize => {
                    if success {
                        param.current_value = (param.current_value + adjustment.abs()).min(param.max_value);
                    } else {
                        param.current_value = (param.current_value - adjustment.abs()).max(param.min_value);
                    }
                }
                OptimizationDirection::Minimize => {
                    if success {
                        param.current_value = (param.current_value - adjustment.abs()).max(param.min_value);
                    } else {
                        param.current_value = (param.current_value + adjustment.abs()).min(param.max_value);
                    }
                }
                OptimizationDirection::Auto => {
                    if recent_success_rate > 0.7 {
                        param.learning_rate *= 0.9;
                    } else if recent_success_rate < 0.3 {
                        if normalized_position > 0.7 || normalized_position < 0.3 {
                            param.current_value = (param.min_value + param.max_value) / 2;
                        }
                        param.learning_rate = param.learning_rate.max(0.05);
                    } else {
                        param.current_value = (param.current_value as f64 * 0.9 + 
                                             ((param.min_value + param.max_value) / 2) as f64 * 0.1) as i64;
                    }
                }
            }

            log::debug!("Parameter '{}' adjusted: {} (success_rate: {:.2}, recent: {:.2})", 
                       param_name, param.current_value, success_rate, recent_success_rate);
        }

        Ok(())
    }

    pub async fn get_strategy_stats(&self, name: &str) -> Result<(f64, f64, usize), String> {
        let strategies = self.strategies.read().await;
        let strategy = strategies.get(name)
            .ok_or_else(|| format!("Strategy not found: {}", name))?;

        let total = (strategy.success_count + strategy.failure_count) as f64;
        let success_rate = if total > 0.0 {
            strategy.success_count as f64 / total
        } else {
            0.0
        };

        Ok((success_rate, strategy.avg_execution_time, total as usize))
    }

    pub async fn get_parameter_history(&self, strategy_name: &str) -> Result<Vec<HashMap<String, i64>>, String> {
        let history = self.execution_history.read().await;
        let param_history: Vec<HashMap<String, i64>> = history
            .iter()
            .filter(|r| r.strategy_name == strategy_name)
            .map(|r| r.parameters.clone())
            .collect();
        
        log::info!("Retrieved parameter history for '{}': {} records", strategy_name, param_history.len());
        Ok(param_history)
    }

    pub async fn reset_strategy(&self, strategy_name: &str) -> Result<(), String> {
        let mut strategies = self.strategies.write().await;
        let strategy = strategies.get_mut(strategy_name)
            .ok_or_else(|| format!("Strategy not found: {}", strategy_name))?;

        strategy.success_count = 0;
        strategy.failure_count = 0;
        strategy.avg_execution_time = 0.0;
        
        for param in strategy.parameters.values_mut() {
            param.current_value = (param.min_value + param.max_value) / 2;
            param.learning_rate = 0.1;
        }

        log::info!("Reset strategy: {}", strategy_name);
        Ok(())
    }

    pub async fn compare_strategies(&self, names: Vec<String>) -> Result<Vec<(String, f64, f64)>, String> {
        let mut results = Vec::new();

        for name in names {
            let (success_rate, avg_time, _total) = self.get_strategy_stats(&name).await?;
            results.push((name, success_rate, avg_time));
        }

        results.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap()
                .then(a.2.partial_cmp(&b.2).unwrap())
        });

        Ok(results)
    }

    pub async fn get_parameter_value(&self, strategy_name: &str, param_name: &str) -> Result<i64, String> {
        let strategies = self.strategies.read().await;
        let strategy = strategies.get(strategy_name)
            .ok_or_else(|| format!("Strategy not found: {}", strategy_name))?;

        let param = strategy.parameters.get(param_name)
            .ok_or_else(|| format!("Parameter not found: {}", param_name))?;

        Ok(param.current_value)
    }
}
