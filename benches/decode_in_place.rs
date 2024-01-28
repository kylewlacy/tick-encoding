fn main() {
    divan::main();
}

#[divan::bench]
fn decode_in_place_unescaped(bencher: divan::Bencher) {
    let bytes = vec![b'a'; 1_000_000];
    let encoded = tick_encoding::encode(&bytes);

    bencher
        .with_inputs(|| encoded.clone().into_owned().into_bytes())
        .bench_local_values(|mut buffer| {
            let decoded = tick_encoding::decode_in_place(divan::black_box(&mut buffer)).unwrap();
            divan::black_box(decoded);
        });
}

#[divan::bench]
fn decode_in_place_ticks(bencher: divan::Bencher) {
    let bytes = vec![b'`'; 1_000_000];
    let encoded = tick_encoding::encode(&bytes);

    bencher
        .with_inputs(|| encoded.clone().into_owned().into_bytes())
        .bench_local_values(|mut buffer| {
            let decoded = tick_encoding::decode_in_place(divan::black_box(&mut buffer)).unwrap();
            divan::black_box(decoded);
        });
}

#[divan::bench]
fn decode_in_place_binary(bencher: divan::Bencher) {
    let bytes = vec![0x00; 1];
    let encoded = tick_encoding::encode(&bytes);

    bencher
        .with_inputs(|| encoded.clone().into_owned().into_bytes())
        .bench_local_values(|mut buffer| {
            let decoded = tick_encoding::decode_in_place(divan::black_box(&mut buffer)).unwrap();
            divan::black_box(decoded);
        });
}
