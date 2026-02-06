use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::runtime::Runtime;
use std::sync::atomic::{AtomicU64, Ordering};
use dashmap::DashMap;
use crossbeam::queue::SegQueue;

// Mock connection types for benchmarking
#[derive(Clone)]
struct MockConnection {
    data: Vec<u8>,
}

// Old implementation (RwLock-based)
struct LockBasedRegistry {
    connections: HashMap<u64, MockConnection>,
    next_id: u64,
}

impl LockBasedRegistry {
    fn new() -> Self {
        LockBasedRegistry {
            connections: HashMap::new(),
            next_id: 1,
        }
    }

    fn add(&mut self, conn: MockConnection) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.connections.insert(id, conn);
        id
    }

    fn get_mut(&mut self, id: u64) -> Option<&mut MockConnection> {
        self.connections.get_mut(&id)
    }

    fn remove(&mut self, id: u64) {
        self.connections.remove(&id);
    }
}

// New atomic implementation (lock-free)
struct AtomicRegistry {
    connections: DashMap<u64, MockConnection>,
    next_id: AtomicU64,
    free_ids: SegQueue<u64>,
}

impl AtomicRegistry {
    fn new() -> Self {
        AtomicRegistry {
            connections: DashMap::new(),
            next_id: AtomicU64::new(1),
            free_ids: SegQueue::new(),
        }
    }

    fn allocate_id(&self) -> u64 {
        if let Some(id) = self.free_ids.pop() {
            return id;
        }
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    fn add(&self, conn: MockConnection) -> u64 {
        let id = self.allocate_id();
        self.connections.insert(id, conn);
        id
    }

    fn get_mut(&self, id: u64) -> Option<dashmap::mapref::one::RefMut<u64, MockConnection>> {
        self.connections.get_mut(&id)
    }

    fn remove(&self, id: u64) {
        if self.connections.remove(&id).is_some() {
            self.free_ids.push(id);
        }
    }
}

fn bench_sequential_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential_add");
    
    for size in [100, 1000, 10000].iter() {
        // Lock-based
        group.bench_with_input(BenchmarkId::new("lock_based", size), size, |b, &size| {
            b.iter(|| {
                let registry = Arc::new(Mutex::new(LockBasedRegistry::new()));
                for _ in 0..size {
                    let conn = MockConnection { data: vec![0; 1024] };
                    registry.lock().unwrap().add(conn);
                }
            });
        });

        // Atomic
        group.bench_with_input(BenchmarkId::new("atomic", size), size, |b, &size| {
            b.iter(|| {
                let registry = AtomicRegistry::new();
                for _ in 0..size {
                    let conn = MockConnection { data: vec![0; 1024] };
                    registry.add(conn);
                }
            });
        });
    }
    group.finish();
}

fn bench_concurrent_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_add");
    let rt = Runtime::new().unwrap();
    
    for num_threads in [4, 8, 16].iter() {
        // Lock-based
        group.bench_with_input(BenchmarkId::new("lock_based", num_threads), num_threads, |b, &num_threads| {
            b.iter(|| {
                let registry = Arc::new(Mutex::new(LockBasedRegistry::new()));
                rt.block_on(async {
                    let mut handles = vec![];
                    for _ in 0..num_threads {
                        let reg = registry.clone();
                        let handle = tokio::spawn(async move {
                            for _ in 0..100 {
                                let conn = MockConnection { data: vec![0; 1024] };
                                reg.lock().unwrap().add(conn);
                            }
                        });
                        handles.push(handle);
                    }
                    for handle in handles {
                        handle.await.unwrap();
                    }
                });
            });
        });

        // Atomic
        group.bench_with_input(BenchmarkId::new("atomic", num_threads), num_threads, |b, &num_threads| {
            b.iter(|| {
                let registry = Arc::new(AtomicRegistry::new());
                rt.block_on(async {
                    let mut handles = vec![];
                    for _ in 0..num_threads {
                        let reg = registry.clone();
                        let handle = tokio::spawn(async move {
                            for _ in 0..100 {
                                let conn = MockConnection { data: vec![0; 1024] };
                                reg.add(conn);
                            }
                        });
                        handles.push(handle);
                    }
                    for handle in handles {
                        handle.await.unwrap();
                    }
                });
            });
        });
    }
    group.finish();
}

fn bench_mixed_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_operations");
    let rt = Runtime::new().unwrap();
    
    for num_threads in [4, 8].iter() {
        // Lock-based
        group.bench_with_input(BenchmarkId::new("lock_based", num_threads), num_threads, |b, &num_threads| {
            b.iter(|| {
                let registry = Arc::new(Mutex::new(LockBasedRegistry::new()));
                rt.block_on(async {
                    // Pre-populate
                    for _ in 0..100 {
                        let conn = MockConnection { data: vec![0; 1024] };
                        registry.lock().unwrap().add(conn);
                    }
                    
                    let mut handles = vec![];
                    for _ in 0..num_threads {
                        let reg = registry.clone();
                        let handle = tokio::spawn(async move {
                            for i in 0..50 {
                                if i % 3 == 0 {
                                    // Add
                                    let conn = MockConnection { data: vec![0; 1024] };
                                    reg.lock().unwrap().add(conn);
                                } else if i % 3 == 1 {
                                    // Read
                                    let _ = reg.lock().unwrap().get_mut(i as u64 / 2);
                                } else {
                                    // Remove
                                    reg.lock().unwrap().remove(i as u64 / 2);
                                }
                            }
                        });
                        handles.push(handle);
                    }
                    for handle in handles {
                        handle.await.unwrap();
                    }
                });
            });
        });

        // Atomic
        group.bench_with_input(BenchmarkId::new("atomic", num_threads), num_threads, |b, &num_threads| {
            b.iter(|| {
                let registry = Arc::new(AtomicRegistry::new());
                rt.block_on(async {
                    // Pre-populate
                    for _ in 0..100 {
                        let conn = MockConnection { data: vec![0; 1024] };
                        registry.add(conn);
                    }
                    
                    let mut handles = vec![];
                    for _ in 0..num_threads {
                        let reg = registry.clone();
                        let handle = tokio::spawn(async move {
                            for i in 0..50 {
                                if i % 3 == 0 {
                                    // Add
                                    let conn = MockConnection { data: vec![0; 1024] };
                                    reg.add(conn);
                                } else if i % 3 == 1 {
                                    // Read
                                    let _ = reg.get_mut(i as u64 / 2);
                                } else {
                                    // Remove
                                    reg.remove(i as u64 / 2);
                                }
                            }
                        });
                        handles.push(handle);
                    }
                    for handle in handles {
                        handle.await.unwrap();
                    }
                });
            });
        });
    }
    group.finish();
}

fn bench_1000_concurrent_connections(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("1000_concurrent_lock_based", |b| {
        b.iter(|| {
            let registry = Arc::new(Mutex::new(LockBasedRegistry::new()));
            rt.block_on(async {
                let mut handles = vec![];
                for _ in 0..1000 {
                    let reg = registry.clone();
                    let handle = tokio::spawn(async move {
                        let conn = MockConnection { data: vec![0; 1024] };
                        let id = reg.lock().unwrap().add(conn);
                        let _ = reg.lock().unwrap().get_mut(id);
                    });
                    handles.push(handle);
                }
                for handle in handles {
                    handle.await.unwrap();
                }
            });
        });
    });

    c.bench_function("1000_concurrent_atomic", |b| {
        b.iter(|| {
            let registry = Arc::new(AtomicRegistry::new());
            rt.block_on(async {
                let mut handles = vec![];
                for _ in 0..1000 {
                    let reg = registry.clone();
                    let handle = tokio::spawn(async move {
                        let conn = MockConnection { data: vec![0; 1024] };
                        let id = reg.add(conn);
                        let _ = reg.get_mut(id);
                    });
                    handles.push(handle);
                }
                for handle in handles {
                    handle.await.unwrap();
                }
            });
        });
    });
}

criterion_group!(
    benches,
    bench_sequential_add,
    bench_concurrent_add,
    bench_mixed_operations,
    bench_1000_concurrent_connections
);
criterion_main!(benches);
