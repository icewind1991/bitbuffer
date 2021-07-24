use crate::Endianness;
use std::cmp::min;
use std::iter::{once, repeat};
use std::marker::PhantomData;
use std::mem::size_of;

const USIZE_BITS: usize = size_of::<usize>() * 8;

pub struct WriteBuffer<'a, E: Endianness> {
    bit_len: usize,
    bytes: &'a mut Vec<u8>,
    endianness: PhantomData<E>,
}

impl<'a, E: Endianness> WriteBuffer<'a, E> {
    pub fn new(bytes: &'a mut Vec<u8>, _endianness: E) -> Self {
        WriteBuffer {
            bit_len: 0,
            bytes,
            endianness: PhantomData,
        }
    }

    /// The number of written bits in the buffer
    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    pub fn push_non_fit_bits<I>(&mut self, bits: I, count: usize)
    where
        I: ExactSizeIterator,
        I: DoubleEndedIterator<Item = u8>,
    {
        let full_bytes = min(bits.len() - 1, count / 8);

        let counts = repeat(8)
            .take(full_bytes)
            .chain(once(count - full_bytes * 8));
        if E::is_le() {
            bits.zip(counts)
                .for_each(|(chunk, count)| self.push_bits(chunk as usize, count))
        } else {
            bits.take(count / 8 + 1)
                .rev()
                .zip(counts)
                .for_each(|(chunk, count)| self.push_bits(chunk as usize, count))
        }
    }

    /// Push up to an usize worth of bits
    pub fn push_bits(&mut self, bits: usize, count: usize) {
        if count == 0 {
            return;
        }
        debug_assert!(count < USIZE_BITS - 8);

        // ensure there are no stray bits
        let bits = bits & (usize::MAX >> (USIZE_BITS - count));

        let bit_offset = self.bit_len & 7;
        let last_written_byte = if bit_offset > 0 {
            self.bytes.pop().unwrap_or(0)
        } else {
            0
        };
        let merged_byte_count = (count + bit_offset + 7) / 8;

        if E::is_le() {
            let merged = last_written_byte as usize | bits << bit_offset;
            self.bytes
                .extend_from_slice(&merged.to_le_bytes()[0..merged_byte_count]);
        } else {
            let merged = ((last_written_byte as usize) << (USIZE_BITS - 8))
                | (bits << (USIZE_BITS - bit_offset - count));
            self.bytes
                .extend_from_slice(&merged.to_be_bytes()[0..merged_byte_count]);
        }
        self.bit_len += count;
    }

    pub fn set_at(&mut self, pos: usize, bits: u64, count: usize) {
        debug_assert!(count < 64 - 8);

        let bit_offset = pos & 7;
        let byte_pos = pos / 8;
        let byte_count = (count + bit_offset + 7) / 8;

        let mut old = [0; 8];
        old[0..byte_count].copy_from_slice(&self.bytes[byte_pos..byte_pos + byte_count]);

        let old = u64::from_le_bytes(old);
        let merged = old | (bits << bit_offset);
        let merged = merged.to_le_bytes();
        self.bytes[byte_pos..byte_pos + byte_count].copy_from_slice(&merged[0..byte_count]);
    }

    pub fn extends_from_slice(&mut self, slice: &[u8]) {
        debug_assert_eq!(0, self.bit_len & 7);
        self.bytes.extend_from_slice(slice);
        self.bit_len += slice.len() * 8
    }
}
