use std::borrow::Cow;

pub fn encode(input: &[u8]) -> Cow<str> {
    // Get the first index that needs to be escaped
    let escape_index = input.iter().position(|byte| requires_escape(*byte));

    match escape_index {
        Some(index) => {
            // We know everything up to `index` does not require escaping
            let validated = &input[..index];
            debug_assert!(validated.is_ascii());

            // SAFETY: We know the input up to this point is valid ASCII and
            // UTF-8, since nothing up to this point needs escaping
            let validated = unsafe { std::str::from_utf8_unchecked(validated) };

            let mut output = String::with_capacity(input.len() + 1);
            output.push_str(validated);

            // Encode the remainder of the input
            let requires_encoding = &input[index..];
            encode_to_string(requires_encoding, &mut output);
            Cow::Owned(output)
        }
        None => {
            debug_assert!(input.is_ascii());

            // SAFETY: We know the entire input is valid ASCII and UTF-8, and
            // additionally doesn't require any bytes to be escaped
            Cow::Borrowed(unsafe { std::str::from_utf8_unchecked(input) })
        }
    }
}

pub fn decode(input: &[u8]) -> Result<Cow<[u8]>, DecodeError> {
    // Get the first index that isn't already a valid unescaped byte
    let escape_index = input.iter().position(|byte| requires_escape(*byte));

    match escape_index {
        Some(index) => {
            // We know everything up to `index` does not need to be unescaped
            let validated = &input[..index];

            let mut output = Vec::with_capacity(validated.len() + 1);
            output.extend_from_slice(validated);

            // Decode the remainder of the input
            let requires_decoding = &input[index..];
            decode_to_vec(requires_decoding, &mut output)?;
            Ok(Cow::Owned(output))
        }
        None => Ok(Cow::Borrowed(input)),
    }
}

pub fn requires_escape(byte: u8) -> bool {
    match byte {
        b'`' => true,
        b'\t' | b'\n' | b'\r' | b' '..=b'~' => false,
        _ => true,
    }
}

pub fn encode_to_string(input: &[u8], output: &mut String) -> usize {
    let mut written = 0;
    output.reserve(input.len());
    for byte in input {
        if *byte == b'`' {
            output.push_str("``");
            written += 2;
        } else if requires_escape(*byte) {
            let [high, low] = byte_to_hex_chars(*byte);
            output.push('`');
            output.push(high);
            output.push(low);

            written += 3;
        } else {
            output.push(*byte as char);
            written += 1;
        }
    }

    written
}

pub fn encode_to_vec(input: &[u8], output: &mut Vec<u8>) -> usize {
    let mut written = 0;
    output.reserve(input.len());
    for byte in input {
        if *byte == b'`' {
            output.extend_from_slice(b"``");
            written += 2;
        } else if requires_escape(*byte) {
            let [high, low] = byte_to_hex_bytes(*byte);
            output.extend_from_slice(&[b'`', high, low]);

            written += 3;
        } else {
            output.push(*byte);
            written += 1;
        }
    }

    written
}

pub fn decode_to_vec(input: &[u8], output: &mut Vec<u8>) -> Result<usize, DecodeError> {
    let mut written = 0;
    let mut iter = input.iter();
    while let Some(byte) = iter.next() {
        if *byte == b'`' {
            let escaped = iter.next().ok_or(DecodeError::UnexpectedEnd)?;
            match escaped {
                b'`' => {
                    output.push(b'`');
                    written += 1;
                }
                high => {
                    let low = iter.next().ok_or(DecodeError::UnexpectedEnd)?;
                    let byte = hex_bytes_to_byte([*high, *low])?;
                    output.push(byte);
                    written += 1;
                }
            }
        } else if requires_escape(*byte) {
            return Err(DecodeError::InvalidByte(*byte));
        } else {
            output.push(*byte);
            written += 1;
        }
    }

    Ok(written)
}

fn byte_to_hex_bytes(byte: u8) -> [u8; 2] {
    let high = byte >> 4;
    let low = byte & 0x0F;

    let high_byte = match high {
        0..=9 => b'0' + high,
        10..=15 => b'A' + high - 10,
        _ => unreachable!(),
    };
    let low_byte = match low {
        0..=9 => b'0' + low,
        10..=15 => b'A' + low - 10,
        _ => unreachable!(),
    };

    [high_byte, low_byte]
}

fn byte_to_hex_chars(byte: u8) -> [char; 2] {
    let [high_byte, low_byte] = byte_to_hex_bytes(byte);
    [high_byte as char, low_byte as char]
}

fn hex_bytes_to_byte([high, low]: [u8; 2]) -> Result<u8, DecodeError> {
    enum HexCharResult {
        Valid(u8),
        Lowercase(char),
        Invalid(char),
    }

    let high_value = match high {
        b'0'..=b'9' => HexCharResult::Valid(high - b'0'),
        b'A'..=b'F' => HexCharResult::Valid(high - b'A' + 10),
        b'a'..=b'f' => HexCharResult::Lowercase(high as char),
        _ => HexCharResult::Invalid(high as char),
    };

    let low_value = match low {
        b'0'..=b'9' => HexCharResult::Valid(low - b'0'),
        b'A'..=b'F' => HexCharResult::Valid(low - b'A' + 10),
        b'a'..=b'f' => HexCharResult::Lowercase(low as char),
        _ => HexCharResult::Invalid(low as char),
    };

    let byte = match (high_value, low_value) {
        (HexCharResult::Valid(high_value), HexCharResult::Valid(low_value)) => {
            (high_value << 4) | low_value
        }
        (HexCharResult::Invalid(_), _) | (_, HexCharResult::Invalid(_)) => {
            return Err(DecodeError::InvalidHex(EscapedHex(
                high as char,
                low as char,
            )));
        }
        (HexCharResult::Lowercase(_), _) | (_, HexCharResult::Lowercase(_)) => {
            return Err(DecodeError::LowercaseHex(EscapedHex(
                high as char,
                low as char,
            )));
        }
    };

    if !requires_escape(byte) {
        return Err(DecodeError::UnexpectedEscape(
            EscapedHex(high as char, low as char),
            byte as char,
        ));
    }

    Ok(byte)
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("invalid encoded byte 0x{0:02x}")]
    InvalidByte(u8),
    #[error("unexpected end after `")]
    UnexpectedEnd,
    #[error("unexpected escape {0}, expected {1}")]
    UnexpectedEscape(EscapedHex, char),
    #[error("expected uppercase hex sequence, found {0}")]
    LowercaseHex(EscapedHex),
    #[error("invalid hex sequence {0}")]
    InvalidHex(EscapedHex),
}

#[derive(Debug)]
pub struct EscapedHex(pub char, pub char);

impl std::fmt::Display for EscapedHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}{}", self.0, self.1)
    }
}
