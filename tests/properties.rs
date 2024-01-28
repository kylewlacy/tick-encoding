#![cfg(feature = "alloc")]

use std::borrow::Cow;

use assert_matches::assert_matches;
use proptest::prelude::*;

proptest! {
    #[test]
    fn encode_any_bytes(bytes in any::<Vec<u8>>()) {
        let encoded = tick_encoding::encode(&bytes);
        assert!(encoded.len() >= bytes.len());
    }

    #[test]
    fn decode_any_valid_encoding(encoded_string in "([\t\n\r -_a-~]|`(`|0[0-8]|1[124-9A-F]|[89A-F][0-9A-F]))*") {
        let decoded = tick_encoding::decode(encoded_string.as_bytes()).unwrap();
        assert!(decoded.len() <= encoded_string.len());
    }

    #[test]
    fn decode_invalid_encoding_error(non_encoded_string in "([\t\n\r -_a-~]|`(`|0[0-8]|1[124-9A-F]|[89A-F][0-9A-F]))*([^\t\n\r -~])([\t\n\r -_a-~]|`(`|0[0-8]|1[124-9A-F]|[89A-F][0-9A-F]))*") {
        let result = tick_encoding::decode(non_encoded_string.as_bytes());
        assert_matches!(result, Err(_));
    }

    #[test]
    fn encode_then_decode(bytes in any::<Vec<u8>>()) {
        let encoded = tick_encoding::encode(&bytes);
        let decoded = tick_encoding::decode(encoded.as_bytes()).unwrap();

        assert_eq!(decoded, bytes);
    }

    #[test]
    fn decode_then_encode(encoded_string in "([\t\n\r -_a-~]|`(`|0[0-8]|1[124-9A-F]|[89A-F][0-9A-F]))*") {
        let decoded = tick_encoding::decode(encoded_string.as_bytes()).unwrap();
        let re_encoded = tick_encoding::encode(&decoded);
        assert_eq!(encoded_string, re_encoded);
    }

    #[test]
    fn decode_then_encode_canonical(possibly_encoded_string in "[\t\n\r -~]*") {
        // Iterate over the string until we hit a decoding error. Then, get
        // the index of the iterator (the index where we hit the error).
        let mut possibly_encoded_iter = possibly_encoded_string.as_bytes().iter();
        let _decode_error = tick_encoding::decode_iter(possibly_encoded_iter.by_ref().copied()).find_map(|result| result.err());
        let first_invalid_index = possibly_encoded_string.len() - possibly_encoded_iter.as_slice().len();

        // Because we may have hit an `UnexpectedEnd` error, we might need to
        // cut a few more bytes off the end of the string to get a valid
        // encoding
        let mut encoded = &possibly_encoded_string[..first_invalid_index];
        while tick_encoding::decode(encoded.as_bytes()).is_err() {
            encoded = &encoded[..encoded.len() - 1];
        }

        let decoded = tick_encoding::decode(encoded.as_bytes()).unwrap();
        let re_encoded = tick_encoding::encode(&decoded);
        let re_decoded = tick_encoding::decode(re_encoded.as_bytes()).unwrap();

        assert_eq!(encoded, re_encoded);
        assert_eq!(decoded, re_decoded);
    }

    #[test]
    fn encode_unescaped_borrows(unescaped_string in "[\t\n\r -_a-~]*") {
        let encoded = tick_encoding::encode(unescaped_string.as_bytes());
        assert_matches!(encoded, Cow::Borrowed(_));
    }

    #[test]
    fn decode_unescaped_borrows(unescaped_string in "[\t\n\r -_a-~]*") {
        let decoded = tick_encoding::decode(unescaped_string.as_bytes()).unwrap();
        assert_matches!(decoded, Cow::Borrowed(_));
    }

    #[test]
    fn encode_to_string(bytes in any::<Vec<u8>>()) {
        let encoded = tick_encoding::encode(&bytes);

        let mut buffer = String::new();
        let count = tick_encoding::encode_to_string(&bytes, &mut buffer);
        assert_eq!(buffer, encoded);
        assert_eq!(count, encoded.len());
    }

    #[test]
    fn encode_to_string_append(bytes in any::<Vec<u8>>(), prefix in any::<String>()) {
        let encoded = tick_encoding::encode(&bytes);

        let mut buffer = prefix.clone();
        let count = tick_encoding::encode_to_string(&bytes, &mut buffer);

        let buffer_prefix = &buffer[..prefix.len()];
        let buffer_appended = &buffer[prefix.len()..];

        assert_eq!(buffer_prefix, prefix);
        assert_eq!(buffer_appended, encoded);
        assert_eq!(count, encoded.len());
    }

    #[test]
    fn encode_to_vec(bytes in any::<Vec<u8>>()) {
        let encoded = tick_encoding::encode(&bytes);

        let mut buffer = vec![];
        let count = tick_encoding::encode_to_vec(&bytes, &mut buffer);
        assert_eq!(buffer, encoded.as_bytes());
        assert_eq!(count, encoded.len());
    }

    #[test]
    fn encode_to_vec_append(bytes in any::<Vec<u8>>(), prefix in any::<Vec<u8>>()) {
        let encoded = tick_encoding::encode(&bytes);

        let mut buffer = prefix.clone();
        let count = tick_encoding::encode_to_vec(&bytes, &mut buffer);

        let buffer_prefix = &buffer[..prefix.len()];
        let buffer_appended = &buffer[prefix.len()..];

        assert_eq!(buffer_prefix, prefix);
        assert_eq!(buffer_appended, encoded.as_bytes());
        assert_eq!(count, encoded.len());
    }

    #[test]
    fn decode_to_vec(bytes in any::<Vec<u8>>()) {
        let encoded = tick_encoding::encode(&bytes);

        let mut buffer = vec![];
        let count = tick_encoding::decode_to_vec(encoded.as_bytes(), &mut buffer).unwrap();
        assert_eq!(buffer, bytes);
        assert_eq!(count, bytes.len());
    }

    #[test]
    fn decode_to_vec_appended(bytes in any::<Vec<u8>>(), prefix in any::<Vec<u8>>()) {
        let encoded = tick_encoding::encode(&bytes);

        let mut buffer = prefix.clone();
        let count = tick_encoding::decode_to_vec(encoded.as_bytes(), &mut buffer).unwrap();

        let buffer_prefix = &buffer[..prefix.len()];
        let buffer_appended = &buffer[prefix.len()..];

        assert_eq!(buffer_prefix, prefix);
        assert_eq!(buffer_appended, bytes);
        assert_eq!(count, bytes.len());
    }

    #[test]
    fn decode_in_place(bytes in any::<Vec<u8>>()) {
        let encoded = tick_encoding::encode(&bytes);

        let mut buffer = encoded.clone().into_owned().into_bytes();
        let decoded = tick_encoding::decode_in_place(&mut buffer).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn encode_iter(bytes in any::<Vec<u8>>()) {
        let encoded = tick_encoding::encode(&bytes);

        let encode_iter = tick_encoding::encode_iter(bytes.iter().copied());
        let iter_encoded = encode_iter.collect::<String>();

        assert_eq!(encoded, iter_encoded);
    }

    #[test]
    fn decode_iter(bytes in any::<Vec<u8>>()) {
        let encoded = tick_encoding::encode(&bytes);

        let decode_iter = tick_encoding::decode_iter(encoded.bytes());
        let iter_decoded = decode_iter.collect::<Result<Vec<u8>, tick_encoding::DecodeError>>().unwrap();

        assert_eq!(bytes, iter_decoded);
    }
}
