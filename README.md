[![Crates.io](https://img.shields.io/crates/v/tick-encoding)](https://crates.io/crates/tick-encoding)
[![docs.rs](https://img.shields.io/docsrs/tick-encoding)](https://docs.rs/tick-encoding/)
[![Minimum Supported Rust Version](https://img.shields.io/crates/msrv/tick-encoding)](https://crates.io/crates/tick-encoding)
[![Tests](https://img.shields.io/github/actions/workflow/status/kylewlacy/tick-encoding/.github%2Fworkflows%2Fci.yml?label=tests)
](https://github.com/kylewlacy/tick-encoding/actions/workflows/ci.yml)

Tick Encoding is a simple encoding scheme that encodes arbitrary binary data into an ASCII string. It's primarily designed for stuffing usually-ASCII data into JSON strings. It's very similar to percent encoding / URL encoding, but with a few key differences:

- Uses backtick (\`) instead of percent (`%`) as the escape character
- One canonical encoding for any binary data
- One consistent set of characters that require escaping
- Less characters need escaping

## Usage

Install `tick-encoding` as a dependency by running `cargo add tick-encoding`.

```rust
// Encode the input into a tick-encoded ASCII string
let encoded = tick_encoding::encode("hello, world! 🙂".as_bytes());
assert_eq!(encoded, "hello, world! `F0`9F`99`82");

// Decode it back into a UTF-8 string
let decoded = tick_encoding::decode(encoded.as_bytes()).unwrap();
let decoded_str = std::str::from_utf8(&decoded).unwrap();
assert_eq!(decoded_str, "hello, world! 🙂");
```

## Crate features

The `tick-encoding` crate includes the following features:

- `std` (default): Enables functionality using Rust's standard library. Disable to build in `#![no_std]` mode.
- `alloc` (default): Enables functionality that depends on the global allocator. Disabling this will only give access to a subset of functionality.
- `safe`: Avoid unsafe code. By default, a small amount of unsafe is used (all checked with extensive unit tests, property tests, and Miri checks). Enabling this feature enables the `#![deny(unsafe_code)]` lint at the crate level, and switches to purely safe code.

## Encoding scheme

The encoding scheme for Tick Encoding is straightforward:

- All printable ASCII bytes plus space and excluding backtick (\`) are encoded as-is (`0x20` to `0x5F` and `0x61` to `0x7E`)
- ASCII tabs, newlines, and carriage returns are encoded as-is (`0x09`, `0x0A`, and `0x0D`)
- Backtick (\`) is encoded as two backticks (`0x60` becomes `0x60 0x60`)
- All other bytes are encoded as backtick followed by two uppercase hexadecimal characters

Decoding just reverses the process. To ensure that decoding and re-encoding produces the same output string, the encoded string is validated while decoding:

- The encoded string can only contain printable ASCII characters, spaces, tabs, newlines, and carriage returns
- A backtick must be followed by a backtick or two uppercase hexadecimal characters
