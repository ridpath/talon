// ═══════════════════════════════════════════════════════════════════════════
// CYCLIC PATTERN TOOLKIT - DE BRUIJN SEQUENCES
// ═══════════════════════════════════════════════════════════════════════════
// Provides pwntools-style cyclic patterns for finding buffer overflow offsets

const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const ALPHABET_SIZE: usize = 26;
const SUBSEQUENCE_LENGTH: usize = 4; // 4-byte patterns for 32/64-bit addresses

/// Generate a De Bruijn sequence (cyclic pattern)
///
/// # Arguments
/// * `length` - Desired length of the pattern
///
/// # Returns
/// A byte vector containing the cyclic pattern
///
/// # Example
/// ```
/// let pattern = cyclic(200);
/// // pattern = "aaaabaaacaaadaaa..."
/// ```
pub fn cyclic(length: usize) -> Vec<u8> {
    let mut result = Vec::with_capacity(length);
    let mut sequence = vec![0u8; SUBSEQUENCE_LENGTH];
    let mut a = vec![0usize; ALPHABET_SIZE * SUBSEQUENCE_LENGTH];

    de_bruijn(
        &mut sequence,
        &mut a,
        1,
        1,
        ALPHABET_SIZE,
        SUBSEQUENCE_LENGTH,
        &mut result,
    );

    // Repeat pattern if needed to reach desired length
    while result.len() < length {
        result.push(result[result.len() % (ALPHABET_SIZE.pow(SUBSEQUENCE_LENGTH as u32))]);
    }

    result.truncate(length);
    result
}

/// Find the offset of a pattern in a cyclic sequence
///
/// # Arguments
/// * `value` - The 4-byte value found at crash (e.g., from register)
///
/// # Returns
/// The offset where this pattern appears
///
/// # Example
/// ```
/// let offset = cyclic_find(0x61616162); // "baaa" in little-endian
/// // offset = 4
/// ```
pub fn cyclic_find(value: u64) -> Option<usize> {
    // Convert value to bytes (little-endian)
    let bytes = if value <= 0xFFFFFFFF {
        // 32-bit value
        (value as u32).to_le_bytes().to_vec()
    } else {
        // 64-bit value - take first 4 bytes
        value.to_le_bytes()[0..4].to_vec()
    };

    cyclic_find_bytes(&bytes)
}

/// Find the offset of a byte pattern in a cyclic sequence
///
/// # Arguments
/// * `pattern` - The byte pattern to find
///
/// # Returns
/// The offset where this pattern first appears
pub fn cyclic_find_bytes(pattern: &[u8]) -> Option<usize> {
    if pattern.len() < 4 {
        return None;
    }

    // Generate a large cyclic pattern to search in
    let search_pattern = cyclic(20000);

    // Search for the pattern
    for (i, window) in search_pattern.windows(pattern.len()).enumerate() {
        if window == pattern {
            return Some(i);
        }
    }

    None
}

/// Find offset from a hex string
///
/// # Example
/// ```
/// let offset = cyclic_find_hex("0x61616162");
/// ```
pub fn cyclic_find_hex(hex_str: &str) -> Option<usize> {
    let hex_str = hex_str.trim_start_matches("0x");
    if let Ok(value) = u64::from_str_radix(hex_str, 16) {
        cyclic_find(value)
    } else {
        None
    }
}

/// Internal De Bruijn sequence generator using FKM algorithm
fn de_bruijn(
    sequence: &mut [u8],
    a: &mut [usize],
    t: usize,
    p: usize,
    k: usize,
    n: usize,
    result: &mut Vec<u8>,
) {
    if t > n {
        if n % p == 0 {
            for j in 1..=p {
                result.push(ALPHABET[a[j]]);
            }
        }
    } else {
        a[t] = a[t - p];
        de_bruijn(sequence, a, t + 1, p, k, n, result);

        for j in (a[t - p] + 1)..k {
            a[t] = j;
            de_bruijn(sequence, a, t + 1, t, k, n, result);
        }
    }
}

/// Generate a cyclic pattern with a custom alphabet
pub fn cyclic_custom(length: usize, alphabet: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(length);
    let alpha_len = alphabet.len();

    for i in 0..length {
        let idx = (i / (alpha_len * alpha_len * alpha_len)) % alpha_len;
        result.push(alphabet[idx]);
    }

    // Use a simpler pattern for custom alphabets
    for i in 0..length {
        let a = i % alpha_len;
        let b = (i / alpha_len) % alpha_len;
        let c = (i / (alpha_len * alpha_len)) % alpha_len;
        let d = (i / (alpha_len * alpha_len * alpha_len)) % alpha_len;

        if result.len() >= length {
            break;
        }
        result[i] = alphabet[(a + b + c + d) % alpha_len];
    }

    result.truncate(length);
    result
}

/// Print a cyclic pattern in a formatted way
pub fn cyclic_display(pattern: &[u8], width: usize) -> String {
    let mut result = String::new();
    for (i, chunk) in pattern.chunks(width).enumerate() {
        result.push_str(&format!("{:04x}: ", i * width));
        for byte in chunk {
            result.push(*byte as char);
        }
        result.push('\n');
    }
    result
}

/// Determine the padding needed for a buffer overflow
///
/// # Arguments
/// * `binary_path` - Path to the vulnerable binary
/// * `max_size` - Maximum pattern size to try
///
/// # Returns
/// The offset to control EIP/RIP
pub fn find_overflow_offset(binary_path: &str, max_size: usize) -> Result<usize, String> {
    log::info!("Finding overflow offset for {}", binary_path);
    log::info!("Generating cyclic pattern of {} bytes", max_size);

    let _pattern = cyclic(max_size);

    // In a real implementation, this would:
    // 1. Run the binary with the pattern as input
    // 2. Capture the crash address
    // 3. Find the offset using cyclic_find()

    // For now, return a placeholder
    log::warn!("Automatic offset detection requires process execution - returning manual mode");
    Err(
        "Manual mode: Run binary with cyclic() pattern, then use cyclic_find(crash_addr)"
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cyclic_generation() {
        let pattern = cyclic(100);
        assert_eq!(pattern.len(), 100);

        // Should start with "aaaa"
        assert_eq!(&pattern[0..4], b"aaaa");
    }

    #[test]
    fn test_cyclic_find() {
        // Generate pattern
        let pattern = cyclic(300);

        // Extract a 4-byte sequence from offset 264
        let bytes = &pattern[264..268];
        let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        // Find it
        let offset = cyclic_find(value as u64);
        assert_eq!(offset, Some(264));
    }

    #[test]
    fn test_cyclic_find_bytes() {
        let pattern = cyclic(300);
        let search = &pattern[100..104];
        let offset = cyclic_find_bytes(search);
        assert_eq!(offset, Some(100));
    }

    #[test]
    fn test_cyclic_uniqueness() {
        let pattern = cyclic(1000);

        // Check that 4-byte windows are unique (at least for first 500 bytes)
        let mut seen = std::collections::HashSet::new();
        for window in pattern[..500].windows(4) {
            let key = u32::from_le_bytes([window[0], window[1], window[2], window[3]]);
            assert!(!seen.contains(&key), "Duplicate pattern found!");
            seen.insert(key);
        }
    }
}
