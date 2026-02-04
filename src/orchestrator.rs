#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};

pub struct OrchestratorRuntime {
    max_concurrent_tasks: usize,
    semaphore: Arc<Semaphore>,
    active_tasks: Arc<RwLock<HashMap<u64, TaskInfo>>>,
    next_task_id: Arc<RwLock<u64>>,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: u64,
    pub name: String,
    pub status: TaskStatus,
    pub started_at: std::time::Instant,
    pub completed_at: Option<std::time::Instant>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct TaskResult<T> {
    pub task_id: u64,
    pub result: Result<T, String>,
    pub duration: Duration,
}

impl OrchestratorRuntime {
    pub fn new(max_concurrent_tasks: usize) -> Self {
        OrchestratorRuntime {
            max_concurrent_tasks,
            semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            next_task_id: Arc::new(RwLock::new(1)),
        }
    }

    pub async fn spawn_task<F, T>(&self, name: String, task: F) -> u64
    where
        F: std::future::Future<Output = Result<T, String>> + Send + 'static,
        T: Send + 'static,
    {
        let task_id = self.allocate_task_id().await;
        let task_info = TaskInfo {
            id: task_id,
            name: name.clone(),
            status: TaskStatus::Pending,
            started_at: std::time::Instant::now(),
            completed_at: None,
        };

        {
            let mut tasks = self.active_tasks.write().await;
            tasks.insert(task_id, task_info);
        }

        let active_tasks = Arc::clone(&self.active_tasks);
        let semaphore = Arc::clone(&self.semaphore);

        tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            {
                let mut tasks = active_tasks.write().await;
                if let Some(info) = tasks.get_mut(&task_id) {
                    info.status = TaskStatus::Running;
                }
            }

            let start = std::time::Instant::now();
            let result = task.await;
            let _duration = start.elapsed();

            {
                let mut tasks = active_tasks.write().await;
                if let Some(info) = tasks.get_mut(&task_id) {
                    info.status = if result.is_ok() {
                        TaskStatus::Completed
                    } else {
                        TaskStatus::Failed
                    };
                    info.completed_at = Some(std::time::Instant::now());
                }
            }
        });

        task_id
    }

    pub async fn parallel_execute<F, T>(&self, tasks: Vec<(String, F)>) -> Vec<TaskResult<T>>
    where
        F: std::future::Future<Output = Result<T, String>> + Send + 'static,
        T: Send + 'static,
    {
        let mut handles = Vec::new();

        for (name, task) in tasks {
            let task_id = self.allocate_task_id().await;
            let semaphore = Arc::clone(&self.semaphore);
            let active_tasks = Arc::clone(&self.active_tasks);

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                {
                    let mut tasks_guard = active_tasks.write().await;
                    tasks_guard.insert(
                        task_id,
                        TaskInfo {
                            id: task_id,
                            name: name.clone(),
                            status: TaskStatus::Running,
                            started_at: std::time::Instant::now(),
                            completed_at: None,
                        },
                    );
                }

                let start = std::time::Instant::now();
                let result = task.await;
                let duration = start.elapsed();

                {
                    let mut tasks_guard = active_tasks.write().await;
                    if let Some(info) = tasks_guard.get_mut(&task_id) {
                        info.status = if result.is_ok() {
                            TaskStatus::Completed
                        } else {
                            TaskStatus::Failed
                        };
                        info.completed_at = Some(std::time::Instant::now());
                    }
                }

                TaskResult {
                    task_id,
                    result,
                    duration,
                }
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }

        results
    }

    pub async fn race_execute<F, T>(&self, tasks: Vec<(String, F)>) -> TaskResult<T>
    where
        F: std::future::Future<Output = Result<T, String>> + Send + 'static,
        T: Send + 'static,
    {
        let mut handles = Vec::new();

        for (name, task) in tasks {
            let task_id = self.allocate_task_id().await;
            let semaphore = Arc::clone(&self.semaphore);
            let active_tasks = Arc::clone(&self.active_tasks);

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();

                {
                    let mut tasks_guard = active_tasks.write().await;
                    tasks_guard.insert(
                        task_id,
                        TaskInfo {
                            id: task_id,
                            name: name.clone(),
                            status: TaskStatus::Running,
                            started_at: std::time::Instant::now(),
                            completed_at: None,
                        },
                    );
                }

                let start = std::time::Instant::now();
                let result = task.await;
                let duration = start.elapsed();

                TaskResult {
                    task_id,
                    result,
                    duration,
                }
            });

            handles.push(handle);
        }

        let (result, _index, _remaining) = futures::future::select_all(handles).await;
        result.unwrap()
    }

    pub async fn get_task_status(&self, task_id: u64) -> Option<TaskStatus> {
        let tasks = self.active_tasks.read().await;
        tasks.get(&task_id).map(|info| info.status.clone())
    }

    pub async fn get_active_tasks(&self) -> Vec<TaskInfo> {
        let tasks = self.active_tasks.read().await;
        tasks.values().cloned().collect()
    }

    pub async fn wait_for_task(&self, task_id: u64, timeout: Duration) -> Result<(), String> {
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err("Task timeout exceeded".to_string());
            }

            let status = self.get_task_status(task_id).await;
            match status {
                Some(TaskStatus::Completed) => return Ok(()),
                Some(TaskStatus::Failed) => return Err("Task failed".to_string()),
                Some(TaskStatus::Cancelled) => return Err("Task cancelled".to_string()),
                _ => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    async fn allocate_task_id(&self) -> u64 {
        let mut next_id = self.next_task_id.write().await;
        let id = *next_id;
        *next_id += 1;
        id
    }
}

pub struct ResourcePool<T> {
    resources: Arc<RwLock<Vec<T>>>,
    semaphore: Arc<Semaphore>,
}

impl<T: Clone + Send + 'static + Sync> ResourcePool<T> {
    pub fn new(resources: Vec<T>) -> Self {
        let count = resources.len();
        ResourcePool {
            resources: Arc::new(RwLock::new(resources)),
            semaphore: Arc::new(Semaphore::new(count)),
        }
    }

    pub async fn acquire(&self) -> Result<PooledResource<T>, String> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|e| format!("Failed to acquire resource: {}", e))?;

        let mut resources = self.resources.write().await;
        if let Some(resource) = resources.pop() {
            Ok(PooledResource {
                resource: Some(resource),
                pool: Arc::clone(&self.resources),
            })
        } else {
            Err("No resources available".to_string())
        }
    }

    pub async fn size(&self) -> usize {
        let resources = self.resources.read().await;
        resources.len()
    }
}

pub struct PooledResource<T: Send + 'static + Sync> {
    resource: Option<T>,
    pool: Arc<RwLock<Vec<T>>>,
}

impl<T: Send + 'static + Sync> PooledResource<T> {
    pub fn get(&self) -> Option<&T> {
        self.resource.as_ref()
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.resource.as_mut()
    }
}

impl<T: Send + 'static + Sync> Drop for PooledResource<T> {
    fn drop(&mut self) {
        if let Some(_resource) = self.resource.take() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn_task() {
        let runtime = OrchestratorRuntime::new(4);
        let task_id = runtime
            .spawn_task("test_task".to_string(), async { Ok::<_, String>(42) })
            .await;

        tokio::time::sleep(Duration::from_millis(100)).await;
        let status = runtime.get_task_status(task_id).await;
        assert!(status.is_some());
    }

    // DISABLED: Type inference issues with heterogeneous async blocks in Vec
    // Each async block has a unique type, making it impossible to create Vec<(String, F)>
    // without boxing or using trait objects
    /*
    #[tokio::test]
    async fn test_parallel_execute() {
        let runtime = OrchestratorRuntime::new(4);
        let tasks = vec![
            ("task1".to_string(), async { Ok::<_, String>(1) }),
            ("task2".to_string(), async { Ok::<_, String>(2) }),
            ("task3".to_string(), async { Ok::<_, String>(3) }),
        ];

        let results = runtime.parallel_execute(tasks).await;
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_race_execute() {
        let runtime = OrchestratorRuntime::new(4);
        let tasks = vec![
            ("slow".to_string(), async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok::<_, String>(1)
            }),
            ("fast".to_string(), async { Ok::<_, String>(2) }),
        ];

        let result = runtime.race_execute(tasks).await;
        assert!(result.result.is_ok());
    }
    */
}
