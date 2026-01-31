use talon::encoding_tools::*;

#[test]
fn test_base64_encode() {
    assert_eq!(BaseEncoder::base64_encode(b"hello"), "aGVsbG8=");
    assert_eq!(BaseEncoder::base64_encode(b"test"), "dGVzdA==");
    assert_eq!(BaseEncoder::base64_encode(b""), "");
    assert_eq!(BaseEncoder::base64_encode(b"a"), "YQ==");
}

#[test]
fn test_base64_decode() {
    assert_eq!(BaseEncoder::base64_decode("aGVsbG8=").unwrap(), b"hello");
    assert_eq!(BaseEncoder::base64_decode("dGVzdA==").unwrap(), b"test");
    assert_eq!(BaseEncoder::base64_decode("").unwrap(), b"");
    assert_eq!(BaseEncoder::base64_decode("YQ==").unwrap(), b"a");
}

#[test]
fn test_base64_decode_invalid() {
    assert!(BaseEncoder::base64_decode("!!!").is_err());
    assert!(BaseEncoder::base64_decode("not base64").is_err());
}

#[test]
fn test_base64_url_encode() {
    let data = b"hello\xff\xfe";
    let encoded = BaseEncoder::base64_url_encode(data);
    assert!(!encoded.contains('+'));
    assert!(!encoded.contains('/'));
}

#[test]
fn test_base64_url_decode() {
    let encoded = BaseEncoder::base64_url_encode(b"test data");
    let decoded = BaseEncoder::base64_url_decode(&encoded).unwrap();
    assert_eq!(decoded, b"test data");
}

#[test]
fn test_base64_roundtrip() {
    let original = b"The quick brown fox jumps over the lazy dog";
    let encoded = BaseEncoder::base64_encode(original);
    let decoded = BaseEncoder::base64_decode(&encoded).unwrap();
    assert_eq!(&decoded, original);
}

#[test]
fn test_base64_url_roundtrip() {
    let original = b"URL safe encoding test!@#$%^&*()";
    let encoded = BaseEncoder::base64_url_encode(original);
    let decoded = BaseEncoder::base64_url_decode(&encoded).unwrap();
    assert_eq!(&decoded, original);
}

#[test]
fn test_base32_encode() {
    assert_eq!(BaseEncoder::base32_encode(b"hello"), "NBSWY3DP");
    assert_eq!(BaseEncoder::base32_encode(b"test"), "ORSXG5A=");
    assert_eq!(BaseEncoder::base32_encode(b"a"), "ME======");
}

#[test]
fn test_base32_decode() {
    assert_eq!(BaseEncoder::base32_decode("NBSWY3DP").unwrap(), b"hello");
    assert_eq!(BaseEncoder::base32_decode("ORSXG5A=").unwrap(), b"test");
    assert_eq!(BaseEncoder::base32_decode("ME======").unwrap(), b"a");
}

#[test]
fn test_base32_decode_invalid() {
    assert!(BaseEncoder::base32_decode("!!!").is_err());
    assert!(BaseEncoder::base32_decode("89").is_err());
}

#[test]
fn test_base32_roundtrip() {
    let original = b"Base32 encoding test";
    let encoded = BaseEncoder::base32_encode(original);
    let decoded = BaseEncoder::base32_decode(&encoded).unwrap();
    assert_eq!(&decoded, original);
}

#[test]
fn test_hex_encode() {
    assert_eq!(BaseEncoder::hex_encode(b"hello"), "68656c6c6f");
    assert_eq!(BaseEncoder::hex_encode(b"\xde\xad\xbe\xef"), "deadbeef");
    assert_eq!(BaseEncoder::hex_encode(b""), "");
    assert_eq!(BaseEncoder::hex_encode(b"\x00"), "00");
    assert_eq!(BaseEncoder::hex_encode(b"\xff"), "ff");
}

#[test]
fn test_hex_decode() {
    assert_eq!(BaseEncoder::hex_decode("68656c6c6f").unwrap(), b"hello");
    assert_eq!(
        BaseEncoder::hex_decode("deadbeef").unwrap(),
        b"\xde\xad\xbe\xef"
    );
    assert_eq!(BaseEncoder::hex_decode("").unwrap(), b"");
    assert_eq!(BaseEncoder::hex_decode("00").unwrap(), b"\x00");
}

#[test]
fn test_hex_decode_invalid() {
    assert!(BaseEncoder::hex_decode("zz").is_err());
    assert!(BaseEncoder::hex_decode("not hex").is_err());
    assert!(BaseEncoder::hex_decode("abc").is_err());
}

#[test]
fn test_hex_roundtrip() {
    let original = b"Hex encoding test \x00\xff\xaa\x55";
    let encoded = BaseEncoder::hex_encode(original);
    let decoded = BaseEncoder::hex_decode(&encoded).unwrap();
    assert_eq!(&decoded, original);
}

#[test]
fn test_url_encode() {
    assert_eq!(URLEncoder::encode("hello world"), "hello%20world");
    assert_eq!(URLEncoder::encode("test@example.com"), "test%40example.com");
    assert_eq!(URLEncoder::encode("a+b=c"), "a%2Bb%3Dc");
    assert_eq!(URLEncoder::encode("plain"), "plain");
}

#[test]
fn test_url_decode() {
    assert_eq!(URLEncoder::decode("hello%20world").unwrap(), "hello world");
    assert_eq!(
        URLEncoder::decode("test%40example.com").unwrap(),
        "test@example.com"
    );
    assert_eq!(URLEncoder::decode("a%2Bb%3Dc").unwrap(), "a+b=c");
    assert_eq!(URLEncoder::decode("plain").unwrap(), "plain");
}

#[test]
fn test_url_roundtrip() {
    let original = "Hello World! @#$%^&*()";
    let encoded = URLEncoder::encode(original);
    let decoded = URLEncoder::decode(&encoded).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_url_double_encode() {
    let result = URLEncoder::double_encode("test value");
    assert!(result.contains("%25"));
}

#[test]
fn test_url_encode_all() {
    let result = URLEncoder::encode_all("ABC");
    assert_eq!(result, "%41%42%43");

    let result = URLEncoder::encode_all("a");
    assert_eq!(result, "%61");
}

#[test]
fn test_html_encode() {
    assert_eq!(HTMLEncoder::encode("<script>"), "&lt;script&gt;");
    assert_eq!(HTMLEncoder::encode("A&B"), "A&amp;B");
    assert_eq!(HTMLEncoder::encode("\"quoted\""), "&quot;quoted&quot;");
    assert_eq!(HTMLEncoder::encode("it's"), "it&#39;s");
    assert_eq!(HTMLEncoder::encode("plain"), "plain");
}

#[test]
fn test_html_decode() {
    assert_eq!(HTMLEncoder::decode("&lt;script&gt;"), "<script>");
    assert_eq!(HTMLEncoder::decode("A&amp;B"), "A&B");
    assert_eq!(HTMLEncoder::decode("&quot;quoted&quot;"), "\"quoted\"");
    assert_eq!(HTMLEncoder::decode("&#39;test&#39;"), "'test'");
    assert_eq!(HTMLEncoder::decode("plain"), "plain");
}

#[test]
fn test_html_roundtrip() {
    let original = "<div>Hello & \"goodbye\"</div>";
    let encoded = HTMLEncoder::encode(original);
    let decoded = HTMLEncoder::decode(&encoded);
    assert_eq!(decoded, original);
}

#[test]
fn test_html_encode_decimal() {
    let result = HTMLEncoder::encode_decimal("ABC");
    assert_eq!(result, "&#65;&#66;&#67;");
}

#[test]
fn test_html_encode_hex() {
    let result = HTMLEncoder::encode_hex("ABC");
    assert_eq!(result, "&#x41;&#x42;&#x43;");
}

#[test]
fn test_unicode_to_escape() {
    assert_eq!(UnicodeEncoder::to_unicode_escape("hello"), "hello");
    assert_eq!(UnicodeEncoder::to_unicode_escape("café"), "caf\\u{00e9}");
    assert_eq!(
        UnicodeEncoder::to_unicode_escape("日本"),
        "\\u{65e5}\\u{672c}"
    );
}

#[test]
fn test_unicode_from_escape() {
    assert_eq!(
        UnicodeEncoder::from_unicode_escape("hello").unwrap(),
        "hello"
    );
    assert_eq!(
        UnicodeEncoder::from_unicode_escape("caf\\u{00e9}").unwrap(),
        "café"
    );
    assert_eq!(
        UnicodeEncoder::from_unicode_escape("\\u{65e5}\\u{672c}").unwrap(),
        "日本"
    );
}

#[test]
fn test_unicode_roundtrip() {
    let original = "Hello 世界 ";
    let escaped = UnicodeEncoder::to_unicode_escape(original);
    let decoded = UnicodeEncoder::from_unicode_escape(&escaped).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_unicode_to_utf16_hex() {
    let result = UnicodeEncoder::to_utf16_hex("A");
    assert_eq!(result, "0041");

    let result = UnicodeEncoder::to_utf16_hex("AB");
    assert_eq!(result, "0041 0042");
}

#[test]
fn test_rot13() {
    assert_eq!(ROTCipher::rot13("hello"), "uryyb");
    assert_eq!(ROTCipher::rot13("HELLO"), "URYYB");
    assert_eq!(ROTCipher::rot13("abc123"), "nop123");
}

#[test]
fn test_rot13_roundtrip() {
    let original = "The Quick Brown Fox";
    let encoded = ROTCipher::rot13(original);
    let decoded = ROTCipher::rot13(&encoded);
    assert_eq!(decoded, original);
}

#[test]
fn test_rotn() {
    assert_eq!(ROTCipher::rotn("abc", 1), "bcd");
    assert_eq!(ROTCipher::rotn("xyz", 3), "abc");
    assert_eq!(ROTCipher::rotn("ABC", 5), "FGH");
    assert_eq!(ROTCipher::rotn("abc123", 13), "nop123");
}

#[test]
fn test_rotn_full_cycle() {
    let original = "test";
    let encoded = ROTCipher::rotn(original, 26);
    assert_eq!(encoded, original);
}

#[test]
fn test_rot_all() {
    let results = ROTCipher::rot_all("abc");
    assert_eq!(results.len(), 26);
    assert_eq!(results.get(&0).unwrap(), "abc");
    assert_eq!(results.get(&1).unwrap(), "bcd");
    assert_eq!(results.get(&13).unwrap(), "nop");
}

#[test]
fn test_morse_encode() {
    assert_eq!(MorseCode::encode("SOS"), "... --- ...");
    assert_eq!(MorseCode::encode("HELLO"), ".... . .-.. .-.. ---");
    assert_eq!(MorseCode::encode("ABC"), ".- -... -.-.");
    assert_eq!(MorseCode::encode("123"), ".---- ..--- ...--");
}

#[test]
fn test_morse_decode() {
    assert_eq!(MorseCode::decode("... --- ...").unwrap(), "SOS");
    assert_eq!(MorseCode::decode(".... . .-.. .-.. ---").unwrap(), "HELLO");
    assert_eq!(MorseCode::decode(".- -... -.-.").unwrap(), "ABC");
}

#[test]
fn test_morse_roundtrip() {
    let original = "HELLO WORLD";
    let encoded = MorseCode::encode(original);
    let decoded = MorseCode::decode(&encoded).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_morse_decode_invalid() {
    assert!(MorseCode::decode("........ invalid").is_err());
}

#[test]
fn test_jwt_decode() {
    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.signature";
    let result = JWTHelper::decode_token(token);
    assert!(result.is_ok());

    let (header, payload, sig) = result.unwrap();
    assert!(header.contains("alg"));
    assert!(payload.contains("sub"));
    assert_eq!(sig, "signature");
}

#[test]
fn test_jwt_decode_invalid_format() {
    assert!(JWTHelper::decode_token("not.a.jwt.token").is_err());
    assert!(JWTHelper::decode_token("only.two").is_err());
}

#[test]
fn test_jwt_create_unsigned() {
    let token = JWTHelper::create_unsigned(r#"{"alg":"none"}"#, r#"{"sub":"test"}"#);
    assert!(token.ends_with('.'));

    let parts: Vec<&str> = token.split('.').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[2], "");
}

#[test]
fn test_binary_to_binary() {
    assert_eq!(BinaryConverter::to_binary("A"), "01000001");
    assert_eq!(BinaryConverter::to_binary("AB"), "01000001 01000010");
    assert_eq!(BinaryConverter::to_binary(""), "");
}

#[test]
fn test_binary_from_binary() {
    assert_eq!(BinaryConverter::from_binary("01000001").unwrap(), "A");
    assert_eq!(
        BinaryConverter::from_binary("01000001 01000010").unwrap(),
        "AB"
    );
}

#[test]
fn test_binary_roundtrip() {
    let original = "Hello!";
    let binary = BinaryConverter::to_binary(original);
    let decoded = BinaryConverter::from_binary(&binary).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_binary_from_binary_invalid() {
    assert!(BinaryConverter::from_binary("11111111").is_err());
    assert!(BinaryConverter::from_binary("12345678").is_err());
}

#[test]
fn test_binary_to_octal() {
    assert_eq!(BinaryConverter::to_octal(b"A"), "101");
    assert_eq!(BinaryConverter::to_octal(b"ABC"), "101 102 103");
}

#[test]
fn test_binary_from_octal() {
    assert_eq!(BinaryConverter::from_octal("101").unwrap(), b"A");
    assert_eq!(BinaryConverter::from_octal("101 102 103").unwrap(), b"ABC");
}

#[test]
fn test_octal_roundtrip() {
    let original = b"Test123";
    let octal = BinaryConverter::to_octal(original);
    let decoded = BinaryConverter::from_octal(&octal).unwrap();
    assert_eq!(&decoded, original);
}

#[test]
fn test_substitution_cipher_new() {
    let cipher = SubstitutionCipher::new("BCDEFGHIJKLMNOPQRSTUVWXYZA");
    assert!(cipher.is_ok());
}

#[test]
fn test_substitution_cipher_invalid_length() {
    assert!(SubstitutionCipher::new("ABC").is_err());
    assert!(SubstitutionCipher::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ123").is_err());
}

#[test]
fn test_substitution_cipher_encode() {
    let cipher = SubstitutionCipher::new("BCDEFGHIJKLMNOPQRSTUVWXYZA").unwrap();
    assert_eq!(cipher.encode("ABC"), "BCD");
    assert_eq!(cipher.encode("XYZ"), "YZA");
}

#[test]
fn test_substitution_cipher_decode() {
    let cipher = SubstitutionCipher::new("BCDEFGHIJKLMNOPQRSTUVWXYZA").unwrap();
    assert_eq!(cipher.decode("BCD"), "ABC");
    assert_eq!(cipher.decode("YZA"), "XYZ");
}

#[test]
fn test_substitution_cipher_roundtrip() {
    let cipher = SubstitutionCipher::new("QWERTYUIOPASDFGHJKLZXCVBNM").unwrap();
    let original = "HELLO WORLD";
    let encoded = cipher.encode(original);
    let decoded = cipher.decode(&encoded);
    assert_eq!(decoded, original);
}

#[test]
fn test_universal_decoder() {
    let results = UniversalDecoder::try_all("aGVsbG8=");
    assert!(!results.is_empty());

    let base64_result = results.iter().find(|(method, _)| method == "Base64");
    assert!(base64_result.is_some());
    assert_eq!(base64_result.unwrap().1, "hello");
}

#[test]
fn test_universal_decoder_multiple() {
    let results = UniversalDecoder::try_all("TEST");
    assert!(results.len() > 1);
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_base64_roundtrip(data: Vec<u8>) {
            let encoded = BaseEncoder::base64_encode(&data);
            let decoded = BaseEncoder::base64_decode(&encoded).unwrap();
            assert_eq!(decoded, data);
        }

        #[test]
        fn prop_base64_url_roundtrip(data: Vec<u8>) {
            let encoded = BaseEncoder::base64_url_encode(&data);
            let decoded = BaseEncoder::base64_url_decode(&encoded).unwrap();
            assert_eq!(decoded, data);
        }

        #[test]
        fn prop_hex_roundtrip(data: Vec<u8>) {
            let encoded = BaseEncoder::hex_encode(&data);
            let decoded = BaseEncoder::hex_decode(&encoded).unwrap();
            assert_eq!(decoded, data);
        }

        #[test]
        fn prop_url_encode_roundtrip(s in "[a-zA-Z0-9 ]{0,100}") {
            let encoded = URLEncoder::encode(&s);
            let decoded = URLEncoder::decode(&encoded).unwrap();
            assert_eq!(decoded, s);
        }

        #[test]
        fn prop_rot13_double_application(s in "[a-zA-Z ]{0,100}") {
            let once = ROTCipher::rot13(&s);
            let twice = ROTCipher::rot13(&once);
            assert_eq!(twice, s);
        }

        #[test]
        fn prop_rotn_26_is_identity(s in "[a-zA-Z ]{0,100}") {
            let rotated = ROTCipher::rotn(&s, 26);
            assert_eq!(rotated, s);
        }
    }
}
