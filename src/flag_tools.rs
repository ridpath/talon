use regex::Regex;
use std::collections::HashMap;

pub struct FlagFinder {
    patterns: Vec<Regex>,
}

impl FlagFinder {
    pub fn new() -> Self {
        let patterns = vec![
            Regex::new(r"flag\{[^}]+\}").unwrap(),
            Regex::new(r"FLAG\{[^}]+\}").unwrap(),
            Regex::new(r"CTF\{[^}]+\}").unwrap(),
            Regex::new(r"ctf\{[^}]+\}").unwrap(),
            Regex::new(r"[A-Z0-9]{31}=").unwrap(),
            Regex::new(r"HTB\{[^}]+\}").unwrap(),
            Regex::new(r"picoCTF\{[^}]+\}").unwrap(),
            Regex::new(r"RACTF\{[^}]+\}").unwrap(),
            Regex::new(r"[a-f0-9]{32}").unwrap(),
            Regex::new(r"[a-f0-9]{64}").unwrap(),
        ];

        FlagFinder { patterns }
    }

    pub fn with_custom_pattern(mut self, pattern: &str) -> Result<Self, String> {
        let regex = Regex::new(pattern).map_err(|e| format!("Invalid regex pattern: {}", e))?;
        self.patterns.push(regex);
        Ok(self)
    }

    pub fn search(&self, data: &[u8]) -> Vec<String> {
        let text = String::from_utf8_lossy(data);
        let mut results = Vec::new();

        for pattern in &self.patterns {
            for capture in pattern.find_iter(&text) {
                let flag = capture.as_str().to_string();
                if !results.contains(&flag) {
                    results.push(flag);
                }
            }
        }

        results
    }

    pub fn search_file(&self, filepath: &str) -> Result<Vec<String>, String> {
        let data = std::fs::read(filepath).map_err(|e| format!("Failed to read file: {}", e))?;
        Ok(self.search(&data))
    }

    pub fn search_recursive(&self, directory: &str) -> Result<Vec<FlagMatch>, String> {
        use walkdir::WalkDir;

        let mut matches = Vec::new();

        for entry in WalkDir::new(directory).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Ok(data) = std::fs::read(entry.path()) {
                    let flags = self.search(&data);
                    for flag in flags {
                        matches.push(FlagMatch {
                            flag,
                            filepath: entry.path().display().to_string(),
                        });
                    }
                }
            }
        }

        Ok(matches)
    }
}

#[derive(Debug, Clone)]
pub struct FlagMatch {
    pub flag: String,
    pub filepath: String,
}

pub struct FlagSubmitter {
    url: String,
    headers: HashMap<String, String>,
}

impl FlagSubmitter {
    pub fn new(url: String) -> Self {
        FlagSubmitter {
            url,
            headers: HashMap::new(),
        }
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.headers
            .insert("Authorization".to_string(), format!("Bearer {}", token));
        self
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn submit(&self, flag: &str) -> Result<SubmitResponse, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let mut request = client.post(&self.url);

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let body = serde_json::json!({ "flag": flag });

        let response = request
            .json(&body)
            .send()
            .map_err(|e| format!("Failed to submit flag: {}", e))?;

        let status = response.status();
        let body_text = response
            .text()
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if status.is_success() {
            Ok(SubmitResponse {
                success: true,
                message: body_text,
            })
        } else {
            Ok(SubmitResponse {
                success: false,
                message: format!("HTTP {}: {}", status, body_text),
            })
        }
    }

    pub fn submit_ctfd(&self, flag: &str, challenge_id: i64) -> Result<SubmitResponse, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let mut request = client.post(&format!("{}/api/v1/challenges/attempt", self.url));

        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        let body = serde_json::json!({
            "challenge_id": challenge_id,
            "submission": flag
        });

        let response = request
            .json(&body)
            .send()
            .map_err(|e| format!("Failed to submit flag: {}", e))?;

        let _status = response.status();
        let json: serde_json::Value = response
            .json()
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(SubmitResponse {
            success: json["success"].as_bool().unwrap_or(false),
            message: json["message"].as_str().unwrap_or("").to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SubmitResponse {
    pub success: bool,
    pub message: String,
}

pub fn flag_search(data: &[u8]) -> Vec<String> {
    FlagFinder::new().search(data)
}

pub fn flag_search_custom(data: &[u8], pattern: &str) -> Result<Vec<String>, String> {
    let finder = FlagFinder::new().with_custom_pattern(pattern)?;
    Ok(finder.search(data))
}

pub fn flag_search_file(filepath: &str) -> Result<Vec<String>, String> {
    FlagFinder::new().search_file(filepath)
}

pub fn flag_search_dir(directory: &str) -> Result<Vec<FlagMatch>, String> {
    FlagFinder::new().search_recursive(directory)
}

pub fn flag_submit(url: &str, flag: &str) -> Result<SubmitResponse, String> {
    FlagSubmitter::new(url.to_string()).submit(flag)
}

pub fn flag_submit_with_token(
    url: &str,
    flag: &str,
    token: &str,
) -> Result<SubmitResponse, String> {
    FlagSubmitter::new(url.to_string())
        .with_token(token.to_string())
        .submit(flag)
}

pub fn flag_submit_ctfd(
    url: &str,
    flag: &str,
    challenge_id: i64,
    token: &str,
) -> Result<SubmitResponse, String> {
    FlagSubmitter::new(url.to_string())
        .with_token(token.to_string())
        .submit_ctfd(flag, challenge_id)
}
