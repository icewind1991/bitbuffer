#![feature(test)]
#![warn(missing_docs)]

//! Tools for reading integers of arbitrary bit length and non byte-aligned integers and other data types

extern crate test;

use is_signed::IsSigned;
use num_traits::PrimInt;
use std::cmp::min;
use std::mem::size_of;
use std::ops::BitOrAssign;

#[cfg(test)]
mod tests;
mod is_signed;

/// Errors that can be returned when trying to read from a buffer
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ReadError {
    /// Too many bits requested to fit in the requested data type
    TooManyBits {
        /// The number of bits requested to read
        requested: usize,
        /// The number of bits that fit in the requested data type
        max: usize,
    },
    /// Not enough data in the buffer to read all requested bits
    NotEnoughData {
        /// The number of bits requested to read
        requested: usize,
        /// the number of bits left in the buffer
        bits_left: usize,
    },
}

/// Either the read bits in the requested format or a [`ReadError`](enum.ReadError.html)
pub type Result<T> = std::result::Result<T, ReadError>;

/// Buffer that allows reading integers of arbitrary bit length and non byte-aligned integers
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

const USIZE_SIZE: usize = size_of::<usize>();

impl<'a> BitBuffer<'a> {
    /// Create a new BitBuffer from a byte slice with included padding
    ///
    /// The padding is required because the optimized method for reading bits can overshoot the last requested bit
    ///
    /// # Panics
    ///
    /// When not enough padding is provided (3 bytes on 32bit systems, 7 bytes on 64 bit systems)
    /// this method will panic
    ///
    /// # Examples
    ///
    /// ```
    /// use bitbuffer::BitBuffer;
    ///
    /// let bytes:&[u8] =&[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111,
    ///     0, 0, 0, 0, 0, 0, 0, 0
    /// ];
    /// let buffer = BitBuffer::from_padded_slice(bytes, 8);
    /// ```
    ///
    ///
    pub fn from_padded_slice(bytes: &'a [u8], byte_len: usize) -> BitBuffer<'a> {
        if byte_len > bytes.len() - (USIZE_SIZE - 1) {
            panic!("Not enough padding on slice, at least {} bytes of padding are required", USIZE_SIZE - 1);
        }
        BitBuffer {
            bytes,
            byte_len,
            bit_len: byte_len * 8,
        }
    }

    /// The available number of bits in the buffer
    ///
    /// Note that this does not included any padding from the source slice
    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    /// The available number of bytes in the buffer
    ///
    /// Note that this does not included any padding from the source slice
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

    /// Read a single bit from the buffer as boolean
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`](enum.ReadError.html#variant.NotEnoughData): not enough bits available in the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use bitbuffer::BitBuffer;
    ///
    /// let bytes:&[u8] =&[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111,
    ///     0, 0, 0, 0, 0, 0, 0, 0
    /// ];
    /// let buffer = BitBuffer::from_padded_slice(bytes, 8);
    /// let result = buffer.read_bool(6);
    /// ```
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

    /// Read a sequence of bits from the buffer as integer
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`](enum.ReadError.html#variant.NotEnoughData): not enough bits available in the buffer
    /// - [`ReadError::TooManyBits`](enum.ReadError.html#variant.TooManyBits): to many bits requested for the chosen integer type
    ///
    /// # Examples
    ///
    /// ```
    /// use bitbuffer::BitBuffer;
    ///
    /// let bytes:&[u8] =&[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111,
    ///     0, 0, 0, 0, 0, 0, 0, 0
    /// ];
    /// let buffer = BitBuffer::from_padded_slice(bytes, 8);
    /// let result = buffer.read::<u16>(10, 9);
    /// ```
    pub fn read<T>(&self, position: usize, count: usize) -> Result<T>
        where T: PrimInt + BitOrAssign + IsSigned
    {
        let value = {
            let type_bit_size = size_of::<T>() * 8;
            let usize_bit_size = size_of::<usize>() * 8;

            if type_bit_size < count {
                return Err(ReadError::TooManyBits {
                    requested: count,
                    max: type_bit_size,
                });
            }

            if size_of::<usize>() > size_of::<T>() || count < usize_bit_size - 8 {
                let raw = self.read_usize(position, count)?;
                let max_signed_value = (1 << (type_bit_size - 1)) - 1;
                if T::is_signed() && raw > max_signed_value {
                    return Ok(T::zero() - T::from(raw & max_signed_value).unwrap());
                } else {
                    T::from(raw).unwrap()
                }
            } else {
                let mut left_to_read = count;
                let mut partial = T::zero();
                let max_read = (size_of::<usize>() - 1) * 8;
                let mut read_pos = position;
                let mut bit_offset = 0;
                while left_to_read > 0 {
                    let bits_left = self.bit_len - read_pos;
                    let read = min(min(left_to_read, max_read), bits_left);
                    partial |= T::from(self.read_usize(read_pos, read)?).unwrap() << bit_offset;
                    bit_offset += read;
                    read_pos += read;
                    left_to_read -= read;
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

    /// Read a series of bytes from the buffer
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`](enum.ReadError.html#variant.NotEnoughData): not enough bits available in the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use bitbuffer::BitBuffer;
    ///
    /// let bytes:&[u8] =&[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111,
    ///     0, 0, 0, 0, 0, 0, 0, 0
    /// ];
    /// let buffer = BitBuffer::from_padded_slice(bytes, 8);
    /// let bytes = buffer.read_bytes(5, 3);
    /// ```
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