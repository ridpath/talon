use crate::ast::Command;
use crate::interpreter::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeculativeResult {
    pub outcome: ExecutionOutcome,
    pub final_state: HashMap<String, String>,
    pub side_effects: Vec<String>,
    pub suggestion: Option<String>,
    pub probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionOutcome {
    Success,
    Crash,
    Hang,
    SecurityViolation,
    Unknown,
}

pub struct SpeculativeExecutor {
    futures: Arc<RwLock<HashMap<String, SpeculativeResult>>>,
    sandbox_id_counter: Arc<RwLock<u64>>,
}

impl SpeculativeExecutor {
    pub fn new() -> Self {
        SpeculativeExecutor {
            futures: Arc::new(RwLock::new(HashMap::new())),
            sandbox_id_counter: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn speculate(
        &self,
        commands: Vec<Command>,
        current_state: HashMap<String, Value>,
    ) -> Result<SpeculativeResult, String> {
        let sandbox_id = {
            let mut counter = self.sandbox_id_counter.write().await;
            *counter += 1;
            *counter
        };

        let result = self
            .execute_in_sandbox(sandbox_id, commands, current_state)
            .await?;

        self.futures
            .write()
            .await
            .insert(format!("sandbox_{}", sandbox_id), result.clone());

        Ok(result)
    }

    async fn execute_in_sandbox(
        &self,
        _sandbox_id: u64,
        commands: Vec<Command>,
        current_state: HashMap<String, Value>,
    ) -> Result<SpeculativeResult, String> {
        let outcome = self.simulate_execution(&commands, &current_state).await?;

        let final_state: HashMap<String, String> = current_state
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect();

        let side_effects = self.detect_side_effects(&commands).await;

        let suggestion = self.generate_suggestion(&outcome, &side_effects).await;

        let probability = self.calculate_probability(&outcome, &commands).await;

        Ok(SpeculativeResult {
            outcome,
            final_state,
            side_effects,
            suggestion,
            probability,
        })
    }

    async fn simulate_execution(
        &self,
        commands: &[Command],
        _state: &HashMap<String, Value>,
    ) -> Result<ExecutionOutcome, String> {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;

            let commands_clone = commands.to_vec();
            let timeout_ms = 5000;

            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || unsafe {
                let pid = libc::fork();

                if pid == 0 {
                    let result = Self::execute_in_child(&commands_clone);
                    std::process::exit(if result.is_ok() { 0 } else { 1 });
                } else if pid > 0 {
                    let start = Instant::now();
                    loop {
                        let mut status: libc::c_int = 0;
                        let result = libc::waitpid(pid, &mut status, libc::WNOHANG);

                        if result == pid {
                            if libc::WIFEXITED(status) {
                                let exit_code = libc::WEXITSTATUS(status);
                                tx.send(if exit_code == 0 {
                                    ExecutionOutcome::Success
                                } else {
                                    ExecutionOutcome::Unknown
                                })
                                .ok();
                            } else if libc::WIFSIGNALED(status) {
                                let signal = libc::WTERMSIG(status);
                                tx.send(if signal == libc::SIGSEGV || signal == libc::SIGBUS {
                                    ExecutionOutcome::Crash
                                } else {
                                    ExecutionOutcome::SecurityViolation
                                })
                                .ok();
                            }
                            break;
                        } else if result == 0 {
                            if start.elapsed().as_millis() > timeout_ms as u128 {
                                libc::kill(pid, libc::SIGKILL);
                                libc::waitpid(pid, &mut status, 0);
                                tx.send(ExecutionOutcome::Hang).ok();
                                break;
                            }
                            std::thread::sleep(Duration::from_millis(10));
                        } else {
                            tx.send(ExecutionOutcome::Unknown).ok();
                            break;
                        }
                    }
                } else {
                    tx.send(ExecutionOutcome::Unknown).ok();
                }
            });

            match rx.recv_timeout(Duration::from_millis(timeout_ms + 1000)) {
                Ok(outcome) => {
                    log::info!("Speculative execution completed: {:?}", outcome);
                    Ok(outcome)
                }
                Err(_) => {
                    log::warn!("Speculative execution timed out");
                    Ok(ExecutionOutcome::Hang)
                }
            }
        }

        #[cfg(not(unix))]
        {
            log::warn!("Fork-based speculation not available on this platform");
            for cmd in commands {
                if let Command::Expr(crate::ast::Expr::Call { name, .. }) = cmd {
                    if name.contains("crash") || name.contains("segfault") {
                        return Ok(ExecutionOutcome::Crash);
                    }
                }
            }
            Ok(ExecutionOutcome::Success)
        }
    }

    fn execute_in_child(commands: &[Command]) -> Result<(), String> {
        for (i, _cmd) in commands.iter().enumerate() {
            if i > 1000 {
                return Err("Command limit exceeded".to_string());
            }
        }
        Ok(())
    }

    async fn detect_side_effects(&self, commands: &[Command]) -> Vec<String> {
        let mut effects = Vec::new();

        for cmd in commands {
            match cmd {
                Command::WriteFile { .. } => {
                    effects.push("filesystem_modification".to_string());
                }
                Command::Connect { .. } => {
                    effects.push("network_activity".to_string());
                }
                Command::DumpMemory { .. } => {
                    effects.push("memory_access".to_string());
                }
                _ => {}
            }
        }

        effects
    }

    async fn generate_suggestion(
        &self,
        outcome: &ExecutionOutcome,
        side_effects: &[String],
    ) -> Option<String> {
        match outcome {
            ExecutionOutcome::Crash => {
                Some("This sequence will likely crash. Consider: 1) Check memory alignment 2) Validate addresses 3) Add null checks".to_string())
            }
            ExecutionOutcome::Hang => {
                Some("Potential infinite loop detected. Consider: 1) Add timeout 2) Review loop conditions 3) Use race primitive".to_string())
            }
            ExecutionOutcome::SecurityViolation => {
                Some("Security violation detected. This may trigger defensive mechanisms. Consider: 1) Add obfuscation 2) Use polymorphic variants 3) Delay execution".to_string())
            }
            ExecutionOutcome::Success => {
                if side_effects.contains(&"network_activity".to_string()) {
                    Some("Network activity detected. Ensure connection is established before sending data.".to_string())
                } else {
                    None
                }
            }
            ExecutionOutcome::Unknown => {
                Some("Execution outcome uncertain. Consider adding explicit error handling.".to_string())
            }
        }
    }

    async fn calculate_probability(&self, outcome: &ExecutionOutcome, commands: &[Command]) -> f64 {
        #[cfg(unix)]
        let base_probability = match outcome {
            ExecutionOutcome::Success => 0.95,
            ExecutionOutcome::Crash => 0.95,
            ExecutionOutcome::Hang => 0.90,
            ExecutionOutcome::SecurityViolation => 0.85,
            ExecutionOutcome::Unknown => 0.60,
        };

        #[cfg(not(unix))]
        let base_probability = match outcome {
            ExecutionOutcome::Success => 0.70,
            ExecutionOutcome::Crash => 0.75,
            ExecutionOutcome::Hang => 0.60,
            ExecutionOutcome::SecurityViolation => 0.50,
            ExecutionOutcome::Unknown => 0.40,
        };

        let complexity_factor = 1.0 - (commands.len() as f64 * 0.005).min(0.15);
        let probability = base_probability * complexity_factor;

        log::debug!(
            "Probability calculation: {:?} outcome with {} commands = {:.2}%",
            outcome,
            commands.len(),
            probability * 100.0
        );

        probability
    }

    pub async fn get_sandbox_count(&self) -> usize {
        self.futures.read().await.len()
    }

    pub async fn clear_futures(&self) {
        self.futures.write().await.clear();
        log::info!("Cleared all cached futures");
    }

    pub async fn precompute_futures(
        &self,
        branches: Vec<(String, Vec<Command>)>,
        current_state: HashMap<String, Value>,
    ) -> Result<HashMap<String, SpeculativeResult>, String> {
        let mut results = HashMap::new();

        for (branch_name, commands) in branches {
            let result = self.speculate(commands, current_state.clone()).await?;
            results.insert(branch_name, result);
        }

        Ok(results)
    }

    pub async fn select_best_future(
        &self,
        results: HashMap<String, SpeculativeResult>,
    ) -> Option<(String, SpeculativeResult)> {
        results
            .into_iter()
            .filter(|(_, r)| matches!(r.outcome, ExecutionOutcome::Success))
            .max_by(|a, b| a.1.probability.partial_cmp(&b.1.probability).unwrap())
    }
}
