#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data.len() > 10_000 {
        return;
    }

    let _ = talon::crypto_tools::sha256(data);
    let _ = talon::crypto_tools::sha1(data);
    let _ = talon::crypto_tools::md5(data);

    if data.len() >= 32 {
        let key = &data[0..32];
        if data.len() > 32 {
            let plaintext = &data[32..];
            let _ = talon::crypto_tools::aes_encrypt(plaintext, key);

            if let Ok(ciphertext) = talon::crypto_tools::aes_encrypt(plaintext, key) {
                let _ = talon::crypto_tools::aes_decrypt(&ciphertext, key);
            }
        }
    }

    if data.len() >= 16 {
        let key = data[0];
        let _ = talon::crypto_tools::xor_encrypt(&data[16..], key);
    }

    if let Ok(s) = std::str::from_utf8(data) {
        let _ = talon::crypto_tools::base64_encode(data);
        let _ = talon::crypto_tools::base64_decode(s);
        let _ = talon::crypto_tools::hex_encode(data);
        let _ = talon::crypto_tools::hex_decode(s);
    }
});
