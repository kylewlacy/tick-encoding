fn main() {
    divan::main();
}

#[divan::bench]
fn decode_unescaped(bencher: divan::Bencher) {
    let bytes = vec![b'a'; 1_000_000];
    let encoded = tick_encoding::encode(&bytes);

    bencher.bench_local(|| {
        let decoded = tick_encoding::decode(divan::black_box(encoded.as_bytes())).unwrap();
        divan::black_box(decoded);
    });
}

#[divan::bench]
fn decode_ticks(bencher: divan::Bencher) {
    let bytes = vec![b'`'; 1_000_000];
    let encoded = tick_encoding::encode(&bytes);

    bencher.bench_local(|| {
        let decoded = tick_encoding::decode(divan::black_box(encoded.as_bytes())).unwrap();
        divan::black_box(decoded);
    });
}

#[divan::bench]
fn decode_binary(bencher: divan::Bencher) {
    let bytes = vec![0x00; 1_000_000];
    let encoded = tick_encoding::encode(&bytes);

    bencher.bench_local(|| {
        let decoded = tick_encoding::decode(divan::black_box(encoded.as_bytes())).unwrap();
        divan::black_box(decoded);
    });
}
