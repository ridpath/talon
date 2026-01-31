use crate::session_state::ExploitSession;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ResilientExecutor {
    max_attempts: usize,
    auto_rollback: bool,
    strategy: RetryStrategy,
}

#[derive(Debug, Clone)]
pub enum RetryStrategy {
    Immediate,
    ExponentialBackoff { initial_delay_ms: u64, max_delay_ms: u64 },
    Linear { delay_ms: u64 },
}

#[derive(Debug, Clone)]
pub struct AttemptResult<T> {
    pub success: bool,
    pub result: Option<T>,
    pub error: Option<String>,
    pub attempt_number: usize,
    pub rolled_back: bool,
}

#[derive(Debug, Clone)]
pub struct ResilientBlock {
    pub attempts: Vec<AttemptConfig>,
    pub recovery: Option<RecoveryConfig>,
}

#[derive(Debug, Clone)]
pub struct AttemptConfig {
    pub name: String,
    pub timeout_ms: Option<u64>,
    pub expected_failure: bool,
}

#[derive(Debug, Clone)]
pub struct RecoveryConfig {
    pub on_failure: RecoveryAction,
    pub max_recovery_attempts: usize,
}

#[derive(Debug, Clone)]
pub enum RecoveryAction {
    Rollback,
    Continue,
    Abort,
    Custom(String),
}

impl ResilientExecutor {
    pub fn new() -> Self {
        ResilientExecutor {
            max_attempts: 3,
            auto_rollback: true,
            strategy: RetryStrategy::ExponentialBackoff {
                initial_delay_ms: 100,
                max_delay_ms: 5000,
            },
        }
    }

    pub fn with_max_attempts(mut self, max: usize) -> Self {
        self.max_attempts = max;
        self
    }

    pub fn with_strategy(mut self, strategy: RetryStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn with_auto_rollback(mut self, enabled: bool) -> Self {
        self.auto_rollback = enabled;
        self
    }

    pub async fn execute_resilient<F, T>(
        &self,
        session: &ExploitSession,
        operation: F,
    ) -> AttemptResult<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send>>,
        T: Clone,
    {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < self.max_attempts {
            attempt += 1;

            let checkpoint = if self.auto_rollback {
                session.checkpoint().await.ok()
            } else {
                None
            };

            let result = operation().await;

            match result {
                Ok(value) => {
                    return AttemptResult {
                        success: true,
                        result: Some(value),
                        error: None,
                        attempt_number: attempt,
                        rolled_back: false,
                    };
                }
                Err(e) => {
                    last_error = Some(e);

                    if let Some(checkpoint_id) = checkpoint {
                        let _ = session.rewind(checkpoint_id).await;
                    }

                    if attempt < self.max_attempts {
                        self.apply_delay(attempt).await;
                    }
                }
            }
        }

        AttemptResult {
            success: false,
            result: None,
            error: last_error,
            attempt_number: attempt,
            rolled_back: self.auto_rollback,
        }
    }

    pub async fn execute_block<F, T>(
        &self,
        session: &ExploitSession,
        block: ResilientBlock,
        operations: Vec<F>,
    ) -> Result<Vec<T>, String>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send>>,
        T: Clone,
    {
        let mut results = Vec::new();
        let initial_checkpoint = session.checkpoint().await?;

        for (idx, operation) in operations.into_iter().enumerate() {
            let attempt_config = block.attempts.get(idx).ok_or("Missing attempt config")?;
            
            let checkpoint = session.checkpoint().await?;
            
            match operation().await {
                Ok(value) => {
                    results.push(value);
                }
                Err(e) => {
                    session.rewind(checkpoint).await?;

                    if !attempt_config.expected_failure {
                        if let Some(recovery) = &block.recovery {
                            match recovery.on_failure {
                                RecoveryAction::Rollback => {
                                    session.rewind(initial_checkpoint).await?;
                                    return Err(format!("Operation failed: {}", e));
                                }
                                RecoveryAction::Continue => {
                                    continue;
                                }
                                RecoveryAction::Abort => {
                                    return Err(format!("Aborted: {}", e));
                                }
                                RecoveryAction::Custom(ref action) => {
                                    return Err(format!("Custom recovery needed: {}", action));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    async fn apply_delay(&self, attempt: usize) {
        let delay = match self.strategy {
            RetryStrategy::Immediate => return,
            RetryStrategy::Linear { delay_ms } => delay_ms,
            RetryStrategy::ExponentialBackoff { initial_delay_ms, max_delay_ms } => {
                let delay = initial_delay_ms * 2_u64.pow(attempt as u32 - 1);
                delay.min(max_delay_ms)
            }
        };

        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
    }
}

pub struct RecoveryManager {
    checkpoints: Arc<RwLock<Vec<RecoveryPoint>>>,
}

#[derive(Debug, Clone)]
pub struct RecoveryPoint {
    pub id: u64,
    pub session_checkpoint: u64,
    pub context: String,
    pub timestamp: std::time::Instant,
}

impl RecoveryManager {
    pub fn new() -> Self {
        RecoveryManager {
            checkpoints: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn create_recovery_point(
        &self,
        session: &ExploitSession,
        context: String,
    ) -> Result<u64, String> {
        let session_checkpoint = session.checkpoint().await?;
        
        let mut checkpoints = self.checkpoints.write().await;
        let id = checkpoints.len() as u64 + 1;
        
        checkpoints.push(RecoveryPoint {
            id,
            session_checkpoint,
            context,
            timestamp: std::time::Instant::now(),
        });

        Ok(id)
    }

    pub async fn recover_to_point(
        &self,
        session: &ExploitSession,
        point_id: u64,
    ) -> Result<(), String> {
        let checkpoints = self.checkpoints.read().await;
        
        let point = checkpoints
            .iter()
            .find(|p| p.id == point_id)
            .ok_or("Recovery point not found")?;

        session.rewind(point.session_checkpoint).await?;
        Ok(())
    }

    pub async fn list_recovery_points(&self) -> Vec<RecoveryPoint> {
        let checkpoints = self.checkpoints.read().await;
        checkpoints.clone()
    }
}

pub async fn resilient_with_session<F, T>(
    session: &ExploitSession,
    label: &str,
    operation: F,
) -> Result<T, String>
where
    F: std::future::Future<Output = Result<T, String>>,
{
    let checkpoint = session.checkpoint_labeled(label.to_string()).await?;
    
    match operation.await {
        Ok(result) => Ok(result),
        Err(e) => {
            session.rewind(checkpoint).await?;
            Err(format!("Operation '{}' failed and rolled back: {}", label, e))
        }
    }
}

pub async fn attempt_with_fallback<F1, F2, T>(
    session: &ExploitSession,
    primary: F1,
    fallback: F2,
) -> Result<T, String>
where
    F1: std::future::Future<Output = Result<T, String>>,
    F2: std::future::Future<Output = Result<T, String>>,
{
    let checkpoint = session.checkpoint().await?;
    
    match primary.await {
        Ok(result) => Ok(result),
        Err(_) => {
            session.rewind(checkpoint).await?;
            fallback.await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resilient_executor() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        
        let executor = ResilientExecutor::new();
        let session = ExploitSession::new();
        
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);
        let result = executor.execute_resilient(&session, move || {
            let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                if count < 1 {
                    Err("Simulated failure".to_string())
                } else {
                    Ok(42)
                }
            })
        }).await;

        assert!(result.success);
        assert_eq!(result.result, Some(42));
        assert_eq!(result.attempt_number, 2);
    }

    #[tokio::test]
    async fn test_recovery_manager() {
        let manager = RecoveryManager::new();
        let session = ExploitSession::new();
        
        session.set_libc_base(0x1000).await;
        let point_id = manager.create_recovery_point(&session, "test".to_string()).await.unwrap();
        
        session.set_libc_base(0x2000).await;
        assert_eq!(session.get_libc_base().await, Some(0x2000));
        
        manager.recover_to_point(&session, point_id).await.unwrap();
        assert_eq!(session.get_libc_base().await, Some(0x1000));
    }
}
