use crate::{nibble_to_hex, requires_escape};

/// Encoder state machine.
#[derive(Debug, Clone, Copy)]
pub struct Encoder {
    /// Precomputed characters to emit.
    chars: [u8; 2],
    /// Count of remaining characters to emit: 0, 1, or 2.
    pending: u8,
}

impl Default for Encoder {
    #[inline]
    fn default() -> Self {
        Self {
            chars: [0, 0],
            pending: 0,
        }
    }
}

impl Encoder {
    #[inline]
    pub fn next(&mut self) -> Option<char> {
        if self.pending == 0 {
            return None;
        }

        // Map pending count to array index:
        // - pending = 2 → index = 0 (first)
        // - pending = 1 → index = 1 (second)
        let index = (2 - self.pending) as usize;
        self.pending -= 1;

        Some(self.chars[index] as char)
    }

    #[inline]
    pub fn push(&mut self, byte: u8) -> char {
        if byte == b'`' {
            // Store at index = 1 because next() will access:
            // - pending = 1 → index = 1
            self.chars[1] = b'`';
            self.pending = 1;
            '`'
        } else if requires_escape(byte) {
            self.chars[0] = nibble_to_hex(byte >> 4);
            self.chars[1] = nibble_to_hex(byte & 0x0F);
            self.pending = 2;
            '`'
        } else {
            self.pending = 0;
            byte as char
        }
    }
}
