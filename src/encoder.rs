use crate::{byte_to_hex_chars, requires_escape};

#[derive(Debug, Default, Clone, Copy)]
pub enum Encoder {
    #[default]
    Ready,
    EmitOne([char; 1]),
    EmitTwo([char; 2]),
}

impl Encoder {
    #[inline]
    pub fn next(&mut self) -> Option<char> {
        match *self {
            Self::Ready => None,
            Self::EmitOne([byte]) => {
                *self = Self::Ready;
                Some(byte)
            }
            Self::EmitTwo([a, b]) => {
                *self = Self::EmitOne([b]);
                Some(a)
            }
        }
    }

    #[inline]
    pub fn push(&mut self, byte: u8) -> char {
        if byte == b'`' {
            *self = Self::EmitOne(['`']);
            '`'
        } else if requires_escape(byte) {
            let hex = byte_to_hex_chars(byte);
            *self = Self::EmitTwo(hex);
            '`'
        } else {
            *self = Self::Ready;
            byte as char
        }
    }
}
