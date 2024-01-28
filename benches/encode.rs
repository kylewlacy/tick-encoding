fn main() {
    divan::main();
}

#[divan::bench]
fn encode_unescaped(bencher: divan::Bencher) {
    let bytes = vec![b'a'; 1_000_000];

    bencher.bench_local(|| {
        let encoded = tick_encoding::encode(divan::black_box(&bytes));
        divan::black_box(encoded);
    });
}

#[divan::bench]
fn encode_ticks(bencher: divan::Bencher) {
    let bytes = vec![b'`'; 1_000_000];

    bencher.bench_local(|| {
        let encoded = tick_encoding::encode(divan::black_box(&bytes));
        divan::black_box(encoded);
    });
}

#[divan::bench]
fn encode_binary(bencher: divan::Bencher) {
    let bytes = vec![0x00; 1_000_000];

    bencher.bench_local(|| {
        let encoded = tick_encoding::encode(divan::black_box(&bytes));
        divan::black_box(encoded);
    });
}
