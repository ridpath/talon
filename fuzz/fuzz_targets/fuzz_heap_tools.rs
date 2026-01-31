#![no_main]

use libfuzzer_sys::fuzz_target;
use talon::heap_tools::{HeapAnalyzer, ChunkMetadata, HeapAllocator};

fuzz_target!(|data: &[u8]| {
    if data.len() < 32 {
        return;
    }

    let heap_base = u64::from_le_bytes([
        data[0], data[1], data[2], data[3],
        data[4], data[5], data[6], data[7],
    ]);

    let chunk_size = u64::from_le_bytes([
        data[8], data[9], data[10], data[11],
        data[12], data[13], data[14], data[15],
    ]);

    if chunk_size > 1024 * 1024 * 10 {
        return;
    }

    let mut analyzer = HeapAnalyzer::new(heap_base);

    let chunk = ChunkMetadata {
        address: heap_base,
        size: chunk_size,
        prev_size: 0,
        in_use: data[16] & 0x01 != 0,
        is_mmapped: data[16] & 0x02 != 0,
        prev_in_use: data[16] & 0x04 != 0,
    };

    analyzer.add_chunk(chunk);

    let _ = analyzer.find_overlapping_chunks();
    let _ = analyzer.detect_double_free(heap_base);
    let _ = analyzer.validate_chunk_integrity(heap_base);

    if data.len() > 24 {
        let target_size = u64::from_le_bytes([
            data[17], data[18], data[19], data[20],
            data[21], data[22], data[23], data[24],
        ]) & 0xFFFF;

        if target_size > 0 && target_size < 10000 {
            let _ = analyzer.find_suitable_chunk(target_size as usize);
        }
    }

    let allocator_type = match data.get(25).unwrap_or(&0) % 3 {
        0 => HeapAllocator::Glibc,
        1 => HeapAllocator::Jemalloc,
        2 => HeapAllocator::Tcmalloc,
        _ => HeapAllocator::Glibc,
    };

    if data.len() > 32 {
        let tcache_idx = data[26] as usize % 64;
        let _ = analyzer.poison_tcache_entry(tcache_idx, heap_base + 0x1000);

        let fastbin_idx = data[27] as usize % 10;
        let _ = analyzer.analyze_fastbin(fastbin_idx);

        if data.len() > 40 {
            let fd = u64::from_le_bytes([
                data[32], data[33], data[34], data[35],
                data[36], data[37], data[38], data[39],
            ]);
            let _ = analyzer.validate_tcache_key(heap_base, fd);
        }
    }
});
