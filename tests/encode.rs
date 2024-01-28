#![cfg(feature = "alloc")]

use tick_encoding::encode;

#[test]
fn test_encode() {
    assert_eq!(encode(b""), "");
    assert_eq!(encode(b"hello"), "hello");
    assert_eq!(encode(b"`"), "``");
    assert_eq!(encode(&[0xFF]), "`FF");
    assert_eq!(
        encode(b"hello world!\r\n\thi there"),
        "hello world!\r\n\thi there"
    );
    assert_eq!(encode("foo bar 🙂".as_bytes()), "foo bar `F0`9F`99`82");
}
