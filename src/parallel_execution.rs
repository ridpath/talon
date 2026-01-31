#![allow(clippy::type_complexity)]
#![allow(clippy::extra_unused_type_parameters)]

use futures::future::join_all;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct ParallelExecutor {
    max_concurrency: usize,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug, Clone)]
pub struct ParallelResult<T> {
    pub index: usize,
    pub result: Result<T, String>,
    pub duration: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct RaceResult<T> {
    pub winner_index: usize,
    pub result: T,
    pub duration: std::time::Duration,
}

impl ParallelExecutor {
    pub fn new(max_concurrency: usize) -> Self {
        ParallelExecutor {
            max_concurrency,
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
        }
    }

    pub async fn parallel_for<F, T, I>(&self, items: Vec<I>, operation: F) -> Vec<ParallelResult<T>>
    where
        F: Fn(I) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send>>
            + Send
            + Sync
            + 'static,
        T: Send + 'static,
        I: Send + 'static,
    {
        let operation = Arc::new(operation);
        let mut handles = Vec::new();

        for (index, item) in items.into_iter().enumerate() {
            let semaphore = Arc::clone(&self.semaphore);
            let operation = Arc::clone(&operation);

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let start = std::time::Instant::now();
                let result = operation(item).await;
                let duration = start.elapsed();

                ParallelResult {
                    index,
                    result,
                    duration,
                }
            });

            handles.push(handle);
        }

        let results = join_all(handles).await;
        results.into_iter().filter_map(|r| r.ok()).collect()
    }

    pub async fn parallel_map<F, T, U, I>(&self, items: Vec<I>, mapper: F) -> Vec<U>
    where
        F: Fn(I) -> std::pin::Pin<Box<dyn std::future::Future<Output = U> + Send>>
            + Send
            + Sync
            + 'static,
        T: Send + 'static,
        U: Send + 'static,
        I: Send + 'static,
    {
        let mapper = Arc::new(mapper);
        let mut handles = Vec::new();

        for item in items {
            let semaphore = Arc::clone(&self.semaphore);
            let mapper = Arc::clone(&mapper);

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                mapper(item).await
            });

            handles.push(handle);
        }

        let results = join_all(handles).await;
        results.into_iter().filter_map(|r| r.ok()).collect()
    }

    pub async fn race<F, T>(&self, operations: Vec<F>) -> RaceResult<T>
    where
        F: std::future::Future<Output = Result<T, String>> + Send + 'static,
        T: Send + 'static,
    {
        let mut handles = Vec::new();
        let start = std::time::Instant::now();

        for (index, operation) in operations.into_iter().enumerate() {
            let handle = tokio::spawn(async move { (index, operation.await) });
            handles.push(handle);
        }

        let (result, _winner_index, _remaining) = futures::future::select_all(handles).await;
        let duration = start.elapsed();

        match result {
            Ok((idx, Ok(value))) => RaceResult {
                winner_index: idx,
                result: value,
                duration,
            },
            Ok((_idx, Err(e))) => panic!("Race winner failed: {}", e),
            Err(e) => panic!("Race task panicked: {}", e),
        }
    }

    pub async fn race_against_target<F, T>(
        &self,
        _target: String,
        strategies: Vec<F>,
    ) -> RaceResult<T>
    where
        F: std::future::Future<Output = Result<T, String>> + Send + 'static,
        T: Send + 'static,
    {
        self.race(strategies).await
    }

    pub async fn batch_execute<F, T>(
        &self,
        operations: Vec<F>,
        _batch_size: usize,
    ) -> Vec<ParallelResult<T>>
    where
        F: std::future::Future<Output = Result<T, String>> + Send + 'static,
        T: Send + 'static,
    {
        let mut handles = Vec::new();

        for (idx, operation) in operations.into_iter().enumerate() {
            let semaphore = Arc::clone(&self.semaphore);

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let start = std::time::Instant::now();
                let result = operation.await;
                let duration = start.elapsed();

                ParallelResult {
                    index: idx,
                    result,
                    duration,
                }
            });

            handles.push(handle);
        }

        let results = join_all(handles).await;
        results.into_iter().filter_map(|r| r.ok()).collect()
    }
}

pub struct ConcurrentStrategies<T> {
    strategies: Vec<Strategy<T>>,
    timeout: Option<std::time::Duration>,
}

pub struct Strategy<T> {
    pub name: String,
    pub priority: u32,
    pub timeout: Option<std::time::Duration>,
    pub operation: Arc<
        dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send>>
            + Send
            + Sync,
    >,
}

impl<T: Send + 'static> ConcurrentStrategies<T> {
    pub fn new() -> Self {
        ConcurrentStrategies {
            strategies: Vec::new(),
            timeout: None,
        }
    }

    pub fn add_strategy<F>(mut self, name: String, priority: u32, operation: F) -> Self
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send>>
            + Send
            + Sync
            + 'static,
    {
        self.strategies.push(Strategy {
            name,
            priority,
            timeout: None,
            operation: Arc::new(operation),
        });
        self
    }

    pub fn with_global_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub async fn execute_all(&self) -> Vec<StrategyResult<T>> {
        let mut handles = Vec::new();

        for strategy in &self.strategies {
            let name = strategy.name.clone();
            let operation = Arc::clone(&strategy.operation);

            let handle = tokio::spawn(async move {
                let start = std::time::Instant::now();
                let result = operation().await;
                let duration = start.elapsed();

                StrategyResult {
                    name,
                    result,
                    duration,
                }
            });

            handles.push(handle);
        }

        let results = join_all(handles).await;
        results.into_iter().filter_map(|r| r.ok()).collect()
    }

    pub async fn execute_until_success(&self) -> Option<StrategyResult<T>> {
        let mut strategy_refs: Vec<(
            String,
            u32,
            Arc<
                dyn Fn() -> std::pin::Pin<
                        Box<dyn std::future::Future<Output = Result<T, String>> + Send>,
                    > + Send
                    + Sync,
            >,
        )> = self
            .strategies
            .iter()
            .map(|s| (s.name.clone(), s.priority, Arc::clone(&s.operation)))
            .collect();
        strategy_refs.sort_by(|a, b| b.1.cmp(&a.1));

        for (name, _priority, operation) in strategy_refs {
            let start = std::time::Instant::now();
            let result = operation().await;
            let duration = start.elapsed();

            if result.is_ok() {
                return Some(StrategyResult {
                    name,
                    result,
                    duration,
                });
            }
        }

        None
    }

    pub async fn race_strategies(&self) -> Option<StrategyResult<T>> {
        let mut handles = Vec::new();

        for strategy in &self.strategies {
            let name = strategy.name.clone();
            let operation = Arc::clone(&strategy.operation);

            let handle = tokio::spawn(async move {
                let start = std::time::Instant::now();
                let result = operation().await;
                let duration = start.elapsed();

                StrategyResult {
                    name,
                    result,
                    duration,
                }
            });

            handles.push(handle);
        }

        let (result, _index, _remaining) = futures::future::select_all(handles).await;
        result
            .ok()
            .and_then(|r| if r.result.is_ok() { Some(r) } else { None })
    }
}

#[derive(Debug, Clone)]
pub struct StrategyResult<T> {
    pub name: String,
    pub result: Result<T, String>,
    pub duration: std::time::Duration,
}

pub async fn parallel_attack<F, T>(targets: Vec<String>, attack_fn: F) -> Vec<AttackResult<T>>
where
    F: Fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send>>
        + Send
        + Sync
        + 'static,
    T: Send + 'static,
{
    let executor = ParallelExecutor::new(10);
    let results = executor.parallel_for(targets, attack_fn).await;

    results
        .into_iter()
        .map(|pr| AttackResult {
            target_index: pr.index,
            result: pr.result,
            duration: pr.duration,
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct AttackResult<T> {
    pub target_index: usize,
    pub result: Result<T, String>,
    pub duration: std::time::Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_for() {
        let executor = ParallelExecutor::new(4);
        let items = vec![1, 2, 3, 4, 5];

        let results = executor
            .parallel_for(items, |item| {
                Box::pin(async move { Ok::<_, String>(item * 2) })
            })
            .await;

        assert_eq!(results.len(), 5);
        assert!(results.iter().all(|r| r.result.is_ok()));
    }

    #[tokio::test]
    async fn test_race() {
        use std::future::Future;
        use std::pin::Pin;

        let executor = ParallelExecutor::new(4);

        let operations: Vec<Pin<Box<dyn Future<Output = Result<i32, String>> + Send>>> = vec![
            Box::pin(async {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                Ok::<_, String>(1)
            }),
            Box::pin(async { Ok::<_, String>(2) }),
        ];

        let result = executor.race(operations).await;
        assert_eq!(result.result, 2);
    }

    #[tokio::test]
    async fn test_concurrent_strategies() {
        let strategies = ConcurrentStrategies::new()
            .add_strategy("slow".to_string(), 1, || {
                Box::pin(async {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    Ok::<_, String>(1)
                })
            })
            .add_strategy("fast".to_string(), 2, || {
                Box::pin(async { Ok::<_, String>(2) })
            });

        let result = strategies.race_strategies().await;
        assert!(result.is_some());
    }
}
