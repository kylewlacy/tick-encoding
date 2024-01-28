# tick-encoding

`tick-encoding` is a simple encoding scheme that encodes arbitrary binary data into an ASCII string. It's primarily designed for stuffing usually-ASCII data into JSON strings. It's very similar to percent encoding / URL encoding, but with a few key differences:

- Uses backtick (\`) instead of percent (`%`) as the escape character
- One canonical encoding for any binary data
- One consistent set of characters that require escaping
- Less characters need escaping
