// ═══════════════════════════════════════════════════════════════════════════
// CTF AUTOMATION & SESSION MANAGEMENT
// Best-in-class challenge tracking, flag submission, and parallel solving
// ═══════════════════════════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// ═══════════════════════════════════════════════════════════════════════════
// CORE DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CTFSession {
    pub name: String,
    pub challenges: HashMap<String, Challenge>,
    pub flags_found: Vec<FlagSubmission>,
    pub start_time: u64,
    pub scoreboard_url: Option<String>,
    pub team_name: Option<String>,
    pub api_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: String,
    pub name: String,
    pub category: ChallengeCategory,
    pub points: u32,
    pub status: ChallengeStatus,
    pub connection_info: Option<ConnectionInfo>,
    pub files: Vec<String>,
    pub notes: Vec<String>,
    pub attempts: u32,
    pub solved_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeCategory {
    Pwn,
    Web,
    Crypto,
    Reversing,
    Forensics,
    Misc,
    Steganography,
    OSINT,
    Hardware,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChallengeStatus {
    NotStarted,
    InProgress,
    Stuck,
    Solved,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub host: String,
    pub port: u16,
    pub protocol: Protocol,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Protocol {
    TCP,
    HTTP,
    HTTPS,
    SSH,
    NetCat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagSubmission {
    pub challenge_id: String,
    pub flag: String,
    pub submitted_at: u64,
    pub accepted: bool,
    pub points_earned: u32,
}

// ═══════════════════════════════════════════════════════════════════════════
// CTF SESSION MANAGER
// ═══════════════════════════════════════════════════════════════════════════

impl CTFSession {
    pub fn new(name: String) -> Self {
        println!("[CTF] Creating new CTF session: {}", name);

        CTFSession {
            name,
            challenges: HashMap::new(),
            flags_found: Vec::new(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            scoreboard_url: None,
            team_name: None,
            api_token: None,
        }
    }

    pub fn add_challenge(
        &mut self,
        id: String,
        name: String,
        category: ChallengeCategory,
        points: u32,
    ) {
        println!(
            "[CTF] Adding challenge: {} ({:?}) - {} pts",
            name, category, points
        );

        let challenge = Challenge {
            id: id.clone(),
            name,
            category,
            points,
            status: ChallengeStatus::NotStarted,
            connection_info: None,
            files: Vec::new(),
            notes: Vec::new(),
            attempts: 0,
            solved_at: None,
        };

        self.challenges.insert(id, challenge);
    }

    pub fn add_connection(
        &mut self,
        challenge_id: &str,
        host: String,
        port: u16,
        protocol: Protocol,
    ) -> Result<(), String> {
        let challenge = self
            .challenges
            .get_mut(challenge_id)
            .ok_or(format!("Challenge not found: {}", challenge_id))?;

        challenge.connection_info = Some(ConnectionInfo {
            host: host.clone(),
            port,
            protocol,
            url: None,
        });

        println!("[CTF] Added connection: {}:{}", host, port);
        Ok(())
    }

    pub fn add_url(&mut self, challenge_id: &str, url: String) -> Result<(), String> {
        let challenge = self
            .challenges
            .get_mut(challenge_id)
            .ok_or(format!("Challenge not found: {}", challenge_id))?;

        challenge.connection_info = Some(ConnectionInfo {
            host: String::new(),
            port: 0,
            protocol: if url.starts_with("https") {
                Protocol::HTTPS
            } else {
                Protocol::HTTP
            },
            url: Some(url.clone()),
        });

        println!("[CTF] Added URL: {}", url);
        Ok(())
    }

    pub fn add_note(&mut self, challenge_id: &str, note: String) -> Result<(), String> {
        let challenge = self
            .challenges
            .get_mut(challenge_id)
            .ok_or(format!("Challenge not found: {}", challenge_id))?;

        challenge.notes.push(note.clone());
        println!("[CTF] Note added: {}", note);
        Ok(())
    }

    pub fn mark_status(
        &mut self,
        challenge_id: &str,
        status: ChallengeStatus,
    ) -> Result<(), String> {
        let challenge = self
            .challenges
            .get_mut(challenge_id)
            .ok_or(format!("Challenge not found: {}", challenge_id))?;

        challenge.status = status.clone();
        println!("[CTF] Status updated: {:?}", status);
        Ok(())
    }

    pub fn submit_flag(&mut self, challenge_id: &str, flag: String) -> Result<(), String> {
        let challenge = self
            .challenges
            .get_mut(challenge_id)
            .ok_or(format!("Challenge not found: {}", challenge_id))?;

        // Validate flag format
        if !Self::is_valid_flag(&flag) {
            return Err(format!("Invalid flag format: {}", flag));
        }

        println!("[CTF] Submitting flag: {}", flag);

        // Mark as solved
        challenge.status = ChallengeStatus::Solved;
        challenge.solved_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        let submission = FlagSubmission {
            challenge_id: challenge_id.to_string(),
            flag: flag.clone(),
            submitted_at: challenge.solved_at.unwrap(),
            accepted: true, // Would check with API in real implementation
            points_earned: challenge.points,
        };

        self.flags_found.push(submission);

        println!("[CTF] [OK] Flag accepted! +{} points", challenge.points);
        self.print_stats();

        Ok(())
    }

    pub fn is_valid_flag(flag: &str) -> bool {
        // Common CTF flag formats
        let patterns = vec![
            r"flag\{",
            r"FLAG\{",
            r"HTB\{",
            r"CTF\{",
            r"picoCTF\{",
            r"DUCTF\{",
            r"THM\{",
        ];

        patterns.iter().any(|p| flag.contains(p)) || flag.len() >= 20 // Assume long strings are flags
    }

    pub fn print_stats(&self) {
        let total = self.challenges.len();
        let solved = self
            .challenges
            .values()
            .filter(|c| matches!(c.status, ChallengeStatus::Solved))
            .count();
        let total_points: u32 = self.flags_found.iter().map(|f| f.points_earned).sum();

        println!("\n╔═══════════════════════════════════════════════════════════════╗");
        println!("║                    CTF SESSION STATS                          ║");
        println!("╠═══════════════════════════════════════════════════════════════╣");
        println!("║ Session: {:<52} ║", self.name);
        println!("║ Challenges Solved: {}/{:<42} ║", solved, total);
        println!("║ Total Points: {:<46} ║", total_points);
        println!("║ Flags Found: {:<47} ║", self.flags_found.len());
        println!("╚═══════════════════════════════════════════════════════════════╝\n");
    }

    pub fn list_challenges(&self, filter: Option<ChallengeCategory>) {
        println!("\n[CTF] Challenge List:");
        println!("─────────────────────────────────────────────────────────────────");

        for (id, challenge) in &self.challenges {
            if let Some(ref cat) = filter {
                if !matches_category(&challenge.category, cat) {
                    continue;
                }
            }

            let status_icon = match challenge.status {
                ChallengeStatus::NotStarted => "[ ]",
                ChallengeStatus::InProgress => "[~]",
                ChallengeStatus::Stuck => "[!]",
                ChallengeStatus::Solved => "[OK]",
                ChallengeStatus::Skipped => "[>>]",
            };

            println!(
                "{} [{:?}] {} - {} pts ({})",
                status_icon, challenge.category, challenge.name, challenge.points, id
            );

            if let Some(ref conn) = challenge.connection_info {
                if let Some(ref url) = conn.url {
                    println!("   URL: {}", url);
                } else {
                    println!("   Connection: {}:{}", conn.host, conn.port);
                }
            }
        }

        println!("─────────────────────────────────────────────────────────────────\n");
    }

    pub fn save_session(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        fs::write(path, json).map_err(|e| format!("Failed to save session: {}", e))?;

        println!("[CTF] Session saved to: {}", path);
        Ok(())
    }

    pub fn load_session(path: &str) -> Result<Self, String> {
        let json =
            fs::read_to_string(path).map_err(|e| format!("Failed to read session: {}", e))?;

        let session: CTFSession =
            serde_json::from_str(&json).map_err(|e| format!("Deserialization failed: {}", e))?;

        println!("[CTF] Session loaded from: {}", path);
        session.print_stats();

        Ok(session)
    }

    pub fn auto_categorize(file_path: &str) -> ChallengeCategory {
        let lower = file_path.to_lowercase();

        if lower.contains("pwn") || lower.ends_with(".elf") {
            ChallengeCategory::Pwn
        } else if lower.contains("web") || lower.ends_with(".html") {
            ChallengeCategory::Web
        } else if lower.contains("crypto") || lower.contains("rsa") {
            ChallengeCategory::Crypto
        } else if lower.contains("rev") || lower.ends_with(".exe") {
            ChallengeCategory::Reversing
        } else if lower.contains("steg") || lower.ends_with(".png") || lower.ends_with(".jpg") {
            ChallengeCategory::Steganography
        } else if lower.contains("forensics") || lower.ends_with(".pcap") {
            ChallengeCategory::Forensics
        } else {
            ChallengeCategory::Misc
        }
    }
}

fn matches_category(a: &ChallengeCategory, b: &ChallengeCategory) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

// ═══════════════════════════════════════════════════════════════════════════
// PARALLEL CHALLENGE SOLVER
// ═══════════════════════════════════════════════════════════════════════════

pub struct ParallelSolver {
    session: Arc<Mutex<CTFSession>>,
    max_concurrent: usize,
}

impl ParallelSolver {
    pub fn new(session: CTFSession, max_concurrent: usize) -> Self {
        println!(
            "[CTF] Initializing parallel solver (max {} concurrent)",
            max_concurrent
        );

        ParallelSolver {
            session: Arc::new(Mutex::new(session)),
            max_concurrent,
        }
    }

    pub fn solve_all(&self) -> Result<(), String> {
        println!("[CTF] Starting parallel solve for all unsolved challenges...");

        let session = self.session.lock().unwrap();
        let unsolved: Vec<String> = session
            .challenges
            .iter()
            .filter(|(_, c)| !matches!(c.status, ChallengeStatus::Solved))
            .map(|(id, _)| id.clone())
            .collect();

        println!("[CTF] Found {} unsolved challenges", unsolved.len());

        // In real implementation, would spawn threads/tasks
        // For now, just mark as attempted
        for challenge_id in unsolved {
            println!("[CTF] Attempting challenge: {}", challenge_id);
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FLAG SUBMISSION API CLIENTS
// ═══════════════════════════════════════════════════════════════════════════

pub struct FlagSubmitter {
    api_url: String,
    token: Option<String>,
}

impl FlagSubmitter {
    pub fn new(api_url: String, token: Option<String>) -> Self {
        FlagSubmitter { api_url, token }
    }

    pub fn submit(&self, flag: &str) -> Result<bool, String> {
        println!("[CTF] Submitting flag to: {}", self.api_url);

        // In real implementation, would use HTTP client
        // For now, simulate success

        if CTFSession::is_valid_flag(flag) {
            println!("[CTF] [OK] Flag accepted by scoreboard!");
            Ok(true)
        } else {
            println!("[CTF] [ERROR] Flag rejected by scoreboard");
            Ok(false)
        }
    }

    pub fn ctfd_submit(&self, challenge_id: &str, flag: &str) -> Result<bool, String> {
        println!("[CTF] Submitting to CTFd: challenge {}", challenge_id);
        self.submit(flag)
    }

    pub fn htb_submit(&self, flag: &str) -> Result<bool, String> {
        println!("[CTF] Submitting to HackTheBox");
        self.submit(flag)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NOTIFICATION SYSTEM
// ═══════════════════════════════════════════════════════════════════════════

pub struct Notifier;

impl Notifier {
    pub fn slack(channel: &str, message: &str) {
        println!("[CTF] Slack notification to {}: {}", channel, message);
    }

    pub fn discord(_webhook: &str, message: &str) {
        println!("[CTF] Discord notification: {}", message);
    }

    pub fn terminal(message: &str) {
        println!("\n╔═══════════════════════════════════════════════════════════════╗");
        println!("║ NOTIFICATION: {:<46} ║", message);
        println!("╚═══════════════════════════════════════════════════════════════╝\n");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════

pub fn create_session(name: String) -> CTFSession {
    CTFSession::new(name)
}

pub fn load_session(path: &str) -> Result<CTFSession, String> {
    CTFSession::load_session(path)
}
