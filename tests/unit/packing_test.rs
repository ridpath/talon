use talon::packing_tools::*;

#[test]
fn test_pack64_little_endian() {
    assert_eq!(pack64(0xdeadbeef), vec![0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00]);
    assert_eq!(pack64(0x4142434445464748), vec![0x48, 0x47, 0x46, 0x45, 0x44, 0x43, 0x42, 0x41]);
    assert_eq!(pack64(0), vec![0; 8]);
    assert_eq!(pack64(u64::MAX), vec![0xff; 8]);
}

#[test]
fn test_pack64_big_endian() {
    assert_eq!(pack64_be(0xdeadbeef), vec![0x00, 0x00, 0x00, 0x00, 0xde, 0xad, 0xbe, 0xef]);
    assert_eq!(pack64_be(0x4142434445464748), vec![0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48]);
    assert_eq!(pack64_be(0), vec![0; 8]);
    assert_eq!(pack64_be(u64::MAX), vec![0xff; 8]);
}

#[test]
fn test_unpack64_little_endian() {
    let bytes = vec![0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00];
    assert_eq!(unpack64(&bytes).unwrap(), 0xdeadbeef);
    
    let bytes = vec![0x48, 0x47, 0x46, 0x45, 0x44, 0x43, 0x42, 0x41];
    assert_eq!(unpack64(&bytes).unwrap(), 0x4142434445464748);
    
    let bytes = vec![0; 8];
    assert_eq!(unpack64(&bytes).unwrap(), 0);
    
    let bytes = vec![0xff; 8];
    assert_eq!(unpack64(&bytes).unwrap(), u64::MAX);
}

#[test]
fn test_unpack64_big_endian() {
    let bytes = vec![0x00, 0x00, 0x00, 0x00, 0xde, 0xad, 0xbe, 0xef];
    assert_eq!(unpack64_be(&bytes).unwrap(), 0xdeadbeef);
    
    let bytes = vec![0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48];
    assert_eq!(unpack64_be(&bytes).unwrap(), 0x4142434445464748);
}

#[test]
fn test_unpack64_insufficient_bytes() {
    let bytes = vec![0x41, 0x42, 0x43];
    assert!(unpack64(&bytes).is_err());
    assert!(unpack64_be(&bytes).is_err());
    
    let empty: Vec<u8> = vec![];
    assert!(unpack64(&empty).is_err());
}

#[test]
fn test_unpack64_extra_bytes() {
    let bytes = vec![0xef, 0xbe, 0xad, 0xde, 0x00, 0x00, 0x00, 0x00, 0x99, 0x88];
    assert_eq!(unpack64(&bytes).unwrap(), 0xdeadbeef);
}

#[test]
fn test_pack32_little_endian() {
    assert_eq!(pack32(0x41424344), vec![0x44, 0x43, 0x42, 0x41]);
    assert_eq!(pack32(0xdeadbeef), vec![0xef, 0xbe, 0xad, 0xde]);
    assert_eq!(pack32(0), vec![0; 4]);
    assert_eq!(pack32(u32::MAX), vec![0xff; 4]);
}

#[test]
fn test_pack32_big_endian() {
    assert_eq!(pack32_be(0x41424344), vec![0x41, 0x42, 0x43, 0x44]);
    assert_eq!(pack32_be(0xdeadbeef), vec![0xde, 0xad, 0xbe, 0xef]);
    assert_eq!(pack32_be(0), vec![0; 4]);
    assert_eq!(pack32_be(u32::MAX), vec![0xff; 4]);
}

#[test]
fn test_unpack32_little_endian() {
    let bytes = vec![0x44, 0x43, 0x42, 0x41];
    assert_eq!(unpack32(&bytes).unwrap(), 0x41424344);
    
    let bytes = vec![0xef, 0xbe, 0xad, 0xde];
    assert_eq!(unpack32(&bytes).unwrap(), 0xdeadbeef);
    
    let bytes = vec![0; 4];
    assert_eq!(unpack32(&bytes).unwrap(), 0);
    
    let bytes = vec![0xff; 4];
    assert_eq!(unpack32(&bytes).unwrap(), u32::MAX);
}

#[test]
fn test_unpack32_big_endian() {
    let bytes = vec![0x41, 0x42, 0x43, 0x44];
    assert_eq!(unpack32_be(&bytes).unwrap(), 0x41424344);
    
    let bytes = vec![0xde, 0xad, 0xbe, 0xef];
    assert_eq!(unpack32_be(&bytes).unwrap(), 0xdeadbeef);
}

#[test]
fn test_unpack32_insufficient_bytes() {
    let bytes = vec![0x41, 0x42];
    assert!(unpack32(&bytes).is_err());
    assert!(unpack32_be(&bytes).is_err());
}

#[test]
fn test_pack16_little_endian() {
    assert_eq!(pack16(0x4142), vec![0x42, 0x41]);
    assert_eq!(pack16(0xbeef), vec![0xef, 0xbe]);
    assert_eq!(pack16(0), vec![0; 2]);
    assert_eq!(pack16(u16::MAX), vec![0xff; 2]);
}

#[test]
fn test_pack16_big_endian() {
    assert_eq!(pack16_be(0x4142), vec![0x41, 0x42]);
    assert_eq!(pack16_be(0xbeef), vec![0xbe, 0xef]);
    assert_eq!(pack16_be(0), vec![0; 2]);
    assert_eq!(pack16_be(u16::MAX), vec![0xff; 2]);
}

#[test]
fn test_unpack16_little_endian() {
    let bytes = vec![0x42, 0x41];
    assert_eq!(unpack16(&bytes).unwrap(), 0x4142);
    
    let bytes = vec![0xef, 0xbe];
    assert_eq!(unpack16(&bytes).unwrap(), 0xbeef);
    
    let bytes = vec![0; 2];
    assert_eq!(unpack16(&bytes).unwrap(), 0);
    
    let bytes = vec![0xff; 2];
    assert_eq!(unpack16(&bytes).unwrap(), u16::MAX);
}

#[test]
fn test_unpack16_big_endian() {
    let bytes = vec![0x41, 0x42];
    assert_eq!(unpack16_be(&bytes).unwrap(), 0x4142);
    
    let bytes = vec![0xbe, 0xef];
    assert_eq!(unpack16_be(&bytes).unwrap(), 0xbeef);
}

#[test]
fn test_unpack16_insufficient_bytes() {
    let bytes = vec![0x41];
    assert!(unpack16(&bytes).is_err());
    assert!(unpack16_be(&bytes).is_err());
    
    let empty: Vec<u8> = vec![];
    assert!(unpack16(&empty).is_err());
}

#[test]
fn test_pack8() {
    assert_eq!(pack8(0x41), vec![0x41]);
    assert_eq!(pack8(0), vec![0]);
    assert_eq!(pack8(255), vec![255]);
}

#[test]
fn test_unpack8() {
    assert_eq!(unpack8(&[0x41]).unwrap(), 0x41);
    assert_eq!(unpack8(&[0]).unwrap(), 0);
    assert_eq!(unpack8(&[255]).unwrap(), 255);
}

#[test]
fn test_unpack8_empty() {
    let empty: Vec<u8> = vec![];
    assert!(unpack8(&empty).is_err());
}

#[test]
fn test_pack_struct_valid() {
    let result = pack_struct("Q", &[0x4142434445464748]).unwrap();
    assert_eq!(result, vec![0x48, 0x47, 0x46, 0x45, 0x44, 0x43, 0x42, 0x41]);
    
    let result = pack_struct("I", &[0x41424344]).unwrap();
    assert_eq!(result, vec![0x44, 0x43, 0x42, 0x41]);
    
    let result = pack_struct("H", &[0x4142]).unwrap();
    assert_eq!(result, vec![0x42, 0x41]);
    
    let result = pack_struct("B", &[0x41]).unwrap();
    assert_eq!(result, vec![0x41]);
}

#[test]
fn test_pack_struct_multiple() {
    let result = pack_struct("QIH", &[0x4142434445464748, 0x41424344, 0x4142]).unwrap();
    assert_eq!(result.len(), 8 + 4 + 2);
    assert_eq!(&result[0..8], &[0x48, 0x47, 0x46, 0x45, 0x44, 0x43, 0x42, 0x41]);
    assert_eq!(&result[8..12], &[0x44, 0x43, 0x42, 0x41]);
    assert_eq!(&result[12..14], &[0x42, 0x41]);
}

#[test]
fn test_pack_struct_mixed() {
    let result = pack_struct("QIHB", &[0x1122334455667788, 0xaabbccdd, 0xeeff, 0x99]).unwrap();
    assert_eq!(result.len(), 8 + 4 + 2 + 1);
}

#[test]
fn test_pack_struct_invalid_format() {
    let result = pack_struct("X", &[0x41]);
    assert!(result.is_err());
    
    let result = pack_struct("QQQ", &[0x41]);
    assert!(result.is_err());
}

#[test]
fn test_flat_pack() {
    let result = flat_pack(&[0x41, 0x42, 0x43]);
    assert_eq!(result.len(), 24);
    
    assert_eq!(&result[0..8], &pack64(0x41));
    assert_eq!(&result[8..16], &pack64(0x42));
    assert_eq!(&result[16..24], &pack64(0x43));
}

#[test]
fn test_flat_pack_empty() {
    let result = flat_pack(&[]);
    assert_eq!(result.len(), 0);
}

#[test]
fn test_flat_pack_single() {
    let result = flat_pack(&[0xdeadbeef]);
    assert_eq!(result, pack64(0xdeadbeef));
}

#[test]
fn test_cyclic_buffer() {
    let pattern = b"ABCD";
    let result = cyclic_buffer(10, pattern);
    assert_eq!(result.len(), 10);
    assert_eq!(&result[0..4], pattern);
    assert_eq!(&result[4..8], pattern);
    assert_eq!(&result[8..10], &pattern[0..2]);
}

#[test]
fn test_cyclic_buffer_exact() {
    let pattern = b"ABC";
    let result = cyclic_buffer(9, pattern);
    assert_eq!(result.len(), 9);
    assert_eq!(result, b"ABCABCABC");
}

#[test]
fn test_cyclic_buffer_single_byte() {
    let pattern = b"X";
    let result = cyclic_buffer(5, pattern);
    assert_eq!(result, b"XXXXX");
}

#[test]
fn test_hex_to_bytes() {
    assert_eq!(hex_to_bytes("deadbeef").unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
    assert_eq!(hex_to_bytes("0xdeadbeef").unwrap(), vec![0xde, 0xad, 0xbe, 0xef]);
    assert_eq!(hex_to_bytes("41424344").unwrap(), vec![0x41, 0x42, 0x43, 0x44]);
    assert_eq!(hex_to_bytes("00").unwrap(), vec![0x00]);
    assert_eq!(hex_to_bytes("ff").unwrap(), vec![0xff]);
}

#[test]
fn test_hex_to_bytes_invalid() {
    assert!(hex_to_bytes("gg").is_err());
    assert!(hex_to_bytes("xyz").is_err());
    assert!(hex_to_bytes("12g").is_err());
}

#[test]
fn test_bytes_to_hex() {
    assert_eq!(bytes_to_hex(&[0xde, 0xad, 0xbe, 0xef]), "0xdeadbeef");
    assert_eq!(bytes_to_hex(&[0x41, 0x42, 0x43, 0x44]), "0x41424344");
    assert_eq!(bytes_to_hex(&[0x00]), "0x00");
    assert_eq!(bytes_to_hex(&[0xff]), "0xff");
}

#[test]
fn test_hex_roundtrip() {
    let original = vec![0xde, 0xad, 0xbe, 0xef, 0x01, 0x02, 0x03];
    let hex = bytes_to_hex(&original);
    let decoded = hex_to_bytes(&hex).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_pack_unpack_roundtrip_64() {
    let values = [0u64, 1, 0xdeadbeef, 0x4142434445464748, u64::MAX];
    for &val in &values {
        assert_eq!(unpack64(&pack64(val)).unwrap(), val);
        assert_eq!(unpack64_be(&pack64_be(val)).unwrap(), val);
    }
}

#[test]
fn test_pack_unpack_roundtrip_32() {
    let values = [0u32, 1, 0xdeadbeef, 0x41424344, u32::MAX];
    for &val in &values {
        assert_eq!(unpack32(&pack32(val)).unwrap(), val);
        assert_eq!(unpack32_be(&pack32_be(val)).unwrap(), val);
    }
}

#[test]
fn test_pack_unpack_roundtrip_16() {
    let values = [0u16, 1, 0xbeef, 0x4142, u16::MAX];
    for &val in &values {
        assert_eq!(unpack16(&pack16(val)).unwrap(), val);
        assert_eq!(unpack16_be(&pack16_be(val)).unwrap(), val);
    }
}

#[test]
fn test_pack_unpack_roundtrip_8() {
    let values = [0u8, 1, 0x41, 0xff];
    for &val in &values {
        assert_eq!(unpack8(&pack8(val)).unwrap(), val);
    }
}

#[test]
fn test_endianness_difference() {
    let value: u64 = 0x0102030405060708;
    let le = pack64(value);
    let be = pack64_be(value);
    
    assert_ne!(le, be);
    assert_eq!(le, vec![0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
    assert_eq!(be, vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
}

#[test]
fn test_pack_struct_all_formats() {
    let result = pack_struct("QIHB", &[0x1122334455667788, 0xaabbccdd, 0xeeff, 0x99]).unwrap();
    
    assert_eq!(&result[0..8], &pack64(0x1122334455667788));
    assert_eq!(&result[8..12], &pack32(0xaabbccdd));
    assert_eq!(&result[12..14], &pack16(0xeeff));
    assert_eq!(&result[14..15], &pack8(0x99));
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_pack64_unpack64_roundtrip(value: u64) {
            assert_eq!(unpack64(&pack64(value)).unwrap(), value);
        }

        #[test]
        fn prop_pack64_be_unpack64_be_roundtrip(value: u64) {
            assert_eq!(unpack64_be(&pack64_be(value)).unwrap(), value);
        }

        #[test]
        fn prop_pack32_unpack32_roundtrip(value: u32) {
            assert_eq!(unpack32(&pack32(value)).unwrap(), value);
        }

        #[test]
        fn prop_pack16_unpack16_roundtrip(value: u16) {
            assert_eq!(unpack16(&pack16(value)).unwrap(), value);
        }

        #[test]
        fn prop_pack8_unpack8_roundtrip(value: u8) {
            assert_eq!(unpack8(&pack8(value)).unwrap(), value);
        }

        #[test]
        fn prop_hex_roundtrip(bytes: Vec<u8>) {
            let hex = bytes_to_hex(&bytes);
            let decoded = hex_to_bytes(&hex).unwrap();
            assert_eq!(bytes, decoded);
        }
    }
}
