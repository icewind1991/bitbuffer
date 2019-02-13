#![feature(test)]

extern crate test;

use std::mem::size_of;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ReadError {
    TooManyBits {
        requested: usize,
        max: usize,
    },
    NotEnoughData {
        requested: usize,
        bits_left: usize,
    },
}

pub type Result<T> = std::result::Result<T, ReadError>;

pub struct BitBuffer {
    bytes: Vec<u8>,
    bit_len: usize,
    byte_len: usize,
}

macro_rules! array_ref {
    ($arr:expr, $offset:expr, $len:expr) => {{
        {
            #[inline]
            unsafe fn as_array<T>(slice: &[T]) -> &[T; $len] {
                &*(slice.as_ptr() as *const [_; $len])
            }
            let offset = $offset;
            let slice = & $arr[offset..offset + $len];
            #[allow(unused_unsafe)]
            unsafe {
                as_array(slice)
            }
        }
    }}
}

const USIZE_SIZE: usize = std::mem::size_of::<usize>();

macro_rules! bitreader_unsigned_le {
    ($buffer:expr, $type:ty, $position:expr, $count:expr) => {
        {
            let size: usize = size_of::<$type>() * 8;
            if $count > size {
                return Err(ReadError::TooManyBits { requested: $count, max: size });
            }

            let bits_left = $buffer.bit_len() - $position;

            if $count > bits_left {
                return Err(ReadError::NotEnoughData { requested: $count, bits_left});
            }

            let byte_index = $position / 8;
            let bit_offset = $position & 7;
            let bytes:&[u8; USIZE_SIZE] = array_ref!($buffer.bytes(), byte_index, USIZE_SIZE);
            let container_le = unsafe {
                std::mem::transmute::<[u8; USIZE_SIZE], usize>(*bytes)
            };
            let container = usize::from_le(container_le);
            let shifted = container >> bit_offset;
            let mask = if $count == (USIZE_SIZE * 8) {usize::max_value()} else {!(usize::max_value() << $count)};
            let value = shifted & mask;

            Ok(value as $type)
        }
    }
}


macro_rules! make_signed {
    ($unsigned:expr, $type:ty, $count:expr) => {
        {
            let sign_bits = $unsigned >> ($count - 1) & 1;
            let high_bits = 0 - sign_bits as $type;
            high_bits << $count | $unsigned as $type
        }
    }
}

impl BitBuffer {
    pub fn from_slice(data: &[u8]) -> BitBuffer {
        let mut bytes = vec![];
        bytes.extend_from_slice(data);
        BitBuffer::from_vec(bytes)
    }

    pub fn from_vec(mut bytes: Vec<u8>) -> BitBuffer {
        let byte_len = bytes.len();
        // add leading 0 bytes for overflow during reading
        bytes.resize(byte_len + size_of::<usize>(), 0);
        BitBuffer::from_padded_vec(&bytes, byte_len)
    }

    pub fn from_padded_vec(bytes: &Vec<u8>, byte_len: usize) -> BitBuffer {
        BitBuffer {
            bytes: bytes.to_vec(),
            bit_len: byte_len * 8,
            byte_len,
        }
    }

    fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    pub fn byte_len(&self) -> usize {
        self.byte_len
    }

    pub fn read_u8(&self, position: usize, count: usize) -> Result<u8> {
        self.bytes.
        bitreader_unsigned_le!(self, u8, position, count)
    }

    pub fn read_u16(&self, position: usize, count: usize) -> Result<u16> {
        bitreader_unsigned_le!(self, u16, position, count)
    }

    pub fn read_u32(&self, position: usize, count: usize) -> Result<u32> {
        bitreader_unsigned_le!(self, u32, position, count)
    }

    pub fn read_i8(&self, position: usize, count: usize) -> Result<i8> {
        let unsigned = self.read_u8(position, count)?;
        Ok(make_signed!(unsigned, i8, count))
    }

    pub fn read_i16(&self, position: usize, count: usize) -> Result<i16> {
        let unsigned = self.read_u16(position, count)?;
        Ok(make_signed!(unsigned, i16, count))
    }

    pub fn read_i32(&self, position: usize, count: usize) -> Result<i32> {
        let unsigned = self.read_u32(position, count)?;
        Ok(make_signed!(unsigned, i32, count))
    }
}