#![warn(missing_docs)]

//! Tools for reading integers of arbitrary bit length and non byte-aligned integers and other data types

// for bench on nightly
//extern crate test;

pub use endianness::{BigEndian, LittleEndian};
use endianness::Endianness;
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
    /// The requested position is outside the bounds of the buffer
    IndexOutOfBounds {
        /// The requested position
        pos: usize,
        /// the number of bits in the buffer
        size: usize,
    },
}

/// Mark source slice as not including padding
pub struct NonPadded;

/// Mark source slice as including padding
pub struct Padded;

/// Determine whether or not the source slice is padded
pub trait IsPadded {
    /// Whether or not the slice is padded
    fn is_padded() -> bool;
}

impl IsPadded for NonPadded {
    #[inline]
    fn is_padded() -> bool {
        false
    }
}

impl IsPadded for Padded {
    #[inline]
    fn is_padded() -> bool {
        true
    }
}

/// Either the read bits in the requested format or a [`ReadError`](enum.ReadError.html)
pub type Result<T> = std::result::Result<T, ReadError>;

/// Buffer that allows reading integers of arbitrary bit length and non byte-aligned integers
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
/// let buffer = BitBuffer::new(bytes, LittleEndian);
/// ```
///
/// You can also provide a slice padded with at least `size_of::<usize>() - 1` bytes,
/// when the input slice is padded, the BitBuffer can use some optimizations which result in a ~1.5 time performance increase
///
/// ```
/// use bitstream_reader::{BitBuffer, LittleEndian};
///
/// let bytes: &[u8] = &[
///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111,
///     0, 0, 0, 0, 0, 0, 0, 0
/// ];
/// let buffer = BitBuffer::from_padded_slice(bytes, 8, LittleEndian);
/// ```
pub struct BitBuffer<'a, E, S>
    where
        E: Endianness,
        S: IsPadded,
{
    bytes: &'a [u8],
    bit_len: usize,
    byte_len: usize,
    endianness: PhantomData<E>,
    is_padded: PhantomData<S>,
}

impl<'a, E> BitBuffer<'a, E, NonPadded>
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
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// ```
    pub fn new(bytes: &'a [u8], _endianness: E) -> Self {
        let byte_len = bytes.len();
        BitBuffer {
            bytes,
            byte_len,
            bit_len: byte_len * 8,
            endianness: PhantomData,
            is_padded: PhantomData,
        }
    }
}

impl<'a, E> BitBuffer<'a, E, Padded>
    where
        E: Endianness,
{
    /// Create a new BitBuffer from a byte slice with included padding
    ///
    /// by including at least `size_of::<usize>() - 1` bytes of padding reading can be further optimized
    ///
    /// # Panics
    ///
    /// Panics if not enough bytes of padding are included
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111,
    ///     0, 0, 0, 0, 0, 0, 0, 0
    /// ];
    /// let buffer = BitBuffer::from_padded_slice(bytes, 8, LittleEndian);
    /// ```
    pub fn from_padded_slice(bytes: &'a [u8], byte_len: usize, _endianness: E) -> Self {
        if bytes.len() < byte_len + USIZE_SIZE - 1 {
            panic!(
                "not enough padding bytes, {} required, {} provided",
                USIZE_SIZE - 1,
                byte_len - bytes.len()
            )
        }
        BitBuffer {
            bytes,
            byte_len,
            bit_len: byte_len * 8,
            endianness: PhantomData,
            is_padded: PhantomData,
        }
    }
}

impl<'a, E, S> BitBuffer<'a, E, S>
    where
        E: Endianness,
        S: IsPadded,
{
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
        let byte_index = if S::is_padded() {
            position / 8
        } else {
            min(position / 8, self.byte_len - USIZE_SIZE)
        };
        //let byte_index = position / 8;
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
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
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
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
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
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
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
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
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

/// Stream that provides an easy way to iterate trough a BitBuffer
///
/// # Examples
///
/// ```
/// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
///
/// let bytes: &[u8] = &[
///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
/// ];
/// let buffer = BitBuffer::new(bytes, LittleEndian);
/// let mut stream = BitStream::new(&buffer, None, None);
/// ```
pub struct BitStream<'a, E, S>
    where
        E: Endianness,
        S: IsPadded,
{
    buffer: &'a BitBuffer<'a, E, S>,
    start_pos: usize,
    pos: usize,
    bit_len: usize,
}

impl<'a, E, S> BitStream<'a, E, S>
    where
        E: Endianness,
        S: IsPadded, {
    /// Create a new stream for a buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitStream::new(&buffer, None, None);
    /// ```
    pub fn new(buffer: &'a BitBuffer<'a, E, S>, start_pos: Option<usize>, bit_len: Option<usize>) -> Self {
        BitStream {
            start_pos: start_pos.unwrap_or(0),
            pos: start_pos.unwrap_or(0),
            bit_len: bit_len.unwrap_or(buffer.bit_len()),
            buffer,
        }
    }

    fn verify_bits_left(&self, count: usize) -> Result<()> {
        if self.bits_left() < count {
            Err(ReadError::NotEnoughData {
                bits_left: self.bits_left(),
                requested: count,
            })
        } else {
            Ok(())
        }
    }

    /// Read a single bit from the stream as boolean
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`](enum.ReadError.html#variant.NotEnoughData): not enough bits available in the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitStream::new(&buffer, None, None);
    /// assert_eq!(stream.read_bool().unwrap(), true);
    /// assert_eq!(stream.read_bool().unwrap(), false);
    /// assert_eq!(stream.pos(), 2);
    /// ```
    pub fn read_bool(&mut self) -> Result<bool> {
        self.verify_bits_left(1)?;
        let result = self.buffer.read_bool(self.pos);
        match result {
            Ok(_) => self.pos += 1,
            Err(_) => {}
        }
        result
    }

    /// Read a sequence of bits from the stream as integer
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`](enum.ReadError.html#variant.NotEnoughData): not enough bits available in the buffer
    /// - [`ReadError::TooManyBits`](enum.ReadError.html#variant.TooManyBits): to many bits requested for the chosen integer type
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitStream::new(&buffer, None, None);
    /// assert_eq!(stream.read::<u16>(3).unwrap(), 0b101);
    /// assert_eq!(stream.read::<u16>(3).unwrap(), 0b110);
    /// assert_eq!(stream.pos(), 6);
    /// ```
    pub fn read<T>(&mut self, count: usize) -> Result<T>
        where
            T: PrimInt + BitOrAssign + IsSigned, {
        self.verify_bits_left(count)?;
        let result = self.buffer.read(self.pos, count);
        match result {
            Ok(_) => self.pos += count,
            Err(_) => {}
        }
        result
    }

    /// Read a sequence of bits from the stream as float
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`](enum.ReadError.html#variant.NotEnoughData): not enough bits available in the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitStream::new(&buffer, None, None);
    /// let result = stream.read_float::<f32>().unwrap();
    /// assert_eq!(stream.pos(), 32);
    /// ```
    pub fn read_float<T>(&mut self) -> Result<T>
        where
            T: Float, {
        let count = size_of::<T>() * 8;
        self.verify_bits_left(count)?;
        let result = self.buffer.read_float(self.pos);
        match result {
            Ok(_) => self.pos += count,
            Err(_) => {}
        }
        result
    }

    /// Read a series of bytes from the stream
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`](enum.ReadError.html#variant.NotEnoughData): not enough bits available in the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitStream::new(&buffer, None, None);
    /// let bytes = stream.read_bytes(3).unwrap();
    /// assert_eq!(bytes, &[0b1011_0101, 0b0110_1010, 0b1010_1100]);
    /// assert_eq!(stream.pos(), 24);
    /// ```
    pub fn read_bytes(&mut self, byte_count: usize) -> Result<Vec<u8>> {
        let count = byte_count * 8;
        self.verify_bits_left(count)?;
        let result = self.buffer.read_bytes(self.pos, byte_count);
        match result {
            Ok(_) => self.pos += count,
            Err(_) => {}
        }
        result
    }

    /// Read a sequence of bits from the stream as a BitStream
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`](enum.ReadError.html#variant.NotEnoughData): not enough bits available in the buffer
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitStream::new(&buffer, None, None);
    /// let mut bits = stream.read_bits(3).unwrap();
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(bits.pos(), 0);
    /// assert_eq!(bits.bit_len(), 3);
    /// assert_eq!(stream.read::<u8>(3).unwrap(), 0b110);
    /// assert_eq!(bits.read::<u8>(3).unwrap(), 0b101);
    /// ```
    pub fn read_bits(&mut self, count: usize) -> Result<Self> {
        self.verify_bits_left(count)?;
        let result = BitStream::new(&self.buffer, Some(self.pos), Some(count));
        self.pos += count;
        Ok(result)
    }

    /// Skip a number of bits in the stream
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`](enum.ReadError.html#variant.NotEnoughData): not enough bits available in the buffer to skip
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitStream::new(&buffer, None, None);
    /// stream.skip(3).unwrap();
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(stream.read::<u8>(3).unwrap(), 0b110);
    /// ```
    pub fn skip(&mut self, count: usize) -> Result<()> {
        self.verify_bits_left(count)?;
        self.pos += count;
        Ok(())
    }

    /// Set the position of the stream
    ///
    /// # Errors
    ///
    /// - [`ReadError::IndexOutOfBounds`](enum.ReadError.html#variant.IndexOutOfBounds): new position is outside the bounds of the stream
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitStream::new(&buffer, None, None);
    /// stream.set_pos(3).unwrap();
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(stream.read::<u8>(3).unwrap(), 0b110);
    /// ```
    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        if pos > self.bit_len {
            return Err(ReadError::IndexOutOfBounds {
                pos,
                size: self.bit_len,
            });
        }
        self.pos = pos + self.start_pos;
        Ok(())
    }

    /// Get the length of the stream in bits
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitStream::new(&buffer, None, None);
    /// assert_eq!(stream.bit_len(), 64);
    /// ```
    pub fn bit_len(&self) -> usize {
        self.bit_len
    }

    /// Get the current position in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitStream::new(&buffer, None, None);
    /// assert_eq!(stream.pos(), 0);
    /// stream.skip(5).unwrap();
    /// assert_eq!(stream.pos(), 5);
    /// ```
    pub fn pos(&self) -> usize {
        self.pos - self.start_pos
    }

    /// Get the number of bits left in the stream
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0b1011_0101, 0b0110_1010, 0b1010_1100, 0b1001_1001,
    ///     0b1001_1001, 0b1001_1001, 0b1001_1001, 0b1110_0111
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitStream::new(&buffer, None, None);
    /// assert_eq!(stream.bits_left(), 64);
    /// stream.skip(5).unwrap();
    /// assert_eq!(stream.bits_left(), 59);
    /// ```
    pub fn bits_left(&self) -> usize {
        self.bit_len - self.pos()
    }
}