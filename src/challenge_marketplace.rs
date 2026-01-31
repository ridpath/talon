use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: String,
    pub title: String,
    pub author: String,
    pub description: String,
    pub difficulty: String,
    pub category: String,
    pub points: u32,
    pub downloads: u32,
    pub rating: f32,
    pub tags: Vec<String>,
    pub replay_file: Option<String>,
}

pub struct ChallengeMarketplace;

impl ChallengeMarketplace {
    pub fn browse() -> Result<Vec<Challenge>, String> {
        Ok(vec![
            Challenge {
                id: "1".to_string(),
                title: "Baby's First Buffer Overflow".to_string(),
                author: "TALON Team".to_string(),
                description: "Learn the basics of buffer overflow exploitation".to_string(),
                difficulty: "beginner".to_string(),
                category: "pwn".to_string(),
                points: 100,
                downloads: 1523,
                rating: 4.8,
                tags: vec!["buffer-overflow".to_string(), "rop".to_string()],
                replay_file: Some("baby_bof.talonrec".to_string()),
            },
            Challenge {
                id: "2".to_string(),
                title: "Format String Madness".to_string(),
                author: "0xdeadbeef".to_string(),
                description: "Master format string vulnerabilities".to_string(),
                difficulty: "intermediate".to_string(),
                category: "pwn".to_string(),
                points: 250,
                downloads: 892,
                rating: 4.6,
                tags: vec!["format-string".to_string()],
                replay_file: Some("fmt_madness.talonrec".to_string()),
            },
        ])
    }

    pub fn download(challenge_id: &str) -> Result<String, String> {
        println!("[OK] Downloading challenge {}...", challenge_id);
        Ok(format!("challenge_{}.talonrec", challenge_id))
    }

    pub fn rate(_challenge_id: &str, _rating: f32) -> Result<(), String> {
        println!("[OK] Rating submitted");
        Ok(())
    }

    pub fn upload(_challenge: &Challenge) -> Result<String, String> {
        println!("[OK] Challenge uploaded successfully");
        Ok("new_challenge_id".to_string())
    }
}
