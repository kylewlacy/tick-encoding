#![cfg(feature = "alloc")]

use assert_matches::assert_matches;
use tick_encoding::{decode, DecodeError, EscapedHex};

#[test]
fn test_decode() {
    assert_eq!(decode(b"").unwrap(), &b""[..]);
    assert_eq!(decode(b"hello").unwrap(), &b"hello"[..]);
    assert_eq!(decode(b"``").unwrap(), &b"`"[..]);
    assert_eq!(decode(b"`FF").unwrap(), &[0xFF][..]);
    assert_eq!(
        decode(b"hello world!\r\n\thi there").unwrap(),
        &b"hello world!\r\n\thi there"[..]
    );
    assert_eq!(
        decode(b"foo bar `F0`9F`99`82").unwrap(),
        "foo bar 🙂".as_bytes()
    );
}

#[test]
fn test_decode_invalid_byte_error() {
    assert_matches!(decode(&[0xFF]), Err(DecodeError::InvalidByte(0xFF)));
    assert_matches!(decode(&[0x00]), Err(DecodeError::InvalidByte(0x00)));
}

#[test]
fn test_decode_unexpected_end_error() {
    assert_matches!(decode(b"`"), Err(DecodeError::UnexpectedEnd));
    assert_matches!(decode(b"`F"), Err(DecodeError::UnexpectedEnd));
    assert_matches!(decode(b"`F0`"), Err(DecodeError::UnexpectedEnd));
    assert_matches!(decode(b"`F0`9"), Err(DecodeError::UnexpectedEnd));
}

#[test]
fn test_decode_lowercase_hex_error() {
    assert_matches!(
        decode(b"`fe"),
        Err(DecodeError::LowercaseHex(EscapedHex(b'f', b'e')))
    );
    assert_matches!(
        decode(b"`0e"),
        Err(DecodeError::LowercaseHex(EscapedHex(b'0', b'e')))
    );
    assert_matches!(
        decode(b"`f0"),
        Err(DecodeError::LowercaseHex(EscapedHex(b'f', b'0')))
    );
}

#[test]
fn test_decode_invalid_hex_error() {
    assert_matches!(
        decode(b"`GE"),
        Err(DecodeError::InvalidHex(EscapedHex(b'G', b'E')))
    );
    assert_matches!(
        decode(b"`0G"),
        Err(DecodeError::InvalidHex(EscapedHex(b'0', b'G')))
    );
    assert_matches!(
        decode(b"`G0"),
        Err(DecodeError::InvalidHex(EscapedHex(b'G', b'0')))
    );

    assert_matches!(
        decode(b"`fG"),
        Err(DecodeError::InvalidHex(EscapedHex(b'f', b'G')))
    );
    assert_matches!(
        decode(b"`gF"),
        Err(DecodeError::InvalidHex(EscapedHex(b'g', b'F')))
    );
}

#[test]
fn test_decode_unexpected_escape_error() {
    assert_matches!(
        decode(b"`65"),
        Err(DecodeError::UnexpectedEscape(EscapedHex(b'6', b'5'), 'e'))
    );
}
