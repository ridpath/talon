use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub type ObservableId = u64;
pub type SubscriberId = u64;

#[derive(Clone)]
pub struct Observable<T: Clone + Send + Sync> {
    id: ObservableId,
    value: Arc<RwLock<T>>,
    subscribers: Arc<RwLock<Vec<Subscriber<T>>>>,
    dependencies: Arc<RwLock<Vec<ObservableId>>>,
    manager: Arc<ObservableManager>,
}

pub struct Subscriber<T> {
    id: SubscriberId,
    callback: Arc<dyn Fn(&T) -> () + Send + Sync>,
}

pub struct ObservableManager {
    next_id: Arc<RwLock<ObservableId>>,
    next_subscriber_id: Arc<RwLock<SubscriberId>>,
    observables: Arc<RwLock<HashMap<ObservableId, Box<dyn std::any::Any + Send + Sync>>>>,
}

impl ObservableManager {
    pub fn new() -> Self {
        ObservableManager {
            next_id: Arc::new(RwLock::new(1)),
            next_subscriber_id: Arc::new(RwLock::new(1)),
            observables: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn allocate_id(&self) -> ObservableId {
        let mut next = self.next_id.write().await;
        let id = *next;
        *next += 1;
        id
    }

    async fn allocate_subscriber_id(&self) -> SubscriberId {
        let mut next = self.next_subscriber_id.write().await;
        let id = *next;
        *next += 1;
        id
    }

    pub async fn create<T: Clone + Send + Sync + 'static>(
        self: &Arc<Self>,
        initial_value: T,
    ) -> Observable<T> {
        let id = self.allocate_id().await;
        let obs = Observable {
            id,
            value: Arc::new(RwLock::new(initial_value)),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            dependencies: Arc::new(RwLock::new(Vec::new())),
            manager: Arc::clone(self),
        };

        let mut observables = self.observables.write().await;
        observables.insert(id, Box::new(obs.clone()));

        obs
    }
}

impl<T: Clone + Send + Sync + 'static> Observable<T> {
    pub async fn get(&self) -> T {
        let value = self.value.read().await;
        value.clone()
    }

    pub async fn set(&self, new_value: T) {
        {
            let mut value = self.value.write().await;
            *value = new_value.clone();
        }
        self.notify_subscribers().await;
    }

    pub async fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        {
            let mut value = self.value.write().await;
            updater(&mut *value);
        }
        self.notify_subscribers().await;
    }

    async fn notify_subscribers(&self) {
        let value = self.value.read().await;
        let subscribers = self.subscribers.read().await;
        
        for subscriber in subscribers.iter() {
            (subscriber.callback)(&*value);
        }
    }

    pub async fn subscribe<F>(&self, callback: F) -> SubscriberId
    where
        F: Fn(&T) -> () + Send + Sync + 'static,
    {
        let id = self.manager.allocate_subscriber_id().await;
        let subscriber = Subscriber {
            id,
            callback: Arc::new(callback),
        };

        let mut subscribers = self.subscribers.write().await;
        subscribers.push(subscriber);
        id
    }

    pub async fn unsubscribe(&self, subscriber_id: SubscriberId) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.retain(|s| s.id != subscriber_id);
    }

    pub async fn map<U, F>(&self, mapper: F) -> Observable<U>
    where
        U: Clone + Send + Sync + 'static,
        F: Fn(&T) -> U + Send + Sync + 'static,
    {
        let mapped_value = {
            let value = self.value.read().await;
            mapper(&*value)
        };

        let mapped = self.manager.create(mapped_value).await;

        {
            let mut deps = mapped.dependencies.write().await;
            deps.push(self.id);
        }

        let mapped_clone = mapped.clone();
        let mapper = Arc::new(mapper);
        
        self.subscribe(move |value| {
            let mapped_value = mapper(value);
            let mapped_clone2 = mapped_clone.clone();
            tokio::spawn(async move {
                mapped_clone2.set(mapped_value).await;
            });
        }).await;

        mapped
    }

    pub async fn filter<F>(&self, predicate: F) -> Observable<Option<T>>
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        let initial = {
            let value = self.value.read().await;
            if predicate(&*value) {
                Some(value.clone())
            } else {
                None
            }
        };

        let filtered = self.manager.create(initial).await;

        {
            let mut deps = filtered.dependencies.write().await;
            deps.push(self.id);
        }

        let filtered_clone = filtered.clone();
        let predicate = Arc::new(predicate);

        self.subscribe(move |value| {
            let result = if predicate(value) {
                Some(value.clone())
            } else {
                None
            };
            let filtered_clone2 = filtered_clone.clone();
            tokio::spawn(async move {
                filtered_clone2.set(result).await;
            });
        }).await;

        filtered
    }

    pub async fn combine<U, R, F>(
        &self,
        other: &Observable<U>,
        combiner: F,
    ) -> Observable<R>
    where
        U: Clone + Send + Sync + 'static,
        R: Clone + Send + Sync + 'static,
        F: Fn(&T, &U) -> R + Send + Sync + 'static,
    {
        let initial = {
            let val1 = self.value.read().await;
            let val2 = other.value.read().await;
            combiner(&*val1, &*val2)
        };

        let combined = self.manager.create(initial).await;

        {
            let mut deps = combined.dependencies.write().await;
            deps.push(self.id);
            deps.push(other.id);
        }

        let combiner = Arc::new(combiner);
        let other_clone = other.clone();
        let combined_clone = combined.clone();
        let combiner_clone = Arc::clone(&combiner);
        let self_clone_for_sub1 = self.clone();

        self.subscribe(move |_| {
            let other_clone2 = other_clone.clone();
            let combined_clone2 = combined_clone.clone();
            let combiner_clone2 = Arc::clone(&combiner_clone);
            let self_clone = self_clone_for_sub1.clone();
            
            tokio::spawn(async move {
                let val1 = self_clone.get().await;
                let val2 = other_clone2.get().await;
                let result = combiner_clone2(&val1, &val2);
                combined_clone2.set(result).await;
            });
        }).await;

        let combined_clone2 = combined.clone();
        let combiner_clone2 = Arc::clone(&combiner);
        let self_clone = self.clone();
        let other_clone_for_sub2 = other.clone();
        let other_clone_for_closure = other_clone_for_sub2.clone();

        other_clone_for_sub2.subscribe(move |_| {
            let self_clone2 = self_clone.clone();
            let combined_clone3 = combined_clone2.clone();
            let combiner_clone3 = Arc::clone(&combiner_clone2);
            let other_clone2 = other_clone_for_closure.clone();
            
            tokio::spawn(async move {
                let val1 = self_clone2.get().await;
                let val2 = other_clone2.get().await;
                let result = combiner_clone3(&val1, &val2);
                combined_clone3.set(result).await;
            });
        }).await;

        combined
    }

    pub async fn throttle(&self, duration: std::time::Duration) -> Observable<T> {
        let throttled = self.manager.create(self.get().await).await;

        {
            let mut deps = throttled.dependencies.write().await;
            deps.push(self.id);
        }

        let last_update = Arc::new(RwLock::new(std::time::Instant::now()));
        let throttled_clone = throttled.clone();

        self.subscribe(move |value| {
            let throttled_clone2 = throttled_clone.clone();
            let last_update_clone = Arc::clone(&last_update);
            let value_clone = value.clone();

            tokio::spawn(async move {
                let mut last = last_update_clone.write().await;
                let now = std::time::Instant::now();
                
                if now.duration_since(*last) >= duration {
                    *last = now;
                    drop(last);
                    throttled_clone2.set(value_clone).await;
                }
            });
        }).await;

        throttled
    }

    pub async fn debounce(&self, duration: std::time::Duration) -> Observable<T> {
        let debounced = self.manager.create(self.get().await).await;

        {
            let mut deps = debounced.dependencies.write().await;
            deps.push(self.id);
        }

        let pending = Arc::new(RwLock::new(None::<tokio::task::JoinHandle<()>>));
        let debounced_clone = debounced.clone();

        self.subscribe(move |value| {
            let debounced_clone2 = debounced_clone.clone();
            let pending_clone = Arc::clone(&pending);
            let value_clone = value.clone();

            tokio::spawn(async move {
                {
                    let mut pending_guard = pending_clone.write().await;
                    if let Some(handle) = pending_guard.take() {
                        handle.abort();
                    }

                    let debounced_clone3 = debounced_clone2.clone();
                    *pending_guard = Some(tokio::spawn(async move {
                        tokio::time::sleep(duration).await;
                        debounced_clone3.set(value_clone).await;
                    }));
                }
            });
        }).await;

        debounced
    }

    pub fn id(&self) -> ObservableId {
        self.id
    }

    pub async fn dependencies(&self) -> Vec<ObservableId> {
        let deps = self.dependencies.read().await;
        deps.clone()
    }
}

#[derive(Clone)]
pub struct ReactiveContext {
    manager: Arc<ObservableManager>,
    variables: Arc<RwLock<HashMap<String, ObservableId>>>,
}

impl ReactiveContext {
    pub fn new() -> Self {
        ReactiveContext {
            manager: Arc::new(ObservableManager::new()),
            variables: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_observable<T: Clone + Send + Sync + 'static>(
        &self,
        name: String,
        initial_value: T,
    ) -> Observable<T> {
        let obs = self.manager.create(initial_value).await;
        let mut vars = self.variables.write().await;
        vars.insert(name, obs.id);
        obs
    }

    pub async fn get_observable<T: Clone + Send + Sync + 'static>(
        &self,
        name: &str,
    ) -> Option<Observable<T>> {
        let vars = self.variables.read().await;
        let id = vars.get(name)?;
        
        let observables = self.manager.observables.read().await;
        let any_obs = observables.get(id)?;
        
        any_obs.downcast_ref::<Observable<T>>().cloned()
    }

    pub fn manager(&self) -> &Arc<ObservableManager> {
        &self.manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_observable_basic() {
        let ctx = ReactiveContext::new();
        let obs = ctx.create_observable("test".to_string(), 42).await;
        
        assert_eq!(obs.get().await, 42);
        
        obs.set(100).await;
        assert_eq!(obs.get().await, 100);
    }

    #[tokio::test]
    async fn test_observable_subscribe() {
        let ctx = ReactiveContext::new();
        let obs = ctx.create_observable("test".to_string(), 0).await;
        
        let received = Arc::new(RwLock::new(Vec::new()));
        let received_clone = Arc::clone(&received);
        
        obs.subscribe(move |value| {
            let received = Arc::clone(&received_clone);
            tokio::spawn(async move {
                let mut r = received.write().await;
                r.push(*value);
            });
        }).await;
        
        obs.set(1).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        obs.set(2).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        obs.set(3).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        let r = received.read().await;
        assert_eq!(*r, vec![1, 2, 3]);
    }

    #[tokio::test]
    async fn test_observable_map() {
        let ctx = ReactiveContext::new();
        let obs = ctx.create_observable("test".to_string(), 5).await;
        let mapped = obs.map(|x| x * 2).await;
        
        assert_eq!(mapped.get().await, 10);
        
        obs.set(10).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        assert_eq!(mapped.get().await, 20);
    }

    #[tokio::test]
    async fn test_observable_filter() {
        let ctx = ReactiveContext::new();
        let obs = ctx.create_observable("test".to_string(), 5).await;
        let filtered = obs.filter(|x| *x > 10).await;
        
        assert_eq!(filtered.get().await, None);
        
        obs.set(15).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        assert_eq!(filtered.get().await, Some(15));
    }

    #[tokio::test]
    async fn test_observable_combine() {
        let ctx = ReactiveContext::new();
        let obs1 = ctx.create_observable("a".to_string(), 5).await;
        let obs2 = ctx.create_observable("b".to_string(), 10).await;
        let combined = obs1.combine(&obs2, |a, b| a + b).await;
        
        assert_eq!(combined.get().await, 15);
        
        obs1.set(20).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        assert_eq!(combined.get().await, 30);
        
        obs2.set(5).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        assert_eq!(combined.get().await, 25);
    }
}
