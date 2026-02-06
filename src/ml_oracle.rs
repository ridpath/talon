// ML Oracle - AI-assisted vulnerability analysis and exploit generation
// Supports dual-mode: LM Studio HTTP API or local GGUF models
// Graceful fallback: LM Studio → Local GGUF → Disable AI

use crate::oracle::VulnerabilityReport;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub backend: AiBackend,
    pub endpoint: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AiBackend {
    LmStudio,
    LocalGguf,
    Disabled,
}

impl Default for AiConfig {
    fn default() -> Self {
        AiConfig {
            backend: AiBackend::LmStudio,
            endpoint: "http://10.5.0.2:1234/v1".to_string(),
            model: "deepseek-coder-6.7b-instruct".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

impl AiConfig {
    pub fn load() -> Self {
        if let Ok(backend) = std::env::var("TALON_AI_BACKEND") {
            let mut config = AiConfig::default();
            config.backend = match backend.as_str() {
                "lm_studio" => AiBackend::LmStudio,
                "local_gguf" => AiBackend::LocalGguf,
                "disabled" => AiBackend::Disabled,
                _ => AiBackend::LmStudio,
            };
            
            if let Ok(endpoint) = std::env::var("TALON_LM_STUDIO_ENDPOINT") {
                config.endpoint = endpoint;
            }
            
            return config;
        }

        let config_path = dirs::home_dir()
            .map(|home| home.join(".talon").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from(".talon/config.toml"));

        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                    return Self::from_toml(&config);
                }
            }
        }

        AiConfig::default()
    }

    fn from_toml(config: &toml::Value) -> Self {
        let mut ai_config = AiConfig::default();
        
        if let Some(ai_section) = config.get("ai") {
            if let Some(backend) = ai_section.get("backend").and_then(|v| v.as_str()) {
                ai_config.backend = match backend {
                    "lm_studio" => AiBackend::LmStudio,
                    "local_gguf" => AiBackend::LocalGguf,
                    "disabled" => AiBackend::Disabled,
                    _ => AiBackend::LmStudio,
                };
            }
            
            if let Some(endpoint) = ai_section.get("endpoint").and_then(|v| v.as_str()) {
                ai_config.endpoint = endpoint.to_string();
            }
            
            if let Some(model) = ai_section.get("model").and_then(|v| v.as_str()) {
                ai_config.model = model.to_string();
            }
            
            if let Some(temp) = ai_section.get("temperature").and_then(|v| v.as_float()) {
                ai_config.temperature = temp as f32;
            }
            
            if let Some(max_tokens) = ai_section.get("max_tokens").and_then(|v| v.as_integer()) {
                ai_config.max_tokens = max_tokens as usize;
            }
        }
        
        ai_config
    }
}

pub struct MlOracle {
    config: AiConfig,
    client: reqwest::Client,
    available: bool,
}

impl MlOracle {
    pub fn new() -> Self {
        let config = AiConfig::load();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        MlOracle {
            config,
            client,
            available: false,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), String> {
        match self.config.backend {
            AiBackend::LmStudio => {
                match self.check_lm_studio_available().await {
                    Ok(true) => {
                        self.available = true;
                        Ok(())
                    }
                    Ok(false) => {
                        log::warn!("LM Studio not available, trying local GGUF fallback");
                        self.config.backend = AiBackend::LocalGguf;
                        self.check_local_gguf_available()
                    }
                    Err(e) => {
                        log::warn!("LM Studio check failed: {}, disabling AI features", e);
                        self.config.backend = AiBackend::Disabled;
                        self.available = false;
                        Err(format!("AI features disabled: {}", e))
                    }
                }
            }
            AiBackend::LocalGguf => self.check_local_gguf_available(),
            AiBackend::Disabled => {
                self.available = false;
                Err("AI features manually disabled".to_string())
            }
        }
    }

    async fn check_lm_studio_available(&self) -> Result<bool, String> {
        let url = format!("{}/models", self.config.endpoint);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => Err(format!("LM Studio connection failed: {}", e)),
        }
    }

    fn check_local_gguf_available(&mut self) -> Result<(), String> {
        let models_dir = dirs::home_dir()
            .map(|home| home.join(".talon").join("models"))
            .unwrap_or_else(|| PathBuf::from(".talon/models"));

        if !models_dir.exists() {
            fs::create_dir_all(&models_dir)
                .map_err(|e| format!("Failed to create models directory: {}", e))?;
        }

        let has_models = models_dir
            .read_dir()
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .any(|e| {
                        e.path()
                            .extension()
                            .and_then(|ext| ext.to_str())
                            == Some("gguf")
                    })
            })
            .unwrap_or(false);

        if has_models {
            self.available = true;
            Ok(())
        } else {
            log::warn!("No GGUF models found in {:?}, disabling AI features", models_dir);
            self.config.backend = AiBackend::Disabled;
            self.available = false;
            Err("No AI models available".to_string())
        }
    }

    pub fn is_available(&self) -> bool {
        self.available
    }

    pub async fn analyze_vulnerability(
        &self,
        binary_path: &str,
        disassembly: &str,
        heuristic_report: &VulnerabilityReport,
    ) -> Result<String, String> {
        if !self.available {
            return Err("AI features not available".to_string());
        }

        let prompt = Self::build_vulnerability_analysis_prompt(
            binary_path,
            disassembly,
            heuristic_report,
        );

        self.query_ai(&prompt).await
    }

    pub async fn generate_exploit(
        &self,
        vuln_report: &VulnerabilityReport,
        target_info: &str,
    ) -> Result<String, String> {
        if !self.available {
            return Err("AI features not available".to_string());
        }

        let prompt = Self::build_exploit_generation_prompt(vuln_report, target_info);
        self.query_ai(&prompt).await
    }

    pub async fn suggest_gadgets(
        &self,
        binary_arch: &str,
        objective: &str,
        available_gadgets: &[String],
    ) -> Result<String, String> {
        if !self.available {
            return Err("AI features not available".to_string());
        }

        let prompt = Self::build_gadget_suggestion_prompt(binary_arch, objective, available_gadgets);
        self.query_ai(&prompt).await
    }

    pub async fn explain_error(&self, error_message: &str, context: &str) -> Result<String, String> {
        if !self.available {
            return Err("AI features not available".to_string());
        }

        let prompt = Self::build_error_explanation_prompt(error_message, context);
        self.query_ai(&prompt).await
    }

    pub async fn suggest_code_improvements(&self, code: &str) -> Result<String, String> {
        if !self.available {
            return Err("AI features not available".to_string());
        }

        let prompt = Self::build_code_review_prompt(code);
        self.query_ai(&prompt).await
    }

    async fn query_ai(&self, prompt: &str) -> Result<String, String> {
        match self.config.backend {
            AiBackend::LmStudio => self.query_lm_studio(prompt).await,
            AiBackend::LocalGguf => self.query_local_gguf(prompt).await,
            AiBackend::Disabled => Err("AI features disabled".to_string()),
        }
    }

    async fn query_lm_studio(&self, prompt: &str) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.config.endpoint);
        
        let payload = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are an expert security researcher specializing in binary exploitation, vulnerability analysis, and exploit development. Provide technical, precise, and actionable analysis."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
            "stream": false
        });

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("LM Studio request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("LM Studio returned error: {}", response.status()));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse LM Studio response: {}", e))?;

        let content = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        if content.is_empty() {
            return Err("LM Studio returned empty response".to_string());
        }

        Ok(content)
    }

    async fn query_local_gguf(&self, _prompt: &str) -> Result<String, String> {
        Err("Local GGUF support not yet implemented".to_string())
    }

    fn build_vulnerability_analysis_prompt(
        binary_path: &str,
        disassembly: &str,
        heuristic: &VulnerabilityReport,
    ) -> String {
        let truncated_disasm = Self::truncate_for_context(disassembly, 4000);
        
        format!(
            r#"Analyze this binary for vulnerabilities beyond the heuristic detection.

Binary: {}
Heuristic Detection: {} (confidence: {:.2}, exploitability: {})
Location: {}

Disassembly snippet:
```
{}
```

Provide:
1. Validation of heuristic findings
2. Additional logic flaws or vulnerabilities not caught by heuristics
3. Potential exploit strategies
4. Recommended gadgets or techniques
5. Risk assessment

Be concise but technically precise."#,
            binary_path,
            heuristic.vuln_type,
            heuristic.confidence,
            match heuristic.exploitability {
                crate::oracle::Exploitability::Critical => "Critical",
                crate::oracle::Exploitability::High => "High",
                crate::oracle::Exploitability::Medium => "Medium",
                crate::oracle::Exploitability::Low => "Low",
                crate::oracle::Exploitability::None => "None",
            },
            heuristic.location,
            truncated_disasm
        )
    }

    fn build_exploit_generation_prompt(vuln: &VulnerabilityReport, target_info: &str) -> String {
        format!(
            r#"Generate a TALON exploit script for this vulnerability.

Vulnerability: {}
Location: {}
Confidence: {:.2}
Exploitability: {}
Details: {}

Target Info:
{}

Generate a complete TALON script with:
1. Binary loading and analysis
2. Gadget finding or shellcode selection
3. Payload construction
4. Exploitation logic
5. Error handling

Output only the TALON code, no explanations outside comments."#,
            vuln.vuln_type,
            vuln.location,
            vuln.confidence,
            match vuln.exploitability {
                crate::oracle::Exploitability::Critical => "Critical",
                crate::oracle::Exploitability::High => "High",
                crate::oracle::Exploitability::Medium => "Medium",
                crate::oracle::Exploitability::Low => "Low",
                crate::oracle::Exploitability::None => "None",
            },
            vuln.details,
            target_info
        )
    }

    fn build_gadget_suggestion_prompt(
        arch: &str,
        objective: &str,
        gadgets: &[String],
    ) -> String {
        let gadget_list = if gadgets.len() > 50 {
            format!("{} gadgets available (showing first 50):\n{}", 
                gadgets.len(),
                gadgets[..50].join("\n"))
        } else {
            gadgets.join("\n")
        };
        
        format!(
            r#"Suggest an optimal ROP chain strategy for this objective.

Architecture: {}
Objective: {}

Available Gadgets:
{}

Provide:
1. Recommended gadget sequence
2. Register setup strategy
3. Stack alignment considerations
4. Alternative approaches if primary fails

Be specific with gadget addresses and register operations."#,
            arch, objective, gadget_list
        )
    }

    fn build_error_explanation_prompt(error: &str, context: &str) -> String {
        format!(
            r#"Explain this TALON error and suggest fixes.

Error: {}

Context:
{}

Provide:
1. Root cause explanation
2. Common reasons this error occurs
3. Step-by-step fix instructions
4. Example corrected code

Be helpful and educational."#,
            error, context
        )
    }

    fn build_code_review_prompt(code: &str) -> String {
        let truncated = Self::truncate_for_context(code, 6000);
        
        format!(
            r#"Review this TALON exploit code for improvements.

Code:
```talon
{}
```

Provide:
1. Security considerations
2. Reliability improvements
3. Code quality suggestions
4. Performance optimizations
5. Edge cases to handle

Focus on actionable feedback."#,
            truncated
        )
    }

    fn truncate_for_context(text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            let half = max_len / 2;
            format!(
                "{}...[truncated {} bytes]...\n{}",
                &text[..half],
                text.len() - max_len,
                &text[text.len() - half..]
            )
        }
    }
}

impl Default for MlOracle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_config_default() {
        let config = AiConfig::default();
        assert_eq!(config.backend, AiBackend::LmStudio);
        assert_eq!(config.endpoint, "http://10.5.0.2:1234/v1");
        assert_eq!(config.model, "deepseek-coder-6.7b-instruct");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.max_tokens, 2048);
    }

    #[test]
    fn test_ml_oracle_creation() {
        let oracle = MlOracle::new();
        assert!(!oracle.is_available());
    }

    #[test]
    fn test_truncate_for_context() {
        let short_text = "Hello, world!";
        let result = MlOracle::truncate_for_context(short_text, 100);
        assert_eq!(result, short_text);

        let long_text = "A".repeat(1000);
        let result = MlOracle::truncate_for_context(&long_text, 100);
        assert!(result.contains("truncated"));
        assert!(result.len() < long_text.len());
    }

    #[test]
    fn test_prompt_building() {
        let prompt = MlOracle::build_error_explanation_prompt(
            "Connection refused",
            "Trying to connect to localhost:1234",
        );
        assert!(prompt.contains("Connection refused"));
        assert!(prompt.contains("localhost:1234"));
        assert!(prompt.contains("Root cause"));
    }

    #[test]
    fn test_config_from_env() {
        std::env::set_var("TALON_AI_BACKEND", "disabled");
        let config = AiConfig::load();
        assert_eq!(config.backend, AiBackend::Disabled);
        std::env::remove_var("TALON_AI_BACKEND");
    }

    #[test]
    fn test_gadget_prompt_truncation() {
        let gadgets: Vec<String> = (0..100).map(|i| format!("gadget_{}", i)).collect();
        let prompt = MlOracle::build_gadget_suggestion_prompt("x64", "spawn shell", &gadgets);
        assert!(prompt.contains("showing first 50"));
        assert!(prompt.contains("100 gadgets"));
    }
}
