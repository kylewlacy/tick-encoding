use assert_matches::assert_matches;
use tick_encoding::{DecodeError, EscapedHex};

fn decode_iter(bytes: &[u8]) -> impl Iterator<Item = Result<u8, DecodeError>> + '_ {
    tick_encoding::decode_iter(bytes.iter().copied())
}

fn decode_iter_collect(bytes: &[u8]) -> Result<Vec<u8>, DecodeError> {
    decode_iter(bytes).collect()
}

#[test]
fn test_decode() {
    assert_eq!(decode_iter_collect(b"").unwrap(), &b""[..]);
    assert_eq!(decode_iter_collect(b"hello").unwrap(), &b"hello"[..]);
    assert_eq!(decode_iter_collect(b"``").unwrap(), &b"`"[..]);
    assert_eq!(decode_iter_collect(b"`FF").unwrap(), &[0xFF][..]);
    assert_eq!(
        decode_iter_collect(b"hello world!\r\n\thi there").unwrap(),
        &b"hello world!\r\n\thi there"[..]
    );
    assert_eq!(
        decode_iter_collect(b"foo bar `F0`9F`99`82").unwrap(),
        "foo bar 🙂".as_bytes()
    );
}

#[test]
fn test_decode_invalid_byte_error() {
    let mut iter = decode_iter(&[b'a', 0xFF]);
    assert_matches!(iter.next(), Some(Ok(b'a')));
    assert_matches!(iter.next(), Some(Err(DecodeError::InvalidByte(0xFF))));
    assert_matches!(iter.next(), None);

    let mut iter = decode_iter(&[b'x', b'`', b'`', 0x00]);
    assert_matches!(iter.next(), Some(Ok(b'x')));
    assert_matches!(iter.next(), Some(Ok(b'`')));
    assert_matches!(iter.next(), Some(Err(DecodeError::InvalidByte(0x00))));
    assert_matches!(iter.next(), None);
}

#[test]
fn test_decode_unexpected_end_error() {
    let mut iter = decode_iter(b"x`");
    assert_matches!(iter.next(), Some(Ok(b'x')));
    assert_matches!(iter.next(), Some(Err(DecodeError::UnexpectedEnd)));
    assert_matches!(iter.next(), None);

    let mut iter = decode_iter(b"x`F");
    assert_matches!(iter.next(), Some(Ok(b'x')));
    assert_matches!(iter.next(), Some(Err(DecodeError::UnexpectedEnd)));
    assert_matches!(iter.next(), None);
}

#[test]
fn test_decode_lowercase_hex_error() {
    assert_matches!(
        decode_iter_collect(b"`fe"),
        Err(DecodeError::LowercaseHex(EscapedHex(b'f', b'e')))
    );
    assert_matches!(
        decode_iter_collect(b"`0e"),
        Err(DecodeError::LowercaseHex(EscapedHex(b'0', b'e')))
    );
    assert_matches!(
        decode_iter_collect(b"`f0"),
        Err(DecodeError::LowercaseHex(EscapedHex(b'f', b'0')))
    );
}

#[test]
fn test_decode_invalid_hex_error() {
    assert_matches!(
        decode_iter_collect(b"`GE"),
        Err(DecodeError::InvalidHex(EscapedHex(b'G', b'E')))
    );
    assert_matches!(
        decode_iter_collect(b"`0G"),
        Err(DecodeError::InvalidHex(EscapedHex(b'0', b'G')))
    );
    assert_matches!(
        decode_iter_collect(b"`G0"),
        Err(DecodeError::InvalidHex(EscapedHex(b'G', b'0')))
    );

    assert_matches!(
        decode_iter_collect(b"`fG"),
        Err(DecodeError::InvalidHex(EscapedHex(b'f', b'G')))
    );
    assert_matches!(
        decode_iter_collect(b"`gF"),
        Err(DecodeError::InvalidHex(EscapedHex(b'g', b'F')))
    );
}

#[test]
fn test_decode_unexpected_escape_error() {
    assert_matches!(
        decode_iter_collect(b"`65"),
        Err(DecodeError::UnexpectedEscape(EscapedHex(b'6', b'5'), 'e'))
    );
}
