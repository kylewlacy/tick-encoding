#![cfg_attr(feature = "safe", deny(unsafe_code))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "std", doc = include_str!("../README.md"))]

pub(crate) mod decoder;
pub(crate) mod encoder;
pub mod iter;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
use alloc::{borrow::Cow, string::String, vec::Vec};

/// Encode the given input as a string, escaping any bytes that require it.
/// If no bytes require escaping, then the result will be borrowed from
/// the input.
///
/// ## Example
///
/// ```
/// # #![cfg(feature = "alloc")]
/// let encoded = tick_encoding::encode(b"hello world!");
/// assert_eq!(encoded, "hello world!");
///
/// let encoded = tick_encoding::encode(&[0x00, 0xFF]);
/// assert_eq!(encoded, "`00`FF");
/// ```
#[cfg(feature = "alloc")]
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
            let validated = from_utf8_unchecked_potentially_unsafe(validated);

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
            Cow::Borrowed(from_utf8_unchecked_potentially_unsafe(input))
        }
    }
}

/// Return an iterator that encodes the bytes from the input iterator.
///
/// ## Example
///
/// ```
/// let iter = tick_encoding::encode_iter(b"x: \x00".iter().copied());
/// assert_eq!(iter.collect::<String>(), "x: `00");
/// ```
pub fn encode_iter<I>(iter: I) -> iter::EncodeIter<I::IntoIter>
where
    I: IntoIterator<Item = u8>,
{
    iter::EncodeIter::new(iter.into_iter())
}

/// Decode the given encoded input into a byte array. If no bytes need to
/// be un-escapeed, then the result will be borrowed from the input.
///
/// Returns an error if the input isn't a valid ASCII string, or isn't a
/// valid canonical tick-encoding.
///
/// ## Example
///
/// ```
/// # #![cfg(feature = "alloc")]
/// let decoded = tick_encoding::decode(b"hello world!").unwrap();
/// assert_eq!(decoded, "hello world!".as_bytes());
///
/// let decoded = tick_encoding::decode(b"`00`FF").unwrap();
/// assert_eq!(decoded, [0x00, 0xFF].as_slice());
/// ```
#[cfg(feature = "alloc")]
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

/// Return an iterator that decodes the tick-encoded characters from the input
/// iterator. Returns `Some(Err(_))` if the input character sequence is invalid,
/// then returns `None` after that.
///
/// ## Example
///
/// ```
/// let iter = tick_encoding::decode_iter(b"`00`01".iter().copied());
/// assert_eq!(iter.collect::<Result<Vec<_>, _>>().unwrap(), vec![0x00, 0x01]);
/// ```
pub fn decode_iter<I>(iter: I) -> iter::DecodeIter<I::IntoIter>
where
    I: IntoIterator<Item = u8>,
{
    iter::DecodeIter::new(iter.into_iter())
}

/// Take a byte slice containing a tick-encoded ASCII string, and decode it
/// in-place, writing back into the same byte slice. Returns a sub-slice
/// containing just the decoded bytes (the bytes past the returned sub-slice
/// are left unchanged).
///
/// ## Example
///
/// ```rust
/// let mut buffer = b"bytes: `00`01`02`03".to_vec();
/// let decoded = tick_encoding::decode_in_place(&mut buffer).unwrap();
/// assert_eq!(decoded, b"bytes: \x00\x01\x02\x03");
/// ```
pub fn decode_in_place(input: &mut [u8]) -> Result<&mut [u8], DecodeError> {
    // Get the first index that isn't already a valid unescaped byte
    let Some(escape_index) = input.iter().position(|byte| requires_escape(*byte)) else {
        // Nothing needs to be unescaped
        return Ok(input);
    };

    // Walk through the rest of the input. The bytes between `0..head` have been
    // decoded, and the bytes between `tail..input.len()` are still encoded.
    // Since the encoded form is always as long as the decoded form or longer,
    // `head` will always be less than or equal to `tail`.
    //
    // This technique is very similar to the one from `in-place-string-map` (see
    // https://crates.io/crates/in-place-string-map), but works on a byte slice
    // instead.
    let mut head = escape_index;
    let mut tail = escape_index;
    while tail < input.len() {
        if input[tail] == b'`' {
            let escaped = input.get(tail + 1).ok_or(DecodeError::UnexpectedEnd)?;
            match escaped {
                b'`' => {
                    input[head] = b'`';
                    tail += 2;
                    head += 1;
                }
                high => {
                    let low = input.get(tail + 2).ok_or(DecodeError::UnexpectedEnd)?;
                    let byte = hex_bytes_to_byte([*high, *low])?;
                    input[head] = byte;
                    tail += 3;
                    head += 1;
                }
            }
        } else if requires_escape(input[tail]) {
            return Err(DecodeError::InvalidByte(input[tail]));
        } else {
            input[head] = input[tail];
            tail += 1;
            head += 1;
        }
    }

    let decoded = &mut input[..head];
    Ok(decoded)
}

/// Returns true if the given byte must be escaped with a backtick.
///
/// The following ASCII bytes **do not** require escaping, and are left
/// un-escaped in a tick-encoded string:
///
/// - Tab (`\t`, 0x09)
/// - Newline (`\n`, 0x0A)
/// - Carriage return (`\r`, 0x0D)
/// - Space (` `, 0x20)
/// - Printable characters except bactick (0x21 to 0x59, 0x61 to 0x7E)
pub fn requires_escape(byte: u8) -> bool {
    match byte {
        b'`' => true,
        b'\t' | b'\n' | b'\r' | b' '..=b'~' => false,
        _ => true,
    }
}

/// Encode the given input, and append the result to `output`. Returns
/// the number of bytes / characters appended (only ASCII characters are
/// appended).
///
/// ## Example
///
/// ```
/// # #![cfg(feature = "alloc")]
/// let mut output = String::new();
/// let count = tick_encoding::encode_to_string("hello, world! 🙂".as_bytes(), &mut output);
/// assert_eq!(output, "hello, world! `F0`9F`99`82");
/// assert_eq!(count, 26);
/// ```
#[cfg(feature = "alloc")]
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

/// Encode the given input, and append the result to `output`. Returns
/// the number of bytes appended.
///
/// ## Example
///
/// ```
/// let mut output = vec![];
/// let count = tick_encoding::encode_to_vec("hello, world! 🙂".as_bytes(), &mut output);
/// assert_eq!(output, b"hello, world! `F0`9F`99`82");
/// assert_eq!(count, 26);
/// ```
#[cfg(feature = "alloc")]
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

/// Decode the given tick-encoded ASCII input, and append the result to
/// `output`. Returns the number of bytes appended. Returns an error
/// if the result isn't a valid ASCII string, or isn't a valid canonical
/// tick-encoding.
///
/// ## Example
///
/// ```
/// let mut output = vec![];
/// let count = tick_encoding::decode_to_vec(b"hello, world! `F0`9F`99`82", &mut output).unwrap();
/// let output_str = core::str::from_utf8(&output).unwrap();
/// assert_eq!(output_str, "hello, world! 🙂");
/// assert_eq!(count, 18);
/// ```
#[cfg(feature = "alloc")]
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
            return Err(DecodeError::InvalidHex(EscapedHex(high, low)));
        }
        (HexCharResult::Lowercase(_), _) | (_, HexCharResult::Lowercase(_)) => {
            return Err(DecodeError::LowercaseHex(EscapedHex(high, low)));
        }
    };

    if byte == b'`' || !requires_escape(byte) {
        return Err(DecodeError::UnexpectedEscape(
            EscapedHex(high, low),
            byte as char,
        ));
    }

    Ok(byte)
}

#[cfg(feature = "safe")]
fn from_utf8_unchecked_potentially_unsafe(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).unwrap()
}

#[cfg(not(feature = "safe"))]
fn from_utf8_unchecked_potentially_unsafe(bytes: &[u8]) -> &str {
    debug_assert!(bytes.is_ascii());
    unsafe { core::str::from_utf8_unchecked(bytes) }
}

/// An error trying to decode a tick-encoded string.
#[derive(Debug)]
#[cfg_attr(feature = "dep:thiserror", derive(thiserror::Error))]
pub enum DecodeError {
    /// Encountered an invalid byte in the string. This could either by a
    /// non-ASCII byte or an ASCII byte that requires escaping (see
    /// [requires_escape]).
    #[cfg_attr(feature = "dep:thiserror", error("invalid encoded byte 0x{0:02x}"))]
    InvalidByte(u8),
    /// Reached the end of the string following a backtick (\`). A backtick
    /// must be followed by either another backtick or a 2-digit hex value.
    #[cfg_attr(feature = "dep:thiserror", error("unexpected end after `"))]
    UnexpectedEnd,
    /// Tried to decode a 2-digit hex value, but the value does not require
    /// escaping (see [requires_escape]).
    #[cfg_attr(
        feature = "dep:thiserror",
        error("unexpected escape {0}, expected {1}")
    )]
    UnexpectedEscape(EscapedHex, char),
    /// Tried to decode a 2-digit hex value, but the hex value contained
    /// the values `[a-f]`. Escaped hex values must use `[A-F]`.
    #[cfg_attr(
        feature = "dep:thiserror",
        error("expected uppercase hex sequence, found {0}")
    )]
    LowercaseHex(EscapedHex),
    /// Tried to decode a 2-digit hex value, but an invalid hex digit
    /// was found. Escaped hex values must use the characters `[0-9A-F]`.
    #[cfg_attr(feature = "dep:thiserror", error("invalid hex sequence {0}"))]
    InvalidHex(EscapedHex),
}

/// A two-digit escaped hex sequence, prefixed with a backtick.
pub struct EscapedHex(pub u8, pub u8);

impl core::fmt::Debug for EscapedHex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self(high, low) = self;
        if requires_escape(*high) || requires_escape(*low) {
            f.debug_tuple("EscapedHex")
                .field(&self.0)
                .field(&self.1)
                .finish()
        } else {
            f.debug_tuple("EscapedHex")
                .field(&(*high as char))
                .field(&(*low as char))
                .finish()
        }
    }
}

impl core::fmt::Display for EscapedHex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self(high, low) = self;
        if requires_escape(*high) || requires_escape(*low) {
            write!(f, "0x{:02X} 0x{:02X}", high, low)
        } else {
            write!(f, "`{}{}", *high as char, *low as char)
        }
    }
}
