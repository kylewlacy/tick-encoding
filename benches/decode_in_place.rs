fn main() {
    divan::main();
}

/// 100% ASCII - best case (no escaping needed)
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

/// 100% tick characters
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

/// 100% binary - worst case (all bytes need escaping)
#[divan::bench]
fn decode_in_place_binary(bencher: divan::Bencher) {
    let bytes = vec![0x00; 1_000_000];
    let encoded = tick_encoding::encode(&bytes);

    bencher
        .with_inputs(|| encoded.clone().into_owned().into_bytes())
        .bench_local_values(|mut buffer| {
            let decoded = tick_encoding::decode_in_place(divan::black_box(&mut buffer)).unwrap();
            divan::black_box(decoded);
        });
}

/// 90% ASCII, 10% binary - mostly text content
#[divan::bench]
fn decode_in_place_mixed_90_10(bencher: divan::Bencher) {
    let original: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 10 == 0 { 0x00 } else { b'a' })
        .collect();
    let encoded = tick_encoding::encode(&original);

    bencher
        .with_inputs(|| encoded.clone().into_owned().into_bytes())
        .bench_local_values(|mut buffer| {
            let decoded = tick_encoding::decode_in_place(divan::black_box(&mut buffer)).unwrap();
            divan::black_box(decoded);
        });
}

/// 50% ASCII, 50% binary - mix content
#[divan::bench]
fn decode_in_place_mixed_50_50(bencher: divan::Bencher) {
    let original: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 2 == 0 { 0x00 } else { b'a' })
        .collect();
    let encoded = tick_encoding::encode(&original);

    bencher
        .with_inputs(|| encoded.clone().into_owned().into_bytes())
        .bench_local_values(|mut buffer| {
            let decoded = tick_encoding::decode_in_place(divan::black_box(&mut buffer)).unwrap();
            divan::black_box(decoded);
        });
}

/// 10% ASCII, 90% binary - mostly binary content
#[divan::bench]
fn decode_in_place_mixed_10_90(bencher: divan::Bencher) {
    let original: Vec<u8> = (0..1_000_000)
        .map(|i| if i % 10 == 0 { b'a' } else { 0x00 })
        .collect();
    let encoded = tick_encoding::encode(&original);

    bencher
        .with_inputs(|| encoded.clone().into_owned().into_bytes())
        .bench_local_values(|mut buffer| {
            let decoded = tick_encoding::decode_in_place(divan::black_box(&mut buffer)).unwrap();
            divan::black_box(decoded);
        });
}
