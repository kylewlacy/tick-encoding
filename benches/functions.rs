fn main() {
    divan::main();
}

/// Benchmark `requires_escape()` across all byte values
#[divan::bench]
fn requires_escape_all_bytes(bencher: divan::Bencher) {
    let bytes: Vec<u8> = (0..=255u8).collect();

    bencher.bench_local(|| {
        let mut count = 0u32;
        for _ in 0..4000 {
            // Repeat to get measurable time
            for &b in &bytes {
                if tick_encoding::requires_escape(divan::black_box(b)) {
                    count += 1;
                }
            }
        }
        divan::black_box(count)
    });
}

/// Benchmark `encode_to_vec` with unescaped content
#[divan::bench]
fn encode_to_vec_unescaped(bencher: divan::Bencher) {
    let bytes = vec![b'a'; 1_000_000];

    bencher.bench_local(|| {
        let mut output = Vec::new();
        tick_encoding::encode_to_vec(divan::black_box(&bytes), &mut output);
        divan::black_box(output)
    });
}

/// Benchmark `encode_to_vec` with binary content (worst case)
#[divan::bench]
fn encode_to_vec_binary(bencher: divan::Bencher) {
    let bytes = vec![0x00; 1_000_000];

    bencher.bench_local(|| {
        let mut output = Vec::new();
        tick_encoding::encode_to_vec(divan::black_box(&bytes), &mut output);
        divan::black_box(output)
    });
}

/// Benchmark `encode_to_vec` with mixed content
#[divan::bench]
fn encode_to_vec_mixed_50_50(bencher: divan::Bencher) {
    let bytes: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 2 == 0 { 0x00 } else { b'a' })
        .collect();

    bencher.bench_local(|| {
        let mut output = Vec::new();
        tick_encoding::encode_to_vec(divan::black_box(&bytes), &mut output);
        divan::black_box(output)
    });
}

/// Benchmark `encode_to_string` with unescaped content
#[divan::bench]
fn encode_to_string_unescaped(bencher: divan::Bencher) {
    let bytes = vec![b'a'; 1_000_000];

    bencher.bench_local(|| {
        let mut output = String::new();
        tick_encoding::encode_to_string(divan::black_box(&bytes), &mut output);
        divan::black_box(output)
    });
}

/// Benchmark `encode_to_string` with binary content (worst case)
#[divan::bench]
fn encode_to_string_binary(bencher: divan::Bencher) {
    let bytes = vec![0x00; 1_000_000];

    bencher.bench_local(|| {
        let mut output = String::new();
        tick_encoding::encode_to_string(divan::black_box(&bytes), &mut output);
        divan::black_box(output)
    });
}

/// Benchmark `encode_to_string` with mixed content
#[divan::bench]
fn encode_to_string_mixed_50_50(bencher: divan::Bencher) {
    let bytes: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 2 == 0 { 0x00 } else { b'a' })
        .collect();

    bencher.bench_local(|| {
        let mut output = String::new();
        tick_encoding::encode_to_string(divan::black_box(&bytes), &mut output);
        divan::black_box(output)
    });
}

/// Benchmark `decode_to_vec` with unescaped content
#[divan::bench]
fn decode_to_vec_unescaped(bencher: divan::Bencher) {
    let bytes = vec![b'a'; 1_000_000];

    bencher.bench_local(|| {
        let mut output = Vec::new();
        tick_encoding::decode_to_vec(divan::black_box(&bytes), &mut output).unwrap();
        divan::black_box(output)
    });
}

/// Benchmark `decode_to_vec` with binary content (worst case)
#[divan::bench]
fn decode_to_vec_binary(bencher: divan::Bencher) {
    let original = vec![0x00; 1_000_000];
    let encoded = tick_encoding::encode(&original);
    let encoded_bytes = encoded.as_bytes().to_vec();

    bencher.bench_local(|| {
        let mut output = Vec::new();
        tick_encoding::decode_to_vec(divan::black_box(&encoded_bytes), &mut output).unwrap();
        divan::black_box(output)
    });
}

/// Benchmark `decode_to_vec` with mixed content
#[divan::bench]
fn decode_to_vec_mixed_50_50(bencher: divan::Bencher) {
    let original: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 2 == 0 { 0x00 } else { b'a' })
        .collect();
    let encoded = tick_encoding::encode(&original);
    let encoded_bytes = encoded.as_bytes().to_vec();

    bencher.bench_local(|| {
        let mut output = Vec::new();
        tick_encoding::decode_to_vec(divan::black_box(&encoded_bytes), &mut output).unwrap();
        divan::black_box(output)
    });
}
