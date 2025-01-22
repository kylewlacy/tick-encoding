use crate::{hex_bytes_to_byte, requires_escape, DecodeError};

#[derive(Debug, Default, Clone, Copy)]
pub enum Decoder {
    #[default]
    Ready,
    Finished,
    Tick,
    TickHalfHex(u8),
}

impl Decoder {
    pub fn push(&mut self, input: Option<u8>) -> DecodeStatus {
        match (*self, input) {
            (Self::Finished, _) => DecodeStatus::Emit(None),
            (Self::Ready, Some(input)) => {
                if input == b'`' {
                    *self = Self::Tick;
                    DecodeStatus::NeedMore
                } else if requires_escape(input) {
                    *self = Self::Finished;
                    DecodeStatus::Emit(Some(Err(DecodeError::InvalidByte(input))))
                } else {
                    DecodeStatus::Emit(Some(Ok(input)))
                }
            }
            (Self::Ready, None) => {
                *self = Self::Finished;
                DecodeStatus::Emit(None)
            }
            (Self::Tick, Some(input)) => {
                if input == b'`' {
                    *self = Self::Ready;
                    DecodeStatus::Emit(Some(Ok(b'`')))
                } else {
                    *self = Self::TickHalfHex(input);
                    DecodeStatus::NeedMore
                }
            }
            (Self::Tick, None) => {
                *self = Self::Finished;
                DecodeStatus::Emit(Some(Err(DecodeError::UnexpectedEnd)))
            }
            (Self::TickHalfHex(high), Some(low)) => {
                let byte_result = hex_bytes_to_byte([high, low]);
                match byte_result {
                    Ok(byte) => {
                        *self = Self::Ready;
                        DecodeStatus::Emit(Some(Ok(byte)))
                    }
                    Err(error) => {
                        *self = Self::Finished;
                        DecodeStatus::Emit(Some(Err(error)))
                    }
                }
            }
            (Self::TickHalfHex(_), None) => {
                *self = Self::Finished;
                DecodeStatus::Emit(Some(Err(DecodeError::UnexpectedEnd)))
            }
        }
    }
}

pub enum DecodeStatus {
    NeedMore,
    Emit(Option<Result<u8, DecodeError>>),
}
