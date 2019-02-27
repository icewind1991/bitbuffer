use std::mem::size_of;
use std::ops::BitOrAssign;

use num_traits::{Float, PrimInt};

use crate::buffer::IsPadded;
use crate::endianness::Endianness;
use crate::is_signed::IsSigned;
use crate::BitBuffer;
use crate::{ReadError, Result};

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
/// let mut stream = BitStream::new(buffer, None, None);
/// ```
pub struct BitStream<'a, E, S>
where
    E: Endianness,
    S: IsPadded,
{
    buffer: BitBuffer<'a, E, S>,
    start_pos: usize,
    pos: usize,
    bit_len: usize,
}

impl<'a, E, S> BitStream<'a, E, S>
where
    E: Endianness,
    S: IsPadded,
{
    /// Create a new stream for a buffer
    ///
    /// # Panics
    ///
    /// - If the start_pos is higher than the bit length of the buffer
    ///
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
    /// let mut stream = BitStream::new(buffer, None, None);
    /// ```
    pub fn new(
        buffer: BitBuffer<'a, E, S>,
        start_pos: Option<usize>,
        bit_len: Option<usize>,
    ) -> Self {
        let buffer_len = buffer.bit_len();
        let start = start_pos.unwrap_or_default();
        if start > buffer_len {
            panic!("start_pos out opf bounds of the buffer")
        }
        BitStream {
            start_pos: start,
            pos: start,
            bit_len: bit_len.unwrap_or(buffer_len - start),
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
    /// let mut stream = BitStream::new(buffer, None, None);
    /// assert_eq!(stream.read_bool().unwrap(), true);
    /// assert_eq!(stream.read_bool().unwrap(), false);
    /// assert_eq!(stream.pos(), 2);
    /// ```
    pub fn read_bool(&mut self) -> Result<bool> {
        self.verify_bits_left(1)?;
        let result = self.buffer.read_bool(self.pos);
        if result.is_ok() {
            self.pos += 1;
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
    /// let mut stream = BitStream::new(buffer, None, None);
    /// assert_eq!(stream.read_int::<u16>(3).unwrap(), 0b101);
    /// assert_eq!(stream.read_int::<u16>(3).unwrap(), 0b110);
    /// assert_eq!(stream.pos(), 6);
    /// ```
    pub fn read_int<T>(&mut self, count: usize) -> Result<T>
    where
        T: PrimInt + BitOrAssign + IsSigned,
    {
        self.verify_bits_left(count)?;
        let result = self.buffer.read_int(self.pos, count);
        if result.is_ok() {
            self.pos += count;
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
    /// let mut stream = BitStream::new(buffer, None, None);
    /// let result = stream.read_float::<f32>().unwrap();
    /// assert_eq!(stream.pos(), 32);
    /// ```
    pub fn read_float<T>(&mut self) -> Result<T>
    where
        T: Float,
    {
        let count = size_of::<T>() * 8;
        self.verify_bits_left(count)?;
        let result = self.buffer.read_float(self.pos);
        if result.is_ok() {
            self.pos += count;
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
    /// let mut stream = BitStream::new(buffer, None, None);
    /// assert_eq!(stream.read_bytes(3).unwrap(), &[0b1011_0101, 0b0110_1010, 0b1010_1100]);
    /// assert_eq!(stream.pos(), 24);
    /// ```
    pub fn read_bytes(&mut self, byte_count: usize) -> Result<Vec<u8>> {
        let count = byte_count * 8;
        self.verify_bits_left(count)?;
        let result = self.buffer.read_bytes(self.pos, byte_count);
        if result.is_ok() {
            self.pos += count;
        }
        result
    }

    /// Read a series of bytes from the stream as utf8 string
    ///
    /// You can either read a fixed number of bytes, or a dynamic length null-terminated string
    ///
    /// # Errors
    ///
    /// - [`ReadError::NotEnoughData`](enum.ReadError.html#variant.NotEnoughData): not enough bits available in the buffer
    /// - [`ReadError::Utf8Error`](enum.ReadError.html#variant.Utf8Error): the read bytes are not valid utf8
    ///
    /// # Examples
    ///
    /// ```
    /// use bitstream_reader::{BitBuffer, BitStream, LittleEndian};
    ///
    /// let bytes: &[u8] = &[
    ///     0x48, 0x65, 0x6c, 0x6c,
    ///     0x6f, 0x20, 0x77, 0x6f,
    ///     0x72, 0x6c, 0x64, 0,
    ///     0,    0,    0,    0
    /// ];
    /// let buffer = BitBuffer::new(bytes, LittleEndian);
    /// let mut stream = BitStream::new(buffer, None, None);
    /// // Fixed length string
    /// stream.set_pos(0);
    /// assert_eq!(stream.read_string(Some(11)).unwrap(), "Hello world".to_owned());
    /// assert_eq!(11 * 8, stream.pos());
    /// // fixed length with null padding
    /// stream.set_pos(0);
    /// assert_eq!(stream.read_string(Some(16)).unwrap(), "Hello world".to_owned());
    /// assert_eq!(16 * 8, stream.pos());
    /// // null terminated
    /// stream.set_pos(0);
    /// assert_eq!(stream.read_string(None).unwrap(), "Hello world".to_owned());
    /// assert_eq!(12 * 8, stream.pos()); // 1 more for the terminating null byte
    /// ```
    pub fn read_string(&mut self, byte_len: Option<usize>) -> Result<String> {
        let result = self.buffer.read_string(self.pos, byte_len)?;
        let read = match byte_len {
            Some(len) => len * 8,
            None => (result.len() + 1) * 8,
        };
        self.pos += read;
        Ok(result)
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
    /// let mut stream = BitStream::new(buffer, None, None);
    /// let mut bits = stream.read_bits(3).unwrap();
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(bits.pos(), 0);
    /// assert_eq!(bits.bit_len(), 3);
    /// assert_eq!(stream.read_int::<u8>(3).unwrap(), 0b110);
    /// assert_eq!(bits.read_int::<u8>(3).unwrap(), 0b101);
    /// ```
    pub fn read_bits(&mut self, count: usize) -> Result<Self> {
        self.verify_bits_left(count)?;
        let result = BitStream::new(self.buffer.clone(), Some(self.pos), Some(count));
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
    /// let mut stream = BitStream::new(buffer, None, None);
    /// stream.skip(3).unwrap();
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(stream.read_int::<u8>(3).unwrap(), 0b110);
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
    /// let mut stream = BitStream::new(buffer, None, None);
    /// stream.set_pos(3).unwrap();
    /// assert_eq!(stream.pos(), 3);
    /// assert_eq!(stream.read_int::<u8>(3).unwrap(), 0b110);
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
    /// let mut stream = BitStream::new(buffer, None, None);
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
    /// let mut stream = BitStream::new(buffer, None, None);
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
    /// let mut stream = BitStream::new(buffer, None, None);
    /// assert_eq!(stream.bits_left(), 64);
    /// stream.skip(5).unwrap();
    /// assert_eq!(stream.bits_left(), 59);
    /// ```
    pub fn bits_left(&self) -> usize {
        self.bit_len - self.pos()
    }
}
