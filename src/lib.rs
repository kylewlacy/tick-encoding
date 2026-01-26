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

/// Lookup table for knowing if a byte requires escaping.
const REQUIRES_ESCAPE_TABLE: [bool; 256] = {
    let mut table = [true; 256]; // Default: requires escape

    // Whitespace that doesn't require escaping
    table[b'\t' as usize] = false;
    table[b'\n' as usize] = false;
    table[b'\r' as usize] = false;

    // Printable ASCII (space through tilde) except backtick
    let mut i = b' ';
    while i <= b'~' {
        if i != b'`' {
            table[i as usize] = false;
        }
        i += 1;
    }

    table
};

const HEX_NIBBLE_DECODE_INVALID_ERR: u8 = 0xFF;
const HEX_NIBBLE_DECODE_LOWERCASE_ERR: u8 = 0xFE;

/// Lookup table for hex ASCII character to nibble.
///
/// Values:
/// - 0x00-0x0F: Valid uppercase hex digit
/// - `HEX_LOWERCASE`: Lowercase hex digit (a-f)
/// - `HEX_INVALID`: Invalid character
const HEX_NIBBLE_DECODE_TABLE: [u8; 256] = {
    let mut table = [HEX_NIBBLE_DECODE_INVALID_ERR; 256];

    // Digits '0'-'9' -> 0-9
    let mut i = b'0';
    while i <= b'9' {
        table[i as usize] = i - b'0';
        i += 1;
    }

    // Uppercase 'A'-'F' -> 10-15
    i = b'A';
    while i <= b'F' {
        table[i as usize] = i - b'A' + 10;
        i += 1;
    }

    // Lowercase 'a'-'f' -> lowercase error
    i = b'a';
    while i <= b'f' {
        table[i as usize] = HEX_NIBBLE_DECODE_LOWERCASE_ERR;
        i += 1;
    }

    table
};

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
#[must_use]
pub fn encode(input: &[u8]) -> Cow<'_, str> {
    // Get the first index that needs to be escaped
    input
        .iter()
        .position(|byte| requires_escape(*byte))
        // If no escape needed, borrow input. Otherwise encode from that index
        .map_or_else(
            || {
                debug_assert!(input.is_ascii());

                // SAFETY: We know the entire input is valid ASCII and UTF-8, and
                // additionally doesn't require any bytes to be escaped
                Cow::Borrowed(from_utf8_unchecked_potentially_unsafe(input))
            },
            |index| {
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
            },
        )
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
/// # Errors
///
/// Returns a [`DecodeError`] if the input is not valid tick-encoded data.
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
pub fn decode(input: &[u8]) -> Result<Cow<'_, [u8]>, DecodeError> {
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

/// Decode a tick-encoded ASCII string in-place.
///
/// Takes a byte slice containing a tick-encoded ASCII string, and decodes it
/// in-place, writing back into the same byte slice. Returns a sub-slice
/// containing just the decoded bytes (the bytes past the returned sub-slice
/// are left unchanged).
///
/// # Errors
///
/// Returns a [`DecodeError`] if the input is not valid tick-encoded data.
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
                    let byte = hex_bytes_to_byte(*high, *low)?;
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
/// - Printable characters except backtick (0x21 to 0x59, 0x61 to 0x7E)
#[must_use]
pub const fn requires_escape(byte: u8) -> bool {
    REQUIRES_ESCAPE_TABLE[byte as usize]
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
    for &byte in input {
        if byte == b'`' {
            output.push_str("``");
            written += 2;
        } else if requires_escape(byte) {
            let [high, low] = byte_to_hex_chars(byte);
            output.push('`');
            output.push(high);
            output.push(low);

            written += 3;
        } else {
            output.push(byte as char);
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
    for &byte in input {
        if byte == b'`' {
            output.extend_from_slice(b"``");
            written += 2;
        } else if requires_escape(byte) {
            let [high, low] = byte_to_hex_bytes(byte);
            output.extend_from_slice(&[b'`', high, low]);

            written += 3;
        } else {
            output.push(byte);
            written += 1;
        }
    }

    written
}

/// Decode tick-encoded ASCII input and append the result to a vector.
///
/// Returns the number of bytes appended. Returns an error if the result
/// isn't a valid ASCII string, or isn't a valid canonical tick-encoding.
///
/// # Errors
///
/// Returns a [`DecodeError`] if the input is not valid tick-encoded data.
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
    while let Some(&byte) = iter.next() {
        if byte == b'`' {
            let escaped = iter.next().ok_or(DecodeError::UnexpectedEnd)?;
            match escaped {
                b'`' => {
                    output.push(b'`');
                    written += 1;
                }
                high => {
                    let low = iter.next().ok_or(DecodeError::UnexpectedEnd)?;
                    let byte = hex_bytes_to_byte(*high, *low)?;
                    output.push(byte);
                    written += 1;
                }
            }
        } else if requires_escape(byte) {
            return Err(DecodeError::InvalidByte(byte));
        } else {
            output.push(byte);
            written += 1;
        }
    }

    Ok(written)
}

/// Convert a nibble to its uppercase hex ASCII character.
const fn nibble_to_hex(n: u8) -> u8 {
    // 0-9 → '0'-'9'
    // 10-15 → 'A'-'F' (add 7 to skip the ASCII gap between '9' and 'A')
    n + b'0' + ((n > 9) as u8) * 7
}

/// Convert a byte to its two-character uppercase hex representation.
const fn byte_to_hex_bytes(byte: u8) -> [u8; 2] {
    [nibble_to_hex(byte >> 4), nibble_to_hex(byte & 0x0F)]
}

const fn byte_to_hex_chars(byte: u8) -> [char; 2] {
    let [high_byte, low_byte] = byte_to_hex_bytes(byte);
    [high_byte as char, low_byte as char]
}

/// Decode two hex ASCII characters into a single byte.
///
/// Returns an error if:
/// - Either character is not a valid hex digit (`InvalidHex`)
/// - Either character is lowercase a-f (`LowercaseHex`)
/// - The decoded byte doesn't require escaping (`UnexpectedEscape`)
const fn hex_bytes_to_byte(high: u8, low: u8) -> Result<u8, DecodeError> {
    let high_value = HEX_NIBBLE_DECODE_TABLE[high as usize];
    let low_value = HEX_NIBBLE_DECODE_TABLE[low as usize];

    match (high_value, low_value) {
        // Both valid hex digits (0x00-0x0F)
        (0..=0x0F, 0..=0x0F) => {
            let byte = (high_value << 4) | low_value;

            if byte == b'`' || !requires_escape(byte) {
                return Err(DecodeError::UnexpectedEscape(
                    EscapedHex(high, low),
                    byte as char,
                ));
            }

            Ok(byte)
        }
        // At least one invalid character
        (HEX_NIBBLE_DECODE_INVALID_ERR, _) | (_, HEX_NIBBLE_DECODE_INVALID_ERR) => {
            Err(DecodeError::InvalidHex(EscapedHex(high, low)))
        }
        // Must be lowercase
        _ => Err(DecodeError::LowercaseHex(EscapedHex(high, low))),
    }
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
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum DecodeError {
    /// Encountered an invalid byte in the string. This could either by a
    /// non-ASCII byte or an ASCII byte that requires escaping (see
    /// [`requires_escape`]).
    #[cfg_attr(feature = "std", error("invalid encoded byte 0x{0:02x}"))]
    InvalidByte(u8),
    /// Reached the end of the string following a backtick (\`). A backtick
    /// must be followed by either another backtick or a 2-digit hex value.
    #[cfg_attr(feature = "std", error("unexpected end after `"))]
    UnexpectedEnd,
    /// Tried to decode a 2-digit hex value, but the value does not require
    /// escaping (see [`requires_escape`]).
    #[cfg_attr(feature = "std", error("unexpected escape {0}, expected {1}"))]
    UnexpectedEscape(EscapedHex, char),
    /// Tried to decode a 2-digit hex value, but the hex value contained
    /// the values `[a-f]`. Escaped hex values must use `[A-F]`.
    #[cfg_attr(feature = "std", error("expected uppercase hex sequence, found {0}"))]
    LowercaseHex(EscapedHex),
    /// Tried to decode a 2-digit hex value, but an invalid hex digit
    /// was found. Escaped hex values must use the characters `[0-9A-F]`.
    #[cfg_attr(feature = "std", error("invalid hex sequence {0}"))]
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
            write!(f, "0x{high:02X} 0x{low:02X}")
        } else {
            write!(f, "`{}{}", *high as char, *low as char)
        }
    }
}
