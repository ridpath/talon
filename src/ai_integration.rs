// AI Integration Layer - Manages AI features across TALON with caching and token budgets
// Provides optional AI assistance throughout the framework with graceful fallback

use crate::ml_oracle::{AiConfig, MlOracle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex as TokioMutex;
use std::sync::Mutex as StdMutex;

#[derive(Clone)]
pub struct AiIntegration {
    oracle: Arc<TokioMutex<MlOracle>>,
    cache: Arc<StdMutex<AiCache>>,
    token_budget: Arc<StdMutex<TokenBudget>>,
    config: AiConfig,
    enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedResponse {
    response: String,
    timestamp: u64,
    tokens_used: usize,
}

struct AiCache {
    entries: HashMap<String, CachedResponse>,
    max_entries: usize,
    ttl_seconds: u64,
}

impl AiCache {
    fn new(max_entries: usize, ttl_seconds: u64) -> Self {
        AiCache {
            entries: HashMap::new(),
            max_entries,
            ttl_seconds,
        }
    }

    fn get(&mut self, key: &str) -> Option<String> {
        if let Some(cached) = self.entries.get(key) {
            let now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if now - cached.timestamp < self.ttl_seconds {
                return Some(cached.response.clone());
            } else {
                self.entries.remove(key);
            }
        }
        None
    }

    fn set(&mut self, key: String, response: String, tokens_used: usize) {
        if self.entries.len() >= self.max_entries {
            if let Some(oldest_key) = self.entries.keys().next().cloned() {
                self.entries.remove(&oldest_key);
            }
        }

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.entries.insert(key, CachedResponse {
            response,
            timestamp,
            tokens_used,
        });
    }

    fn clear(&mut self) {
        self.entries.clear();
    }

    fn stats(&self) -> (usize, usize) {
        (self.entries.len(), self.max_entries)
    }
}

struct TokenBudget {
    tokens_used: usize,
    max_tokens_per_hour: usize,
    window_start: SystemTime,
}

impl TokenBudget {
    fn new(max_tokens_per_hour: usize) -> Self {
        TokenBudget {
            tokens_used: 0,
            max_tokens_per_hour,
            window_start: SystemTime::now(),
        }
    }

    fn can_use(&mut self, tokens: usize) -> bool {
        self.reset_if_expired();
        
        if self.tokens_used + tokens <= self.max_tokens_per_hour {
            self.tokens_used += tokens;
            true
        } else {
            false
        }
    }

    fn reset_if_expired(&mut self) {
        let elapsed = SystemTime::now()
            .duration_since(self.window_start)
            .unwrap_or(Duration::from_secs(0));
        
        if elapsed >= Duration::from_secs(3600) {
            self.tokens_used = 0;
            self.window_start = SystemTime::now();
        }
    }

    fn remaining(&mut self) -> usize {
        self.reset_if_expired();
        self.max_tokens_per_hour.saturating_sub(self.tokens_used)
    }

    fn stats(&mut self) -> (usize, usize, u64) {
        self.reset_if_expired();
        let remaining_seconds = 3600u64.saturating_sub(
            SystemTime::now()
                .duration_since(self.window_start)
                .unwrap_or(Duration::from_secs(0))
                .as_secs()
        );
        (self.tokens_used, self.max_tokens_per_hour, remaining_seconds)
    }
}

impl AiIntegration {
    pub fn new(enabled: bool) -> Self {
        let config = AiConfig::load();
        let oracle = Arc::new(TokioMutex::new(MlOracle::new()));
        
        let cache = Arc::new(StdMutex::new(AiCache::new(1000, 3600)));
        
        let token_budget = Arc::new(StdMutex::new(TokenBudget::new(100000)));

        AiIntegration {
            oracle,
            cache,
            token_budget,
            config,
            enabled,
        }
    }

    pub async fn initialize(&self) -> Result<(), String> {
        if !self.enabled {
            return Err("AI features disabled via --no-ai flag".to_string());
        }

        let mut oracle = self.oracle.lock().await;
        oracle.initialize().await
    }

    pub fn is_available(&self) -> bool {
        self.enabled
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub async fn explain_error(&self, error_msg: &str, context: &str) -> Result<String, String> {
        if !self.enabled {
            return Err("AI features not available".to_string());
        }

        let cache_key = format!("error:{}", error_msg);
        
        if let Some(cached) = self.cache.lock().unwrap().get(&cache_key) {
            return Ok(cached);
        }

        let estimated_tokens = 300;
        if !self.token_budget.lock().unwrap().can_use(estimated_tokens) {
            return Err("Token budget exceeded, try again later".to_string());
        }

        let oracle = self.oracle.lock().await;
        match oracle.explain_error(error_msg, context).await {
            Ok(response) => {
                self.cache.lock().unwrap().set(cache_key, response.clone(), estimated_tokens);
                Ok(response)
            }
            Err(e) => Err(format!("AI error explanation failed: {}", e))
        }
    }

    pub async fn suggest_rop_chain(&self, binary_arch: &str, objective: &str, available_gadgets: &[String]) -> Result<String, String> {
        if !self.enabled {
            return Err("AI features not available".to_string());
        }

        let cache_key = format!("rop:{}:{}", binary_arch, objective);
        
        if let Some(cached) = self.cache.lock().unwrap().get(&cache_key) {
            return Ok(cached);
        }

        let estimated_tokens = 500;
        if !self.token_budget.lock().unwrap().can_use(estimated_tokens) {
            return Err("Token budget exceeded, try again later".to_string());
        }

        let oracle = self.oracle.lock().await;
        match oracle.suggest_gadgets(binary_arch, objective, available_gadgets).await {
            Ok(response) => {
                self.cache.lock().unwrap().set(cache_key, response.clone(), estimated_tokens);
                Ok(response)
            }
            Err(e) => Err(format!("AI ROP suggestion failed: {}", e))
        }
    }

    pub async fn optimize_shellcode(&self, shellcode_desc: &str, constraints: &str) -> Result<String, String> {
        if !self.enabled {
            return Err("AI features not available".to_string());
        }

        let cache_key = format!("shellcode:{}:{}", shellcode_desc, constraints);
        
        if let Some(cached) = self.cache.lock().unwrap().get(&cache_key) {
            return Ok(cached);
        }

        let estimated_tokens = 400;
        if !self.token_budget.lock().unwrap().can_use(estimated_tokens) {
            return Err("Token budget exceeded, try again later".to_string());
        }

        let code_with_context = format!(
            "Shellcode optimization request:\nDescription: {}\nConstraints: {}\n\nProvide optimization strategy and alternative approaches.",
            shellcode_desc, constraints
        );

        let oracle = self.oracle.lock().await;
        match oracle.suggest_code_improvements(&code_with_context).await {
            Ok(response) => {
                self.cache.lock().unwrap().set(cache_key, response.clone(), estimated_tokens);
                Ok(response)
            }
            Err(e) => Err(format!("AI shellcode optimization failed: {}", e))
        }
    }

    pub async fn review_exploit(&self, script_content: &str) -> Result<String, String> {
        if !self.enabled {
            return Err("AI features not available".to_string());
        }

        let estimated_tokens = 800;
        if !self.token_budget.lock().unwrap().can_use(estimated_tokens) {
            return Err("Token budget exceeded, try again later".to_string());
        }

        let oracle = self.oracle.lock().await;
        match oracle.suggest_code_improvements(script_content).await {
            Ok(response) => Ok(response),
            Err(e) => Err(format!("AI exploit review failed: {}", e))
        }
    }

    pub async fn fix_script(&self, script_content: &str, error_msg: &str) -> Result<String, String> {
        if !self.enabled {
            return Err("AI features not available".to_string());
        }

        let estimated_tokens = 600;
        if !self.token_budget.lock().unwrap().can_use(estimated_tokens) {
            return Err("Token budget exceeded, try again later".to_string());
        }

        let code_with_error = format!(
            "Fix this TALON exploitation script:\n\n```talon\n{}\n```\n\nError: {}\n\nProvide the corrected script with explanations.",
            script_content, error_msg
        );

        let oracle = self.oracle.lock().await;
        match oracle.suggest_code_improvements(&code_with_error).await {
            Ok(response) => Ok(response),
            Err(e) => Err(format!("AI script fix failed: {}", e))
        }
    }

    pub async fn generate_documentation(&self, script_content: &str) -> Result<String, String> {
        if !self.enabled {
            return Err("AI features not available".to_string());
        }

        let estimated_tokens = 700;
        if !self.token_budget.lock().unwrap().can_use(estimated_tokens) {
            return Err("Token budget exceeded, try again later".to_string());
        }

        let code_with_request = format!(
            "Generate inline documentation for this TALON exploitation script:\n\n```talon\n{}\n```\n\nAdd comments explaining techniques and logic.",
            script_content
        );

        let oracle = self.oracle.lock().await;
        match oracle.suggest_code_improvements(&code_with_request).await {
            Ok(response) => Ok(response),
            Err(e) => Err(format!("AI documentation generation failed: {}", e))
        }
    }

    pub async fn tutorial_hint(&self, level: &str, user_progress: &str) -> Result<String, String> {
        if !self.enabled {
            return Err("AI features not available".to_string());
        }

        let cache_key = format!("tutorial:{}:{}", level, user_progress);
        
        if let Some(cached) = self.cache.lock().unwrap().get(&cache_key) {
            return Ok(cached);
        }

        let estimated_tokens = 250;
        if !self.token_budget.lock().unwrap().can_use(estimated_tokens) {
            return Err("Token budget exceeded, try again later".to_string());
        }

        let hint_request = format!(
            "Tutorial context: Level {}\nCurrent progress: {}\n\nProvide a helpful hint without revealing the full solution.",
            level, user_progress
        );

        let oracle = self.oracle.lock().await;
        match oracle.explain_error("Stuck on tutorial", &hint_request).await {
            Ok(response) => {
                self.cache.lock().unwrap().set(cache_key, response.clone(), estimated_tokens);
                Ok(response)
            }
            Err(e) => Err(format!("AI tutorial hint failed: {}", e))
        }
    }

    pub async fn general_help(&self, query: &str) -> Result<String, String> {
        if !self.enabled {
            return Err("AI features not available".to_string());
        }

        let cache_key = format!("help:{}", query);
        
        if let Some(cached) = self.cache.lock().unwrap().get(&cache_key) {
            return Ok(cached);
        }

        let estimated_tokens = 350;
        if !self.token_budget.lock().unwrap().can_use(estimated_tokens) {
            return Err("Token budget exceeded, try again later".to_string());
        }

        let help_context = format!(
            "Question about TALON exploitation framework:\n\n{}\n\nProvide a technical answer with examples where appropriate.",
            query
        );

        let oracle = self.oracle.lock().await;
        match oracle.suggest_code_improvements(&help_context).await {
            Ok(response) => {
                self.cache.lock().unwrap().set(cache_key, response.clone(), estimated_tokens);
                Ok(response)
            }
            Err(e) => Err(format!("AI help failed: {}", e))
        }
    }

    pub fn cache_stats(&self) -> (usize, usize) {
        self.cache.lock().unwrap().stats()
    }

    pub fn token_stats(&self) -> (usize, usize, u64) {
        self.token_budget.lock().unwrap().stats()
    }

    pub fn clear_cache(&self) {
        self.cache.lock().unwrap().clear();
    }
}

impl Default for AiIntegration {
    fn default() -> Self {
        Self::new(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_integration_creation() {
        let ai = AiIntegration::new(true);
        assert!(ai.is_available());
    }

    #[test]
    fn test_ai_integration_disabled() {
        let mut ai = AiIntegration::new(true);
        ai.disable();
        assert!(!ai.is_available());
    }

    #[test]
    fn test_cache_basic() {
        let mut cache = AiCache::new(10, 3600);
        cache.set("test".to_string(), "response".to_string(), 100);
        assert_eq!(cache.get("test"), Some("response".to_string()));
    }

    #[test]
    fn test_cache_expiry() {
        let mut cache = AiCache::new(10, 0);
        cache.set("test".to_string(), "response".to_string(), 100);
        std::thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(cache.get("test"), None);
    }

    #[test]
    fn test_cache_max_entries() {
        let mut cache = AiCache::new(2, 3600);
        cache.set("test1".to_string(), "response1".to_string(), 100);
        cache.set("test2".to_string(), "response2".to_string(), 100);
        cache.set("test3".to_string(), "response3".to_string(), 100);
        
        let (entries, max) = cache.stats();
        assert_eq!(entries, 2);
        assert_eq!(max, 2);
    }

    #[test]
    fn test_token_budget() {
        let mut budget = TokenBudget::new(1000);
        assert!(budget.can_use(500));
        assert!(budget.can_use(400));
        assert!(!budget.can_use(200));
    }

    #[test]
    fn test_token_budget_remaining() {
        let mut budget = TokenBudget::new(1000);
        budget.can_use(300);
        assert_eq!(budget.remaining(), 700);
    }

    #[test]
    fn test_cache_stats() {
        let ai = AiIntegration::new(true);
        let (entries, max) = ai.cache_stats();
        assert_eq!(entries, 0);
        assert_eq!(max, 1000);
    }

    #[test]
    fn test_token_stats() {
        let ai = AiIntegration::new(true);
        let (used, max, _remaining_seconds) = ai.token_stats();
        assert_eq!(used, 0);
        assert_eq!(max, 100000);
    }

    #[test]
    fn test_clear_cache() {
        let ai = AiIntegration::new(true);
        ai.clear_cache();
        let (entries, _max) = ai.cache_stats();
        assert_eq!(entries, 0);
    }
}
