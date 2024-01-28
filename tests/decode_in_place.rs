use assert_matches::assert_matches;
use tick_encoding::{decode_in_place, DecodeError, EscapedHex};

#[test]
fn test_decode_in_place() {
    assert_eq!(decode_in_place(&mut b"".to_vec()).unwrap(), &b""[..]);
    assert_eq!(
        decode_in_place(&mut b"hello".to_vec()).unwrap(),
        &b"hello"[..]
    );
    assert_eq!(decode_in_place(&mut b"``".to_vec()).unwrap(), &b"`"[..]);
    assert_eq!(decode_in_place(&mut b"`FF".to_vec()).unwrap(), &[0xFF][..]);
    assert_eq!(
        decode_in_place(&mut b"hello world!\r\n\thi there".to_vec()).unwrap(),
        &b"hello world!\r\n\thi there"[..]
    );
    assert_eq!(
        decode_in_place(&mut b"foo bar `F0`9F`99`82".to_vec()).unwrap(),
        "foo bar 🙂".as_bytes()
    );
}

#[test]
fn test_decode_in_place_invalid_byte_error() {
    assert_matches!(
        decode_in_place(&mut [0xFF]),
        Err(DecodeError::InvalidByte(0xFF))
    );
    assert_matches!(
        decode_in_place(&mut [0x00]),
        Err(DecodeError::InvalidByte(0x00))
    );
}

#[test]
fn test_decode_in_place_unexpected_end_error() {
    assert_matches!(
        decode_in_place(&mut b"`".to_vec()),
        Err(DecodeError::UnexpectedEnd)
    );
    assert_matches!(
        decode_in_place(&mut b"`F".to_vec()),
        Err(DecodeError::UnexpectedEnd)
    );
    assert_matches!(
        decode_in_place(&mut b"`F0`".to_vec()),
        Err(DecodeError::UnexpectedEnd)
    );
    assert_matches!(
        decode_in_place(&mut b"`F0`9".to_vec()),
        Err(DecodeError::UnexpectedEnd)
    );
}

#[test]
fn test_decode_in_place_lowercase_hex_error() {
    assert_matches!(
        decode_in_place(&mut b"`fe".to_vec()),
        Err(DecodeError::LowercaseHex(EscapedHex(b'f', b'e')))
    );
    assert_matches!(
        decode_in_place(&mut b"`0e".to_vec()),
        Err(DecodeError::LowercaseHex(EscapedHex(b'0', b'e')))
    );
    assert_matches!(
        decode_in_place(&mut b"`f0".to_vec()),
        Err(DecodeError::LowercaseHex(EscapedHex(b'f', b'0')))
    );
}

#[test]
fn test_decode_in_place_invalid_hex_error() {
    assert_matches!(
        decode_in_place(&mut b"`GE".to_vec()),
        Err(DecodeError::InvalidHex(EscapedHex(b'G', b'E')))
    );
    assert_matches!(
        decode_in_place(&mut b"`0G".to_vec()),
        Err(DecodeError::InvalidHex(EscapedHex(b'0', b'G')))
    );
    assert_matches!(
        decode_in_place(&mut b"`G0".to_vec()),
        Err(DecodeError::InvalidHex(EscapedHex(b'G', b'0')))
    );

    assert_matches!(
        decode_in_place(&mut b"`fG".to_vec()),
        Err(DecodeError::InvalidHex(EscapedHex(b'f', b'G')))
    );
    assert_matches!(
        decode_in_place(&mut b"`gF".to_vec()),
        Err(DecodeError::InvalidHex(EscapedHex(b'g', b'F')))
    );
}

#[test]
fn test_decode_in_place_unexpected_escape_error() {
    assert_matches!(
        decode_in_place(&mut b"`65".to_vec()),
        Err(DecodeError::UnexpectedEscape(EscapedHex(b'6', b'5'), 'e'))
    );
}
