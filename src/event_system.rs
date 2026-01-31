use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::ast::{Command, Expr};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    MemoryChange,
    RegisterModified,
    FunctionCalled,
    BreakpointHit,
    ConnectionEstablished,
    DataReceived,
    ExploitSuccess,
    ExploitFailure,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct EventHandler {
    pub event_type: EventType,
    pub condition: Option<Expr>,
    pub callback: Vec<Command>,
    pub enabled: bool,
}

pub struct EventSystem {
    handlers: Arc<RwLock<Vec<EventHandler>>>,
    register_watches: Arc<RwLock<HashMap<String, RegisterWatch>>>,
}

#[derive(Debug, Clone)]
pub struct RegisterWatch {
    pub register: String,
    pub min_value: Option<u64>,
    pub max_value: Option<u64>,
    pub callback: Vec<Command>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: EventType,
    pub timestamp: u64,
    pub data: HashMap<String, String>,
}

impl EventSystem {
    pub fn new() -> Self {
        EventSystem {
            handlers: Arc::new(RwLock::new(Vec::new())),
            register_watches: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_handler(&self, event_type: EventType, condition: Option<Expr>, callback: Vec<Command>) -> Result<(), String> {
        let handler = EventHandler {
            event_type,
            condition,
            callback,
            enabled: true,
        };

        self.handlers.write().await.push(handler);
        Ok(())
    }

    pub async fn register_watch(&self, register: String, range: Option<(u64, u64)>, callback: Vec<Command>) -> Result<(), String> {
        let (min_value, max_value) = if let Some((min, max)) = range {
            (Some(min), Some(max))
        } else {
            (None, None)
        };

        let watch = RegisterWatch {
            register: register.clone(),
            min_value,
            max_value,
            callback,
        };

        self.register_watches.write().await.insert(register, watch);
        Ok(())
    }

    pub async fn emit_event(&self, event: Event) -> Result<Vec<Vec<Command>>, String> {
        let mut triggered_callbacks = Vec::new();
        let handlers = self.handlers.read().await;

        for handler in handlers.iter() {
            if handler.enabled && self.matches_event_type(&handler.event_type, &event.event_type) {
                triggered_callbacks.push(handler.callback.clone());
            }
        }

        Ok(triggered_callbacks)
    }

    fn matches_event_type(&self, handler_type: &EventType, event_type: &EventType) -> bool {
        match (handler_type, event_type) {
            (EventType::Custom(h), EventType::Custom(e)) => h == e,
            _ => std::mem::discriminant(handler_type) == std::mem::discriminant(event_type)
        }
    }

    pub async fn check_register(&self, register: &str, value: u64) -> Result<Option<Vec<Command>>, String> {
        let watches = self.register_watches.read().await;
        
        if let Some(watch) = watches.get(register) {
            let in_range = match (watch.min_value, watch.max_value) {
                (Some(min), Some(max)) => value >= min && value <= max,
                (Some(min), None) => value >= min,
                (None, Some(max)) => value <= max,
                (None, None) => true,
            };

            if in_range {
                return Ok(Some(watch.callback.clone()));
            }
        }

        Ok(None)
    }

    pub async fn on_memory_change(&self, address: u64, old_value: &[u8], new_value: &[u8]) -> Result<Vec<Vec<Command>>, String> {
        let mut data = HashMap::new();
        data.insert("address".to_string(), format!("0x{:x}", address));
        data.insert("old_value".to_string(), hex::encode(old_value));
        data.insert("new_value".to_string(), hex::encode(new_value));

        let event = Event {
            event_type: EventType::MemoryChange,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            data,
        };

        self.emit_event(event).await
    }
}
