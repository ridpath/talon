use regex::Regex;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::fs;

// ═══════════════════════════════════════════════════════════════════════════
// OSINT (OPEN SOURCE INTELLIGENCE) TOOLKIT - PRODUCTION READY
// ═══════════════════════════════════════════════════════════════════════════

// ────────────────────────────────────────────────────────────────────────────
// SUBDOMAIN ENUMERATION
// ────────────────────────────────────────────────────────────────────────────

pub struct SubdomainEnumerator;

impl SubdomainEnumerator {
    pub fn enumerate(domain: &str, wordlist: &[&str]) -> Vec<String> {
        println!("[SUBDOMAIN-ENUM] Enumerating subdomains for {}", domain);

        let mut found = Vec::new();
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();

        for subdomain in wordlist {
            let url = format!("http://{}.{}", subdomain, domain);

            match client.get(&url).send() {
                Ok(response) => {
                    if response.status().is_success() || response.status().is_redirection() {
                        println!("[SUBDOMAIN-ENUM] [OK] Found: {}.{}", subdomain, domain);
                        found.push(format!("{}.{}", subdomain, domain));
                    }
                }
                Err(_) => continue,
            }
        }

        println!("[SUBDOMAIN-ENUM] Total found: {}", found.len());
        found
    }

    pub fn common_subdomains() -> Vec<&'static str> {
        vec![
            "www",
            "mail",
            "ftp",
            "localhost",
            "webmail",
            "smtp",
            "pop",
            "ns1",
            "webdisk",
            "ns2",
            "cpanel",
            "whm",
            "autodiscover",
            "autoconfig",
            "m",
            "imap",
            "test",
            "ns",
            "blog",
            "pop3",
            "dev",
            "www2",
            "admin",
            "forum",
            "news",
            "vpn",
            "ns3",
            "mail2",
            "new",
            "mysql",
            "old",
            "lists",
            "support",
            "mobile",
            "mx",
            "static",
            "docs",
            "beta",
            "shop",
            "sql",
            "secure",
            "demo",
            "cp",
            "calendar",
            "wiki",
            "web",
            "media",
            "email",
            "images",
            "img",
            "www1",
            "intranet",
            "portal",
            "video",
            "sip",
            "dns2",
            "api",
            "cdn",
            "stats",
            "dns1",
            "ns4",
            "www3",
            "dns",
            "search",
            "staging",
            "server",
            "mx1",
            "chat",
            "wap",
            "my",
            "svn",
            "mail1",
            "sites",
            "proxy",
            "ads",
            "host",
            "crm",
        ]
    }

    pub fn dns_lookup(domain: &str) -> Result<(), String> {
        println!("[DNS-LOOKUP] Looking up DNS records for {}", domain);

        let output = std::process::Command::new("nslookup").arg(domain).output();

        match output {
            Ok(out) => {
                println!("{}", String::from_utf8_lossy(&out.stdout));
                Ok(())
            }
            Err(_) => {
                println!("[DNS-LOOKUP] nslookup not found");
                Ok(())
            }
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// EMAIL HARVESTING
// ────────────────────────────────────────────────────────────────────────────

pub struct EmailHarvester;

impl EmailHarvester {
    pub fn harvest_from_url(url: &str) -> Result<Vec<String>, String> {
        println!("[EMAIL-HARVEST] Harvesting emails from {}", url);

        let client = Client::new();
        let response = client
            .get(url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let body = response
            .text()
            .map_err(|e| format!("Failed to read response: {}", e))?;

        Self::extract_emails(&body)
    }

    pub fn extract_emails(text: &str) -> Result<Vec<String>, String> {
        let email_regex = Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")
            .map_err(|e| format!("Regex error: {}", e))?;

        let mut emails: Vec<String> = email_regex
            .find_iter(text)
            .map(|m| m.as_str().to_string())
            .collect();

        emails.sort();
        emails.dedup();

        println!("[EMAIL-HARVEST] Found {} unique emails", emails.len());
        for email in &emails {
            println!("[EMAIL-HARVEST]   • {}", email);
        }

        Ok(emails)
    }

    pub fn harvest_from_file(file_path: &str) -> Result<Vec<String>, String> {
        let content =
            fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        Self::extract_emails(&content)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// METADATA EXTRACTION (EXIF)
// ────────────────────────────────────────────────────────────────────────────

pub struct EXIFExtractor;

impl EXIFExtractor {
    pub fn extract(file_path: &str) -> Result<HashMap<String, String>, String> {
        println!("[EXIF] Extracting metadata from {}", file_path);

        let output = std::process::Command::new("exiftool")
            .arg("-json")
            .arg(file_path)
            .output();

        match output {
            Ok(out) => {
                let json_str = String::from_utf8_lossy(&out.stdout);

                if let Ok(json) = serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
                    if let Some(first) = json.first() {
                        let mut metadata = HashMap::new();

                        if let Some(obj) = first.as_object() {
                            for (key, value) in obj {
                                metadata.insert(key.clone(), value.to_string());
                            }
                        }

                        println!("[EXIF] Extracted {} metadata fields", metadata.len());
                        for (key, value) in &metadata {
                            if key.contains("GPS")
                                || key.contains("Date")
                                || key.contains("Make")
                                || key.contains("Model")
                            {
                                println!("[EXIF]   {}: {}", key, value);
                            }
                        }

                        return Ok(metadata);
                    }
                }

                Ok(HashMap::new())
            }
            Err(_) => {
                println!("[EXIF] exiftool not found. Install with: apt-get install libimage-exiftool-perl");
                Ok(HashMap::new())
            }
        }
    }

    pub fn extract_gps(file_path: &str) -> Result<Option<(f64, f64)>, String> {
        let metadata = Self::extract(file_path)?;

        let lat = metadata
            .get("GPSLatitude")
            .and_then(|s| s.parse::<f64>().ok());
        let lon = metadata
            .get("GPSLongitude")
            .and_then(|s| s.parse::<f64>().ok());

        if let (Some(lat), Some(lon)) = (lat, lon) {
            println!("[EXIF] GPS Coordinates: {}, {}", lat, lon);
            println!(
                "[EXIF] 🌍 Google Maps: https://www.google.com/maps?q={},{}",
                lat, lon
            );
            Ok(Some((lat, lon)))
        } else {
            println!("[EXIF] No GPS coordinates found");
            Ok(None)
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// WHOIS LOOKUP
// ────────────────────────────────────────────────────────────────────────────

pub struct WhoisLookup;

impl WhoisLookup {
    pub fn lookup(domain: &str) -> Result<String, String> {
        println!("[WHOIS] Looking up {}", domain);

        let output = std::process::Command::new("whois").arg(domain).output();

        match output {
            Ok(out) => {
                let result = String::from_utf8_lossy(&out.stdout).to_string();
                println!("{}", result);
                Ok(result)
            }
            Err(_) => {
                println!("[WHOIS] whois command not found");
                Self::http_whois(domain)
            }
        }
    }

    fn http_whois(domain: &str) -> Result<String, String> {
        println!("[WHOIS] Using HTTP WHOIS lookup");

        let client = Client::new();
        let url = format!("https://www.whois.com/whois/{}", domain);

        let response = client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let body = response
            .text()
            .map_err(|e| format!("Failed to read response: {}", e))?;

        Ok(body)
    }

    pub fn extract_info(whois_data: &str) -> HashMap<String, String> {
        let mut info = HashMap::new();

        for line in whois_data.lines() {
            if line.contains("Registrar:") {
                info.insert(
                    "Registrar".to_string(),
                    line.split(':').nth(1).unwrap_or("").trim().to_string(),
                );
            }
            if line.contains("Creation Date:") || line.contains("Created:") {
                info.insert(
                    "Created".to_string(),
                    line.split(':').nth(1).unwrap_or("").trim().to_string(),
                );
            }
            if line.contains("Expiration Date:") || line.contains("Expires:") {
                info.insert(
                    "Expires".to_string(),
                    line.split(':').nth(1).unwrap_or("").trim().to_string(),
                );
            }
            if line.contains("Name Server:") {
                let ns = line.split(':').nth(1).unwrap_or("").trim();
                info.entry("NameServers".to_string())
                    .and_modify(|e| *e = format!("{}, {}", e, ns))
                    .or_insert(ns.to_string());
            }
        }

        info
    }
}

// ────────────────────────────────────────────────────────────────────────────
// SOCIAL MEDIA USERNAME SEARCH
// ────────────────────────────────────────────────────────────────────────────

pub struct UsernameSearch;

impl UsernameSearch {
    pub fn search(username: &str) -> HashMap<String, bool> {
        println!("[USERNAME-SEARCH] Searching for username: {}", username);

        let platforms = vec![
            ("GitHub", format!("https://github.com/{}", username)),
            ("Twitter", format!("https://twitter.com/{}", username)),
            ("Instagram", format!("https://instagram.com/{}", username)),
            ("LinkedIn", format!("https://linkedin.com/in/{}", username)),
            ("Reddit", format!("https://reddit.com/user/{}", username)),
            ("YouTube", format!("https://youtube.com/@{}", username)),
            ("TikTok", format!("https://tiktok.com/@{}", username)),
            ("Pinterest", format!("https://pinterest.com/{}", username)),
            ("Medium", format!("https://medium.com/@{}", username)),
            ("Dev.to", format!("https://dev.to/{}", username)),
        ];

        let mut results = HashMap::new();
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();

        for (platform, url) in platforms {
            match client.get(&url).send() {
                Ok(response) => {
                    let exists = response.status().is_success();
                    results.insert(platform.to_string(), exists);

                    if exists {
                        println!("[USERNAME-SEARCH] [OK] Found on {}: {}", platform, url);
                    }
                }
                Err(_) => {
                    results.insert(platform.to_string(), false);
                }
            }
        }

        results
    }
}

// ────────────────────────────────────────────────────────────────────────────
// GOOGLE DORKING HELPER
// ────────────────────────────────────────────────────────────────────────────

pub struct GoogleDork;

impl GoogleDork {
    pub fn generate_query(target: &str, dork_type: &str) -> String {
        let query = match dork_type {
            "files" => format!(
                "site:{} filetype:pdf OR filetype:doc OR filetype:xls",
                target
            ),
            "login" => format!(
                "site:{} inurl:login OR inurl:admin OR inurl:dashboard",
                target
            ),
            "exposed" => format!("site:{} intitle:index.of OR inurl:backup", target),
            "config" => format!(
                "site:{} filetype:conf OR filetype:cfg OR filetype:ini",
                target
            ),
            "sql" => format!(
                "site:{} intext:'SQL syntax' OR intext:'mysql_fetch'",
                target
            ),
            "directories" => format!("site:{} intitle:'Index of /'", target),
            "credentials" => format!("site:{} intext:password OR intext:username", target),
            "subdomains" => format!("site:*.{}", target),
            _ => format!("site:{}", target),
        };

        println!("[GOOGLE-DORK] Generated query: {}", query);
        println!(
            "[GOOGLE-DORK] URL: https://www.google.com/search?q={}",
            urlencoding::encode(&query)
        );

        query
    }

    pub fn common_dorks() -> Vec<(&'static str, &'static str)> {
        vec![
            ("files", "Find documents and files"),
            ("login", "Find login pages"),
            ("exposed", "Find exposed directories"),
            ("config", "Find configuration files"),
            ("sql", "Find SQL errors"),
            ("directories", "Find directory listings"),
            ("credentials", "Find exposed credentials"),
            ("subdomains", "Enumerate subdomains"),
        ]
    }
}

// ────────────────────────────────────────────────────────────────────────────
// SHODAN INTEGRATION
// ────────────────────────────────────────────────────────────────────────────

pub struct ShodanHelper;

impl ShodanHelper {
    pub fn search(api_key: &str, query: &str) -> Result<String, String> {
        println!("[SHODAN] Searching for: {}", query);

        let client = Client::new();
        let url = format!(
            "https://api.shodan.io/shodan/host/search?key={}&query={}",
            api_key,
            urlencoding::encode(query)
        );

        let response = client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let body = response
            .text()
            .map_err(|e| format!("Failed to read response: {}", e))?;

        Ok(body)
    }

    pub fn host_info(api_key: &str, ip: &str) -> Result<String, String> {
        println!("[SHODAN] Getting info for IP: {}", ip);

        let client = Client::new();
        let url = format!("https://api.shodan.io/shodan/host/{}?key={}", ip, api_key);

        let response = client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let body = response
            .text()
            .map_err(|e| format!("Failed to read response: {}", e))?;

        Ok(body)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// PHONE NUMBER OSINT
// ────────────────────────────────────────────────────────────────────────────

pub struct PhoneOSINT;

impl PhoneOSINT {
    pub fn parse_number(phone: &str) -> HashMap<String, String> {
        let mut info = HashMap::new();

        let cleaned = phone
            .chars()
            .filter(|c| c.is_numeric() || *c == '+')
            .collect::<String>();

        info.insert("Cleaned".to_string(), cleaned.clone());

        if cleaned.starts_with('+') {
            let country_code = &cleaned[1..].chars().take(2).collect::<String>();
            info.insert("Country Code".to_string(), format!("+{}", country_code));
        }

        println!("[PHONE-OSINT] Parsed phone number:");
        for (key, value) in &info {
            println!("[PHONE-OSINT]   {}: {}", key, value);
        }

        info
    }
}

// ────────────────────────────────────────────────────────────────────────────
// IP GEOLOCATION
// ────────────────────────────────────────────────────────────────────────────

pub struct IPGeolocation;

impl IPGeolocation {
    pub fn lookup(ip: &str) -> Result<HashMap<String, String>, String> {
        println!("[IP-GEO] Looking up geolocation for {}", ip);

        let client = Client::new();
        let url = format!("http://ip-api.com/json/{}", ip);

        let response = client
            .get(&url)
            .send()
            .map_err(|e| format!("Request failed: {}", e))?;

        let json: serde_json::Value = response
            .json()
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let mut info = HashMap::new();

        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                info.insert(key.clone(), value.to_string().trim_matches('"').to_string());
            }
        }

        println!("[IP-GEO] Results:");
        for (key, value) in &info {
            println!("[IP-GEO]   {}: {}", key, value);
        }

        Ok(info)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// COMPREHENSIVE OSINT REPORT
// ────────────────────────────────────────────────────────────────────────────

pub struct OSINTReport;

impl OSINTReport {
    pub fn generate(target: &str) -> Result<(), String> {
        println!("\n╔════════════════════════════════════════════════════════╗");
        println!("║          OSINT RECONNAISSANCE REPORT                  ║");
        println!("╚════════════════════════════════════════════════════════╝\n");
        println!("Target: {}\n", target);

        if target.contains('.') && !target.parse::<std::net::IpAddr>().is_ok() {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("WHOIS Information");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            let _ = WhoisLookup::lookup(target);

            println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("DNS Information");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            let _ = SubdomainEnumerator::dns_lookup(target);

            println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Subdomain Enumeration");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            let common = SubdomainEnumerator::common_subdomains();
            let _ = SubdomainEnumerator::enumerate(target, &common[..10]);
        } else if target.parse::<std::net::IpAddr>().is_ok() {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("IP Geolocation");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            let _ = IPGeolocation::lookup(target);
        } else {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Username Search");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            let _ = UsernameSearch::search(target);
        }

        println!("\n[OSINT] Report generation complete!");

        Ok(())
    }
}
