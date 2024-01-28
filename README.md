# tick-encoding

`tick-encoding` is a simple encoding scheme that encodes arbitrary binary data into an ASCII string. It's primarily designed for stuffing usually-ASCII data into JSON strings. It's very similar to percent encoding / URL encoding, but with a few key differences:

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
