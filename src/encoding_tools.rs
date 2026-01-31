use base64;
use hex;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// ENCODING/DECODING TOOLKIT - PRODUCTION READY
// ═══════════════════════════════════════════════════════════════════════════

// ────────────────────────────────────────────────────────────────────────────
// BASE ENCODING/DECODING
// ────────────────────────────────────────────────────────────────────────────

pub struct BaseEncoder;

impl BaseEncoder {
    /// Encode data to base64.
    ///
    /// # Examples
    ///
    /// ```
    /// use talon::encoding_tools::BaseEncoder;
    ///
    /// let data = b"Hello, World!";
    /// let encoded = BaseEncoder::base64_encode(data);
    /// assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");
    /// ```
    pub fn base64_encode(data: &[u8]) -> String {
        base64::encode(data)
    }
    
    /// Decode base64 string to bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use talon::encoding_tools::BaseEncoder;
    ///
    /// let encoded = "SGVsbG8sIFdvcmxkIQ==";
    /// let decoded = BaseEncoder::base64_decode(encoded).unwrap();
    /// assert_eq!(decoded, b"Hello, World!");
    /// ```
    pub fn base64_decode(encoded: &str) -> Result<Vec<u8>, String> {
        base64::decode(encoded)
            .map_err(|e| format!("Base64 decode error: {}", e))
    }
    
    /// Encode data to URL-safe base64.
    ///
    /// # Examples
    ///
    /// ```
    /// use talon::encoding_tools::BaseEncoder;
    ///
    /// let data = b"Test data with special chars: +/=";
    /// let encoded = BaseEncoder::base64_url_encode(data);
    /// assert!(!encoded.contains('+'));
    /// assert!(!encoded.contains('/'));
    /// ```
    pub fn base64_url_encode(data: &[u8]) -> String {
        base64::encode_config(data, base64::URL_SAFE)
    }
    
    /// Decode URL-safe base64 string to bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use talon::encoding_tools::BaseEncoder;
    ///
    /// let data = b"test";
    /// let encoded = BaseEncoder::base64_url_encode(data);
    /// let decoded = BaseEncoder::base64_url_decode(&encoded).unwrap();
    /// assert_eq!(decoded, b"test");
    /// ```
    pub fn base64_url_decode(encoded: &str) -> Result<Vec<u8>, String> {
        base64::decode_config(encoded, base64::URL_SAFE)
            .map_err(|e| format!("Base64 URL decode error: {}", e))
    }
    
    pub fn base32_encode(data: &[u8]) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
        let mut result = String::new();
        let mut bits = 0u32;
        let mut bit_count = 0;
        
        for &byte in data {
            bits = (bits << 8) | byte as u32;
            bit_count += 8;
            
            while bit_count >= 5 {
                bit_count -= 5;
                let index = ((bits >> bit_count) & 0x1F) as usize;
                result.push(ALPHABET[index] as char);
            }
        }
        
        if bit_count > 0 {
            let index = ((bits << (5 - bit_count)) & 0x1F) as usize;
            result.push(ALPHABET[index] as char);
        }
        
        while !result.len().is_multiple_of(8) {
            result.push('=');
        }
        
        result
    }
    
    pub fn base32_decode(encoded: &str) -> Result<Vec<u8>, String> {
        let alphabet_map: HashMap<char, u8> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"
            .chars()
            .enumerate()
            .map(|(i, c)| (c, i as u8))
            .collect();
        
        let mut result = Vec::new();
        let mut bits = 0u32;
        let mut bit_count = 0;
        
        for c in encoded.chars() {
            if c == '=' {
                break;
            }
            
            let value = *alphabet_map.get(&c)
                .ok_or_else(|| format!("Invalid base32 character: {}", c))?;
            
            bits = (bits << 5) | value as u32;
            bit_count += 5;
            
            if bit_count >= 8 {
                bit_count -= 8;
                result.push(((bits >> bit_count) & 0xFF) as u8);
            }
        }
        
        Ok(result)
    }
    
    /// Encode data to hexadecimal string.
    ///
    /// # Examples
    ///
    /// ```
    /// use talon::encoding_tools::BaseEncoder;
    ///
    /// let data = b"\xde\xad\xbe\xef";
    /// let encoded = BaseEncoder::hex_encode(data);
    /// assert_eq!(encoded, "deadbeef");
    /// ```
    pub fn hex_encode(data: &[u8]) -> String {
        hex::encode(data)
    }
    
    /// Decode hexadecimal string to bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use talon::encoding_tools::BaseEncoder;
    ///
    /// let encoded = "deadbeef";
    /// let decoded = BaseEncoder::hex_decode(encoded).unwrap();
    /// assert_eq!(decoded, b"\xde\xad\xbe\xef");
    /// ```
    pub fn hex_decode(encoded: &str) -> Result<Vec<u8>, String> {
        hex::decode(encoded)
            .map_err(|e| format!("Hex decode error: {}", e))
    }
}

// ────────────────────────────────────────────────────────────────────────────
// URL ENCODING/DECODING
// ────────────────────────────────────────────────────────────────────────────

pub struct URLEncoder;

impl URLEncoder {
    pub fn encode(data: &str) -> String {
        urlencoding::encode(data).to_string()
    }
    
    pub fn decode(encoded: &str) -> Result<String, String> {
        urlencoding::decode(encoded)
            .map(|s| s.to_string())
            .map_err(|e| format!("URL decode error: {}", e))
    }
    
    pub fn double_encode(data: &str) -> String {
        let once = Self::encode(data);
        Self::encode(&once)
    }
    
    pub fn encode_all(data: &str) -> String {
        data.bytes()
            .map(|b| format!("%{:02X}", b))
            .collect()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// HTML ENCODING/DECODING
// ────────────────────────────────────────────────────────────────────────────

pub struct HTMLEncoder;

impl HTMLEncoder {
    pub fn encode(data: &str) -> String {
        data.chars()
            .map(|c| match c {
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '&' => "&amp;".to_string(),
                '"' => "&quot;".to_string(),
                '\'' => "&#39;".to_string(),
                _ => c.to_string(),
            })
            .collect()
    }
    
    pub fn decode(encoded: &str) -> String {
        encoded
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&#x27;", "'")
    }
    
    pub fn encode_decimal(data: &str) -> String {
        data.chars()
            .map(|c| format!("&#{};", c as u32))
            .collect()
    }
    
    pub fn encode_hex(data: &str) -> String {
        data.chars()
            .map(|c| format!("&#x{:x};", c as u32))
            .collect()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// UNICODE ENCODING
// ────────────────────────────────────────────────────────────────────────────

pub struct UnicodeEncoder;

impl UnicodeEncoder {
    pub fn to_unicode_escape(data: &str) -> String {
        data.chars()
            .map(|c| {
                if c.is_ascii() {
                    c.to_string()
                } else {
                    format!("\\u{{{:04x}}}", c as u32)
                }
            })
            .collect()
    }
    
    pub fn to_utf16_hex(data: &str) -> String {
        data.encode_utf16()
            .map(|u| format!("{:04x}", u))
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    pub fn from_unicode_escape(encoded: &str) -> Result<String, String> {
        let mut result = String::new();
        let mut chars = encoded.chars().peekable();
        
        while let Some(c) = chars.next() {
            if c == '\\' && chars.peek() == Some(&'u') {
                chars.next();
                if chars.peek() == Some(&'{') {
                    chars.next();
                    let mut hex = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch == '}' {
                            chars.next();
                            break;
                        }
                        hex.push(ch);
                        chars.next();
                    }
                    
                    let code = u32::from_str_radix(&hex, 16)
                        .map_err(|e| format!("Invalid unicode escape: {}", e))?;
                    let character = char::from_u32(code)
                        .ok_or_else(|| "Invalid unicode code point".to_string())?;
                    result.push(character);
                }
            } else {
                result.push(c);
            }
        }
        
        Ok(result)
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ROT CIPHER
// ────────────────────────────────────────────────────────────────────────────

pub struct ROTCipher;

impl ROTCipher {
    pub fn rot13(text: &str) -> String {
        Self::rotn(text, 13)
    }
    
    pub fn rotn(text: &str, n: u8) -> String {
        text.chars()
            .map(|c| {
                if c.is_ascii_alphabetic() {
                    let base = if c.is_ascii_lowercase() { b'a' } else { b'A' };
                    let offset = (c as u8 - base + n) % 26;
                    (base + offset) as char
                } else {
                    c
                }
            })
            .collect()
    }
    
    pub fn rot_all(text: &str) -> HashMap<u8, String> {
        let mut results = HashMap::new();
        
        for i in 0..26 {
            results.insert(i, Self::rotn(text, i));
        }
        
        results
    }
}

// ────────────────────────────────────────────────────────────────────────────
// MORSE CODE
// ────────────────────────────────────────────────────────────────────────────

pub struct MorseCode;

impl MorseCode {
    fn get_morse_map() -> HashMap<char, &'static str> {
        [
            ('A', ".-"), ('B', "-..."), ('C', "-.-."), ('D', "-.."),
            ('E', "."), ('F', "..-."), ('G', "--."), ('H', "...."),
            ('I', ".."), ('J', ".---"), ('K', "-.-"), ('L', ".-.."),
            ('M', "--"), ('N', "-."), ('O', "---"), ('P', ".--."),
            ('Q', "--.-"), ('R', ".-."), ('S', "..."), ('T', "-"),
            ('U', "..-"), ('V', "...-"), ('W', ".--"), ('X', "-..-"),
            ('Y', "-.--"), ('Z', "--.."),
            ('0', "-----"), ('1', ".----"), ('2', "..---"), ('3', "...--"),
            ('4', "....-"), ('5', "....."), ('6', "-...."), ('7', "--..."),
            ('8', "---.."), ('9', "----."),
            (' ', "/"),
        ].iter().cloned().collect()
    }
    
    fn get_reverse_map() -> HashMap<&'static str, char> {
        Self::get_morse_map()
            .iter()
            .map(|(&k, &v)| (v, k))
            .collect()
    }
    
    pub fn encode(text: &str) -> String {
        let map = Self::get_morse_map();
        
        text.to_uppercase()
            .chars()
            .filter_map(|c| map.get(&c)).copied()
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    pub fn decode(morse: &str) -> Result<String, String> {
        let map = Self::get_reverse_map();
        
        morse.split_whitespace()
            .map(|code| {
                map.get(code)
                    .copied()
                    .ok_or_else(|| format!("Invalid morse code: {}", code))
            })
            .collect()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// JWT TOKEN MANIPULATION
// ────────────────────────────────────────────────────────────────────────────

pub struct JWTHelper;

impl JWTHelper {
    pub fn decode_token(token: &str) -> Result<(String, String, String), String> {
        let parts: Vec<&str> = token.split('.').collect();
        
        if parts.len() != 3 {
            return Err("Invalid JWT format".to_string());
        }
        
        let header = BaseEncoder::base64_url_decode(parts[0])?;
        let payload = BaseEncoder::base64_url_decode(parts[1])?;
        
        let header_str = String::from_utf8(header)
            .map_err(|e| format!("Invalid UTF-8 in header: {}", e))?;
        let payload_str = String::from_utf8(payload)
            .map_err(|e| format!("Invalid UTF-8 in payload: {}", e))?;
        
        Ok((header_str, payload_str, parts[2].to_string()))
    }
    
    pub fn create_unsigned(header: &str, payload: &str) -> String {
        let header_encoded = BaseEncoder::base64_url_encode(header.as_bytes());
        let payload_encoded = BaseEncoder::base64_url_encode(payload.as_bytes());
        
        format!("{}.{}.", header_encoded, payload_encoded)
    }
    
    pub fn analyze(token: &str) -> Result<(), String> {
        println!("[JWT] Analyzing token...");
        
        let (header, payload, signature) = Self::decode_token(token)?;
        
        println!("\n[JWT] Header:");
        println!("{}", header);
        
        println!("\n[JWT] Payload:");
        println!("{}", payload);
        
        println!("\n[JWT] Signature:");
        println!("{}", signature);
        
        if signature.is_empty() {
            println!("\n[JWT] WARNING: Token is UNSIGNED (algorithm: none)");
        }
        
        Ok(())
    }
}

// ────────────────────────────────────────────────────────────────────────────
// BINARY/ASCII CONVERSION
// ────────────────────────────────────────────────────────────────────────────

pub struct BinaryConverter;

impl BinaryConverter {
    pub fn to_binary(text: &str) -> String {
        text.bytes()
            .map(|b| format!("{:08b}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    pub fn from_binary(binary: &str) -> Result<String, String> {
        binary.split_whitespace()
            .map(|b| {
                u8::from_str_radix(b, 2)
                    .map(|byte| byte as char)
                    .map_err(|e| format!("Invalid binary: {}", e))
            })
            .collect()
    }
    
    pub fn to_octal(data: &[u8]) -> String {
        data.iter()
            .map(|&b| format!("{:03o}", b))
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    pub fn from_octal(octal: &str) -> Result<Vec<u8>, String> {
        octal.split_whitespace()
            .map(|o| {
                u8::from_str_radix(o, 8)
                    .map_err(|e| format!("Invalid octal: {}", e))
            })
            .collect()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// CUSTOM ALPHABET SUBSTITUTION
// ────────────────────────────────────────────────────────────────────────────

pub struct SubstitutionCipher {
    alphabet: String,
    key: String,
}

impl SubstitutionCipher {
    pub fn new(key: &str) -> Result<Self, String> {
        if key.len() != 26 {
            return Err("Key must be 26 characters".to_string());
        }
        
        Ok(SubstitutionCipher {
            alphabet: "ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string(),
            key: key.to_uppercase(),
        })
    }
    
    pub fn encode(&self, text: &str) -> String {
        let map: HashMap<char, char> = self.alphabet.chars()
            .zip(self.key.chars())
            .collect();
        
        text.to_uppercase()
            .chars()
            .map(|c| {
                if c.is_ascii_alphabetic() {
                    *map.get(&c).unwrap_or(&c)
                } else {
                    c
                }
            })
            .collect()
    }
    
    pub fn decode(&self, text: &str) -> String {
        let map: HashMap<char, char> = self.key.chars()
            .zip(self.alphabet.chars())
            .collect();
        
        text.to_uppercase()
            .chars()
            .map(|c| {
                if c.is_ascii_alphabetic() {
                    *map.get(&c).unwrap_or(&c)
                } else {
                    c
                }
            })
            .collect()
    }
}

// ────────────────────────────────────────────────────────────────────────────
// ALL-IN-ONE DECODER
// ────────────────────────────────────────────────────────────────────────────

pub struct UniversalDecoder;

impl UniversalDecoder {
    pub fn try_all(data: &str) -> Vec<(String, String)> {
        let mut results = Vec::new();
        
        if let Ok(decoded) = BaseEncoder::base64_decode(data) {
            if let Ok(text) = String::from_utf8(decoded) {
                results.push(("Base64".to_string(), text));
            }
        }
        
        if let Ok(decoded) = BaseEncoder::base32_decode(data) {
            if let Ok(text) = String::from_utf8(decoded) {
                results.push(("Base32".to_string(), text));
            }
        }
        
        if let Ok(decoded) = URLEncoder::decode(data) {
            results.push(("URL".to_string(), decoded));
        }
        
        results.push(("ROT13".to_string(), ROTCipher::rot13(data)));
        
        if let Ok(decoded) = MorseCode::decode(data) {
            results.push(("Morse".to_string(), decoded));
        }
        
        let html_decoded = HTMLEncoder::decode(data);
        if html_decoded != data {
            results.push(("HTML".to_string(), html_decoded));
        }
        
        if let Ok(decoded) = BaseEncoder::hex_decode(&data.replace(" ", "")) {
            if let Ok(text) = String::from_utf8(decoded) {
                results.push(("Hex".to_string(), text));
            }
        }
        
        println!("[UNIVERSAL-DECODE] Attempted {} decodings", results.len());
        for (method, result) in &results {
            println!("[UNIVERSAL-DECODE] {}: {}", method, 
                &result.chars().take(50).collect::<String>());
        }
        
        results
    }
}
