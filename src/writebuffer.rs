use crate::Endianness;
use std::cmp::min;
use std::iter::{once, repeat};
use std::marker::PhantomData;
use std::mem::size_of;

const USIZE_BITS: usize = size_of::<usize>() * 8;

pub struct WriteBuffer<'a, E: Endianness>(CowWriteBuffer<'a, E>);

impl<'a, E: Endianness> WriteBuffer<'a, E> {
    pub fn new(bytes: &'a mut Vec<u8>, endianness: E) -> Self {
        WriteBuffer(CowWriteBuffer::ExpandBorrowed(ExpandWriteBuffer::new(
            bytes, endianness,
        )))
    }

    /// The number of written bits in the buffer
    pub fn bit_len(&self) -> usize {
        self.0.bit_len()
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
        self.0.push_bits(bits, count)
    }

    pub fn reserve(&mut self, length: usize) -> (WriteBuffer<E>, WriteBuffer<E>) {
        let (head, tail) = self.0.reserve(length);
        (WriteBuffer(head), WriteBuffer(tail))
    }
}

enum CowWriteBuffer<'a, E: Endianness> {
    FixedBorrowed(FixedWriteBuffer<'a, E>),
    ExpandBorrowed(ExpandWriteBuffer<'a, E>),
}

impl<'a, E: Endianness> CowWriteBuffer<'a, E> {
    /// The number of written bits in the buffer
    fn bit_len(&self) -> usize {
        match self {
            CowWriteBuffer::FixedBorrowed(buffer) => buffer.bit_len(),
            CowWriteBuffer::ExpandBorrowed(buffer) => buffer.bit_len(),
        }
    }

    /// Push up to an usize worth of bits
    fn push_bits(&mut self, bits: usize, count: usize) {
        match self {
            CowWriteBuffer::FixedBorrowed(buffer) => buffer.push_bits(bits, count),
            CowWriteBuffer::ExpandBorrowed(buffer) => buffer.push_bits(bits, count),
        }
    }

    /// Reserve some bits to be written later by splitting of two parts
    fn reserve(&mut self, length: usize) -> (CowWriteBuffer<E>, CowWriteBuffer<E>) {
        match self {
            CowWriteBuffer::FixedBorrowed(buffer) => {
                let (head, tail) = buffer.reserve(length);
                (
                    CowWriteBuffer::FixedBorrowed(head),
                    CowWriteBuffer::FixedBorrowed(tail),
                )
            }
            CowWriteBuffer::ExpandBorrowed(buffer) => {
                let (head, tail) = buffer.reserve(length);
                (
                    CowWriteBuffer::FixedBorrowed(head),
                    CowWriteBuffer::ExpandBorrowed(tail),
                )
            }
        }
    }
}

struct ExpandWriteBuffer<'a, E: Endianness> {
    bit_len: usize,
    bytes: &'a mut Vec<u8>,
    endianness: PhantomData<E>,
}

impl<'a, E: Endianness> ExpandWriteBuffer<'a, E> {
    fn new(bytes: &'a mut Vec<u8>, _endianness: E) -> Self {
        ExpandWriteBuffer {
            bit_len: 0,
            bytes,
            endianness: PhantomData,
        }
    }

    /// The number of written bits in the buffer
    fn bit_len(&self) -> usize {
        self.bit_len
    }

    /// Push up to an usize worth of bits
    fn push_bits(&mut self, bits: usize, count: usize) {
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

    /// Reserve some bits to be written later by splitting of two parts
    ///
    /// One fixed size part and one expanding part
    fn reserve(&mut self, length: usize) -> (FixedWriteBuffer<E>, ExpandWriteBuffer<E>) {
        let byte_count = (length + 7) / 8;

        let bit_offset = self.bit_len & 7;
        let byte_index = self.bit_len / 8;

        let end_byte = byte_index + byte_count;

        self.bytes.resize(end_byte, 0);
        self.bit_len += length;

        // take a mut slice without telling the borrow checker
        // this is safe because
        // 1. the buffers are append only, meaning that the "expand" part can't mess with the reserved bits
        // 2. the underlying vec can only be used again after both parts have been dropped
        let bytes = unsafe {
            let ptr = self.bytes[byte_index..end_byte].as_mut_ptr();
            std::slice::from_raw_parts_mut(ptr, byte_count)
        };
        (
            FixedWriteBuffer::new(bytes, bit_offset, length + bit_offset, E::endianness()),
            ExpandWriteBuffer {
                bit_len: self.bit_len,
                bytes: self.bytes,
                endianness: PhantomData,
            },
        )
    }
}

#[test]
fn test_push_expand_be() {
    use crate::BigEndian;

    let mut buffer = vec![];
    let mut write = ExpandWriteBuffer::new(&mut buffer, BigEndian);
    write.push_bits(0b1101, 4);
    write.push_bits(0b1, 1);
    write.push_bits(0b0, 1);
    write.push_bits(0b101_01010, 8);

    assert_eq!(vec![0b1101_1_0_10, 0b101010_00], buffer)
}

#[test]
fn test_push_expand_le() {
    use crate::LittleEndian;

    let mut buffer = vec![];
    let mut write = ExpandWriteBuffer::new(&mut buffer, LittleEndian);
    write.push_bits(0b1101, 4);
    write.push_bits(0b1, 1);
    write.push_bits(0b0, 1);
    write.push_bits(0b101_01010, 8);

    assert_eq!(vec![0b10_0_1_1101, 0b00101010], buffer)
}

#[test]
fn test_push_expand_reserve_be() {
    use crate::BigEndian;

    let mut buffer = vec![];
    let mut write = ExpandWriteBuffer::new(&mut buffer, BigEndian);
    write.push_bits(0b1101, 4);

    let (mut reserved, mut rest) = write.reserve(2);
    rest.push_bits(0b101_01010, 8);

    reserved.push_bits(0b1, 1);
    reserved.push_bits(0b0, 1);

    assert_eq!(vec![0b1101_1_0_10, 0b101010_00], buffer)
}

#[test]
fn test_push_expand_reserve_le() {
    use crate::LittleEndian;

    let mut buffer = vec![];
    let mut write = ExpandWriteBuffer::new(&mut buffer, LittleEndian);
    write.push_bits(0b1101, 4);

    let (mut reserved, mut rest) = write.reserve(2);
    rest.push_bits(0b101_01010, 8);

    reserved.push_bits(0b1, 1);
    reserved.push_bits(0b0, 1);

    assert_eq!(vec![0b10_0_1_1101, 0b00101010], buffer)
}

struct FixedWriteBuffer<'a, E: Endianness> {
    bit_start: usize,
    bit_len: usize,
    bytes: &'a mut [u8],
    endianness: PhantomData<E>,
    bit_size: usize,
}

impl<'a, E: Endianness> FixedWriteBuffer<'a, E> {
    fn new(bytes: &'a mut [u8], bit_start: usize, bit_size: usize, _endianness: E) -> Self {
        FixedWriteBuffer {
            bit_start,
            bit_len: bit_start,
            bytes,
            endianness: PhantomData,
            bit_size,
        }
    }

    /// The number of written bits in the buffer
    fn bit_len(&self) -> usize {
        self.bit_len - self.bit_start
    }

    /// Push up to an usize worth of bits
    fn push_bits(&mut self, bits: usize, count: usize) {
        debug_assert!(count < USIZE_BITS - 8);
        assert!(self.bit_len + count <= self.bit_size);

        // ensure there are no stray bits
        let bits = bits & (usize::MAX >> (USIZE_BITS - count));

        let bit_offset = self.bit_len & 7;
        let byte_index = self.bit_len / 8;
        let last_written_byte = self.bytes[byte_index];
        let merged_byte_count = (count + bit_offset + 7) / 8;

        if E::is_le() {
            let merged = last_written_byte as usize | bits << bit_offset;
            self.bytes[byte_index..byte_index + merged_byte_count]
                .copy_from_slice(&merged.to_le_bytes()[0..merged_byte_count]);
        } else {
            let merged = ((last_written_byte as usize) << (USIZE_BITS - 8))
                | (bits << (USIZE_BITS - bit_offset - count));
            self.bytes[byte_index..byte_index + merged_byte_count]
                .copy_from_slice(&merged.to_be_bytes()[0..merged_byte_count]);
        }
        self.bit_len += count;
    }

    fn reserve(&mut self, length: usize) -> (FixedWriteBuffer<E>, FixedWriteBuffer<E>) {
        assert!(self.bit_len + length <= self.bit_size);
        let byte_count = (length + 7) / 8;

        let bit_offset = self.bit_len & 7;
        let byte_index = self.bit_len / 8;

        self.bit_len += length;

        // take a mut slice without telling the borrow checker
        // this is safe because
        // 1. the buffers are append only, meaning that the last part can't mess with the reserved bits
        // 2. the underlying vec can only be used again after both parts have been dropped
        let bytes = unsafe {
            let ptr = self.bytes[byte_index..byte_count + byte_count].as_mut_ptr();
            std::slice::from_raw_parts_mut(ptr, byte_count)
        };
        (
            FixedWriteBuffer::new(bytes, bit_offset, length + bit_offset, E::endianness()),
            FixedWriteBuffer::new(self.bytes, self.bit_len, self.bit_size, E::endianness()),
        )
    }
}

#[test]
fn test_push_fixed_be() {
    use crate::BigEndian;

    let mut buffer = vec![0; 2];
    let mut write = FixedWriteBuffer::new(&mut buffer, 0, 16, BigEndian);
    write.push_bits(0b1101, 4);
    assert_eq!(4, write.bit_len());
    write.push_bits(0b1, 1);
    assert_eq!(5, write.bit_len());
    write.push_bits(0b0, 1);
    assert_eq!(6, write.bit_len());
    write.push_bits(0b101_01010, 8);

    assert_eq!(vec![0b1101_1_0_10, 0b101010_00], buffer)
}

#[test]
fn test_push_fixed_le() {
    use crate::LittleEndian;

    let mut buffer = vec![0; 2];
    let mut write = FixedWriteBuffer::new(&mut buffer, 0, 16, LittleEndian);
    write.push_bits(0b1101, 4);
    assert_eq!(4, write.bit_len());
    write.push_bits(0b1, 1);
    assert_eq!(5, write.bit_len());
    write.push_bits(0b0, 1);
    assert_eq!(6, write.bit_len());
    write.push_bits(0b101_01010, 8);

    assert_eq!(vec![0b10_0_1_1101, 0b00101010], buffer)
}

#[test]
fn test_push_fixed_reserve_be() {
    use crate::BigEndian;

    let mut buffer = vec![0; 2];
    let mut write = FixedWriteBuffer::new(&mut buffer, 0, 16, BigEndian);
    write.push_bits(0b1101, 4);

    let (mut reserved, mut rest) = write.reserve(2);
    rest.push_bits(0b101_01010, 8);

    reserved.push_bits(0b1, 1);
    reserved.push_bits(0b0, 1);

    assert_eq!(vec![0b1101_1_0_10, 0b101010_00], buffer)
}

#[test]
fn test_push_fixed_reserve_le() {
    use crate::LittleEndian;

    let mut buffer = vec![0; 2];
    let mut write = FixedWriteBuffer::new(&mut buffer, 0, 16, LittleEndian);
    write.push_bits(0b1101, 4);

    let (mut reserved, mut rest) = write.reserve(2);
    rest.push_bits(0b101_01010, 8);

    reserved.push_bits(0b1, 1);
    reserved.push_bits(0b0, 1);

    assert_eq!(vec![0b10_0_1_1101, 0b00101010], buffer)
}
