#![feature(test)]

extern crate test;

use is_signed::IsSigned;
use num_traits::{PrimInt};
use std::cmp::min;
use std::mem::size_of;
use std::ops::BitOrAssign;

#[cfg(test)]
mod tests;
mod is_signed;

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

pub struct BitBuffer<'a> {
    bytes: &'a [u8],
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

impl<'a> BitBuffer<'a> {
    pub fn from_padded_slice(bytes: &'a [u8], byte_len: usize) -> BitBuffer<'a> {
        BitBuffer {
            bytes,
            byte_len,
            bit_len: byte_len * 8,
        }
    }

    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    pub fn byte_len(&self) -> usize {
        self.byte_len
    }

    fn read_usize(&self, position: usize, count: usize) -> Result<usize> {
        if position + count > self.bit_len {
            return Err(ReadError::NotEnoughData {
                requested: count,
                bits_left: self.bit_len - position,
            });
        }
        let byte_index = position / 8;
        let bit_offset = position & 7;
        let bytes: &[u8; USIZE_SIZE] = array_ref!(self.bytes, byte_index, USIZE_SIZE);
        let container_le = unsafe {
            std::mem::transmute::<[u8; USIZE_SIZE], usize>(*bytes)
        };
        let container = usize::from_le(container_le);
        let shifted = container >> bit_offset;
        let mask = !(usize::max_value() << count);
        Ok(shifted & mask)
    }

    pub fn read_bool(&self, position: usize) -> Result<bool> {
        let byte_index = position / 8;
        let bit_offset = position & 7;

        if position >= self.bit_len {
            return Err(ReadError::NotEnoughData {
                requested: 1,
                bits_left: self.bit_len - position,
            });
        }

        let byte = self.bytes[byte_index];
        let shifted = byte >> bit_offset;
        let mask = 1u8 << bit_offset;
        Ok(shifted & mask == 1)
    }

    pub fn read<T>(&self, position: usize, count: usize) -> Result<T>
        where T: PrimInt + BitOrAssign + IsSigned
    {
        let value = {
            let type_bit_size = size_of::<T>() * 8;

            if type_bit_size < count {
                return Err(ReadError::TooManyBits {
                    requested: count,
                    max: size_of::<T>() * 8,
                });
            }

            if size_of::<usize>() > size_of::<T>() || (count / 8) < size_of::<usize>() {
                let raw = self.read_usize(position, count)?;
                let max_signed_value = (1 << (type_bit_size - 1)) - 1;
                if T::is_signed() && raw > max_signed_value {
                    return Ok(T::zero() - T::from(raw & max_signed_value).unwrap())
                } else {
                    T::from(raw).unwrap()
                }
            } else {
                let mut bits_left = count;
                let mut partial = T::zero();
                let max_read = size_of::<usize>() - 1 * 8;
                let mut read_pos = position;
                let mut bit_offset = 0;
                while bits_left > 0 {
                    let read = min(min(bits_left, max_read), self.bit_len - read_pos);
                    partial |= T::from(self.read_usize(read_pos, read)?).unwrap() << bit_offset;
                    bit_offset += read;
                    read_pos += read;
                    bits_left -= read;
                }

                partial
            }
        };

        if T::is_signed() {
            let sign_bit = value >> (count - 1) & T::one();
            let absolute_value = value & !(T::max_value() << (count - 1));
            let sign = T::one() - sign_bit - sign_bit;
            Ok(absolute_value * sign)
        } else {
            Ok(value)
        }
    }

    pub fn read_bytes(&self, position: usize, byte_count: usize) -> Result<Vec<u8>> {
        let mut data = vec!();
        data.reserve_exact(byte_count);
        let mut byte_left = byte_count;
        let max_read = size_of::<usize>() - 1;
        let mut read_pos = position;
        while byte_left > 0 {
            let read = min(byte_left, max_read);
            let bytes: [u8; USIZE_SIZE] = self.read_usize(read_pos, read * 8)?.to_le_bytes();
            let usable_bytes = &bytes[0..max_read];
            data.extend_from_slice(usable_bytes);
            byte_left -= read;
            read_pos += read;
        }
        Ok(data)
    }
}