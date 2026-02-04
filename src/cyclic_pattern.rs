// Cyclic pattern generation and offset finding for buffer overflow exploitation
// De Bruijn sequence implementation for precise offset calculation

const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const ALPHABET_SIZE: usize = 26;
const SUBSEQUENCE_LENGTH: usize = 4;

pub struct CyclicPattern {
    alphabet: Vec<u8>,
    n: usize,
}

impl CyclicPattern {
    pub fn new() -> Self {
        CyclicPattern {
            alphabet: ALPHABET.to_vec(),
            n: SUBSEQUENCE_LENGTH,
        }
    }

    pub fn generate(&self, length: usize) -> Vec<u8> {
        let mut pattern = Vec::with_capacity(length);
        let mut sequence = vec![0; self.n];
        let alphabet_len = self.alphabet.len();

        let mut a = vec![0; self.n * alphabet_len];

        fn db(
            t: usize,
            p: usize,
            sequence: &mut [usize],
            a: &mut [usize],
            alphabet_len: usize,
            n: usize,
            result: &mut Vec<u8>,
            alphabet: &[u8],
            max_len: usize,
        ) {
            if result.len() >= max_len {
                return;
            }

            if t > n {
                if n % p == 0 {
                    for j in 1..=p {
                        if result.len() >= max_len {
                            return;
                        }
                        if j < sequence.len() {
                            result.push(alphabet[sequence[j]]);
                        }
                    }
                }
            } else {
                if t < a.len() && (t - p) < a.len() {
                    a[t] = a[t - p];
                    db(
                        t + 1,
                        p,
                        sequence,
                        a,
                        alphabet_len,
                        n,
                        result,
                        alphabet,
                        max_len,
                    );

                    for j in (a[t - p] + 1)..alphabet_len {
                        if result.len() >= max_len {
                            return;
                        }
                        if t < a.len() && t < sequence.len() {
                            a[t] = j;
                            sequence[t] = j;
                            db(
                                t + 1,
                                t,
                                sequence,
                                a,
                                alphabet_len,
                                n,
                                result,
                                alphabet,
                                max_len,
                            );
                        }
                    }
                }
            }
        }

        db(
            1,
            1,
            &mut sequence,
            &mut a,
            alphabet_len,
            self.n,
            &mut pattern,
            &self.alphabet,
            length,
        );

        pattern.truncate(length);
        pattern
    }

    pub fn find_offset(&self, pattern_bytes: &[u8], search: &[u8]) -> Option<usize> {
        if search.len() < SUBSEQUENCE_LENGTH {
            return None;
        }

        let search_slice = &search[..SUBSEQUENCE_LENGTH.min(search.len())];

        for (i, window) in pattern_bytes.windows(search_slice.len()).enumerate() {
            if window == search_slice {
                return Some(i);
            }
        }

        None
    }

    pub fn find_offset_from_u64(&self, pattern_bytes: &[u8], value: u64) -> Option<usize> {
        let bytes = value.to_le_bytes();

        for start in 0..=8 {
            let end = (start + 4).min(8);
            if end - start < 4 {
                continue;
            }

            let slice = &bytes[start..end];
            if let Some(offset) = self.find_offset(pattern_bytes, slice) {
                return Some(offset);
            }
        }

        None
    }

    pub fn find_offset_from_string(&self, pattern_bytes: &[u8], s: &str) -> Option<usize> {
        self.find_offset(pattern_bytes, s.as_bytes())
    }
}

pub fn cyclic(length: usize) -> Vec<u8> {
    use colored::Colorize;

    if length == 0 {
        eprintln!(
            "{} {}",
            "[WARNING]".yellow(),
            "Requested cyclic pattern length is 0, returning empty pattern".bright_black()
        );
        return Vec::new();
    }
    if length > 1_000_000 {
        eprintln!(
            "{} {}",
            "[WARNING]".yellow(),
            format!(
                "Requested pattern length {} exceeds 1MB, this may take a while",
                length
            )
            .yellow()
        );
    }

    let generator = CyclicPattern::new();
    generator.generate(length)
}

pub fn cyclic_find(pattern_bytes: &[u8], search_value: &str) -> Option<usize> {
    use colored::Colorize;

    if pattern_bytes.is_empty() {
        eprintln!(
            "{} {}",
            "[ERROR]".red(),
            "Cannot search in empty pattern".red()
        );
        return None;
    }

    if search_value.is_empty() {
        eprintln!(
            "{} {}",
            "[ERROR]".red(),
            "Search value cannot be empty".red()
        );
        return None;
    }

    let generator = CyclicPattern::new();

    if search_value.starts_with("0x") {
        if let Ok(value) = u64::from_str_radix(&search_value[2..], 16) {
            let result = generator.find_offset_from_u64(pattern_bytes, value);
            if result.is_none() {
                eprintln!(
                    "{} {}",
                    "[INFO]".yellow(),
                    format!(
                        "Value {} not found in pattern. Pattern may be too short.",
                        search_value
                    )
                    .cyan()
                );
            }
            return result;
        }
    }

    if let Ok(value) = search_value.parse::<u64>() {
        let result = generator.find_offset_from_u64(pattern_bytes, value);
        if result.is_none() {
            eprintln!(
                "{} {}",
                "[INFO]".yellow(),
                format!(
                    "Value {} not found in pattern. Pattern may be too short.",
                    search_value
                )
                .cyan()
            );
        }
        return result;
    }

    let result = generator.find_offset_from_string(pattern_bytes, search_value);
    if result.is_none() {
        eprintln!(
            "{} {}",
            "[INFO]".yellow(),
            format!(
                "String '{}' not found in pattern. Check for typos or generate a longer pattern.",
                search_value
            )
            .cyan()
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cyclic_generation() {
        let pattern = cyclic(100);
        assert_eq!(pattern.len(), 100);

        assert!(pattern
            .windows(4)
            .enumerate()
            .all(|(i, w)| { pattern.windows(4).skip(i + 1).all(|other| w != other) }));
    }

    #[test]
    fn test_cyclic_find() {
        let pattern = cyclic(1000);

        let offset_100 = &pattern[100..104];
        let found = cyclic_find(&pattern, std::str::from_utf8(offset_100).unwrap());
        assert_eq!(found, Some(100));

        let offset_500 = &pattern[500..504];
        let found = cyclic_find(&pattern, std::str::from_utf8(offset_500).unwrap());
        assert_eq!(found, Some(500));
    }

    #[test]
    fn test_cyclic_find_u64() {
        let pattern = cyclic(1000);
        let generator = CyclicPattern::new();

        let offset = 72;
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&pattern[offset..offset + 8]);
        let value = u64::from_le_bytes(bytes);

        let found = generator.find_offset_from_u64(&pattern, value);
        assert!(found.is_some());
        assert!(found.unwrap() >= offset && found.unwrap() <= offset + 4);
    }
}
