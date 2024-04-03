use crate::Endianness;
use std::cmp::min;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut, Range};

enum WriteData<'a> {
    Vec(&'a mut Vec<u8>),
    Slice { data: &'a mut [u8], length: usize },
}

impl<'a> WriteData<'a> {
    fn pop(&mut self) -> Option<u8> {
        match self {
            WriteData::Vec(vec) => vec.pop(),
            WriteData::Slice { data, length } if *length > 0 => {
                *length -= 1;
                Some(data[*length])
            }
            _ => None,
        }
    }

    fn extend_from_slice(&mut self, other: &[u8]) {
        match self {
            WriteData::Vec(vec) => vec.extend_from_slice(other),
            WriteData::Slice { data, length } => {
                let end = *length + other.len();
                let target = &mut data[*length..end];
                target.copy_from_slice(other);
                *length += other.len();
            }
        }
    }

    fn push(&mut self, byte: u8) {
        match self {
            WriteData::Vec(vec) => vec.push(byte),
            WriteData::Slice { data, length } => {
                data[*length] = byte;
                *length += 1;
            }
        }
    }

    fn last_mut(&mut self) -> Option<&mut u8> {
        match self {
            WriteData::Vec(vec) => vec.last_mut(),
            WriteData::Slice { data, length } if *length > 0 => Some(&mut data[*length - 1]),
            _ => None,
        }
    }
}

impl<'a> Index<usize> for WriteData<'a> {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            WriteData::Vec(vec) => &vec[index],
            WriteData::Slice { data, .. } => &data[index],
        }
    }
}

impl<'a> IndexMut<usize> for WriteData<'a> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self {
            WriteData::Vec(vec) => &mut vec[index],
            WriteData::Slice { data, .. } => &mut data[index],
        }
    }
}

impl<'a> Index<Range<usize>> for WriteData<'a> {
    type Output = [u8];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        match self {
            WriteData::Vec(vec) => &vec[index],
            WriteData::Slice { data, .. } => &data[index],
        }
    }
}

impl<'a> IndexMut<Range<usize>> for WriteData<'a> {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        match self {
            WriteData::Vec(vec) => &mut vec[index],
            WriteData::Slice { data, .. } => &mut data[index],
        }
    }
}

pub struct WriteBuffer<'a, E: Endianness> {
    bit_len: usize,
    bytes: WriteData<'a>,
    endianness: PhantomData<E>,
}

impl<'a, E: Endianness> WriteBuffer<'a, E> {
    pub fn new(bytes: &'a mut Vec<u8>, _endianness: E) -> Self {
        WriteBuffer {
            bit_len: 0,
            bytes: WriteData::Vec(bytes),
            endianness: PhantomData,
        }
    }
    pub fn for_slice(bytes: &'a mut [u8], _endianness: E) -> Self {
        WriteBuffer {
            bit_len: 0,
            bytes: WriteData::Slice {
                data: bytes,
                length: 0,
            },
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
        I: DoubleEndedIterator<Item = (usize, u8)>,
    {
        let mut remaining = count;
        for (chunk, chunk_size) in bits {
            if remaining > 0 {
                let bits = min(remaining, chunk_size as usize);
                self.push_bits(chunk, bits);
                remaining -= bits
            }
        }
    }

    /// Push up to an usize worth of bits
    pub fn push_bits(&mut self, bits: usize, count: usize) {
        if count == 0 {
            return;
        }

        // ensure there are no stray bits
        let bits = bits & (usize::MAX >> (usize::BITS as usize - count));

        let bit_offset = self.bit_len & 7;

        debug_assert!(count <= usize::BITS as usize - bit_offset);

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
            let merged = ((last_written_byte as usize) << (usize::BITS as usize - 8))
                | (bits << (usize::BITS as usize - bit_offset - count));
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

    pub fn push_bool(&mut self, val: bool) {
        let val = val as u8;
        let bit_offset = self.bit_len() % 8;
        let shift = if E::is_le() {
            bit_offset
        } else {
            7 - bit_offset
        };
        if bit_offset == 0 {
            self.bytes.push(val << shift);
        } else {
            *self.bytes.last_mut().unwrap() |= val << shift;
        }
        self.bit_len += 1;
    }
}
