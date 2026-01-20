fn main() {
    divan::main();
}

// ============ encode_iter benchmarks ============

/// 100% ASCII - best case (no escaping needed)
#[divan::bench]
fn encode_iter_unescaped(bencher: divan::Bencher) {
    let bytes = vec![b'a'; 1_000_000];

    bencher.bench_local(|| {
        tick_encoding::encode_iter(divan::black_box(bytes.iter().copied())).count()
    });
}

/// 100% tick characters
#[divan::bench]
fn encode_iter_ticks(bencher: divan::Bencher) {
    let bytes = vec![b'`'; 1_000_000];

    bencher.bench_local(|| {
        tick_encoding::encode_iter(divan::black_box(bytes.iter().copied())).count()
    });
}

/// 100% binary - worst case (all bytes need escaping)
#[divan::bench]
fn encode_iter_binary(bencher: divan::Bencher) {
    let bytes = vec![0x00u8; 1_000_000];

    bencher.bench_local(|| {
        tick_encoding::encode_iter(divan::black_box(bytes.iter().copied())).count()
    });
}

/// 90% ASCII, 10% binary - mostly text content
#[divan::bench]
fn encode_iter_mixed_90_10(bencher: divan::Bencher) {
    let bytes: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 10 == 0 { 0x00 } else { b'a' })
        .collect();

    bencher.bench_local(|| {
        tick_encoding::encode_iter(divan::black_box(bytes.iter().copied())).count()
    });
}

/// 50% ASCII, 50% binary - mix content
#[divan::bench]
fn encode_iter_mixed_50_50(bencher: divan::Bencher) {
    let bytes: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 2 == 0 { 0x00 } else { b'a' })
        .collect();

    bencher.bench_local(|| {
        tick_encoding::encode_iter(divan::black_box(bytes.iter().copied())).count()
    });
}

/// 10% ASCII, 90% binary - mostly binary content
#[divan::bench]
fn encode_iter_mixed_10_90(bencher: divan::Bencher) {
    let bytes: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 10 == 0 { b'a' } else { 0x00 })
        .collect();

    bencher.bench_local(|| {
        tick_encoding::encode_iter(divan::black_box(bytes.iter().copied())).count()
    });
}

// ============ decode_iter benchmarks ============

/// 100% ASCII - best case (no escaping needed)
#[divan::bench]
fn decode_iter_unescaped(bencher: divan::Bencher) {
    let bytes = vec![b'a'; 1_000_000];

    bencher.bench_local(|| {
        tick_encoding::decode_iter(divan::black_box(bytes.iter().copied()))
            .collect::<Result<Vec<_>, _>>()
    });
}

/// 100% tick characters
#[divan::bench]
fn decode_iter_ticks(bencher: divan::Bencher) {
    let original = vec![b'`'; 1_000_000];
    let encoded = tick_encoding::encode(&original);
    let encoded_bytes: Vec<u8> = encoded.as_bytes().to_vec();

    bencher.bench_local(|| {
        tick_encoding::decode_iter(divan::black_box(encoded_bytes.iter().copied()))
            .collect::<Result<Vec<_>, _>>()
    });
}

/// 100% binary - worst case (all bytes need escaping)
#[divan::bench]
fn decode_iter_binary(bencher: divan::Bencher) {
    let original = vec![0x00u8; 1_000_000];
    let encoded = tick_encoding::encode(&original);
    let encoded_bytes: Vec<u8> = encoded.as_bytes().to_vec();

    bencher.bench_local(|| {
        tick_encoding::decode_iter(divan::black_box(encoded_bytes.iter().copied()))
            .collect::<Result<Vec<_>, _>>()
    });
}

/// 90% ASCII, 10% binary
#[divan::bench]
fn decode_iter_mixed_90_10(bencher: divan::Bencher) {
    let original: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 10 == 0 { 0x00 } else { b'a' })
        .collect();
    let encoded = tick_encoding::encode(&original);
    let encoded_bytes: Vec<u8> = encoded.as_bytes().to_vec();

    bencher.bench_local(|| {
        tick_encoding::decode_iter(divan::black_box(encoded_bytes.iter().copied()))
            .collect::<Result<Vec<_>, _>>()
    });
}

/// 50% ASCII, 50% binary
#[divan::bench]
fn decode_iter_mixed_50_50(bencher: divan::Bencher) {
    let original: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 2 == 0 { 0x00 } else { b'a' })
        .collect();
    let encoded = tick_encoding::encode(&original);
    let encoded_bytes: Vec<u8> = encoded.as_bytes().to_vec();

    bencher.bench_local(|| {
        tick_encoding::decode_iter(divan::black_box(encoded_bytes.iter().copied()))
            .collect::<Result<Vec<_>, _>>()
    });
}

/// 10% ASCII, 90% binary
#[divan::bench]
fn decode_iter_mixed_10_90(bencher: divan::Bencher) {
    let original: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 10 == 0 { b'a' } else { 0x00 })
        .collect();
    let encoded = tick_encoding::encode(&original);
    let encoded_bytes: Vec<u8> = encoded.as_bytes().to_vec();

    bencher.bench_local(|| {
        tick_encoding::decode_iter(divan::black_box(encoded_bytes.iter().copied()))
            .collect::<Result<Vec<_>, _>>()
    });
}
