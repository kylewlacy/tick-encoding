fn main() {
    divan::main();
}

/// 100% ASCII - best case (no escaping needed)
#[divan::bench]
fn encode_unescaped(bencher: divan::Bencher) {
    let bytes = vec![b'a'; 1_000_000];

    bencher.bench_local(|| {
        let encoded = tick_encoding::encode(divan::black_box(&bytes));
        divan::black_box(encoded);
    });
}

/// 100% tick characters
#[divan::bench]
fn encode_ticks(bencher: divan::Bencher) {
    let bytes = vec![b'`'; 1_000_000];

    bencher.bench_local(|| {
        let encoded = tick_encoding::encode(divan::black_box(&bytes));
        divan::black_box(encoded);
    });
}

/// 100% binary - worst case (all bytes need escaping)
#[divan::bench]
fn encode_binary(bencher: divan::Bencher) {
    let bytes = vec![0x00; 1_000_000];

    bencher.bench_local(|| {
        let encoded = tick_encoding::encode(divan::black_box(&bytes));
        divan::black_box(encoded);
    });
}

/// 90% ASCII, 10% binary - mostly text content
#[divan::bench]
fn encode_mixed_90_10(bencher: divan::Bencher) {
    let bytes: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 10 == 0 { 0x00 } else { b'a' })
        .collect();

    bencher.bench_local(|| {
        let encoded = tick_encoding::encode(divan::black_box(&bytes));
        divan::black_box(encoded);
    });
}

/// 50% ASCII, 50% binary - mix content
#[divan::bench]
fn encode_mixed_50_50(bencher: divan::Bencher) {
    let bytes: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 2 == 0 { 0x00 } else { b'a' })
        .collect();

    bencher.bench_local(|| {
        let encoded = tick_encoding::encode(divan::black_box(&bytes));
        divan::black_box(encoded);
    });
}

/// 10% ASCII, 90% binary - mostly binary content
#[divan::bench]
fn encode_mixed_10_90(bencher: divan::Bencher) {
    let bytes: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 10 == 0 { b'a' } else { 0x00 })
        .collect();

    bencher.bench_local(|| {
        let encoded = tick_encoding::encode(divan::black_box(&bytes));
        divan::black_box(encoded);
    });
}

#[divan::bench(args = [16, 64, 256, 1024, 4096])]
fn encode_short_ascii(bencher: divan::Bencher, len: usize) {
    let bytes = vec![b'a'; len];

    bencher.bench_local(|| {
        let encoded = tick_encoding::encode(divan::black_box(&bytes));
        divan::black_box(encoded);
    });
}

#[divan::bench(args = [16, 64, 256, 1024, 4096])]
fn encode_short_binary(bencher: divan::Bencher, len: usize) {
    let bytes = vec![0x00; len];

    bencher.bench_local(|| {
        let encoded = tick_encoding::encode(divan::black_box(&bytes));
        divan::black_box(encoded);
    });
}
