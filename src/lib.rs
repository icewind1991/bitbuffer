#![feature(test)]
#![warn(missing_docs)]

//! Tools for reading integers of arbitrary bit length and non byte-aligned integers and other data types

extern crate test;

use endianness::Endianness;
pub use endianness::{BigEndian, LittleEndian};
use is_signed::IsSigned;
use num_traits::{Float, PrimInt};
use std::cmp::min;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::BitOrAssign;

mod endianness;
mod is_signed;
#[cfg(test)]
mod tests;

const USIZE_SIZE: usize = size_of::<usize>();

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
///
/// The endianness used when reading from the buffer is specified as type parameter
pub struct BitBuffer<'a, E>
where
    E: Endianness,
{
    bytes: &'a [u8],
    bit_len: usize,
    byte_len: usize,
    endianness: PhantomData<E>,
}

impl<'a, E> BitBuffer<'a, E>
where
    E: Endianness,
{
    /// Create a new BitBuffer from a byte slice
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer: BitBuffer<LittleEndian> = BitBuffer::new(bytes);
    /// ```
    pub fn new(bytes: &'a [u8]) -> Self {
        let byte_len = bytes.len();
        BitBuffer {
            bytes,
            byte_len,
            bit_len: byte_len * 8,
            endianness: PhantomData,
        }
    }

    /// The available number of bits in the buffer
    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    /// The available number of bytes in the buffer
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
        let byte_index = min(position / 8, self.byte_len - USIZE_SIZE);
        let bit_offset = position - byte_index * 8;
        let slice = &self.bytes[byte_index..byte_index + USIZE_SIZE];
        let bytes: [u8; USIZE_SIZE] = unsafe { *(slice.as_ptr() as *const [u8; USIZE_SIZE]) };
        let container = if E::is_le() {
            usize::from_le_bytes(bytes)
        } else {
            usize::from_be_bytes(bytes)
        };
        let shifted = if E::is_le() {
            container >> bit_offset
        } else {
            container >> USIZE_SIZE * 8 - bit_offset - count
        };
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
    /// use bitstream_reader::{BitBuffer, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer: BitBuffer<LittleEndian> = BitBuffer::new(bytes);
    /// let result = buffer.read_bool(5).unwrap();
    /// assert_eq!(result, true);
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
        Ok(shifted & 1u8 == 1)
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
    /// use bitstream_reader::{BitBuffer, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer: BitBuffer<LittleEndian> = BitBuffer::new(bytes);
    /// let result = buffer.read::<u16>(10, 9).unwrap();
    /// assert_eq!(result, 0b100_0110_10);
    /// ```
    pub fn read<T>(&self, position: usize, count: usize) -> Result<T>
    where
        T: PrimInt + BitOrAssign + IsSigned,
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

            let bit_offset = position & 7;
            if size_of::<usize>() > size_of::<T>() || count + bit_offset < usize_bit_size {
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
                    let data = T::from(self.read_usize(read_pos, read)?).unwrap();
                    if E::is_le() {
                        partial |= data << bit_offset;
                    } else {
                        partial = partial << read;
                        partial |= data;
                    }
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
    /// use bitstream_reader::{BitBuffer, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer: BitBuffer<LittleEndian> = BitBuffer::new(bytes);
    /// let bytes = buffer.read_bytes(5, 3).unwrap();
    /// assert_eq!(bytes, &[0b0_1010_101, 0b0_1100_011, 0b1_1001_101]);
    /// ```
    pub fn read_bytes(&self, position: usize, byte_count: usize) -> Result<Vec<u8>> {
        let mut data = vec![];
        data.reserve_exact(byte_count);
        let mut byte_left = byte_count;
        let max_read = size_of::<usize>() - 1;
        let mut read_pos = position;
        while byte_left > 0 {
            let read = min(byte_left, max_read);
            let bytes: [u8; USIZE_SIZE] = self.read_usize(read_pos, read * 8)?.to_le_bytes();
            let usable_bytes = &bytes[0..read];
            data.extend_from_slice(usable_bytes);
            byte_left -= read;
            read_pos += read;
        }
        Ok(data)
    }

    /// Read a sequence of bits from the buffer as float
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`](enum.ReadError.html#variant.NotEnoughData): not enough bits available in the buffer
    /// - [`ReadError::TooManyBits`](enum.ReadError.html#variant.TooManyBits): to many bits requested for the chosen integer type
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer: BitBuffer<LittleEndian> = BitBuffer::new(bytes);
    /// let result = buffer.read_float::<f32>(10).unwrap();
    /// ```
    pub fn read_float<T>(&self, position: usize) -> Result<T>
    where
        T: Float,
    {
        if size_of::<T>() == 4 {
            let int = self.read::<u32>(position, 32)?;
            Ok(T::from(f32::from_bits(int)).unwrap())
        } else {
            let int = self.read::<u64>(position, 64)?;
            Ok(T::from(f64::from_bits(int)).unwrap())
        }
    }
}
