use crate::{
    decoder::{DecodeStatus, Decoder},
    encoder::Encoder,
    DecodeError,
};

/// The iterator returned by [`crate::encode_iter`].
#[derive(Debug, Clone, Copy)]
pub struct EncodeIter<I> {
    iter: I,
    encoder: Encoder,
}

impl<I> EncodeIter<I> {
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            encoder: Encoder::default(),
        }
    }

    /// Get a reference to the inner iterator.
    pub const fn inner(&self) -> &I {
        &self.iter
    }

    /// Get a mutable reference to the inner iterator.
    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.iter
    }

    /// Take the inner iterator.
    pub fn into_inner(self) -> I {
        self.iter
    }
}

impl<I> Iterator for EncodeIter<I>
where
    I: Iterator<Item = u8>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(encoded) = self.encoder.next() {
            return Some(encoded);
        }

        let byte = self.iter.next()?;
        let encoded = self.encoder.push(byte);
        Some(encoded)
    }
}

/// The iterator returned by [`crate::decode_iter`].
#[derive(Debug, Clone, Copy)]
pub struct DecodeIter<I> {
    iter: I,
    decoder: Decoder,
}

impl<I> DecodeIter<I> {
    pub(crate) fn new(iter: I) -> Self {
        Self {
            iter,
            decoder: Decoder::default(),
        }
    }

    /// Get a reference to the inner iterator.
    pub const fn inner(&self) -> &I {
        &self.iter
    }

    /// Get a mutable reference to the inner iterator.
    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.iter
    }

    /// Take the inner iterator.
    pub fn into_inner(self) -> I {
        self.iter
    }
}

impl<I> Iterator for DecodeIter<I>
where
    I: Iterator<Item = u8>,
{
    type Item = Result<u8, DecodeError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next_byte = self.iter.next();
            match self.decoder.push(next_byte) {
                DecodeStatus::NeedMore => {}
                DecodeStatus::Emit(result) => {
                    return result;
                }
            }
        }
    }
}
