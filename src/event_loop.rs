use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    MemoryWrite,
    MemoryRead,
    ProcessEvent,
    RegisterModify,
    FunctionCall,
    Breakpoint,
    Signal,
    Exception,
    NetworkSend,
    NetworkReceive,
    StateChange,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: u64,
    pub event_type: EventType,
    pub timestamp: std::time::Instant,
    pub session_id: Option<u64>,
    pub data: EventData,
}

#[derive(Debug, Clone)]
pub enum EventData {
    MemoryAccess {
        address: u64,
        size: usize,
        data: Vec<u8>,
    },
    ProcessChange {
        pid: u32,
        event_name: String,
    },
    RegisterChange {
        register: String,
        old_value: u64,
        new_value: u64,
    },
    FunctionInvocation {
        name: String,
        address: u64,
        args: Vec<String>,
    },
    BreakpointHit {
        address: u64,
        condition: Option<String>,
    },
    SignalReceived {
        signal_number: i32,
        signal_name: String,
    },
    ExceptionRaised {
        exception_code: u32,
        address: u64,
        description: String,
    },
    NetworkData {
        direction: NetworkDirection,
        data: Vec<u8>,
        size: usize,
    },
    StateTransition {
        from: String,
        to: String,
        reason: Option<String>,
    },
    CustomData {
        key: String,
        value: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkDirection {
    Send,
    Receive,
}

pub type HandlerFn = Arc<dyn Fn(Event) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

pub struct EventHandler {
    pub id: u64,
    pub event_type: EventType,
    pub filter: Option<EventFilter>,
    pub handler: HandlerFn,
    pub priority: i32,
}

pub enum EventFilter {
    Address(u64),
    AddressRange(u64, u64),
    RegisterName(String),
    FunctionName(String),
    SignalNumber(i32),
    ExceptionCode(u32),
    Custom(Box<dyn Fn(&Event) -> bool + Send + Sync>),
}

pub struct EventLoop {
    running: Arc<RwLock<bool>>,
    handlers: Arc<RwLock<HashMap<EventType, Vec<EventHandler>>>>,
    event_sender: broadcast::Sender<Event>,
    event_receiver: Arc<RwLock<broadcast::Receiver<Event>>>,
    next_event_id: Arc<RwLock<u64>>,
    next_handler_id: Arc<RwLock<u64>>,
    event_queue: Arc<RwLock<Vec<Event>>>,
}

impl EventLoop {
    pub fn new(channel_capacity: usize) -> Self {
        let (tx, rx) = broadcast::channel(channel_capacity);

        EventLoop {
            running: Arc::new(RwLock::new(false)),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            event_sender: tx,
            event_receiver: Arc::new(RwLock::new(rx)),
            next_event_id: Arc::new(RwLock::new(1)),
            next_handler_id: Arc::new(RwLock::new(1)),
            event_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn register_handler<F, Fut>(&self, event_type: EventType, handler: F) -> u64
    where
        F: Fn(Event) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.register_handler_with_priority(event_type, handler, 0)
            .await
    }

    pub async fn register_handler_with_priority<F, Fut>(
        &self,
        event_type: EventType,
        handler: F,
        priority: i32,
    ) -> u64
    where
        F: Fn(Event) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let handler_id = {
            let mut next_id = self.next_handler_id.write().await;
            let id = *next_id;
            *next_id += 1;
            id
        };

        let handler_fn: HandlerFn = Arc::new(move |event| Box::pin(handler(event)));

        let event_handler = EventHandler {
            id: handler_id,
            event_type: event_type.clone(),
            filter: None,
            handler: handler_fn,
            priority,
        };

        let mut handlers = self.handlers.write().await;
        handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(event_handler);

        self.sort_handlers_by_priority(&mut handlers).await;

        handler_id
    }

    pub async fn register_handler_with_filter<F, Fut>(
        &self,
        event_type: EventType,
        filter: EventFilter,
        handler: F,
        priority: i32,
    ) -> u64
    where
        F: Fn(Event) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let handler_id = {
            let mut next_id = self.next_handler_id.write().await;
            let id = *next_id;
            *next_id += 1;
            id
        };

        let handler_fn: HandlerFn = Arc::new(move |event| Box::pin(handler(event)));

        let event_handler = EventHandler {
            id: handler_id,
            event_type: event_type.clone(),
            filter: Some(filter),
            handler: handler_fn,
            priority,
        };

        let mut handlers = self.handlers.write().await;
        handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(event_handler);

        self.sort_handlers_by_priority(&mut handlers).await;

        handler_id
    }

    async fn sort_handlers_by_priority(
        &self,
        handlers: &mut HashMap<EventType, Vec<EventHandler>>,
    ) {
        for handler_list in handlers.values_mut() {
            handler_list.sort_by(|a, b| b.priority.cmp(&a.priority));
        }
    }

    pub async fn unregister_handler(&self, handler_id: u64) -> bool {
        let mut handlers = self.handlers.write().await;

        for handler_list in handlers.values_mut() {
            if let Some(pos) = handler_list.iter().position(|h| h.id == handler_id) {
                handler_list.remove(pos);
                return true;
            }
        }

        false
    }

    pub async fn emit(&self, event_type: EventType, data: EventData) -> Result<(), String> {
        self.emit_with_session(event_type, data, None).await
    }

    pub async fn emit_with_session(
        &self,
        event_type: EventType,
        data: EventData,
        session_id: Option<u64>,
    ) -> Result<(), String> {
        let event_id = {
            let mut next_id = self.next_event_id.write().await;
            let id = *next_id;
            *next_id += 1;
            id
        };

        let event = Event {
            id: event_id,
            event_type,
            timestamp: std::time::Instant::now(),
            session_id,
            data,
        };

        self.event_sender
            .send(event)
            .map_err(|e| format!("Failed to emit event: {}", e))?;

        Ok(())
    }

    pub async fn start(&self) -> Result<(), String> {
        let mut running = self.running.write().await;
        if *running {
            return Err("Event loop is already running".to_string());
        }
        *running = true;
        drop(running);

        let handlers = Arc::clone(&self.handlers);
        let running = Arc::clone(&self.running);
        let mut rx = self.event_sender.subscribe();

        tokio::spawn(async move {
            while *running.read().await {
                match rx.recv().await {
                    Ok(event) => {
                        Self::dispatch_event(&handlers, event).await;
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        eprintln!("[WARNING] Event loop lagged, skipped {} events", skipped);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn dispatch_event(
        handlers: &Arc<RwLock<HashMap<EventType, Vec<EventHandler>>>>,
        event: Event,
    ) {
        let handlers_guard = handlers.read().await;

        if let Some(handler_list) = handlers_guard.get(&event.event_type) {
            for handler in handler_list {
                if Self::matches_filter(&handler.filter, &event) {
                    let handler_fn = Arc::clone(&handler.handler);
                    let event_clone = event.clone();

                    tokio::spawn(async move {
                        handler_fn(event_clone).await;
                    });
                }
            }
        }
    }

    fn matches_filter(filter: &Option<EventFilter>, event: &Event) -> bool {
        match filter {
            None => true,
            Some(EventFilter::Address(addr)) => match &event.data {
                EventData::MemoryAccess { address, .. } => address == addr,
                EventData::BreakpointHit { address, .. } => address == addr,
                EventData::FunctionInvocation { address, .. } => address == addr,
                EventData::ExceptionRaised { address, .. } => address == addr,
                _ => false,
            },
            Some(EventFilter::AddressRange(start, end)) => match &event.data {
                EventData::MemoryAccess { address, .. } => address >= start && address <= end,
                EventData::BreakpointHit { address, .. } => address >= start && address <= end,
                EventData::FunctionInvocation { address, .. } => address >= start && address <= end,
                EventData::ExceptionRaised { address, .. } => address >= start && address <= end,
                _ => false,
            },
            Some(EventFilter::RegisterName(name)) => match &event.data {
                EventData::RegisterChange { register, .. } => register == name,
                _ => false,
            },
            Some(EventFilter::FunctionName(name)) => match &event.data {
                EventData::FunctionInvocation { name: fn_name, .. } => fn_name == name,
                _ => false,
            },
            Some(EventFilter::SignalNumber(num)) => match &event.data {
                EventData::SignalReceived { signal_number, .. } => signal_number == num,
                _ => false,
            },
            Some(EventFilter::ExceptionCode(code)) => match &event.data {
                EventData::ExceptionRaised { exception_code, .. } => exception_code == code,
                _ => false,
            },
            Some(EventFilter::Custom(predicate)) => predicate(event),
        }
    }

    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    pub async fn wait_for_event(
        &self,
        event_type: EventType,
        timeout: Option<std::time::Duration>,
    ) -> Option<Event> {
        let mut rx = self.event_sender.subscribe();

        let result = if let Some(duration) = timeout {
            tokio::time::timeout(duration, async {
                loop {
                    match rx.recv().await {
                        Ok(event) if event.event_type == event_type => return Some(event),
                        Ok(_) => continue,
                        Err(_) => return None,
                    }
                }
            })
            .await
        } else {
            Ok(async {
                loop {
                    match rx.recv().await {
                        Ok(event) if event.event_type == event_type => return Some(event),
                        Ok(_) => continue,
                        Err(_) => return None,
                    }
                }
            }
            .await)
        };

        result.ok().flatten()
    }

    pub async fn get_handler_count(&self) -> usize {
        let handlers = self.handlers.read().await;
        handlers.values().map(|v| v.len()).sum()
    }
}

pub struct EventLoopBuilder {
    channel_capacity: usize,
}

impl EventLoopBuilder {
    pub fn new() -> Self {
        EventLoopBuilder {
            channel_capacity: 1024,
        }
    }

    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.channel_capacity = capacity;
        self
    }

    pub fn build(self) -> EventLoop {
        EventLoop::new(self.channel_capacity)
    }
}

impl Default for EventLoopBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_loop_basic() {
        let event_loop = EventLoop::new(100);

        let received = Arc::new(RwLock::new(Vec::new()));
        let received_clone = Arc::clone(&received);

        event_loop
            .register_handler(EventType::MemoryWrite, move |event| {
                let received = Arc::clone(&received_clone);
                async move {
                    let mut r = received.write().await;
                    r.push(event.id);
                }
            })
            .await;

        event_loop.start().await.unwrap();

        event_loop
            .emit(
                EventType::MemoryWrite,
                EventData::MemoryAccess {
                    address: 0x1000,
                    size: 4,
                    data: vec![0x41, 0x42, 0x43, 0x44],
                },
            )
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let r = received.read().await;
        assert_eq!(r.len(), 1);

        event_loop.stop().await;
    }

    #[tokio::test]
    async fn test_event_filter() {
        let event_loop = EventLoop::new(100);

        let received = Arc::new(RwLock::new(Vec::new()));
        let received_clone = Arc::clone(&received);

        event_loop
            .register_handler_with_filter(
                EventType::MemoryWrite,
                EventFilter::Address(0x1000),
                move |event| {
                    let received = Arc::clone(&received_clone);
                    async move {
                        let mut r = received.write().await;
                        r.push(event.id);
                    }
                },
                0,
            )
            .await;

        event_loop.start().await.unwrap();

        event_loop
            .emit(
                EventType::MemoryWrite,
                EventData::MemoryAccess {
                    address: 0x1000,
                    size: 4,
                    data: vec![0x41, 0x42, 0x43, 0x44],
                },
            )
            .await
            .unwrap();

        event_loop
            .emit(
                EventType::MemoryWrite,
                EventData::MemoryAccess {
                    address: 0x2000,
                    size: 4,
                    data: vec![0x41, 0x42, 0x43, 0x44],
                },
            )
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let r = received.read().await;
        assert_eq!(r.len(), 1);

        event_loop.stop().await;
    }

    #[tokio::test]
    async fn test_handler_priority() {
        let event_loop = EventLoop::new(100);

        let order = Arc::new(RwLock::new(Vec::new()));
        let order_clone1 = Arc::clone(&order);
        let order_clone2 = Arc::clone(&order);

        event_loop
            .register_handler_with_priority(
                EventType::MemoryWrite,
                move |_event| {
                    let order = Arc::clone(&order_clone1);
                    async move {
                        let mut o = order.write().await;
                        o.push(1);
                    }
                },
                1,
            )
            .await;

        event_loop
            .register_handler_with_priority(
                EventType::MemoryWrite,
                move |_event| {
                    let order = Arc::clone(&order_clone2);
                    async move {
                        let mut o = order.write().await;
                        o.push(2);
                    }
                },
                10,
            )
            .await;

        event_loop.start().await.unwrap();

        event_loop
            .emit(
                EventType::MemoryWrite,
                EventData::MemoryAccess {
                    address: 0x1000,
                    size: 4,
                    data: vec![0x41, 0x42, 0x43, 0x44],
                },
            )
            .await
            .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let o = order.read().await;
        assert_eq!(o.len(), 2);

        event_loop.stop().await;
    }
}
